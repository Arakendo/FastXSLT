# OASIS XSLT 1.0 Result-Attribute Recovery

Date: 2026-09-20

## Question

Can FastXSLT implement the XSLT 1.0 permitted recovery for an attribute that
cannot be attached to a result element without weakening the modern static or
dynamic error contract?

## Finding

Yes. The five remaining execution-time `XTDE0410` cases all select the XSLT
1.0 recovery that ignores an unattached or late result attribute. The recovery
must not discard an attribute merely because it was produced by a nested
template: an attribute produced before child content still belongs to the
surrounding element.

The separate Lotus `copy40` case revealed a useful static-ordering distinction.
Its `xsl:copy-of select="@level"` produces an attribute, not child content, so a
following `xsl:attribute` is not late. Correcting that classification advances
the case from `XTDE0410` to the narrower unsupported `text()` attribute-value
expression. It is not counted as a pass.

## Implementation

Compilation selects recovery only for an XSLT 1.0 static context and records it
on the relevant attribute, copy, and copy-of plans. Runtime uses a private
recoverable pending-attribute state:

- element assembly consumes it normally when it precedes child content;
- an unattached top-level result or an attribute following child content is
  omitted under the selected XSLT 1.0 recovery;
- modern pending attributes retain `XTDE0410` in those positions; and
- duplicate attached attributes retain the existing error behavior.

Source-copy static analysis now recognizes `@name`, `@*`, and pipe unions of
those forms as attribute-only copy-of selections. This is compile-time plan
selection rather than a recurring stylesheet-version branch in node traversal.
Temporary-tree materialization and final serialization observe the same typed
state, so no second compatibility evaluator was introduced.

Focused regressions cover nested attributes that must attach, XSLT 1.0 late and
top-level attributes that may be ignored, strict modern failures, and the
attribute-only copy-of ordering distinction.

## Corpus movement

Five unchanged OASIS cases move from execution failure to exact XML-semantic
comparison:

- Lotus `copy50`, `copy61`, and `copy62`;
- Lotus `output112`; and
- Microsoft `BVTs_bvt038`.

| Counter | Before | After | Change |
| --- | ---: | ---: | ---: |
| Initialized | 2,089 | 2,089 | 0 |
| Initialization failures | 1,046 | 1,046 | 0 |
| Executed successfully | 1,998 | 2,003 | +5 |
| Execution failures | 91 | 86 | -5 |
| Exact XML-semantic matches | 1,867 | 1,872 | +5 |
| XML comparison mismatches | 55 | 55 | 0 |

The measured exact compatibility lower bound is now
`1,872 / 3,173 = 59.00%`. No execution or initialization frontier remains under
`XTDE0410`; `copy40` is now classified under `FXXP1012` as described above.

This is local compatibility evidence against the hash-verified,
non-redistributed OASIS CD04 archive; it is not a broad conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features unattached_attributes
cargo test -p fastxslt --all-features source_copy_attribute_ordering
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier XTDE0410
./scripts/verify.ps1
```
