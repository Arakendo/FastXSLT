# XSLT30 Root Context and XDM String Value

Date: 2026-09-02

## Inputs

- W3C XSLT 3.0 test-suite revision
  `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`.
- Complete native test set `tests/fn/root/_root-test-set.xml`.
- Unchanged `root-0101` source, stylesheet, and inline XML assertion.

## Method

A first-party overlay conserves all ten native `fn/root` cases. Its default is
`harness-unsupported/not-run`; only `root-0101` is selected. The adapter checks
the complete native case identity set before it imports the selected source and
stylesheet into a bounded sealed snapshot, compiles once, and executes through
the normal transform-set path.

The implementation admits exactly `root(.)` in `xsl:value-of`. It walks from
the current source node through parent relationships to the root, charging one
XPath node visit for each inspected node, and then obtains that node's XDM
string value through the existing controlled visitor. The equivalent
`document-node()` match spelling compiles to the same document-node pattern as
`/`; it does not create a second selection rule.

The first unchanged execution found that container string values included
descendant comment and processing-instruction content. A focused XDM regression
now proves that document and element string values contain descendant text
nodes only, while a comment or processing instruction selected directly still
has its own content as its string value.

## Results

- Complete conserved denominator: 10 cases.
- Selected and passed: 1 (`root-0101`).
- Visible default not run: 9.
- Engine unsupported: 0.
- Profile excluded: 0.
- The unchanged assertion passes after the shared XDM string-value correction.

This raises conserved XSLT30 accounting to 577 cases: 412 passed comparisons,
3 engine-unsupported cases, 54 profile exclusions, and 108 visible default
not-run cases.

## Limitations

This evidence does not admit arbitrary `root()` arguments. The remaining nine
cases require function-argument location paths, empty sequences, variables,
generated node identity, temporary trees, dynamic resource acquisition, or
broader kind-test expressions. They remain individually visible under the
denominator default rather than being reported as engine failures. The result
also does not claim a general XPath function-call representation; `root(.)` is
a typed private expression variant in the current reference engine.
