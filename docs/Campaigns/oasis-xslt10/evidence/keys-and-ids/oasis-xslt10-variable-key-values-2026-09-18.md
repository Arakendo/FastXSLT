# OASIS XSLT 1.0 Variable Key Values

## Question

Can a literal-name `key()` call accept an XSLT 1.0 variable as its second
argument while preserving the distinct string-conversion rules for atomic,
result-tree-fragment, and source-node-set values?

## Method

- Replace the private lookup's static string field with a typed static-or-
  variable value source.
- Resolve atomic and temporary-tree variables through the existing XSLT 1.0
  string conversion owner.
- Resolve a source-node-set variable to every node string value, then select
  the document-ordered union of matches for those values.
- Perform lookup-value conversion once per key call under normal work and
  cancellation accounting.
- Keep the key name literal and retain all existing positional/tail and
  consumer behavior.
- Test both a temporary text variable and a two-node source variable, then run
  the unchanged, hash-verified 3,173-case OASIS CD04 catalog.

## Result

The focused case selects one key value from a temporary text variable and the
ordered union for two source-node values.

Two OASIS cases leave the initialization frontier. `Microsoft/Keys__10050#1`
compares exactly. `Microsoft/Keys__91732#1` executes the intended lookup and
copy but remains an XML-semantic comparison mismatch because the current HTML
indentation path emits whitespace that differs from the expected result.

The strict exact-result lower bound rises from 1,605 to 1,606. Initialized cases
rise from 1,993 to 1,995 and successfully executed cases rise from 1,889 to
1,891. Initialization failures fall from 1,142 to 1,140; comparison mismatches
rise from 213 to 214. Execution failures and expected-error totals remain
unchanged.

## Boundary conclusion

This slice admits only a variable second argument with a literal key name. It
does not admit a dynamic key name, arbitrary second-argument expressions,
cross-document key context, a retained index, or hidden variable evaluation.
The independent HTML serialization mismatch remains visible.

The result is bounded XSLT 1.0 compatibility evidence, not a general `key()` or
conformance claim.
