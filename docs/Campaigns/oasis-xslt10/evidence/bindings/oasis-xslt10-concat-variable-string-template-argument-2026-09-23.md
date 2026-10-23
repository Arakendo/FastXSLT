# OASIS XSLT 1.0 `concat()` Variable Conversion in Template Arguments

Date: 2026-09-23  
Status: Local compatibility evidence

## Question

Can the existing typed XSLT 1.0 concat evaluator preserve an explicit
`string($variable)` operand when the expression is passed through
`xsl:with-param`?

## Implemented boundary

The XSLT 1.0 concat compiler now recognizes the exact
`string($expanded-variable-name)` operand shape and lowers it to the existing
variable-string conversion used by a concat part. This preserves XSLT 1.0
first-node conversion for source-node sequences and the ordinary atomic and
temporary-tree conversion behavior.

Template invocation compilation can now retain that same typed concat plan as
an invocation argument. Runtime evaluates it against the caller's current
focus and lexical variable frame, then supplies the resulting atomic string to
the callee. Compiled state remains invocation-independent.

This does not add a general nested function evaluator, change parameter
propagation rules, or widen modern XPath semantics.

## Corpus effect

Against the unchanged 3,173-case OASIS archive:

- initialized cases rise from 2,239 to 2,240;
- successful executions rise from 2,179 to 2,180;
- exact XML comparisons rise from 2,030 to 2,031 / 3,173 (64.01%);
- unchanged `Lotus/variable_variable48#1` now produces its expected recursively
  copied tree and parameter values; and
- execution failures remain 60, comparison mismatches remain 66,
  expected-error unexpected successes remain five, and no execution panic
  occurs.

## Verification

Focused tests cover explicit variable conversion inside a value expression,
first-node conversion of a source-node sequence, and the same typed concat plan
crossing a named-template argument boundary. The unchanged corpus case then
exercises apply-template and call-template parameter propagation across a
nested source tree.

The archive remains local and unmodified. This is compatibility evidence, not
a broad conformance claim.
