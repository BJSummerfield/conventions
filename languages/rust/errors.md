# Rust: errors (X)

## The ladder (X1)

| Hop | Lives in | Rust |
|---|---|---|
| vendor → adapter | adapter file | `Database(#[from] sqlx::Error)` plus one `classify(sqlx::Error, ..)` that recognises unique and trigger failures by kind or code |
| adapter → entity | adapter file | `impl From<SqliteEntityRepositoryError> for EntityError`, an exhaustive `match` (X9) |
| entity → service | service file | `#[error(transparent)] Entity(#[from] EntityError)` |
| service → action | `http/<entity>/responses.rs` | `impl From<EntityServiceError> for EntityUpdateError` |
| action → status | same | `impl IntoResponse for EntityUpdateError` |

## Service error (V4, X5)

```rust
#[derive(Debug, thiserror::Error)]
pub enum EntityServiceError {
    #[error(transparent)]
    Entity(#[from] EntityError),

    #[error(transparent)]
    Team(#[from] TeamError),

    #[error("team {0} not found")]
    TeamNotFound(TeamId),
}
```

## The three legal variant shapes (X5)

```rust
#[error(transparent)]                       // 1. transparent
Entity(#[from] EntityError),

#[error("failed to open database: {0}")]    // 2. message-inclusive
Database(String),

#[error("internal error: {0}")]             // 3. redaction point (edge only)
Internal(#[source] EntityServiceError),
```

Illegal — the cause is dropped from the message, so `to_string()` loses the
chain:

```rust
#[error("database failure")]
Database(#[source] sqlx::Error),            // also P4: vendor type outside an adapter
```

## Bootstrap error and `main` (B4, X8)

```rust
#[derive(Debug, thiserror::Error)]
pub enum BootstrapError {
    #[error(transparent)]
    Env(#[from] EnvError),

    #[error("failed to open database: {0}")]
    Database(String),

    #[error(transparent)]
    Server(#[from] ServerError),
}

// src/main.rs
#[tokio::main]
async fn main() -> Result<(), BootstrapError> {
    let logger = bootstrap::logger();
    let env = bootstrap::env().await?;
    let app = bootstrap::app(&env, logger).await?;
    bootstrap::serve(&env, app).await
}
```

Not `Box<dyn Error>`, not `anyhow::Result`.

## Anti-patterns

```rust
// X7: log-and-continue
if let Err(e) = repo.save(&entity).await {
    tracing::warn!("save failed: {e}");          // also O2
}

// X7: fabricated placeholder
let name = EntityName::new(raw).unwrap_or_else(|_| EntityName::new("unknown").unwrap());

// X2: pre-formatted string variant
#[error("{0}")]
Invalid(String),                                 // carry { field, max, actual } instead
```

## Adapter error (A2, A3, X9)

```rust
// src/adapters/sqlite_entity_repository.rs — private to the adapter
#[derive(Debug, thiserror::Error)]
enum SqliteEntityRepositoryError {
    #[error("entity {id} already exists")]
    NameTaken { id: EntityId },
    #[error("invalid row: {column}")]
    InvalidRow { column: &'static str },
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

impl SqliteEntityRepositoryError {
    // The call site passes what it was writing, so the fact can name it.
    fn classify(err: sqlx::Error, id: EntityId) -> Self {
        match err.as_database_error() {
            Some(db) if db.is_unique_violation() => Self::NameTaken { id },
            _ => Self::Database(err),
        }
    }
}

impl From<SqliteEntityRepositoryError> for EntityError {
    fn from(err: SqliteEntityRepositoryError) -> Self {
        match err {
            SqliteEntityRepositoryError::NameTaken { id } => EntityError::AlreadyExists(id),
            SqliteEntityRepositoryError::InvalidRow { .. }
            | SqliteEntityRepositoryError::Database(_) => EntityError::Repository(err.to_string()),
        }
    }
}
```

The port method converts once at its return (`.map_err(EntityError::from)`);
everything inside the adapter works in `SqliteEntityRepositoryError`. The
entity error cannot wrap the adapter error transparently, because the domain
never names an adapter type (P2, S4).
