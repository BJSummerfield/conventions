# B — Bootstrap and the dead zone

All wiring happens in one testable place. The entry point only calls it.

## Rules

### B1 — The entry point is the smallest possible dead zone
The bare startup sequence (observability init → settings → resources →
wiring → serve) and nothing else. If it contains logic, the logic is in the
wrong file.

**Check:** the entry point is a linear sequence with no branches beyond
error propagation.

### B2 — All wiring lives in bootstrap
Adapter construction, service assembly, route composition, middleware — all
in `bootstrap`, exercisable by tests.

### B3 — Bootstrap is the only place concrete adapters are chosen
(V5, A6.)

**Check:** concrete adapter constructors are called only from bootstrap and
tests.

### B4 — Startup failures are their own type
`BootstrapError`, distinct from runtime errors: "the process cannot come up"
is a different fact from "a request failed". It is also the entry point's
concrete exit type (X8).

### B5 — Startup-only services are dropped after startup
`EnvService` and its repositories are constructed inside a bootstrap
function, drained, and dropped — they never enter runtime state. Only their
validated product (`Env`) outlives startup.

## Pseudocode

```
main() -> () | BootstrapError                # dead zone (B1)
  logger = bootstrap.logger()
  env    = bootstrap.env()?                  # EnvService built + dropped inside (B5)
  app    = bootstrap.app(env, logger)?
  bootstrap.serve(env, app)?

bootstrap.app(env, logger) -> Router | BootstrapError
  pool     = Database.connect(env.database_url())?
  entities = EntityRepo.postgres(PostgresEntityRepository.new(pool))   # B3
  service  = EntityService.new(entities)
  return routes(service).with_failure_logging(logger)
```
