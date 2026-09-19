# OASIS XSLT 1.0 Variable-Composed Computed-Attribute Names

Date: 2026-09-18  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the remaining variable-only computed-attribute name AVTs reuse invocation
and global variable ownership without adding a general AVT evaluator or a
second variable store?

## Result

Yes. Under XSLT 1.0 static context, a computed-attribute name may now consist
only of static text and one or more unqualified variable references. Compilation
retains a typed text/variable part sequence. Execution resolves invocation-local
or prepared-global atomic and temporary-tree values through the existing runtime
frame, composes the lexical name, and passes it to the same attribute-specific
runtime `QName` validator as the path and context-name forms.

Paths mixed with variables, general XPath expressions, escaped-brace syntax,
dynamic namespace AVTs, and a public AVT plan remain outside this slice.

## Corpus effect

The complete hash-verified 3,173-case measurement remains conserved.

- All five remaining cases leave `FXST1062`; the frontier falls from 5 to 0.
- Two cases reach their independent static empty-`use-attribute-sets` error.
- Two cases reach the independent unsupported UTF-16 string-serialization
  boundary.
- `Microsoft/AttributeSets__91139#1` correctly reports an unbound variable
  because its local `$x` declaration follows the result construction that uses
  the attribute set.
- No case receives pass credit. The strict expected-result lower bound remains
  1,569.
- Aggregate lifecycle counts become 1,952 initialized, 1,850 executed
  successfully, 1,183 initialization failures, and 102 execution failures.

## Verification

- A focused runtime test composes one lexical attribute name from two prepared
  global text values and verifies the serialized result.
- The complete local OASIS measurement conserves every case identity, reports
  no remaining `FXST1062` case, and keeps each later disposition visible.
- The ordinary workspace verification gates pass.
