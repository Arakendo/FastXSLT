# XSLT30 `template-005` Named-Template Execution Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-26 |
| Suite | W3C XSLT 3.0 test suite |
| Revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Native identity | test set `template`, case `template-005` |
| Dependency | `XSLT10+` |
| Result assertion | native `assert-xml` |
| Outcome | Passed through the private reference path |

## Executed behavior

The unmodified case compiles one named template with one parameter and a
matched `doc` template that calls it. The named template conditionally emits
text and the parameter value, then recursively calls itself with values `2` and
`3`. The result matches the native `assert-xml` element name and string value.

The private semantic slice now includes:

- compile-time named-template collection and duplicate-name rejection;
- compile-time validation of call targets and argument names;
- empty parameter defaults and string-valued `xsl:with-param` content;
- invocation-local variable frames that are not stored in compiled state;
- `$name` value access;
- `$name = integer` conditional evaluation;
- named-template calls and recursion; and
- a private maximum named-template call depth of 128, in addition to the
  existing charged XSLT instruction budget and cooperative cancellation points.

An independent infinite-recursion test reaches the depth boundary and returns
structured limit failure `FXRT0003` instead of overflowing the Rust stack.
The guard was reduced from 256 to 128 after a wider instruction dispatcher
showed that 256 debug-mode frames could exhaust a Windows test-thread stack
before the engine-owned limit fired; 128 restores deterministic failure at the
documented engine boundary.

## Denominator effect

All six cases in the complete pinned XSLT30 `template` test set are now selected
and passing. No case was removed or reclassified as excluded or harness-only.

| Disposition | Count |
| --- | ---: |
| Selected and passed | 6 |
| Engine unsupported | 0 |
| Total | 6 |

## Claim boundary

The implemented expression and parameter behavior is intentionally narrow.
This evidence does not establish general sequence constructors, parameter
tunneling, types, default expressions, the XPath operator grammar, tail-call
optimization, general pattern priority, XSLT 1.0, or XSLT 3.0 conformance. The
depth value is a private safety boundary, not a stabilized host policy default.
