# S — Structure: vertical slices

Code is grouped by what it does for the business, not by what kind of code
it is. A reader looking for "how users are stored" opens the `users` slice and
finds the domain, the ports, and every adapter in one place.

## Rules

### S1 — Organize by capability, not by layer
Code is organized by business capability (slice), not by technical layer. A
slice owns its domain, its adapters, and its edge shapes.

**Check:** top-level source directories are capability names (`users`,
`orders`, `env`, `logging`), plus `http` (or another transport) and
bootstrap. A top-level `domain/`, `adapters/`, `models/`, `services/` or
`repositories/` directory fails.

### S2 — A slice has a fixed shape
An entity slice has this shape (names are placeholders):

```
<slice>                              gate: re-exports the public surface
<slice>/domain                       re-exports; ports stay crate-private
<slice>/domain/types                 newtypes, entities, errors, ports, tests
<slice>/adapters
<slice>/adapters/<entity>_repo                    dispatch wrapper
<slice>/adapters/<vendor>_<entity>_repository     concrete adapter
<slice>/<capability>_service                      orchestration (when there is any)
http/<entity>/requests               inbound wire shapes + conversion
http/<entity>/responses              outbound wire shapes + edge errors
http/<entity>/routes                 routing + thin handlers
```

A domain file that grows past comfortable reading may split by concept
(`domain/password`, `domain/totp`) under the same `domain` gate.

**Check:** for each slice, the files exist in these roles; no adapter lives
in `domain/`, no wire shape lives outside `http/`.

### S3 — One layer vocabulary
The layers are: **domain** (types, rules, ports), **adapter** (vendor
implementations of ports), **service** (orchestration), **edge** (transport:
HTTP, CLI, queue consumer, cron runner), **bootstrap** (wiring), **dead
zone** (the entry point).

**Check:** every source file can be assigned exactly one layer. A file that
needs two labels is doing two jobs.

### S4 — Cross-entity orchestration is its own slice
Joins, cascades, and aggregate projections across entities live in their own
slice with their own domain, errors, and adapters — not smeared across the
entity slices they compose.

**Check:** an entity slice's service does not import another entity slice's
repository. Composition across slices happens in a slice that exists for it.

### S5 — No kitchen-sink modules
No `utils`, `helpers`, `common`, `models`, `types` (at crate root), or shared
`dto`. Everything belongs to a slice or to bootstrap. Code generators are the
one kind of crate-level module, one per generator, named for what they
generate (G6).

**Check:** list module names at every level; none of the names above appear.

### S6 — Duplicate between slices rather than couple them
Duplication between slices is accepted; coupling between slices is not. Two
slices rendering the same shape each own their copy.

**Check:** slices import each other only through the gate's public surface,
and only for domain types (a value object, an entity) — never adapters,
never edge shapes.

## Pseudocode

```
src/
  main                      # dead zone
  bootstrap                 # wiring
  dispatch                  # generator (G6)
  orders                    # gate
  orders/domain/types       # OrderId, Order, OrderError, OrderRepository
  orders/adapters/order_repo
  orders/adapters/postgres_order_repository
  orders/adapters/in_memory_order_repository
  orders/order_service
  fulfillment/...           # S4: orders × inventory composition
  http/orders/requests
  http/orders/responses
  http/orders/routes
```
