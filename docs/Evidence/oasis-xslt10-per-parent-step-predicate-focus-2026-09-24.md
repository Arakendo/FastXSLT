# OASIS XSLT 1.0 Per-Parent Step Predicate Focus

- Date: 2026-09-24
- Status: Verified semantic correction and compatibility evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Unchanged case `Microsoft/Miscellaneous__84425#1` had executed but remained a
comparison mismatch after the bounded XSLT 1.0 expression
`//*[name()=$name]/*[position()<$p and name()=name(current())]` was admitted.
The earlier evidence attributed the extra selected children to a questionable
legacy expected result. Reinspection showed that conclusion was wrong: the
private evaluator applied `position()` over the merged document-order sequence
instead of the focus established separately by each `/*` step.

## Correction

XPath predicates filter each step's node sequence before results from distinct
parent contexts are combined and normalized. Execution now:

1. finds each parent matching the variable-held lexical name;
2. establishes element-child position independently for that parent;
3. evaluates the position and outer-`current()` name test in that local focus;
4. combines the selected children without changing their visible identity.

The traversal retains the existing cancellation and work-budget charges. No
general predicate evaluator, alternate XPath backend, or XSLT-version behavior
outside the existing typed compatibility plan was introduced.

The focused runtime oracle was corrected from `x;y;z;` to `x;y;`, proving that
the second matching parent's first element child uses local position `1` rather
than a position continued from the preceding parent.

## Corpus result

`Microsoft/Miscellaneous__84425#1` now compares exactly. The complete sweep
moves from:

- 2,076 to 2,077 exact expected-result matches; and
- 77 to 76 XML comparison mismatches.

Initialization remains 2,288 cases, successful execution remains 2,236 cases,
and initialization/execution failures remain 847 and 52. The exact
compatibility lower bound is therefore **2,077 / 3,173 (65.46%)**.

This record corrects the interpretation in
`oasis-xslt10-variable-name-sequence-predicates-2026-09-23.md`; the legacy
expected result was not the cause of the mismatch.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_variable_named_parent_children
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Miscellaneous__84425#1'
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
