# S — Structure: hexagonal layers

Code is grouped by the role it plays in the hexagon. A reader looking for "how
orders are stored" opens `adapters/` and finds the order repository next to
every other adapter; a reader looking for the rules of an order opens
`domain/order`. Dependencies point inward, and the compiler's module paths
make every crossing visible.

## Rules

### S1 — Organize by layer
Top-level source modules are the layers: `domain`, `ports`, `adapters`,
`services`, the driving edges (`http`, `runners`, a CLI), `bootstrap`, the
generators (G6) and the entry point.

**Check:** every top-level module under `src/` is one of the layers above. A
top-level module named for a business entity (`users`, `orders`) fails.

### S2 — Each layer has a fixed shape
Names are placeholders.

```
domain/<entity>                       newtypes, entities, commands, projections, errors
ports/<capability>                    one port trait per file
adapters/<port>_repo                  dispatch wrapper (G1)
adapters/<vendor>_<port>              concrete adapter, one per (port, technology)
adapters/<vendor>_<entity>_rows       row types shared by several adapters of one vendor (A8)
services/<capability>_service         orchestration
runners/<job>_runner                  scheduled or background driving edge
http/<area>/requests                  inbound wire shapes + conversion
http/<area>/responses                 edge errors, action responses
http/<area>/views                     view models + markup (hypermedia edges, E16)
http/<area>/routes                    routing + thin handlers
bootstrap                             wiring
```

A domain file that grows past comfortable reading splits by concept
(`domain/order`, `domain/draft`) under the same `domain` gate. An `http` area
is a screen or resource, not necessarily one entity.

**Check:** every file sits in the layer its role names; no adapter in
`domain/`, no wire shape outside the edge, no port outside `ports/`.

### S3 — One layer vocabulary
The layers are: **domain** (types, rules), **ports** (contracts), **adapter**
(vendor implementations of ports), **service** (orchestration), **edge**
(transport: HTTP, CLI, queue consumer, scheduled runner), **bootstrap**
(wiring), **dead zone** (the entry point).

**Check:** every source file can be assigned exactly one layer. A file that
needs two labels is doing two jobs.

### S4 — Dependencies point inward
`domain` depends on nothing in the crate but other domain modules. `ports`
depend on `domain`. `adapters` implement `ports` and use `domain`. `services`
depend on `ports`, `domain` and the dispatch wrappers (V5). Edges depend on
`services` and `domain`. Only `bootstrap` names concrete adapters.

**Check:** no `domain` file imports `ports`, `adapters`, `services` or an edge;
no `services` file imports a concrete adapter; no edge file imports a port,
a repository wrapper or a concrete adapter outside its tests, except the
logger wrapper every edge holds (O5, O6).

### S5 — Services group entities
A use case that spans entities — a cascade, a join, an aggregate projection —
is one service holding every port it needs. When the spanning write must be
atomic, it is one port method whose adapter runs one transaction; the
entities it touches belong together (V7).

**Check:** no edge composes two service calls into one business operation;
no service calls another service.

### S6 — No kitchen-sink modules
No `utils`, `helpers`, `common`, `models`, `types` (at crate root), or shared
`dto`. Everything belongs to a layer and a role. Code generators are the one
kind of crate-level module, one per generator, named for what they generate
(G6).

**Check:** list module names at every level; none of the names above appear.

### S7 — Edge areas do not share wire shapes
Two edge areas rendering the same domain value each own their view model or
body (E9). Duplication between areas is accepted; a shared DTO is not.

**Check:** no `requests`, `responses` or `views` file imports another area's
wire shapes or view models. Shared markup chrome (a page layout, a flash
banner) lives in its own edge module and takes view models, not domain
types.

## Pseudocode

```
src/
  main                          # dead zone
  bootstrap                     # wiring
  dispatch                      # generator (G6)
  domain/order                  # OrderId, Order, MoveOrder, OrderError
  domain/item
  ports/order_repository        # OrderRepository
  adapters/order_repo           # OrderRepo (dispatch)
  adapters/sqlite_order_repository
  adapters/sqlite_order_rows    # OrderRow, OrderLineRow (A8)
  services/fulfilment_service   # orders × items: cancel returns stock (S5)
  runners/session_cleanup_runner
  http/board/requests
  http/board/responses
  http/board/views
  http/board/routes
```
