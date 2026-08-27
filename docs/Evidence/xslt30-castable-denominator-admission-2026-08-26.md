# XSLT30 `expr/castable` Denominator Admission

Date: 2026-08-26

## Candidate selection

The smallest remaining complete XSLT30 expression-family test sets were
reviewed at pinned suite revision
`6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`:

| Family | Cases | Initial-profile observation |
| --- | ---: | --- |
| `for` | 4 | already complete and passing |
| `treat-as` | 4 | test-set-wide `schema_aware` dependency |
| `type-expr` | 4 | test-set-wide `schema_aware` dependency |
| `castable` | 9 | mixed: seven non-schema-aware cases and two schema-aware cases |
| `path` | 10 | already complete and passing |

ADR-0007 deliberately excludes schema-awareness claims. Selecting either
four-case type family would therefore produce an entirely profile-excluded
denominator and no immediate engine pressure. `castable` is the smallest
remaining family with cases inside the initial staged-modern profile. Its two
schema-aware cases remain part of the discovered denominator as explicit
exclusions.

## Inputs and method

- Test set: `tests/expr/castable/_castable-test-set.xml`
- Cases: `castable-001` through `castable-009`
- Shared source environment for cases 001 through 004: `castbl01`
- Assertions: four `assert-xml`, two `assert`, and three compound
  `all-of/assert` results

The first-party overlay has exactly one record for every native case. A focused
admission test resolves case dependencies, referenced environment, stylesheet,
assertion root, file-backed expected results, and all native resource bytes. It
admits the nine stylesheets and four logically distinct source-resource uses
into a bounded, sealed in-memory snapshot after closing import handles.

The four engine-classified stylesheets are passed to the current compiler and
must produce structured `Unsupported` failures. The profile-excluded and
harness-unsupported cases are not opportunistically sent down an inapplicable
execution path.

## Conserved starting disposition

| Cases | Selection | Execution | Principal pressure |
| --- | --- | --- | --- |
| `castable-001` through `castable-004` | selected | engine unsupported | built-in atomic castability, descendant selection, variables, and explicit casts |
| `castable-005`, `castable-006` | excluded by profile | not run | schema-defined union/list types plus higher-order functions |
| `castable-007` through `castable-009` | selected | harness unsupported | standard initial-template entry and compound XPath assertions, before engine capability can be classified |

At this admission checkpoint, the denominator was nine discovered: seven
selected and two excluded by the accepted profile. Within the selected set,
zero passed, four were engine-unsupported, three were harness-unsupported, and
none failed or disappeared. Native `castable-001` subsequently advanced in
[the built-in atomic execution evidence](xslt30-castable-001-built-in-atomic-execution-2026-08-26.md).

## Claim boundary and next pressure

Admission establishes inventory, provenance, metadata classification, bounded
resource ownership, and diagnostic category for the four directly
engine-classified cases. It does not establish `castable as`, casting, an XDM
atomic type system, schema awareness, higher-order functions, standard
initial-template behavior, or general XPath assertion evaluation.

`castable-001` is the first implementation pressure, but it spans string,
boolean, integer, decimal, float, double, duration, date, and time families.
Implementation should first name an owned atomic-value and lexical-validation
boundary rather than embed unrelated validators directly in stylesheet
execution.

## Subsequent disposition

Cases `castable-001` through `castable-004` now pass. Cases `castable-007`
through `castable-009` have also advanced from harness-unsupported to
engine-unsupported through
[standard initial-template classification evidence](xslt30-standard-initial-template-classification-2026-08-26.md).
The selected denominator is therefore four passes and three explicit engine
gaps, with no harness gaps; the two schema-aware cases remain visible profile
exclusions.
