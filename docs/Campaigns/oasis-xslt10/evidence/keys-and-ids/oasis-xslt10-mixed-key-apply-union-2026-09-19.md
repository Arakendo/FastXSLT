# OASIS XSLT 1.0 Mixed Key Apply Union

Date: 2026-09-19  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can `xsl:apply-templates` combine `key()` results with ordinary source paths
and source-node variables while retaining XPath node-set union semantics?

## Repair

- Compile a bounded top-level union containing `key()` into typed key, path,
  and variable alternatives rather than reparsing expression text at runtime.
- Preserve the existing all-key union specialization and cap the mixed form at
  eight alternatives.
- Evaluate every alternative through its existing charged and cancellable
  selector, then sort the combined source-node identities into document order
  and remove duplicates before template dispatch.
- Account for the retained typed alternatives in compiled-state capacity; the
  large key-lookup variant is privately boxed rather than inflating every union
  entry.
- Reject non-source variable values through the existing structured type-error
  path. No resource authority, cache, or public expression representation is
  added.

## Result

A focused runtime regression combines a key lookup, a path, and a local
source-node variable in reverse selection order. The result proves document
order restoration and duplicate removal before template application.

The unchanged Lotus `select54#1` path-plus-key case and `select64#1`
variable-plus-key case now initialize, execute, and match their expected
results. The complete 3,173-case sweep moves to 2,025 initialized cases, 1,925
successful executions, and 1,794 exact XML-semantic matches. The mismatch count
remains 57.

## Boundary conclusion

This admits bounded mixed unions only for `xsl:apply-templates` when each member
is already a supported literal key lookup, source location path, or source-node
variable. It does not admit a general XPath union evaluator, temporary-tree
variables, dynamic key names/values beyond the existing key slice, or new
ordering semantics.
