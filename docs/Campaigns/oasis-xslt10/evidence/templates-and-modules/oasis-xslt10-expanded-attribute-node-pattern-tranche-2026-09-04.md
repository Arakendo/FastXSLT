# OASIS XSLT 1.0 Expanded Attribute-Node Pattern Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 734 definite unchanged XML passes; 1,016 initialized cases |
| Result | 737 definite unchanged XML passes; 1,019 initialized cases |
| Disposition | Shared template-pattern canonicalization; not a conformance claim |

## Change

The template-pattern compiler now recognizes `attribute::node()` as another
spelling of the already-admitted any-attribute pattern. Because the attribute
axis has attribute as its principal node kind, this pattern compiles to the same
`AnyAttribute` operation and node-test default priority as `@*` and
`attribute::*`. Runtime selection is unchanged.

This does not treat `node()` as an unrestricted node-kind wildcard on arbitrary
axes or admit a general expanded-axis pattern grammar.

## Unchanged cases

Three cases move directly to definite XML comparison passes:

- `Lotus/conflictres_conflictres29#1`;
- `Lotus/conflictres_conflictres30#1`; and
- `Lotus/node_node19#1`.

The two conflict-resolution cases verify that the canonical form retains the
same default priority and declaration-order recovery behavior as the equivalent
attribute wildcard patterns.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 1,016 | 1,019 | +3 |
| Execution succeeded | 815 | 818 | +3 |
| XML comparison passes | 734 | 737 | +3 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 19 | 19 | 0 |
| Execution failures | 201 | 201 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **737 / 2,742 = 26.88%** of standard-operation
cases and **737 / 3,173 = 23.23%** of the complete catalog.

## Verification

The focused wildcard-pattern compiler test now also verifies that
`attribute::node()` lowers to `AnyAttribute` with node-test default priority.
The complete local OASIS sweep confirms all three affected cases pass without a
new mismatch, comparator gap, runtime failure, unexpected success, or panic.
