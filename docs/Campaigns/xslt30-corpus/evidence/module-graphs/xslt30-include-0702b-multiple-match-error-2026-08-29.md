# XSLT30 `include-0702b` Multiple-Match Error

Date: 2026-08-29

## Inputs

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/decl/include/_include-test-set.xml`
- Case: `include-0702b`
- Principal: `include-0701.xsl`
- Secondary modules: `include-0701b.xsl` through `include-0701e.xsl`
- Native dependencies: `XSLT10 XSLT20`, `on-multiple-match=error`
- Native expected error pattern: `XTRE0540`

## Executable result

The harness admits the source and all five stylesheet modules into one sealed
snapshot, compiles the same graph used by the adjacent recover/default cases,
and selects the private invocation-local error policy. Inspection conserves
four lower-precedence and six principal-precedence template rules.

Execution reaches two applicable `title` rules at the highest import
precedence and equal priority. FastXSLT returns structured dynamic error
`XTDE0540`, retains request identity `include-0702b`, and supplies a stylesheet
source location. The concrete code satisfies the suite's `XTRE0540` pattern.

## Denominator effect

The complete 16-case include ledger is now 14 selected/passed and two explicit
not-run cases. The remaining cases, `include-0101` and `include-0102`, depend on
DTD behavior deliberately denied by the XML boundary.

## Claim boundary

This proves one exact sealed module graph under the private policy. It does not
select a general legacy compatibility profile, expose policy through the public
Rust or host adapters, admit DTD processing, or claim arbitrary module-graph
support.
