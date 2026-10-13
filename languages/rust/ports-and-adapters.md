# Rust: ports and adapters (P, A)

## Port (P1–P3, P5)

```rust
// src/entities/domain/types.rs
pub(crate) trait EntityRepository {
    async fn fetch(&self, id: &EntityId) -> Result<Option<Entity>, EntityError>;
    async fn fetch_many(&self, ids: &[EntityId]) -> Result<Vec<Entity>, EntityError>;
    async fn save(&self, entity: &Entity) -> Result<(), EntityError>;
}

pub(crate) trait Clock {
    fn now(&self) -> Timestamp;
}
```

Native `async fn` in a `pub(crate)` trait: no `async_trait`, no boxing, no
`dyn` (X8). The `async_fn_in_trait` lint only fires on `pub` traits, which
ports never are (P3).

## Adapter (A1–A4, A6, A7)

```rust
// src/entities/adapters/sqlx_entity_repository.rs
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct SqlxEntityRepository {
    pool: SqlitePool,
}

impl SqlxEntityRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl EntityRepository for SqlxEntityRepository {
    async fn save(&self, entity: &Entity) -> Result<(), EntityError> {
        sqlx::query("INSERT INTO entities (id, name) VALUES (?, ?)")
            .bind(entity.id().to_string())
            .bind(entity.name().as_ref())
            .execute(&self.pool)
            .await
            .map_err(|e| match &e {
                sqlx::Error::Database(db) if db.is_unique_violation() => {
                    EntityError::AlreadyExists(entity.id())
                }
                _ => e.into(),
            })?;
        Ok(())
    }

    async fn fetch(&self, id: &EntityId) -> Result<Option<Entity>, EntityError> {
        let row: Option<(String, String, i64)> =
            sqlx::query_as("SELECT id, name, created FROM entities WHERE id = ?")
                .bind(id.to_string())
                .fetch_optional(&self.pool)
                .await?;
        row.map(|(id, name, created)| {
            Ok(Entity::new(
                EntityId::new(&id)?,
                EntityName::new(&name)?,
                Timestamp::from_unix_seconds(created)?,
            ))
        })
        .transpose()
    }

    // fetch_many …
}

impl From<sqlx::Error> for EntityError {
    fn from(e: sqlx::Error) -> Self {
        EntityError::Repository(e.to_string())
    }
}
```

- The unique violation is a domain fact, mapped at the call site (A3).
- Everything else falls through `From` to the catch-all (A2, A4).
- Rows become entities through the same constructors the edge uses (A7).
- The pool is injected; the adapter never knows if it is a file or memory
  (A6).

## Dispatch wrapper (A5, G1)

```rust
// src/entities/adapters/entity_repo.rs
use crate::dispatch::dispatch;

dispatch! {
    #[derive(Debug)]
    pub enum EntityRepo: EntityRepository {
        sqlx => Sqlx(SqlxEntityRepository),
        in_memory => InMemory(InMemoryEntityRepository),
    }
    async fn fetch(&self, id: &EntityId) -> Result<Option<Entity>, EntityError>;
    async fn fetch_many(&self, ids: &[EntityId]) -> Result<Vec<Entity>, EntityError>;
    async fn save(&self, entity: &Entity) -> Result<(), EntityError>;
}
```

See [generated-code.md](generated-code.md) for what this expands to.

## Service holds the wrapper (V5)

```rust
pub struct EntityService {
    entities: EntityRepo,
}
```
