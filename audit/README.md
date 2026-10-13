# Auditing a repository

The procedure an agent follows when asked to check a codebase against these
conventions. It is built to be split: each rule family can be audited by a
separate agent, and the reports merge cleanly.

## Inputs

- **Target:** the repository (or subdirectory) to audit.
- **Scope:** which rule families (`P, A, X`) or individual rules (`E3`).
  No scope given means all families.
- **Language:** selects `languages/<lang>/checks.md`.
- **Output path:** where the report goes. If you cannot write files, the
  report is your final response.

## Deviations

Before auditing, read the target's `CONVENTIONS.md` if it has one. It may
declare deviations:

```markdown
## Deviations
- X3: the `sync` slice has two catch-alls (`Source`, `Repository`) because
  source and storage fail independently and the edge routes them differently.
```

A declared deviation is honored: report it under **Declared deviations**,
not as a finding. A deviation with no reason is itself a finding.

## Procedure

1. **Map the repo.** List slices, assign every source file one layer (S3),
   list vendor dependencies from the manifest. Write the map at the top of
   the report; every later step uses it.
2. **For each rule in scope:**
   1. Run the rule's mechanical check from `languages/<lang>/checks.md`.
      **Count the matches before reading them.** Never conclude something is
      absent from a truncated list.
   2. Read every hit in context. A grep hit is a lead, not a finding.
   3. Where the rule has no mechanical check, read the files the map says
      the rule governs.
3. **Verify each finding** before reporting it: re-read the code, try to
   argue it is compliant (a declared deviation, a test-only path, a rule
   exception). Drop it if the argument holds; keep it with the reason if it
   does not.
4. **Write the report** in the format below.

An audit reports. It does not fix, refactor, or open PRs unless asked.

## Report format

```markdown
# Conventions audit: <repo> @ <commit>

Scope: <families>   Language: <lang>   Conventions: <this repo> @ <commit>

## Map
<slices, file → layer table, vendor list>

## Findings

### <RULE> — <one-line summary>
- **Where:** `path/to/file.rs:42`
- **Evidence:**
  ```rust
  <the offending lines, quoted>
  ```
- **Why it violates:** <one or two sentences tied to the rule text>
- **Fix:** <the smallest change that complies>
- **Confidence:** confirmed | likely

## Clean
| Rule | Check run | Result |
|---|---|---|
| P4 | `rg -c 'sqlx::' src/` → 6 hits, all in adapters | clean |

## Declared deviations
| Rule | Where | Reason given |

## Not checked
<rules in scope you could not evaluate, and why>
```

Order findings by rule ID. One finding per violation site; a pattern
repeated across eight files is one finding listing eight locations.

## Splitting across agents

Families are independent enough to audit in parallel. A good split:

| Agent | Families |
|---|---|
| structure | S, M, G |
| domain | N, C |
| boundaries | P, A, V, B |
| edge & errors | E, X, O |
| tests | T |

Each agent writes its own report; a final pass merges them under one Map
and drops duplicates (the same site cited under two rules stays as one
finding citing both).
