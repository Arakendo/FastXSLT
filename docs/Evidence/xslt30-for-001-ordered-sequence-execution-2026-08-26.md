# XSLT30 `for-001` Ordered Sequence Execution

Date: 2026-08-26

## Native case

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/expr/for/_for-test-set.xml`
- Case: `for-001`
- Dependency: `XSLT20+`
- Environment/source: `for01` / `for01.xml`
- Stylesheet: `for-001.xsl`
- Assertion: file-backed `for-001.out`

The unmodified stylesheet evaluates:

```xpath
for $a in distinct-values(/bib/book/author)
return ((/bib/book/author[. = $a])[1],
        /bib/book[author = $a]/title)
```

inside `xsl:sequence` beneath a literal `out` result element.

## Implemented seam

The compiler recognizes a narrow but name-independent expression shape: one
`for` binding over `distinct-values` of an absolute unprefixed child path,
followed by an ordered two-part return sequence. The first part selects the
first node whose string value equals the bound value. The second selects named
children of related parent nodes whose named test child has that value.

Static variable and path structure remains in the compiled stylesheet.
Distinct values, selected source nodes, and copied result nodes remain
invocation-local. Evaluation retains source `NodeId` values until result
construction, preserving source document order and avoiding a filename or
serialized-text identity substitute.

`xsl:sequence` copies the selected unnamespaced element/text subtrees into the
semantic result. The private copy seam fails explicitly for selected attributes,
comments, processing instructions, document nodes, or elements with attributes;
it does not silently discard structures the result representation cannot yet
preserve.

## Verification

The metadata-driven test imports the native source and stylesheet into a
bounded sealed snapshot after closing their file handles, compiles the native
stylesheet, executes the normal transform-set path, and compares the complete
serialized result with the upstream file-backed assertion. The author/title
sequence matches exactly, including repeated titles and the order established
by distinct author values.

Navigation is charged as XPath node-visit work. String atomization traversals
are charged through the XDM string-value domain. `xsl:sequence`, result-node and
text construction, and serialization retain their existing independent work
charges.

The conserved four-case denominator is now one passed, two
engine-unsupported, one harness-unsupported, zero failed, and zero metadata
failures.

## Claim boundary

This is not a general XPath sequence, FLWOR, function, comparison, collation, or
XSLT `xsl:sequence` implementation. It does not cover arbitrary return
expressions, multiple clauses, atomic result items, namespaces, source
attribute copying, or other node kinds. Those remain unsupported until native
cases and accepted profile semantics expand the seam.
