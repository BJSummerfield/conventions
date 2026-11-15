# A — Adapters

An adapter is a translator. It speaks the vendor's language on one side and
the port's on the other, and nothing else lives in it.

## Rules

### A1 — One file per (port, technology) pair
Named `<vendor>_<capability>`: `sqlx_entity_repository`, `tracing_logger`,
`local_env_repository`.

**Check:** each adapter file implements exactly one port for exactly one
technology, and its name says which.

### A2 — Each adapter names its own failures
Each adapter has its own error type (`SqliteEntityRepositoryError`) whose
variants name what went wrong in the adapter's terms — a unique index, a
trigger, a row that did not match, an invalid column — plus the vendor error
itself, kept typed. The adapter's internals return this type. The adapter builds only its own error: a fact its own reads discover (no row matched, a count says no) is an adapter variant too, and a domain rule it calls inside a transaction returns a decision the adapter maps, not a domain error. Every entity error therefore comes from the one conversion. It converts
into the entity error in exactly one place, in the adapter file, by an
exhaustive match (X9). The adapter error never appears in a port signature,
a service or the edge, and the entity error never names it: the conversion
is a match into domain-owned variants, not a transparent wrap.

**Check:** every adapter file defines its error type and one conversion into
the entity error; no port, service or edge file names an adapter error type. No adapter error variant holds an entity error type.

### A3 — Domain facts are recognised structurally
Vendor failures that are domain facts — a unique-key violation on insert, a
trigger refusing a change — are recognised from the vendor error's kind,
code or constraint name, once, when the vendor error becomes the adapter
error, and map to a named entity variant (`AlreadyExists(id)`), not the
catch-all. Searching the vendor's message text is never how a fact is
recognised; a trigger's own message, which the app defines, may be compared
in full.

**Check:** for each insert/update, constraint violations that the domain
cares about produce a named variant; no `to_string().contains(` on a vendor
error.

### A4 — Only opaque failures fall through
Only genuinely opaque failures (connection lost, driver bug) reach the
entity error's catch-all variant (X3).

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

### A8 — Rows cross through row types
A query result is read into a row type (`OrderRow`, `#[derive(FromRow)]` or
equivalent) and enters the domain through `TryFrom<OrderRow> for Order`,
which calls the domain constructors (A7) and turns a bad column into the
adapter error's invalid-row variant naming that column (A2). A row type read by several adapters of one
vendor lives in `<vendor>_<entity>_rows`, the one exception to A1; a row type
read by one adapter lives in that adapter's file. Single scalar reads (a
count, a returned id) need no row type.

**Check:** no tuple or positional column reads that build a domain type
outside a row conversion; each row type has exactly one conversion into the
domain.

## Pseudocode

```
# adapters/postgres_entity_repository
error PostgresEntityRepositoryError          # adapter vocabulary (A2)
  NameTaken                                  # unique index entities.name
  NoRow
  InvalidRow { column }                      # A8
  Database(PostgresError)                    # the vendor error, still typed

classify(e: PostgresError) -> PostgresEntityRepositoryError
  if e.kind is UniqueViolation: NameTaken    # by kind, never by message (A3)
  else: Database(e)

convert PostgresEntityRepositoryError -> EntityError   # exhaustive (X9)
  NameTaken             -> AlreadyExists
  NoRow                 -> NotFound
  InvalidRow { column } -> Repository("invalid row: " + column)    # A4
  Database(e)           -> Repository(e.message)

adapter PostgresEntityRepository implements EntityRepository
  constructed with: pool                     # injected (A6)

  save(entity) -> () | EntityError
    pool.insert("entities", row_from(entity)) or fail with classify(e)
    # the port returns EntityError: convert once, here

  fetch(id) -> Entity | EntityError
    row = pool.query_one(...) as EntityRow
    return Entity.try_from(row)              # A7, A8: InvalidRow on a bad column

# adapters/entity_repo   (generated, G1)
dispatch EntityRepo implements EntityRepository
  postgres  => Postgres(PostgresEntityRepository)
  in_memory => InMemory(InMemoryEntityRepository)
```

Rust: [languages/rust/ports-and-adapters.md](../languages/rust/ports-and-adapters.md).
