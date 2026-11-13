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

### T7 — A test names and checks one behaviour
The test name states the behaviour (`cancel_from_placed_returns_stock`), and
the test asserts on what a caller can observe: return values, stored state,
rendered output, published events. It does not assert on private helpers or
on the order of internal calls.

**Check:** no test whose name is a function name alone; no test that asserts
on a private item when a public path reaches the same behaviour.

### T8 — Each contract is tested once, at its own layer
A rule is tested where it lives (T3). An outer layer tests its own
conversion and routing, not the inner rule again. Two tests that would fail
for the same reason and only that reason are one test too many.

**Check:** for each domain rule, its tests sit in the domain file; edge tests
of that rule cover only the edge's mapping of it.

### T9 — Every test can fail
No test without an assertion, no assertion that always holds (a value
compared with itself, a constant against its own definition, a fact the type
system already guarantees), and no test made to pass by weakening what it
checks.

**Check:** mutate the code the test names (flip the condition, drop the
call); some test must fail.

### T10 — Tests are deterministic
Time comes from the clock port's test adapter, ids from the test id
generator. No bare sleeps; waiting is a bounded poll on the observable
condition.

**Check:** no real-clock reads, random ids or sleeps in tests outside the
adapters that wrap them.

### T11 — Exact where it is a contract, anchored elsewhere
Wire shapes and user-facing text are asserted exactly (T3). Rendered markup
is asserted by stable anchors — element ids, classes, text — not by copying
whole fragments, so a layout change breaks only the tests about layout.

**Check:** no test asserts a whole rendered page or a multi-element markup
string verbatim.
