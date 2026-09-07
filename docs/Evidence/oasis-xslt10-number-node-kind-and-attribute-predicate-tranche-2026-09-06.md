# OASIS XSLT 1.0 Number Node-Kind and Attribute-Predicate Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-06 |
| Corpus | OASIS XSLT/XPath 1.0 CD04, locally acquired and hash-verified |
| Scope | `node()`, `@*`, and exact attribute-value predicates in `xsl:number` patterns |
| Disposition | Six new expected-result matches; no new mismatch or execution failure |

## Change

The typed number-pattern representation now distinguishes any child node,
any element, and any attribute. Those atoms compose with the existing static
union representation, so patterns such as `node() | / | @*` retain their node
kind meaning rather than being approximated as element wildcards.

One bounded predicate form is also admitted: an exact element-name pattern
with an exact attribute-value predicate, such as `note[@flag='yes']`.
Unprefixed attributes correctly use no namespace; prefixed element and
attribute names resolve during stylesheet compilation. Runtime attribute
inspection is charged per visited attribute. Known compiled retention accounts
for both expanded names and the retained comparison value.

General predicate expressions, child-sequence patterns, `id()`/`key()`
patterns, and positional arithmetic remain structured `FXST1050` unsupported
outcomes.

## Measurement

`scripts/measure-oasis-xslt10.ps1` changed the conserved measurement as follows:

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Catalog cases | 3,173 | 3,173 | 0 |
| Initialized | 1,227 | 1,233 | +6 |
| Executed successfully | 1,016 | 1,022 | +6 |
| Expected-result XML matches | 906 | 912 | +6 |
| XML comparison mismatches | 76 | 76 | 0 |
| Execution failures | 211 | 211 | 0 |

The strict lower bound over the 2,742 standard-operation cases is now 33.26%.
The complete catalog ratio is 28.74%. Neither is an XSLT 1.0 conformance claim.
The visible next number-pattern frontier includes
`note[position() mod 2 = 1]` and remains uncredited.

