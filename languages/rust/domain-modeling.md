# Rust: domain modeling (N)

## Value object (N1–N6)

```rust
pub const ENTITY_NAME_MAX_LENGTH: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityName(String);

impl EntityName {
    pub fn new(raw: &str) -> Result<Self, EntityError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(EntityError::EmptyName);
        }
        let actual = trimmed.chars().count();
        if actual > ENTITY_NAME_MAX_LENGTH {
            return Err(EntityError::NameTooLong { max: ENTITY_NAME_MAX_LENGTH, actual });
        }
        Ok(Self(trimmed.to_string()))
    }
}

impl AsRef<str> for EntityName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EntityName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for EntityName {
    type Err = EntityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}
```

A vendor type may live *inside* a newtype; the newtype is what crosses
signatures:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(uuid::Uuid);

impl EntityId {
    pub fn new(raw: &str) -> Result<Self, EntityError> {
        uuid::Uuid::parse_str(raw)
            .map(Self)
            .map_err(|_| EntityError::InvalidId { input: raw.to_string() })
    }
}
```

## Entity (N2, N4, N7)

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    id: EntityId,
    name: EntityName,
    created: Timestamp,
}

impl Entity {
    pub fn new(id: EntityId, name: EntityName, created: Timestamp) -> Self {
        Self { id, name, created }
    }

    pub fn id(&self) -> EntityId {
        self.id
    }

    pub fn name(&self) -> &EntityName {
        &self.name
    }

    pub fn age(&self, now: Timestamp) -> Duration {
        now.since(self.created)
    }
}
```

## Cross-field invariant: fallible entity / command (N4, N9)

```rust
pub struct UpdateWindow {
    start: Timestamp,
    end: Timestamp,
}

impl UpdateWindow {
    pub fn new(start: Timestamp, end: Timestamp) -> Result<Self, WindowError> {
        if end <= start {
            return Err(WindowError::EndBeforeStart { start, end });
        }
        Ok(Self { start, end })
    }
}
```

## Errors with their limits (N6, X2)

```rust
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum EntityError {
    #[error("name must not be empty")]
    EmptyName,

    #[error("name is {actual} characters; the limit is {max}")]
    NameTooLong { max: usize, actual: usize },

    #[error("invalid entity id '{input}'")]
    InvalidId { input: String },

    #[error("entity {0} already exists")]
    AlreadyExists(EntityId),

    #[error("repository: {0}")]
    Repository(String),
}
```

## Tests (T1, T3, T4)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod entity_name {
        use super::*;

        #[test]
        fn accepts_the_maximum_length() {
            let raw = "a".repeat(ENTITY_NAME_MAX_LENGTH);
            assert!(EntityName::new(&raw).is_ok());
        }

        #[test]
        fn rejects_one_past_the_maximum() {
            let raw = "a".repeat(ENTITY_NAME_MAX_LENGTH + 1);
            assert_eq!(
                EntityName::new(&raw),
                Err(EntityError::NameTooLong { max: ENTITY_NAME_MAX_LENGTH, actual: 65 })
            );
        }

        #[test]
        fn too_long_message_names_both_numbers() {
            let err = EntityError::NameTooLong { max: 64, actual: 65 };
            assert_eq!(err.to_string(), "name is 65 characters; the limit is 64");
        }
    }
}
```
