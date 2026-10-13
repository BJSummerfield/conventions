# Rust

How the rules look in Rust. Rule files say *what*; these files say *how*.

| File | Rules |
|---|---|
| [domain-modeling.md](domain-modeling.md) | N |
| [ports-and-adapters.md](ports-and-adapters.md) | P, A, V5 |
| [edge.md](edge.md) | E |
| [errors.md](errors.md) | X, B4 |
| [configuration.md](configuration.md) | C, B5 |
| [generated-code.md](generated-code.md) | G |
| [checks.md](checks.md) | audit commands, every family |
| [kit/](kit) | `dispatch!` and `catalog!`, compiled and tested |

## Baseline

- Edition 2024. Ports use native `async fn` in traits — no `async_trait`,
  no `Box<dyn …>`, no `Pin<Box<dyn Future>>` (X8).
- Errors: `thiserror` everywhere. No `anyhow` (X6).
- Serialization: `serde` only under `src/http/` (E1).
- Slice gates are `src/<slice>.rs` files, not `mod.rs`.

## Rule-to-idiom map

| Rule | Rust |
|---|---|
| N2 | private fields, `pub fn field(&self) -> &T` |
| N3 | `TryFrom<&str>`, `FromStr` call `Self::new` |
| N5 | no `Deref`/`Borrow`; `Display`, `AsRef<str>` |
| N6 | `pub const ENTITY_NAME_MAX_LENGTH: usize = 64;` |
| P3 | `pub(crate) trait EntityRepository` |
| A2 | `impl From<sqlx::Error> for EntityError` in the adapter file |
| V4 | `#[error(transparent)] Entity(#[from] EntityError)` |
| E2 | `#[derive(Deserialize)] #[serde(deny_unknown_fields)]`, all `Option<String>` |
| E3 | `impl TryFrom<&CreateEntityRequest> for CreateEntity` |
| E7 | `impl IntoResponse for EntityCreatedResponse` |
| X8 | `fn main() -> Result<(), BootstrapError>` |
| C5 | hand-written `impl Debug` / `Display` printing `***` |
| T1 | `#[cfg(test)] mod tests { mod entity_name { … } }` |
| G1 | `dispatch!` / `catalog!` from `src/dispatch.rs`, `src/catalog.rs` |

## Vendor crates

Treat these as vendor names for P4 / O1 / C1 (extend per repo from
`Cargo.toml`): `sqlx`, `diesel`, `mongodb`, `reqwest`, `hyper`, `axum`,
`actix_web`, `tower`, `tower_http`, `tracing`, `tracing_subscriber`, `log`,
`env_logger`, `serde`, `serde_json`, `chrono`, `time`, `uuid`, `rand`,
`dotenvy`, `std::env`, `std::time::SystemTime`, `tokio::time`, any cloud SDK.

`chrono`/`uuid` inside a newtype's private field are acceptable — the newtype
is the domain type. They become findings when they appear in a port
signature, a service signature, or an error variant (P2).
