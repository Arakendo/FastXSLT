# XSLT30 Root Context and XDM String Value

Date: 2026-09-02

## Inputs

- W3C XSLT 3.0 test-suite revision
  `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`.
- Complete native test set `tests/fn/root/_root-test-set.xml`.
- Unchanged `root-0101` through `root-0104`, `root-0201`, and `root-0601`
  sources, stylesheets, and native assertions.

## Method

A first-party overlay conserves all ten native `fn/root` cases. Its default is
`harness-unsupported/not-run`; six cases are selected. The adapter checks the
complete native case identity set before it imports each selected source and
stylesheet into a bounded sealed snapshot, compiles once, and executes through
the normal transform-set path.

The implementation first admitted `root(.)`, then generalized that private
variant to one bounded location-path argument. It evaluates the argument with
the shared path evaluator, returns empty for an empty selection, reports
`XPTY0004` for more than one selected node, and otherwise walks parent
relationships to the root while charging one XPath node visit for each
inspected node. It obtains that node's XDM string value through the existing
controlled visitor. The equivalent
`document-node()` match spelling compiles to the same document-node pattern as
`/`; it does not create a second selection rule.

The fifth promotion resolves namespace-qualified child name tests from the
stylesheet static context. It lowers those tests to expanded names in the same
location-path representation and evaluator used by unqualified paths. Prefix
spelling therefore does not become node identity, and an unbound prefix is a
static `XPST0081` failure rather than an empty selection.

The sixth promotion retains source-node selections in an invocation-local
variable frame and applies `root($variable)` to those node identities. The
frame owns only bounded `NodeId` sequences; it does not copy source nodes,
detach them from their prepared document, or share the binding across
invocations. Direct and variable arguments share the same zero-or-one
cardinality check and charged ancestor walk.

The first unchanged execution found that container string values included
descendant comment and processing-instruction content. A focused XDM regression
now proves that document and element string values contain descendant text
nodes only, while a comment or processing instruction selected directly still
has its own content as its string value.

## Results

- Complete conserved denominator: 10 cases.
- Selected and passed: 6 (`root-0101` through `root-0104`, `root-0201`, and
  `root-0601`).
- Visible default not run: 4.
- Engine unsupported: 0.
- Profile excluded: 0.
- The unchanged assertion passes after the shared XDM string-value correction.

The fourth case also admits typed child `element()`, `comment()`, and
`processing-instruction()` steps alongside the existing `node()` and `text()`
steps. A focused mixed-child test proves that each kind test selects only its
declared XDM node kind.

This raises conserved XSLT30 accounting within this denominator to six passes
and four visible defaults. With the subsequently conserved `insn/apply-imports`
case, current XSLT30 accounting is 578 cases: 417 passed comparisons, 3
engine-unsupported cases, 54 profile exclusions, and 104 visible default
not-run cases.

## Limitations

This evidence does not admit arbitrary `root()` arguments. The remaining four
cases require generated node identity, temporary trees, or dynamic resource
acquisition. They
remain individually visible under the
denominator default rather than being reported as engine failures. The result
also does not claim a general XPath function-call representation; `root(.)` is
a typed private expression variant in the current reference engine.
