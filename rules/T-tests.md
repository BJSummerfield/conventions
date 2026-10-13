# T — Tests

Every boundary is a contract, and every contract is tested from both sides.

## Rules

### T1 — Unit tests are colocated
At the bottom of every file, grouped by type or concern (Rust:
`#[cfg(test)] mod tests` with nested `mod`s).

### T2 — Real infrastructure over mocks where cheap
An in-memory database through the real adapter beats a mock. Ports make
substitution possible; that does not make mocks the default.

### T3 — Every boundary crossing is tested from both sides
- **domain:** constructors accept/reject, boundary values, display text of
  every error variant;
- **inbound:** conversion collects all field errors, cross-field routing,
  submitted values attached;
- **outbound:** exact wire shape asserted as JSON, status per type, error
  routing per variant, 500 bodies withhold detail, source chain preserved;
- **service:** happy paths, error conversion, cascades.

**Check:** for each item above, find the test. Wire shapes are asserted
exactly, not by partial field checks.

### T4 — Test builders are plain functions
In the test module, named for what they build (`valid_entity()`,
`request_with_bad_name()`).

### T5 — Wiring has a smoke test
Bootstrap composes the full application against in-memory infrastructure.

### T6 — Generators are tested once, against a fixture
A generator (G) has tests over a fixture port/catalog in its own module.
Each use site is then covered by the compiler (G5) and by the tests of the
code that calls it — use sites do not re-test delegation.
