# OASIS XSLT 1.0 Sort Tranche

Date: 2026-09-06

## Question

Can the shared compiler and runtime admit a useful first `xsl:sort` slice without
creating a legacy-only evaluator or claiming locale/collation behavior that has
not been implemented?

## Implemented slice

- `xsl:sort` as the leading children of `xsl:for-each` and
  `xsl:apply-templates`;
- stable, input-order-preserving multi-key sorting;
- location-path, literal, `position()`, and `last()` sort keys, including the
  default `select="."`;
- literal `data-type="text|number"` and `order="ascending|descending"`;
- XPath work charging for key evaluation and a conservative comparison charge;
- the same typed plan and execution path for XSLT 1.0 and modern stylesheets.

Unsupported dynamic AVTs, language/case-order options, custom data types,
temporary-tree sorting, and atomic-sequence sorting remain explicit. Text
ordering currently uses an ASCII case-insensitive primary comparison with
stable ties; this is a bounded compatibility slice, not a general collation
contract.

## Focused verification

A first-party runtime test executes numeric `xsl:for-each` sorting and stable
two-key `xsl:apply-templates` sorting through one stylesheet. It verifies that
sorting changes focus order without changing template dispatch or sibling
semantics.

## Full local OASIS observation

Command:

```powershell
./scripts/measure-oasis-xslt10.ps1
```

Relative to the preceding 808-pass lower bound:

| Observation | Before | After | Change |
| --- | ---: | ---: | ---: |
| Initialized | 1,093 | 1,171 | +78 |
| Executed successfully | 891 | 963 | +72 |
| XML comparison pass | 808 | 857 | +49 |
| XML comparison mismatch | 50 | 72 | +22 |
| Execution failure | 202 | 208 | +6 |

The strict unchanged-result lower bound is therefore 857 of 2,742
standard-operation cases (31.25%), or 857 of all 3,173 catalog cases (27.01%).
The additional mismatches and later failures remain uncredited and visible;
initialization success is not treated as compatibility success.

## Interpretation

The named `xsl:sort` initialization frontier disappeared as a dominant group
and exposed later expression, collation, serialization, and result mismatches.
Forty-nine cases reached their expected XML result. The result supports
continuing the shared sorting implementation, but does not establish complete
XSLT 1.0 sorting, locale-aware collation, or a conformance claim.
