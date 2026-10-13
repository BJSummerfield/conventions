# Rust: errors (X)

## The ladder (X1)

| Hop | Lives in | Rust |
|---|---|---|
| vendor → slice | adapter file | `impl From<sqlx::Error> for EntityError`, or `.map_err` at the call site for domain facts |
| slice → service | service file | `#[error(transparent)] Entity(#[from] EntityError)` |
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
