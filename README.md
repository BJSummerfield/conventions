# Conventions

A ruleset for building services with **hexagonal architecture**, **hard
boundaries**, **newtype-driven domain modeling**, and **structured errors**,
written so an agent (or a human) can both *follow* it and *audit a codebase
against it*.

Built on three guides by howtocodeit:

- [Master Hexagonal Architecture in Rust](https://www.howtocodeit.com/guides/master-hexagonal-architecture-in-rust)
- [The Definitive Guide to Rust Error Handling](https://www.howtocodeit.com/guides/the-definitive-guide-to-rust-error-handling)
- [Ultimate Guide to Rust Newtypes](https://www.howtocodeit.com/guides/ultimate-guide-rust-newtypes)

**One deliberate deviation from the guides:** code is organized as **vertical
slices** (each business capability owns its full stack) rather than a
top-level `domain/` + `adapters/` split.

## Layout

```
rules/              the rules, language-agnostic, one file per family
languages/<lang>/   how each rule looks in that language, plus its checks
  rust/kit/         compiled, tested generators (dispatch!, catalog!)
audit/              the procedure and report format for auditing a repo
AGENTS.md           entry point for an agent pointed at this repo
```

| Family | File | Covers |
|---|---|---|
| S | [Structure](rules/S-structure.md) | vertical slices, layer vocabulary |
| N | [Domain modeling](rules/N-domain-modeling.md) | newtypes, entities, constructors |
| P | [Ports](rules/P-ports.md) | every external capability is a contract |
| A | [Adapters](rules/A-adapters.md) | vendor implementations, dispatch |
| V | [Services](rules/V-services.md) | orchestration |
| E | [Edge](rules/E-edge.md) | inbound/outbound crossings, per-action types |
| X | [Errors](rules/X-errors.md) | the error ladder |
| O | [Observability](rules/O-observability.md) | logging as a port, log once at the edge |
| C | [Configuration](rules/C-configuration.md) | config catalog, secrets |
| B | [Bootstrap](rules/B-bootstrap.md) | wiring, the dead zone |
| T | [Tests](rules/T-tests.md) | colocated, both sides of every boundary |
| G | [Generated code](rules/G-generated-code.md) | macros for dispatch and catalogs |
| M | [Naming](rules/M-naming.md) | names for every role |

Languages: [Rust](languages/rust/README.md).

## Citing rules

Rule IDs are stable. Audits and reviews cite them ("violates P4",
"`users/adapters/user_repo.rs` should use `dispatch!` per G1"). A rule is
never renumbered; a retired rule keeps its ID and is marked retired.

`Entity`, `Action`, `Vendor`, `Capability` in the rules are placeholders —
substitute your own names.

## Using it with an agent

Point the agent at this repository and name the scope:

> Audit `src/` against the conventions at
> `github.com/BJSummerfield/conventions`, families P, A and X. Follow
> `audit/README.md` and write the report it describes.

> Implement the `orders` slice following the conventions at
> `github.com/BJSummerfield/conventions` (Rust).

A repo can declare where it deliberately departs from a rule in a
`CONVENTIONS.md` at its root (see [audit/README.md](audit/README.md#deviations));
auditors honor declared deviations instead of reporting them.
