# OASIS XSLT 1.0 Key Arity Classification

## Question

Can structurally complete `key()` calls with the wrong number of arguments be
classified as invalid independently of the engine's deliberately bounded
lookup-operand support?

## Method

- Count top-level function arguments while respecting nested parentheses and
  quoted commas.
- Report any structurally balanced count other than two as `XPST0017 /
  invalid` before selecting the supported key-name and lookup-value forms.
- Keep malformed delimiters under the existing syntax classification and keep
  valid two-argument forms outside the admitted operand slice unsupported.
- Unit-test zero, one, two, three, nested-call, quoted-comma, and unbalanced
  shapes, then trace the unchanged, hash-verified 3,173-case OASIS CD04 catalog.

## Result

The unchanged expected-error cases `Microsoft/Keys__91722#1`,
`Microsoft/Keys__91723#1`, and `Microsoft/Keys__91724#1` now report
`XPST0017 / invalid` for zero, one, and three arguments respectively.

They were already counted as observed expected errors, so the corpus counters
remain unchanged: the strict exact-result lower bound is 1,612, initialized
cases are 2,002, and successfully executed cases are 1,897.

## Boundary conclusion

Arity is a structural validity property and does not depend on whether the
arguments fall inside the admitted key lookup slice. This classification does
not admit nested key calls, dynamic key names, arbitrary value expressions, or
a general function parser.

The result is diagnostic compatibility evidence, not a broader `key()` or
conformance claim.
