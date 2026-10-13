# X — Errors: how they move

Errors are data. Each boundary has one closed set of things that can go
wrong, and each hop between boundaries is one explicit conversion.

## Rules

### X1 — The ladder: one conversion per hop, on the outer side
```
vendor error
  → SliceError        in the adapter file (A2), or inline where the
                      failure is a domain fact (A3)
  → ServiceError      transparent wrapping, automatic conversion (V4)
  → ActionError       conversion from ServiceError, per action, in the edge (E12)
  → status + body     the response mapping, per variant (E14)
```

**Check:** every arrow above is one conversion definition, defined on the
outer side of its boundary. No layer's error type appears two layers up.

### X2 — Structured variants, never pre-formatted strings
All error enums are derived error types with structured variants carrying
the data to explain themselves (`{ max, actual }`).

**Check:** no variant carries a single message string except the catch-all
(X3).

### X3 — Exactly one catch-all per slice
One adapter-agnostic catch-all variant (`Repository(String)` /
`Database(String)`) for opaque infrastructure failures. Everything else is a
named domain fact.

### X4 — Domain errors never reach clients
Public errors are separate types (E11), and internal detail stops at the
edge (E14).

### X5 — The chain survives by message discipline
Every variant is one of exactly three things:

- **transparent** — delegates its message wholesale to the wrapped error;
- **message-inclusive** — renders its cause into its own message
  (`"failed to create pool: {cause}"`);
- **the redaction point** — the edge's `Internal`, where the full chain is
  captured for the log before a redacted message goes to the client.

Vendor errors were already stringified at the adapter (A2), so a boundary
error's display text IS its full chain: one to-string call, statically
dispatched. Nothing walks a source chain at runtime.

**Check:** no variant whose message drops its cause.

### X6 — No catch-all error libraries in domain or services
Structured enums everywhere (Rust: no `anyhow`). The domain defines errors
and knows nothing about who consumes them.

### X7 — Errors are the communication mechanism
Code never logs-and-continues in place of returning an error, and never
fabricates placeholder values to satisfy a signature.

**Check:** no default/empty value returned on an error path; no log call
followed by continuing as if the operation succeeded.

### X8 — No dynamic dispatch
Error sets are closed enums; each boundary names its concrete carrier type,
and dispatch is `match`. No "any error" type in any signature, including the
entry point's exit type (a bootstrap error enum). A signature that wants to
accept "any error" actually wants *this boundary's* error — name it.

**Check:** no dynamic/virtual-dispatch error or interface types in any
signature.

## Pseudocode

```
error EntityError                          # slice (domain)
  EmptyName
  NameTooLong { max, actual }
  AlreadyExists(EntityId)
  Repository(string)                       # catch-all (X3)

error EntityServiceError                   # service
  Entity(EntityError)                      transparent (V4)
  TeamNotFound(TeamId)

error UpdateEntityError                    # edge, per action (E11)
  Validation(EntityValidationErrors)
  NotFound { resource: "entity", id }
  Internal(source: EntityServiceError)     redaction point (X5)

convert EntityServiceError -> UpdateEntityError    # E12
  Entity(NotFound(id))  -> NotFound { "entity", id }
  TeamNotFound(id)      -> Validation(team_id_error = "unknown team")
  other                 -> Internal(other)
```

Rust: [languages/rust/errors.md](../languages/rust/errors.md).
