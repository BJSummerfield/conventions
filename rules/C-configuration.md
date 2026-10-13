# C — Configuration and environment

Configuration is read once, validated once, and then exists only as a typed
value. Nothing reads the environment at request time.

## Rules

### C1 — Configuration is a full slice
An `EnvRepository` port (fetch one raw value by key), one adapter per source
(`LocalEnvRepository` for the process environment, `KeyVaultEnvRepository`,
…), a dispatch wrapper (`EnvRepo`), and an `EnvService` that owns the routing.
Process-environment access is vendor territory (P4) — named only in the local
adapter.

### C2 — The domain owns a catalog
One enum lists every option (`EnvOption`); each knows its lookup key and
which source kind owns it (one declaration, N8 / G1). Repositories return raw
strings; the domain parses and applies defaults (`Env::from_raw`); the
service only routes options to sources. Adding an option — or moving one from
env to vault — is a catalog change, not a plumbing change.

**Check:** every configurable value is a catalog variant; no key string
appears outside the catalog.

### C3 — The product is a validated value
The service's product (`Env`) has private fields, is built through domain
constructors, and its existence proves startup configuration is sound.
Invalid configuration fails fast at startup with a structured error naming
the key and value. No lazy env reads at request time.

### C4 — Defaults keep zero-setup working
The bare run command boots with no environment. Defaults live in the domain
so every source honors the same fallbacks.

### C5 — Secrets redact
Secrets are newtypes whose debug and display output redact. They are never
serialized, never logged, never in error messages.

**Check:** every credential-bearing type has a redacting debug/display; no
error variant carries a secret's value.

## Pseudocode

```
catalog EnvOption describes EnvOptionMeta { key: string, source: EnvSourceKind, secret: bool }
  BindAddr    { key: "ADDR",         source: Local, secret: false }
  DatabaseUrl { key: "DATABASE_URL", source: Local, secret: false }
  ApiKey      { key: "API_KEY",      source: Vault, secret: true  }

service EnvService                           # startup only (B5)
  holds: local: EnvRepo, vault: EnvRepo
  load() -> Env | EnvError
    raw = for option in EnvOption.ALL:
            repo = option.source() == Local ? local : vault
            raw[option] = repo.fetch(option.key())
    return Env.from_raw(raw)                 # domain parses + defaults (C2, C4)

type ApiKey (private value: string)
  display -> "ApiKey(***)"                   # C5
```

Rust: [languages/rust/configuration.md](../languages/rust/configuration.md).
