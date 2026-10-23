# OASIS XSLT 1.0 Qualified `count()` Text Axes

Date: 2026-09-23  
Status: Local compatibility evidence

## Question

Can a namespace-qualified child path retain a trailing reverse-axis text-node
test when used as the argument to `count()`?

## Implemented boundary

The shared qualified-path parser now composes already supported unqualified
axis steps after a namespace-qualified child or attribute step. The shared
path model also adds the missing `preceding-sibling::text()` node test;
`preceding::text()` already existed. Both continue through the ordinary
work-charged reverse-axis evaluator, document-order normalization, and typed
`count()` value operation.

This slice does not admit the namespace axis, qualified name tests on arbitrary
axes, general function composition, or broader whitespace stripping. The
compiler resolves namespace prefixes from stylesheet static context and
retains only expanded names in the compiled path.

## Corpus effect

Against the unchanged 3,173-case OASIS archive:

- initialized cases rise from 2,240 to 2,248;
- successful executions rise from 2,180 to 2,184;
- exact XML comparisons rise from 2,031 to 2,035 / 3,173 (64.13%);
- unchanged Microsoft whitespace cases `91441`, `91442`, `91445`, and `91446`
  produce their expected preceding or preceding-sibling text-node counts;
- cases `91443`, `91444`, `91447`, and `91448` now initialize but retain the
  explicit `FXRT1014` boundary for `xsl:strip-space` over sources containing
  `xml:space`;
- initialization failures fall from 895 to 887, execution failures rise from
  60 to 64, and comparison mismatches remain 66; and
- expected-error unexpected successes remain five and no execution panic
  occurs.

## Verification

A focused evaluator test composes a qualified child step with both reverse text
axes and checks the selected text nodes in document order. The unchanged OASIS
cases independently exercise compilation through `count()`, namespace
resolution, reverse-axis traversal, and serialization.

The archive remains local and unmodified. This is compatibility evidence, not
a broad conformance claim.
