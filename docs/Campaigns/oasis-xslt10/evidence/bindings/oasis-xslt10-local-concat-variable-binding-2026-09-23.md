# OASIS XSLT 1.0 Local `concat()` Variable Binding

Date: 2026-09-23  
Status: Local compatibility evidence

## Question

Can a select-based local XSLT 1.0 variable retain the already supported typed
`concat()` expression and resolve an outer lexical binding at invocation time?

## Implemented boundary

The compiler now lowers an admitted XSLT 1.0 `concat()` in a local
`xsl:variable/@select` to a dedicated atomic binding instruction. The binding
reuses the existing typed concat plan and charged evaluator, resolves variables
through the current invocation scope, and stores the resulting string in the
ordinary invocation-owned atomic-variable frame.

This does not add a general dynamic-expression variable, alter modern
stylesheet semantics, retain invocation values in compiled state, or introduce
another evaluator. Unsupported local selects continue to fail explicitly.

## Corpus effect

Against the unchanged 3,173-case OASIS archive:

- initialized cases rise from 2,236 to 2,237;
- successful executions rise from 2,176 to 2,177;
- exact XML comparisons rise from 2,027 to 2,028 / 3,173 (63.91%);
- unchanged `Microsoft/Variables_VariableWithinVariable#1` now produces the
  exact expected `<root>local from global</root>` result; and
- execution failures remain 60, comparison mismatches remain 66, expected-error
  unexpected successes remain five, and no execution panic occurs.

## Verification

A focused runtime test shadows a global binding inside a constructed local
variable, evaluates `concat('local from ', $value)`, and then reads the outer
constructed result. The unchanged corpus case exercises the same lexical-scope
shape through the normal compile, execution, and serialization path.

The archive remains local and unmodified. This is compatibility evidence, not
a broad conformance claim.
