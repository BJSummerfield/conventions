# N — Domain modeling: newtypes and entities

If a value exists, it is valid. Validation happens once, at construction, and
the type system carries the proof everywhere else. A function that takes an
`EmailAddress` never re-checks it.

## Rules

### N1 — Every meaningful primitive is a newtype
Every primitive with domain meaning is a newtype: `EntityId(String)`,
`Quantity(i32)`. Raw strings and integers never travel through domain,
service, or port signatures.

**Check:** domain, service, and port signatures contain no bare `string` /
`int` / `uuid` parameters or return types where the value has domain meaning.

### N2 — Private fields, getters, no setters
All fields on domain types are private. No setters. Access is through
getters. An instance existing proves it is valid.

**Check:** no public fields on types in `domain/`; no `set_*` methods.

### N3 — `new()` is the only door
Validation lives in the constructor and nowhere else; every other way in
(parsing, conversion traits, deserialization) delegates to `new()`.

**Check:** every conversion into a domain type calls its constructor. No
second copy of a validation rule exists anywhere.

### N4 — Fallible exactly when there is something to check
A value object validating a primitive returns a result. An entity composed of
already-valid parts returns itself — unless it enforces a cross-field
invariant, in which case it returns a result. No ceremonial results.

**Check:** a constructor whose error path is unreachable fails; a value
object constructor that cannot fail but wraps a constrained primitive fails.

### N5 — No implicit unwrapping
No implicit conversion or dereference to the inner type. Provide a display
form, and string access for string newtypes. Derive equality, copying,
debug output liberally where cheap.

**Check:** no language feature that makes the newtype silently behave as its
inner type (Rust: `Deref`, `Borrow`).

### N6 — Limits are named constants
Validation limits are named constants (`ENTITY_ID_MAX_LENGTH`), referenced
by both the constructor and its error variant.

**Check:** no numeric literal bounds inside constructors.

### N7 — Derived data is computed, never stored
If field B is a function of field A, B is a method, not a field — drift
becomes unrepresentable.

**Check:** no field whose value is assigned from another field of the same
type in the constructor.

### N8 — Closed vocabularies have one source of truth
Closed vocabularies are enums. When variants carry metadata (display name,
category, key, valid range), the enum and all accessors are generated from
one declaration, so adding a variant is one line. See G1.

**Check:** no hand-written parallel `match` arms mapping variants to
metadata; no separate hand-maintained `all()` list.

### N9 — Commands are validated types
Mutation commands (`UpdateEntity`, `CreateEntity`) are validated types built
via `new()`, enforcing the same cross-field rules as the entity.

**Check:** services receive commands, not loose argument lists or request
types.

### N10 — No bypass door
Data re-entering the domain — from the database, a queue, a cache — passes
through the same constructors as user input.

**Check:** adapters build domain types with `new()`; no `unchecked`,
`from_raw_unchecked` or struct-literal construction outside the type's own
module.

## Pseudocode

```
const ENTITY_NAME_MAX_LENGTH = 64

type EntityName (private value: string)
  new(raw: string) -> EntityName | EntityError
    trimmed = trim(raw)
    if trimmed is empty: return EntityError.EmptyName
    if length(trimmed) > ENTITY_NAME_MAX_LENGTH:
      return EntityError.NameTooLong { max: ENTITY_NAME_MAX_LENGTH, actual: length(trimmed) }
    return EntityName(trimmed)
  as_str() -> string

type Entity (private id: EntityId, private name: EntityName, private created: Timestamp)
  new(id, name, created) -> Entity            # parts already valid: infallible (N4)
  id() / name() / created()
  age(now: Timestamp) -> Duration             # derived, not stored (N7)

type Window (private start: Timestamp, private end: Timestamp)
  new(start, end) -> Window | WindowError     # cross-field invariant: fallible (N4)
    if end <= start: return WindowError.EndBeforeStart { start, end }
```

Rust: [languages/rust/domain-modeling.md](../languages/rust/domain-modeling.md).
