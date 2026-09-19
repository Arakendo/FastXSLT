# OASIS XSLT 1.0 Context-Path Key Values

## Question

Can a literal-name `key()` call use an already typed context location path as
its second argument without adding arbitrary expression evaluation?

## Method

- Add a typed context-path value source to the private key lookup plan.
- Evaluate the path from the call's current source context under existing work
  and cancellation accounting.
- Convert every selected node to its string value and select the ordered union
  of matching key entries.
- Retain the existing static/variable values, predicates, optional tails, and
  consumer plans.
- Test both a multi-node path and repeated `key('codes', .)` calls under
  changing focus, then run the unchanged, hash-verified 3,173-case OASIS CD04
  catalog.

## Result

The focused case proves that a two-node context path and two changing singleton
contexts select the same ordered keyed nodes.

Two unchanged OASIS cases leave initialization failure.
`Lotus/idkey_idkey17#1` executes and compares exactly.
`Microsoft/Keys__91730#1` reaches execution and reports the independent
`XTDE0410` result-attribute attachment defect.

The strict exact-result lower bound rises from 1,606 to 1,607. Initialized cases
rise from 1,995 to 1,997; successfully executed cases rise from 1,891 to 1,892;
initialization failures fall from 1,140 to 1,138; and execution failures rise
from 104 to 105. Comparison mismatches and expected-error totals remain
unchanged.

## Boundary conclusion

The admitted value is an existing typed location path evaluated from the
current principal-source context. This does not add arbitrary expressions,
nested `key()` calls, dynamic key names, cross-document context switching, or a
retained index.

The result is bounded XSLT 1.0 compatibility evidence, not a general `key()` or
conformance claim.
