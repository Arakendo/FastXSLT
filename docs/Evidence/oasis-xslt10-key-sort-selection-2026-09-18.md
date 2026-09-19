# OASIS XSLT 1.0 Key Sort Selection

## Question

Can an XSLT 1.0 sort key reuse the shared typed `key()` selector while
preserving candidate focus, first-node conversion, and existing sort behavior?

## Method

- Add one private sort-select variant containing the existing typed key lookup.
- Compile it only under XSLT 1.0 compatibility and retain its exact prepared-
  program capacity.
- Evaluate the lookup from each sort candidate's context through the complete
  charged reference selector.
- Convert the first selected node in document order to its string value, or the
  empty sequence to the empty string, before existing text/numeric sort typing.
- Test a context-path lookup value and numeric chronological ordering, then run
  the unchanged, hash-verified 3,173-case OASIS CD04 catalog.

## Result

The focused case resolves each birthday's month through the same source key and
sorts the candidates as `early`, `middle`, `late`.

The unchanged cases `Lotus/idkey_idkey32#1` and `Lotus/idkey_idkey33#1` now
execute and compare exactly through `xsl:apply-templates` and `xsl:for-each`.
The strict exact-result lower bound rises from 1,610 to 1,612. Initialized cases
rise from 2,000 to 2,002 and successfully executed cases rise from 1,895 to
1,897. Initialization failures fall from 1,135 to 1,133. Execution failures,
comparison mismatches, and expected-error totals remain unchanged.

## Boundary conclusion

This is an additional consumer of the already bounded XSLT 1.0 key lookup. It
does not admit arbitrary functions as sort expressions, dynamic key names,
cross-document key context, a retained key index, or alternate sorting
semantics.

The result is bounded XSLT 1.0 compatibility evidence, not a general `key()`,
sorting, or conformance claim.
