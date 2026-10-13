# G — Generated code

Some code is a pure function of a declaration: a dispatch wrapper is
"for each method, for each variant, delegate"; a catalog accessor is "for
each variant, return its metadata". Writing that by hand is where drift and
copy-paste bugs come from. Generate it, keep the declaration readable, and
keep everything that involves judgment hand-written.

## Rules

### G1 — Mechanical boilerplate is generated
Two patterns are always generated, never hand-written:

- **dispatch wrappers** (A5): the enum, one constructor per backend, and a
  match-and-delegate body for each port method;
- **catalogs** (N8, C2): the enum, the list of all variants, and one
  accessor per metadata field.

**Check:** no hand-written wrapper whose methods are only
`match self { Variant(inner) => inner.method(args) }`; no hand-written
parallel `match` mapping variants to keys, labels, or source kinds.

### G2 — Generators emit only mechanics
Delegation, enumeration, lookup tables. Validation, error mapping, defaults,
and business rules are never generated — they stay in hand-written
constructors (N3), adapters (A2), and services.

**Check:** read each generator; no branch inside it encodes a domain rule.

### G3 — In-repo, small, no new dependency
Prefer the language's built-in hygienic generation (Rust: declarative
`macro_rules!`) over code-generation dependencies or build-time codegen. A
generator is a page of code a reader can follow. A third-party generator is
acceptable only if it covers the case completely, including async ports,
without dynamic dispatch (X8).

### G4 — The invocation reads as a declaration
At the call site a reader sees every variant, every constructor name, every
adapter type, and every delegated method. The generator never hides which
adapters exist or which methods are forwarded.

### G5 — Generated code is checked against its contract
The generated wrapper implements the port interface, so a method declared at
the invocation that drifts from the port fails to compile, and a port method
missing from the invocation fails to compile. Catalog metadata is a struct
literal per variant, so a missing field fails to compile.

### G6 — One module per generator, named for what it generates
`dispatch`, `catalog` — crate-level, crate-private. Never a `macros` or
`utils` grab-bag (S5).

### G7 — Behavior goes in adapters, not in wrapper arms
When a backend needs behavior (a test fake that records calls), it becomes a
real adapter type implementing the port, and the wrapper delegates to it
like any other. A variant whose arm does work inline is a finding.
Test-only backends are variants with a test-only attribute; the generator
carries the attribute onto the variant, its constructor, and every arm.

## Pseudocode

```
generate dispatch EntityRepo implements EntityRepository
  postgres  => Postgres(PostgresEntityRepository)
  in_memory => InMemory(InMemoryEntityRepository)
  [test] recording => Recording(RecordingEntityRepository)     # G7
  methods:
    async fetch(id: EntityId) -> Option<Entity> | EntityError
    async save(entity: Entity) -> () | EntityError

# expands to:
enum EntityRepo { Postgres(...), InMemory(...), [test] Recording(...) }
EntityRepo.postgres(inner) / .in_memory(inner) / [test] .recording(inner)
EntityRepo implements EntityRepository:
  fetch(id): match self { Postgres(i) => i.fetch(id), InMemory(i) => i.fetch(id), ... }
  save(e):   match self { ... }
```

Rust: [languages/rust/generated-code.md](../languages/rust/generated-code.md),
tested implementations in [languages/rust/kit](../languages/rust/kit).
