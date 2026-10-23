# OASIS XSLT 1.0 Key-Use Path Union

Date: 2026-09-19  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can one `xsl:key` declaration derive lookup values from a union of ordinary
source paths while preserving XPath node-set identity and order?

## Repair

- Add a private typed key-use representation containing bounded location-path
  alternatives.
- Reuse the existing top-level union splitter so union characters inside
  predicates or string literals are not misclassified.
- Evaluate every alternative through the controlled location-path evaluator,
  restore document order, and remove duplicate node identities before charged
  string-value conversion.
- Include every retained path and vector allocation in compiled-state capacity
  accounting.
- Keep key lookup, declaration composition, source ownership, and resource
  authority unchanged.

## Result

A focused regression uses `z | x | x | y`, proving both duplicate identity
removal and document-order behavior before key-value construction.

The unchanged Lotus `idkey_idkey59#1` case now initializes, executes, and
matches its expected result. The complete 3,173-case sweep reports 2,017
initialized cases, 1,917 successful executions, and 1,786 exact XML-semantic
matches.

## Boundary conclusion

This admits top-level unions of already-supported location paths in
`xsl:key/@use`. It does not admit arbitrary XPath expressions, alter key
candidate matching, share an index across invocations, or grant any new
resource access.
