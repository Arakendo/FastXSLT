# OASIS XSLT 1.0 Local `key()` Variable Binding

Date: 2026-09-23  
Status: Local compatibility evidence

## Question

Can a select-based local XSLT 1.0 variable retain an already compiled `key()`
lookup and expose its source-node sequence to later instructions without adding
a general expression-valued variable path?

## Implemented boundary

The compiler now lowers an admitted XSLT 1.0 local
`xsl:variable/@select="key(...)"` to a dedicated source-node binding
instruction. The instruction retains the existing typed key-lookup plan in
compiled state, evaluates it against the current invocation context, and binds
the selected source nodes through the ordinary invocation-owned variable
frame.

This does not add a new key index or cache, retain source nodes in compiled
state, change resource authority, or admit arbitrary local XPath expressions.
The lowering is restricted to XSLT 1.0 compatibility mode; an equivalent
modern stylesheet continues to report the existing unsupported-expression
diagnostic.

## Corpus effect

Against the unchanged 3,173-case OASIS archive:

- initialized cases rise from 2,237 to 2,238;
- successful executions rise from 2,177 to 2,178;
- exact XML comparisons rise from 2,028 to 2,029 / 3,173 (63.95%);
- unchanged `Microsoft/Keys_Bug76935#1` now produces its expected copied
  keyed nodes; and
- execution failures remain 60, comparison mismatches remain 66,
  expected-error unexpected successes remain five, and no execution panic
  occurs.

## Verification

A focused runtime test builds a static key, binds the lookup result to a local
variable, and copies that source-node sequence. A cross-version assertion keeps
the same select unsupported in a version 3.0 stylesheet. The unchanged corpus
case exercises the compatibility path through normal compilation, execution,
serialization, and XML-semantic comparison.

The archive remains local and unmodified. This is compatibility evidence, not
a broad conformance claim.
