# OASIS XSLT 1.0 Level-Any Number Tranche

Date: 2026-09-06

## Question

Can `xsl:number level="any"` reuse the bounded static number patterns and XDM
document order while preserving exact work accounting and visible later
boundaries?

## Implemented slice

- charged document-order traversal through the source document, attributes,
  and children;
- counting through the context node with either the context-derived default
  pattern or the admitted static `count` pattern;
- reset at the latest admitted static `from` boundary;
- the same retained number format and result-text construction used by the
  single-level path.

`level="multiple"`, pattern unions and predicates, grouping, and broader
format tokens remain unsupported.

## Focused verification

A first-party runtime test numbers three `a` elements across two `chapter`
boundaries as `1, 2, 1`, proving document-order accumulation and `from` reset
without introducing another source representation.

## Full local OASIS observation

Command:

```powershell
./scripts/measure-oasis-xslt10.ps1
```

Relative to the 880-pass single-pattern checkpoint:

| Observation | Before | After | Change |
| --- | ---: | ---: | ---: |
| Initialized | 1,197 | 1,206 | +9 |
| Executed successfully | 989 | 996 | +7 |
| XML comparison pass | 880 | 887 | +7 |
| XML comparison mismatch | 75 | 75 | 0 |
| Execution failure | 208 | 210 | +2 |

The two later failures reach the existing bounded HTML-serialization boundary;
they are not credited as numbering passes. The strict unchanged-result lower
bound is now 887 of 2,742 standard-operation cases (32.35%), or 887 of all
3,173 catalog cases (27.95%).

## Interpretation

The result reduces the `FXST1048` level frontier from 61 to 40 cases and proves
that the source XDM already owns enough document-order structure for bounded
level-any numbering. It does not imply streaming numbering, multiple-level
number lists, or a generalized pattern API.
