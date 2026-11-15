# Rust: generated code (G)

Two declarative macros, zero dependencies, compiled and tested in
[kit/](kit): [`dispatch!`](kit/src/dispatch.rs) and
[`catalog!`](kit/src/catalog.rs). Copy each file into the target crate as
`src/dispatch.rs` / `src/catalog.rs` and declare them in `lib.rs`:

```rust
pub(crate) mod catalog;
pub(crate) mod dispatch;
```

The module paths matter: `dispatch!` recurses through
`$crate::dispatch::dispatch!`, so it must live at `crate::dispatch` (G6).

## `dispatch!` — dispatch wrappers (A5)

### Before: hand-written

Every port method repeats the same match, once per backend. Adding a backend
touches every method; adding a method means writing one more arm per backend.

```rust
pub enum EntityRepo {
    Sqlx(SqlxEntityRepository),
    InMemory(InMemoryEntityRepository),
}

impl EntityRepo {
    pub fn sqlx(inner: SqlxEntityRepository) -> Self {
        Self::Sqlx(inner)
    }

    pub fn in_memory(inner: InMemoryEntityRepository) -> Self {
        Self::InMemory(inner)
    }
}

impl EntityRepository for EntityRepo {
    async fn fetch(&self, id: &EntityId) -> Result<Option<Entity>, EntityError> {
        match self {
            Self::Sqlx(inner) => inner.fetch(id).await,
            Self::InMemory(inner) => inner.fetch(id).await,
        }
    }

    async fn save(&self, entity: &Entity) -> Result<(), EntityError> {
        match self {
            Self::Sqlx(inner) => inner.save(entity).await,
            Self::InMemory(inner) => inner.save(entity).await,
        }
    }
}
```

### After: declared

```rust
use crate::dispatch::dispatch;

dispatch! {
    #[derive(Debug)]
    pub enum EntityRepo: EntityRepository {
        sqlx => Sqlx(SqlxEntityRepository),
        in_memory => InMemory(InMemoryEntityRepository),
        #[cfg(test)]
        recording => Recording(RecordingEntityRepository),
    }
    async fn fetch(&self, id: &EntityId) -> Result<Option<Entity>, EntityError>;
    async fn save(&self, entity: &Entity) -> Result<(), EntityError>;
}
```

The expansion is exactly the hand-written version above, plus the
`#[cfg(test)]` variant carried onto its constructor and every match arm.

Read the invocation as: `constructor => Variant(AdapterType)`, then the
port's methods, copied from the trait.

### Why the methods are listed again

A declarative macro cannot read another item's definition, so the method
signatures are restated at the invocation. That repetition is checked, not
trusted (G5): the expansion is `impl EntityRepository for EntityRepo`, so

- a signature that drifts from the trait fails to compile, and
- a trait method missing from the list fails to compile
  (`missing fetch in implementation`).

### Supported shapes

| Shape | Supported |
|---|---|
| `async fn m(&self, a: A, b: &B) -> R;` | yes |
| `fn m(&self) -> R;` | yes |
| no return type | yes |
| attributes on variants (`#[cfg(test)]`) | yes — applied to variant, constructor, and arms |
| attributes on the enum (`#[derive(Debug)]`) | yes |
| `&mut self`, `self` | no — ports take `&self` |
| generic methods, `impl Trait` arguments | no — ports take concrete domain types (P2) |
| patterns in argument position (`(a, b): (A, B)`) | no |

If a port needs one of the unsupported shapes, question the port first.

### Test fakes are adapters (G7)

Do not do this:

```rust
pub enum LoggerRepo {
    Tracing(TracingLogger),
    #[cfg(test)]
    Fake(Arc<Mutex<Vec<String>>>),     // behavior lives in the wrapper's arms
}
```

Do this:

```rust
#[cfg(test)]
#[derive(Debug, Default, Clone)]
pub struct RecordingLogger(Arc<Mutex<Vec<String>>>);

#[cfg(test)]
impl Logger for RecordingLogger { /* push messages */ }

dispatch! {
    pub enum LoggerRepo: Logger {
        tracing => Tracing(TracingLogger),
        #[cfg(test)]
        recording => Recording(RecordingLogger),
    }
    fn info(&self, msg: &LogMessage);
    fn warn(&self, msg: &LogMessage);
    fn error(&self, msg: &LogMessage);
}
```

Constructors that do more than wrap (`LoggerRepo::tracing()` that also
installs the subscriber) move to bootstrap (B2), which calls
`TracingLogger::init_once()` and then `LoggerRepo::tracing(TracingLogger)`.

## `catalog!` — closed vocabularies with metadata (N8, C2)

### Before: hand-written

Three places to edit per variant — the enum, the `key()` match, the `all()`
list — and nothing stops them drifting apart.

```rust
pub enum EnvOption { BindAddr, DatabaseUrl }

impl EnvOption {
    pub fn key(&self) -> &'static str {
        match self {
            EnvOption::BindAddr => "ADDR",
            EnvOption::DatabaseUrl => "DATABASE_URL",
        }
    }

    pub fn all() -> &'static [EnvOption] {
        &[EnvOption::BindAddr, EnvOption::DatabaseUrl]
    }
}
```

### After: declared

```rust
use crate::catalog::catalog;

catalog! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum EnvOption describes EnvOptionMeta {
        key: &'static str,
        source: EnvSourceKind,
    }
    {
        BindAddr { key: "ADDR", source: EnvSourceKind::Local },
        DatabaseUrl { key: "DATABASE_URL", source: EnvSourceKind::Local },
    }
}
```

Generates:

- `enum EnvOption { BindAddr, DatabaseUrl }` with the given derives;
- `struct EnvOptionMeta { key, source }`;
- `EnvOption::ALL: &[EnvOption]` in declaration order;
- `const fn key(&self)` and `const fn source(&self)`, one per field.

Each variant's metadata is a struct literal, so a missing field is a compile
error (G5). Field types must be `Copy` (`&'static str`, enums, integers,
`bool`) — catalogs hold constants, not owned data.

Uniqueness of keys is not a type-level property; assert it once in the
catalog's tests:

```rust
#[test]
fn keys_are_unique() {
    let mut keys: Vec<_> = EnvOption::ALL.iter().map(EnvOption::key).collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), EnvOption::ALL.len());
}
```

## Why not the `enum_dispatch` crate

It generates the same shape, but it is a proc-macro dependency that needs
its attribute on the port trait itself (putting a generator concern in the
domain file), and its expansion is invisible at the call site (G3, G4). The
in-repo macro is one page, has no dependencies, and states every forwarded
method where the wrapper is declared.
