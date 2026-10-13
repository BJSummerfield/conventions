# Rust checks

Mechanical checks per rule. Each produces **leads**; read every hit before
calling it a finding (audit/README.md). Commands use `rg` (ripgrep); run from
the repo root. **Count first** (`rg -c`), then read — never decide from a
truncated list.

Paths assume the S2 layout. Adjust slice names to the repo.

## S — Structure

```sh
# S1/S5: layer-named or kitchen-sink modules (expect: none)
fd -t f -t d '^(domain|adapters|models|services|repositories|utils|helpers|common|dto|types|macros)(\.rs)?$' src --max-depth 1

# S6: cross-slice imports of adapters or edge shapes (expect: none)
rg -n 'use crate::\w+::adapters' src --glob '!**/bootstrap.rs' --glob '!src/*/adapters/**'
```

## N — Domain modeling

```sh
# N2: public fields in domain (expect: none)
rg -n '^\s*pub(\(crate\))? [a-z_]+\s*:' src/*/domain

# N2: setters (expect: none)
rg -n 'pub fn set_' src/*/domain

# N5: Deref/Borrow on newtypes (expect: none)
rg -n 'impl\s.*\b(Deref|DerefMut|Borrow)\b' src

# N6: literal bounds in constructors (read hits)
rg -n '(len\(\)|chars\(\)\.count\(\))\s*[<>]=?\s*[0-9]' src/*/domain

# N8/G1: hand-written catalog lookups (read hits: a match from variant to &'static str)
rg -n -A3 'fn \w+\(&self\) -> &.static str' src/*/domain

# N10: struct-literal construction of domain types outside their module (read hits)
rg -n 'unchecked|from_raw_unchecked' src
```

## P — Ports

```sh
# P3: public ports (expect: none)
rg -n '^\s*pub trait \w+(Repository|Logger|Clock|Gateway|Mailer|Sender)\b' src

# P1/P4: vendor names outside adapters/bootstrap/edge (expect: none)
rg -n '\b(sqlx|diesel|mongodb|reqwest|tracing|tracing_subscriber|dotenvy)::|std::env' src \
  --glob '!src/*/adapters/**' --glob '!src/bootstrap.rs' --glob '!src/main.rs'

# P1: direct time/randomness in business code (expect: none)
rg -n 'Utc::now|SystemTime::now|Instant::now|rand::|Uuid::new_v4' src \
  --glob '!src/*/adapters/**' --glob '!src/bootstrap.rs'

# P2: vendor types in port signatures (read the trait bodies)
rg -n -A30 'pub\(crate\) trait \w+' src/*/domain | rg 'chrono::|DateTime<|uuid::|sqlx::|serde_json::'

# P6: framework server types outside the server wrapper (expect: only src/http/server.rs and bootstrap)
rg -ln 'axum::serve|TcpListener|HttpServer::new' src
```

## A — Adapters

```sh
# A2: vendor-error conversions outside adapters (expect: none)
rg -n 'impl From<(sqlx|reqwest|mongodb|serde_json|std::io)::' src --glob '!src/*/adapters/**'

# A5/X8: dynamic dispatch over ports (expect: none)
rg -n 'dyn \w+(Repository|Logger|Clock|Gateway)|Arc<dyn|Box<dyn' src

# A6: hardcoded locations in adapters (read hits)
rg -n '"(https?://|sqlite:|postgres://|mongodb://|/)' src/*/adapters
```

## V — Services

```sh
# V1: edge importing ports or adapters (expect: none)
rg -n 'use crate::\w+::(adapters|domain::types::\w+Repository)' src/http

# V5: services holding concrete adapters (expect: none)
rg -n '(Sqlx|Sqlite|Postgres|Mongo|InMemory|Cosmos)\w*Repository' src/*/*_service.rs
```

## E — Edge

```sh
# E1: serde outside the edge (expect: none)
rg -n 'serde' src --glob '!src/http/**'

# E2: request types missing deny_unknown_fields (compare counts)
rg -c 'derive\(.*Deserialize' src/http
rg -c 'deny_unknown_fields' src/http

# E2: non-Option request fields (read hits inside Deserialize structs)
rg -n -A20 'derive\(.*Deserialize' src/http | rg '^\S+-\s+pub \w+: (?!Option<)'  --pcre2

# E7: domain types implementing IntoResponse (expect: none)
rg -n 'impl IntoResponse for' src --glob '!src/http/**'

# E14: 500 bodies carrying detail (read hits)
rg -n -B2 -A6 'INTERNAL_SERVER_ERROR' src/http
```

## X — Errors

```sh
# X6: anyhow/eyre (expect: none outside main.rs, ideally none at all)
rg -n 'anyhow|eyre' src Cargo.toml

# X2/X3: string-carrying variants (read hits; one catch-all per slice is allowed)
rg -n '^\s*\w+\(String\),?$' src

# X5: variants whose message drops the cause (read hits)
rg -n -A1 '#\[error\("[^{]*"\)\]' src | rg '\(\w+\)|\{ *\w+'

# X7: log-and-continue / placeholder values (read hits)
rg -n 'unwrap_or_default\(\)|unwrap_or\(""|\.ok\(\);' src --glob '!src/http/**'

# X8: dyn anywhere (expect: none)
rg -n '\bdyn ' src
rg -n 'fn main\(\).*Box<' src/main.rs
```

## O — Observability

```sh
# O2: logging below the edge (expect: only the logging adapter and bootstrap)
rg -ln 'println!|eprintln!|dbg!|tracing::|log::(info|warn|error|debug)' src
```

## C — Configuration

```sh
# C1: env reads outside the local env adapter (expect: one file)
rg -ln 'std::env|env::var|dotenvy' src

# C2: env key strings outside the catalog (read hits)
rg -n '"[A-Z][A-Z0-9]*_[A-Z0-9_]+"' src --glob '!src/env/**'

# C5: secret-ish types with derived Debug (read hits)
rg -n -B2 'struct \w*(Secret|Key|Token|Password)\w*' src | rg 'derive\(.*Debug'
```

## B — Bootstrap

```sh
# B1: main.rs logic (read the whole file; expect a linear sequence)
wc -l src/main.rs
rg -n '\b(if|match|for|while|loop)\b' src/main.rs

# B3: concrete adapter constructors outside bootstrap (read hits)
rg -n '\w+Repository::new\(' src --glob '!src/bootstrap.rs' --glob '!src/*/adapters/**'
```

## T — Tests

```sh
# T1: files without colocated tests (read the list; domain/adapters/edge should not appear)
rg --files-without-match '#\[cfg\(test\)\]' -g '*.rs' src

# T3: tests that assert a 500 (expect: > 0; then read them for an exact-body assertion)
rg -n -A12 '#\[(tokio::)?test\]' src/http | rg -c 'INTERNAL_SERVER_ERROR'

# T5: bootstrap smoke test exists
rg -n '#\[cfg\(test\)\]' src/bootstrap.rs
```

## G — Generated code

```sh
# G1: hand-written dispatch wrappers (expect: none; every *_repo.rs uses dispatch!)
rg --files-without-match 'dispatch!' src/*/adapters/*_repo.rs
rg -c 'Self::\w+\((inner|repo|r)\) => (inner|repo|r)\.' src

# G6: generator modules named for the pattern
fd -t f '^(dispatch|catalog)\.rs$' src
fd -t f '^(macros|utils)\.rs$' src           # expect: none

# G7: wrapper arms doing work inline (read hits)
rg -n -A3 '=> \{' src/*/adapters/*_repo.rs
```
