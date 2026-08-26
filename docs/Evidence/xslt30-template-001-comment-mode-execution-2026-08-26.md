# XSLT30 `template-001` Comment and Mode Execution Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-26 |
| Suite | W3C XSLT 3.0 test suite |
| Revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Native identity | test set `template`, case `template-001` |
| Dependency | `XSLT10+` |
| Result assertion | native `assert-xml` |
| Outcome | Passed through the private reference path |

## Executed behavior

The harness resolves the case's referenced environment, principal source,
stylesheet, and expected XML from the pinned test-set metadata. It admits the
source and stylesheet into a bounded sealed snapshot, closes the upstream file
handle, compiles the stylesheet, executes one identified request, serializes the
semantic result, and compares the result element's expanded name and string
value with the native `assert-xml` expectation.

The unmodified case exercises these semantics together:

- no explicit `/` template, requiring the built-in document rule;
- exact `doc` element-template dispatch in the default mode;
- `comment()` child selection;
- an unprefixed named mode on `xsl:apply-templates` and `xsl:template`;
- comment-node pattern matching in that mode; and
- context-item string value through `xsl:value-of select="."`.

The stylesheet also contains a default-mode `comment()` template whose body
states that the test failed. The passing expected result therefore supplies
direct evidence that mode identity is respected rather than ignored.

## Denominator effect

The complete pinned `template` test set remains six cases:

| Disposition | Count |
| --- | ---: |
| Selected and passed | 2 |
| Engine unsupported and not run | 4 |
| Total | 6 |

`template-001` and `template-006` pass. `template-002` through `template-005`
remain present and explicitly unsupported. The overlay changed one case's
disposition; it did not change the discovered denominator.

## Claim boundary

This proves the named behavior and case only. It does not establish general
mode, pattern-priority, node-test, XPath, XSLT 1.0, or XSLT 3.0 conformance.
Processing-instruction, `node()`, attribute, and named-template cases remain
separate implementation pressure.
