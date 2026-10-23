# OASIS XSLT 1.0 Key Descendant Tail

Date: 2026-09-19  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Why did a successfully compiled `key('name', 'value')//p` selection return an
empty sequence even though the keyed elements contained matching descendants?

## Finding

The key-call compiler removed only the first slash from the `//p` suffix and
compiled the remainder as `/p`. That changed a context-relative descendant
tail into a document-rooted child path. Runtime key selection was otherwise
correct: it preserved keyed node identity, evaluated the retained tail once
from every selected node, restored document order, removed duplicates, and
charged the ordinary path evaluator.

## Repair

- Normalize a `//tail` after `key()` to the existing typed `.//tail` path
  origin.
- Continue lowering a single `/tail` as an ordinary child path relative to
  each selected key node.
- Retain the existing complete key scan, predicate, path evaluation, document
  ordering, deduplication, budgets, and cancellation behavior.
- Add a focused multi-key-node case proving nested descendants, ignored sibling
  groups, and final document order.
- Run the complete unchanged, hash-verified 3,173-case OASIS catalog.

## Result

The focused case selects `A`, nested `B`, and later keyed `D` as `ABD`.

The unchanged `Lotus/idkey_idkey34#1` case moves from comparison mismatch to
exact pass. The exact-result lower bound rises from 1,617 to 1,618 and visible
mismatches fall from 215 to 214. Initialization remains 2,008, successful
execution remains 1,903, and all failure and expected-error totals remain
unchanged.

## Boundary conclusion

This repairs the origin of a path already retained by the typed key plan. It
does not add a new expression evaluator, retained key index, match-pattern
support for `key()`, resource authority, or cross-invocation state.

The result is bounded XSLT 1.0 compatibility evidence, not a general `key()` or
conformance claim.
