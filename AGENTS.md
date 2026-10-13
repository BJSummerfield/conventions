# For agents

You were pointed at this repository as the source of truth for how code
should be shaped. Read only what your task needs.

## If you are implementing

1. Read the rule files for the families your change touches (`rules/`).
   A new slice touches S, N, P, A, V, E, X, T and M; a new adapter touches
   P, A, X and T.
2. Read the matching examples under `languages/<lang>/`. When a rule and an
   example disagree, the rule wins — report the disagreement.
3. Generators exist for dispatch wrappers and catalogs (G1). Use them; do not
   hand-write match-and-delegate code.

## If you are auditing

Follow [audit/README.md](audit/README.md) exactly. In short:

- Audit only the families you were given.
- Run each rule's mechanical check from `languages/<lang>/checks.md`, then
  read every hit. **Never truncate a search you draw a conclusion from** —
  count matches before calling something absent.
- Every finding cites a rule ID, a `file:line`, and quoted evidence.
- Honor deviations the target repo declares in its `CONVENTIONS.md`.
- Report clean rules as clean, with the check you ran.

## Ground rules

- Rule IDs are the vocabulary. Cite them; do not paraphrase them into new
  rules.
- These conventions describe the target. They do not authorize refactors you
  were not asked for — an audit reports, it does not fix.
