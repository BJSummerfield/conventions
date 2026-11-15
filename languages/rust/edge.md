# Rust: the edge (E)

Examples use axum; the shape is the same for any framework.

## Request and validation errors (E2, E4, E10)

```rust
// src/http/entities/requests.rs
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateEntityRequest {
    pub name: Option<String>,
    pub team_id: Option<String>,
}

#[derive(Debug, Default, Serialize, PartialEq)]
pub struct EntityValidationErrors {
    pub name_value: String,
    pub name_error: Option<String>,
    pub team_id_value: String,
    pub team_id_error: Option<String>,
}

fn required<T, E: ToString>(
    field: &str,
    value: &Option<String>,
    parse: impl FnOnce(&str) -> Result<T, E>,
) -> Result<T, String> {
    match value.as_deref() {
        None | Some("") => Err(format!("{field} is required")),
        Some(raw) => parse(raw).map_err(|e| e.to_string()),
    }
}
```

## Inbound crossing collects every error (E3, E5)

```rust
impl TryFrom<&CreateEntityRequest> for CreateEntity {
    type Error = EntityValidationErrors;

    fn try_from(req: &CreateEntityRequest) -> Result<Self, Self::Error> {
        let name = required("name", &req.name, EntityName::new);
        let team_id = required("team_id", &req.team_id, TeamId::new);

        match (name, team_id) {
            (Ok(name), Ok(team_id)) => Ok(CreateEntity::new(name, team_id)),
            (name, team_id) => Err(EntityValidationErrors {
                name_value: req.name.clone().unwrap_or_default(),
                name_error: name.err(),
                team_id_value: req.team_id.clone().unwrap_or_default(),
                team_id_error: team_id.err(),
            }),
        }
    }
}
```

Both fields are parsed before the `match`, so a request with two bad fields
reports both. The domain constructor runs exactly once per field — never a
"validate, then construct again with `expect`" double pass.

## Per-action response and error (E7, E11, E12, E14)

```rust
// src/http/entities/responses.rs
#[derive(Debug, Serialize, PartialEq)]
pub struct EntityBody {
    id: String,
    name: String,
}

impl From<&Entity> for EntityBody {
    fn from(entity: &Entity) -> Self {
        Self { id: entity.id().to_string(), name: entity.name().to_string() }
    }
}

pub struct EntityCreatedResponse(EntityBody);

impl IntoResponse for EntityCreatedResponse {
    fn into_response(self) -> Response {
        (StatusCode::CREATED, Json(self.0)).into_response()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EntityCreateError {
    #[error("validation failed")]
    Validation(EntityValidationErrors),

    #[error("{message}")]
    Conflict { message: String },

    #[error("internal error: {0}")]
    Internal(#[source] EntityServiceError),
}

impl From<EntityServiceError> for EntityCreateError {
    fn from(e: EntityServiceError) -> Self {
        match e {
            EntityServiceError::Entity(EntityError::AlreadyExists(id)) => {
                Self::Conflict { message: format!("entity {id} already exists") }
            }
            EntityServiceError::TeamNotFound(id) => Self::Validation(EntityValidationErrors {
                team_id_value: id.to_string(),
                team_id_error: Some("unknown team".into()),
                ..Default::default()
            }),
            // Field validity was settled by the conversion (E3), so these reaching a
            // service are internal faults; listing them means a new variant fails to
            // compile here instead of falling into Internal unseen (X9).
            e @ EntityServiceError::Entity(
                EntityError::Name(_) | EntityError::Id(_) | EntityError::Repository(_),
            ) => Self::Internal(e),
            e @ EntityServiceError::Team(TeamError::Repository(_)) => Self::Internal(e),
        }
    }
}

impl IntoResponse for EntityCreateError {
    fn into_response(self) -> Response {
        match self {
            Self::Validation(errors) => (StatusCode::BAD_REQUEST, Json(errors)).into_response(),
            Self::Conflict { message } => {
                (StatusCode::CONFLICT, Json(json!({ "error": message }))).into_response()
            }
            Self::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "internal error" })))
                    .into_response()
            }
        }
    }
}
```

`Internal` is the redaction point (X5): the failure middleware logs
`error.to_string()` — the whole chain — once (O5), and the client gets the
fixed body.

## Thin handler (E13, E15)

```rust
// src/http/entities/routes.rs
async fn create_entity(
    State(service): State<Arc<EntityService>>,
    Json(req): Json<CreateEntityRequest>,
) -> Result<EntityCreatedResponse, EntityCreateError> {
    let command = CreateEntity::try_from(&req).map_err(EntityCreateError::Validation)?;
    let entity = service.create(command).await.map_err(|e| {
        reattach(EntityCreateError::from(e), &req)
    })?;
    Ok(EntityCreatedResponse(EntityBody::from(&entity)))
}
```

`reattach` copies the submitted values onto a service-stage `Validation`
so the 400 body matches a conversion-stage one (E13).

## Wire-shape test (T3)

```rust
#[tokio::test]
async fn internal_error_body_withholds_detail() {
    let response = EntityCreateError::Internal(EntityServiceError::Entity(
        EntityError::Repository("disk full at /var/lib/db".into()),
    ))
    .into_response();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body_json(response).await, json!({ "error": "internal error" }));
}
```

## Paths cross through one extractor (E17)

```rust
// src/http/parsed.rs
pub struct Parsed<T>(pub T);

impl<S, T> FromRequestParts<S> for Parsed<T>
where
    S: Send + Sync,
    T: TryFrom<T::Raw> + PathCommand,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(raw) = Path::<T::Raw>::from_request_parts(parts, state)
            .await
            .map_err(|_| not_found())?;
        T::try_from(raw).map(Parsed).map_err(|_| not_found())
    }
}

pub trait PathCommand: Sized {
    type Raw: DeserializeOwned + Send;
}
```

```rust
// src/http/board/requests.rs
#[derive(Deserialize)]
pub struct MoveOrderPath { order_id: String, transition: String }

impl TryFrom<MoveOrderPath> for MoveOrder {
    type Error = OrderError;
    fn try_from(path: MoveOrderPath) -> Result<Self, Self::Error> {
        Ok(MoveOrder::new(OrderId::new(&path.order_id)?, path.transition.parse()?))
    }
}

impl PathCommand for MoveOrder { type Raw = MoveOrderPath; }

// the handler receives a domain command; a bad id never reaches it
async fn move_order(State(state): State<AppState>, signed: SignedIn, Parsed(cmd): Parsed<MoveOrder>) -> Response
```

## View models for hypermedia (E16)

```rust
// src/http/board/views.rs
pub struct OrderCard { dom_id: String, number: String, age: String, owner: Owner }

impl OrderCard {
    pub fn new(order: &OrderSummary, viewer: &SignedIn, now: Timestamp) -> Self { /* every display decision here */ }
}

fn order_card(card: &OrderCard) -> Markup {
    html! { li.card id=(card.dom_id) { (card.number) " " (card.age) } }
}
```

The view model's constructor is unit-tested with plain inputs; the markup
function is tested by anchors (T11).
