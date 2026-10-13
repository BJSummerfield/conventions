# A — Adapters

An adapter is a translator. It speaks the vendor's language on one side and
the port's on the other, and nothing else lives in it.

## Rules

### A1 — One file per (port, technology) pair
Named `<vendor>_<capability>`: `sqlx_entity_repository`, `tracing_logger`,
`local_env_repository`.

**Check:** each adapter file implements exactly one port for exactly one
technology, and its name says which.

### A2 — Vendor-error translation lives in the adapter
Converting a vendor error into a domain error lives in the adapter file — the
domain never learns the vendor exists.

**Check:** every conversion from a vendor error type is defined in an adapter
file.

### A3 — Domain facts are mapped at the call site
Vendor errors that are domain facts are mapped inline where context exists: a
unique-key violation on insert becomes `AlreadyExists(id)`, not the
catch-all.

**Check:** for each insert/update, constraint violations that the domain
cares about produce a named variant.

### A4 — Only opaque failures fall through
Only genuinely opaque failures (connection lost, driver bug) reach the
slice's catch-all variant (X3).

**Check:** read every path into the catch-all; each is something the domain
cannot act on.

### A5 — Dispatch over backends is an enum wrapper
One variant per adapter, a constructor per backend (`EntityRepo::sqlx(pool)`),
match-and-delegate for each port method. A new backend is a new variant, not
a refactor. The wrapper is generated (G1), not hand-written.

**Check:** services hold the wrapper type, never a concrete adapter and never
a dynamic/virtual interface object (X8). Wrapper bodies only delegate.

### A6 — Adapters take injected handles
Adapters take injected handles (pool, client, writer). Resource location and
lifetime — file vs memory, endpoint URLs, buffer sizes — are bootstrap
decisions, never hardcoded in the adapter.

**Check:** no URL, path, connection string, or size literal inside adapter
constructors.

### A7 — Rehydration goes through domain constructors
Adapters build entities the same way the edge does (N10).

**Check:** every row/document-to-entity mapping calls domain `new()`.

## Pseudocode

```
# adapters/postgres_entity_repository
adapter PostgresEntityRepository implements EntityRepository
  constructed with: pool                     # injected (A6)

  save(entity):
    result = pool.insert("entities", row_from(entity))
    on UniqueViolation: return EntityError.AlreadyExists(entity.id())    # A3
    on other failure e: return EntityError.Repository(e.message)        # A4

  fetch(id):
    row = pool.query_one(...)
    return Entity.new(EntityId.new(row.id)?, EntityName.new(row.name)?, ...)  # A7

translate PostgresError -> EntityError: EntityError.Repository(message)  # A2

# adapters/entity_repo   (generated, G1)
dispatch EntityRepo implements EntityRepository
  postgres  => Postgres(PostgresEntityRepository)
  in_memory => InMemory(InMemoryEntityRepository)
```

Rust: [languages/rust/ports-and-adapters.md](../languages/rust/ports-and-adapters.md).
