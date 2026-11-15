# Rust: configuration (C)

## Catalog (C2, G1)

```rust
// src/env/domain/types.rs
use crate::catalog::catalog;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvSourceKind {
    Local,
    Vault,
}

catalog! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum EnvOption describes EnvOptionMeta {
        key: &'static str,
        source: EnvSourceKind,
    }
    {
        BindAddr { key: "ADDR", source: EnvSourceKind::Local },
        DatabaseUrl { key: "DATABASE_URL", source: EnvSourceKind::Local },
        ApiKey { key: "API_KEY", source: EnvSourceKind::Vault },
    }
}
```

Moving `ApiKey` from vault to local is a one-word change. Adding an option is
one line plus its parse in `Env::from_raw`.

Options parameterized by another closed set (one key per region) are
written as one variant per instance (`UserSourceNorthEurope`,
`UserSourceJapan`); the catalog stays a flat list of unit variants.

## Port and adapter (C1)

```rust
pub(crate) trait EnvRepository {
    async fn fetch(&self, key: &str) -> Result<Option<String>, EnvError>;
}

// src/env/adapters/local_env_repository.rs — the only file naming std::env
pub struct LocalEnvRepository;

impl EnvRepository for LocalEnvRepository {
    async fn fetch(&self, key: &str) -> Result<Option<String>, EnvError> {
        match std::env::var(key) {
            Ok(value) => Ok(Some(value)),
            Err(std::env::VarError::NotPresent) => Ok(None),
            Err(e) => Err(EnvError::Unreadable { key: key.to_string(), reason: e.to_string() }),
        }
    }
}
```

## Service routes, domain parses (C2, C3, V6)

```rust
pub struct EnvService {
    local: EnvRepo,
    vault: EnvRepo,
}

impl EnvService {
    pub async fn load(&self) -> Result<Env, EnvError> {
        let mut raw = RawEnv::default();
        for option in EnvOption::ALL {
            let repo = match option.source() {
                EnvSourceKind::Local => &self.local,
                EnvSourceKind::Vault => &self.vault,
            };
            raw.insert(*option, repo.fetch(option.key()).await?);
        }
        Env::from_raw(raw)
    }
}
```

## Startup only (B5)

```rust
// src/bootstrap.rs
pub async fn env() -> Result<Env, BootstrapError> {
    let service = EnvService::new(
        EnvRepo::local(LocalEnvRepository),
        EnvRepo::key_vault(KeyVaultEnvRepository::connect().await?),
    );
    Ok(service.load().await?)
}
```

`service` drops at the end of the function; only `Env` survives.

## Secrets (C5)

```rust
#[derive(Clone)]
pub struct ApiKey(String);

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiKey(***)")
    }
}

impl fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}
```

The error for a bad secret names the key, never the value:
`EnvError::InvalidValue { key: "API_KEY", value: "***", reason }`.
