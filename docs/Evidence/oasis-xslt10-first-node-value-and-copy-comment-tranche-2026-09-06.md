# OASIS XSLT 1.0 First-Node Value and Copy-Comment Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-06 |
| Corpus | OASIS XSLT/XPath 1.0 CD04, locally acquired and hash-verified |
| Scope | XSLT 1.0 node-set string conversion, source-comment `xsl:copy`, and case-insensitive legacy HTML element recognition |
| Disposition | Fifteen new expected-result matches; later comparison boundaries remain visible |

## Change

The compiled value-expression plan now distinguishes an XSLT 1.0 location
path from its modern counterpart. The XSLT 1.0 operation converts a selected
node-set using the string value of its first node in document order, including
the empty-string result for an empty selection. Modern stylesheets retain the
existing explicit cardinality failure rather than inheriting the compatibility
rule. Focused tests prove both sides of that boundary.

`xsl:copy` now preserves a source comment when the comment is the current node,
using the existing bounded result-node construction and work accounting. The
attribute-current-node case remains explicit because late-attribute recovery is
a separate, partly discretionary XSLT 1.0 behavior.

The legacy HTML serializer now recognizes raw-text and void element names
case-insensitively and admits a bounded `SCRIPT` without a `type` attribute.
This moves one corpus case through execution to the existing comparator gap:
its raw script text deliberately contains markup characters and therefore is
not parseable by the XML-only comparator. No pass is claimed for that case.

## Measurement

`scripts/measure-oasis-xslt10.ps1` changed the conserved measurement as follows:

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Catalog cases | 3,173 | 3,173 | 0 |
| Initialized | 1,269 | 1,269 | 0 |
| Executed successfully | 1,057 | 1,075 | +18 |
| Expected-result XML matches | 947 | 962 | +15 |
| XML comparison mismatches | 76 | 78 | +2 |
| Comparator-unsupported results | 20 | 21 | +1 |
| Execution failures | 212 | 194 | -18 |

The XSLT 1.0 first-node rule removes the complete 14-case `FXRT1001`
execution frontier. Twelve of those cases match exactly and two expose later
semantic mismatches. Comment copying contributes three exact passes. The
legacy HTML change contributes the one newly visible comparator limitation.

The strict standard-operation lower bound is now `962 / 2,742 = 35.08%`.
Against all catalog cases, including expected-error and infrastructure cases,
the deliberately conservative ratio is `962 / 3,173 = 30.32%`. These are
compatibility measurements, not XSLT 1.0 conformance claims.
