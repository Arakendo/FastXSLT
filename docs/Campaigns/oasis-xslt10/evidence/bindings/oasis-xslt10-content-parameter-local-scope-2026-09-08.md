# OASIS XSLT 1.0 content-parameter local scope -- 2026-09-08

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a content-built `xsl:with-param` evaluate one final `xsl:value-of` after
bounded text-only local variable declarations, while preserving XSLT 1.0
temporary-tree conversion and lexical scope?

## Changes

- The private template-argument plan now carries up to 64 typed text-tree
  bindings followed by one already-admitted `ValueExpression`.
- Compilation accepts only XSLT 1.0 text-only `xsl:variable` declarations in
  this prefix. Other instruction sequences, variable forms, and duplicate
  names remain rejected explicitly.
- Runtime clones the caller's invocation-local variable frame, materializes
  each binding as a charged temporary text tree, evaluates the final value
  expression in that local scope, and discards the scope after constructing
  the argument.
- The argument itself remains an invocation-owned temporary tree. The target
  template therefore observes ordinary XSLT 1.0 result-tree-fragment
  conversion rather than an atomic shortcut.
- Compiled retained-capacity accounting includes binding names, binding text,
  the bounded vector, and the retained value-expression plan.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,509 | 1,510 | +1 |
| Executed successfully | 1,344 | 1,345 | +1 |
| Expected-result XML matches | 1,232 | 1,233 | +1 |
| XML comparison mismatches | 86 | 86 | 0 |
| Execution failures | 165 | 165 | 0 |

The unchanged Lotus `variable14` case now passes. Its argument-local `$test`
is visible to the following `xsl:value-of`, the produced `bar` parameter is
copied as a temporary tree by the called template, and the binding does not
escape its constructor scope.

The strict standard-operation lower bound becomes
`1,233 / 2,742 = 44.97%`; the conservative all-catalog ratio becomes
`1,233 / 3,173 = 38.86%`. These are local compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit arbitrary sequence constructors, constructed local
variables, nested control flow, local variable initializers other than one
literal text node, or more than 64 constructor-local bindings. It does not
introduce a second template executor or change modern variable semantics. No
corpus bytes or expected results were changed.

## Verification

- A focused runtime test proves local shadowing, argument construction, target
  temporary-tree copying, and restoration of the outer variable afterward.
- The unchanged Lotus `variable14` case passes the suite XML comparator.
- The complete 3,173-case local measurement produced the counters above.
- The ordinary workspace verification gate passes.
