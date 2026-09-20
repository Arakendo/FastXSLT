# OASIS XSLT 1.0 Forward Global Dependencies

Date: 2026-09-19  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Finding

The compiler already represented a global variable whose default was another
global variable and the runtime could materialize that alias, but compilation
required the dependency to appear earlier in stylesheet declaration order.
XSLT global bindings do not acquire evaluation order from declaration order.
The artificial restriction left 13 catalog cases at the private `FXST1044`
frontier.

## Repair

- Stably order the admitted global bindings before runtime materialization.
- A binding is ready when its represented dependency is no longer pending;
  otherwise independent declarations retain their relative order.
- Continue to resolve host-supplied global parameters before their dependants.
- Reject a dependency cycle explicitly as `XTDE0640` rather than allowing an
  order-dependent unbound-variable failure during execution.
- Retain the same immutable compiled binding records and invocation-owned
  runtime global maps; no cross-invocation state or lazy global cache was
  introduced.

Focused compiler tests cover forward, backward, and cyclic dependency shapes.
A production runtime test proves that a forward alias observes the later
atomic binding during transformation.

## Corpus result

The `FXST1044` initialization frontier falls from 13 cases to zero. Five cases
advance through execution and unchanged XML comparison:

- initialized: 2,026 -> 2,031;
- successfully executed: 1,925 -> 1,930;
- exact XML-semantic matches: 1,798 -> 1,803;
- visible XML mismatches: unchanged at 53.

The unchanged Lotus `variable_variable33#1` case is a named sentinel and emits
the expected `<out>titi</out>` result. The other former frontier cases that do
not become exact remain visible at their next independent disposition rather
than being credited.

## Boundary conclusion

This tranche orders the currently represented direct global-variable alias
dependencies within a compiled stylesheet program. It does not claim a general
lazy global evaluator, arbitrary dependency extraction from every XPath plan,
or complete cross-module import-precedence composition. Those require their
own executable evidence.
