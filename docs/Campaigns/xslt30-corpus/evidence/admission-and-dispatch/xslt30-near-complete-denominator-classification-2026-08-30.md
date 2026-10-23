# XSLT30 Near-Complete Denominator Classification

| Field | Value |
| --- | --- |
| Date | 2026-08-30 |
| Suite | W3C XSLT 3.0 test suite |
| Revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Test sets | `insn/apply-templates`, `decl/include` |
| Decision authority | ADR-0006, ADR-0007, AR-0008 |

## Result

The final default not-run cases in two nearly complete denominators received
explicit dispositions without changing immutable upstream data or weakening
the engine's XML/resource policy.

| Test set | Cases | Passed | Excluded by profile | Default not run |
| --- | ---: | ---: | ---: | ---: |
| `insn/apply-templates` | 50 | 49 | 1 | 0 |
| `decl/include` | 16 | 14 | 2 | 0 |

`conflict-resolution-1402` declares the native `schema_aware` feature and
depends on typed constructed attributes for the
`attribute(x, xs:integer)` pattern. ADR-0007 deliberately excludes
schema-awareness claims, so execution would not be representative of the case.

`include-0101` obtains declarations from an external DTD and general entity.
`include-0102` selects an embedded stylesheet through an `id` attribute typed
by an internal DTD declaration. AR-0008's current XML experiment denies DTD and
external-entity processing; the admitted embedded-module slice supports
`xml:id`, not DTD-derived typing. Both cases are therefore excluded from the
current profile rather than preprocessed by the harness or mislabeled as
engine failures.

## Executable conservation

Focused tests verify the exact upstream case identities and dependency bytes:

- the apply-templates inventory requires `schema_aware` on case 1402;
- the include inventory requires the external DTD declaration and `&child;`
  reference in case 0101's included module; and
- the include inventory requires case 0102's fragment reference, DTD `ID`
  declaration, ordinary `id`, and absence of `xml:id`.

Each first-party denominator overlay now has one explicit override per native
case. The classifications do not claim schema awareness, DTD support, external
entity authority, or execution success.
