# OASIS XSLT 1.0 recursive content-parameter strings -- 2026-09-08

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a named-template recursion pass a string built by `xsl:with-param` content,
observe XSLT 1.0 result-tree-fragment conversion, compute
`string-length($variable)`, and compare that length with another numeric
variable without introducing a second expression runtime?

## Changes

- XSLT 1.0 value compilation admits typed `string-length($variable)` while the
  modern general `fn:string-length` parser remains unchanged.
- Boolean compilation admits a string-length result compared with a numeric
  variable through the five relational operators. Reversed operand spelling is
  normalized in the typed plan.
- Both routes reuse the existing XSLT 1.0 variable string-value owner for
  atomic values, source node sets, temporary trees, and empty sequences.
  Length is measured in Unicode scalar values and every visited scalar is
  charged to XPath work.
- One `xsl:value-of select="concat(...)"` child may build an XSLT 1.0
  `xsl:with-param` value. The argument is retained as an invocation-owned
  temporary text tree rather than silently changing result-tree-fragment
  semantics into an atomic string.
- The existing named-template recursion ceiling, lexical parameter frame,
  cancellation checks, and temporary-XDM node accounting remain in force.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,503 | 1,504 | +1 |
| Executed successfully | 1,338 | 1,339 | +1 |
| Expected-result XML matches | 1,230 | 1,231 | +1 |
| XML comparison mismatches | 82 | 82 | 0 |
| Execution failures | 165 | 165 | 0 |

The unchanged Lotus `variable23` case now executes its recursive named template
through the 10- through 260-character values and matches the expected XML. The
XML comparator correctly treats the archive's CRLF and the generated LF text
line endings according to XML end-of-line normalization.

The strict standard-operation lower bound becomes
`1,231 / 2,742 = 44.89%`; the conservative all-catalog ratio becomes
`1,231 / 3,173 = 38.80%`. These are local compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit a general XPath function/comparison grammar,
arbitrary sequence constructors in template arguments, mixed content, multiple
value-producing instructions, or modern sequence semantics. No corpus bytes or
expected results were changed.

## Verification

- A focused runtime test covers Unicode codepoint length, recursive
  content-built parameters, temporary-tree parameter binding, and the numeric
  stop condition.
- The unchanged `variable23` case passes its expected XML comparison.
- The complete 3,173-case local measurement produced the counters above.
- The ordinary workspace verification gate passes.
