# P — Ports: every external capability is a contract

The domain says what it needs from the world in its own words. Which vendor
provides it is decided once, at bootstrap, and nowhere else.

## Rules

### P1 — Every external capability has a port
The domain defines an interface (port) for **every** capability the
application needs from the outside world — not just persistence:

| Capability | Port (domain-owned) | Adapter (vendor-named) |
|---|---|---|
| Persistence | `EntityRepository` | `SqlxEntityRepository` |
| Logging | `Logger` | `TracingLogger` |
| Config / env | `EnvRepository` | `LocalEnvRepository`, `KeyVaultEnvRepository` |
| Time | `Clock` | `SystemClock` |
| Randomness | `IdGenerator` | `UuidIdGenerator` |
| Outbound calls | `PaymentGateway`, `Mailer` | `StripeGateway`, `SmtpMailer` |
| Serving | our own server wrapper type | the framework it drives |

**Check:** list every vendor dependency; each is reached through a port or is
the transport framework wrapped per P6. Reading the clock or a random source
directly from business code fails.

### P2 — Ports speak domain language
Domain types in, domain types out, domain errors on failure. A port
signature that mentions a vendor type is a boundary violation.

**Check:** every type in every port signature is defined in the domain or is
a language primitive with no domain meaning (e.g. a count).

### P3 — Ports are crate-private
Wiring happens inside the crate at bootstrap; consumers reach capabilities
only through services.

**Check:** port interfaces are not part of the crate's public API.

### P4 — Vendor names stay in adapters and bootstrap
Vendor library names (database driver, logging library, HTTP client,
framework types, process-environment access) appear **only** in adapter files
and bootstrap. Never in domain, service, or edge business code.

**Check:** search for every vendor library name; every hit is in an adapter
file, bootstrap, or the edge's framework glue.

### P5 — Batch operations by design
Ports include batch operations where callers would otherwise loop
(`get_by_ids`) — the N+1 is designed out at the contract.

**Check:** no service calls a single-item port method inside a loop.

### P6 — The server is wrapped in our own type
The transport server is wrapped in our own type
(`HttpServer::new(...).run(router)`) with our own error type. Bootstrap hands
it a fully composed router; it runs whatever it is given.

**Check:** the entry point never touches the framework's server/listener
types directly.

## Pseudocode

```
# domain/types
port EntityRepository                         # crate-private (P3)
  fetch(id: EntityId) -> Option<Entity> | EntityError
  fetch_many(ids: List<EntityId>) -> List<Entity> | EntityError    # P5
  save(entity: Entity) -> () | EntityError

port Clock
  now() -> Timestamp                           # Timestamp is a domain type
```

Rust: [languages/rust/ports-and-adapters.md](../languages/rust/ports-and-adapters.md).
