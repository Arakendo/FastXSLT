# OASIS XSLT 1.0 Dynamic Decimal-Format Selection

Date: 2026-09-21  
Status: Local compatibility evidence

## Question

Can `format-number()` select a compiled named decimal format through an XSLT
1.0 variable, preserve the stylesheet's static QName context, and format large
lexical decimals without floating-point or fixed-width truncation?

## Method

- Retain a variable-valued third `format-number()` argument as a typed operand.
- Resolve its runtime lexical QName against the instruction's compiled static
  namespace bindings, then select only from compiled decimal-format
  declarations.
- Fold a literal-only `concat()` used as the third argument without admitting
  general dynamic function dispatch.
- Replace the formatting step's `i128` magnitude with checked decimal-string
  scaling and rounding while retaining exact-rational evaluation for arithmetic
  expressions.
- Recognize apostrophe-quoted picture literals and ignore quoted pattern
  separators and active characters.
- Execute the corpus through the workbench's bounded byte-serialization lane,
  then decode only declared UTF-8 and ISO-8859-1 output for XML-semantic
  comparison. This avoids treating a selected non-UTF-8 output encoding as a
  failure of the convenience Rust `String` lane.
- Exercise the complete lifecycle with a global variable naming a European
  decimal format, then rerun the unchanged 3,173-case OASIS catalog.

## Result

The focused lifecycle formats `1234.5` with a variable-selected European
format as `1.234,50`. Unit evidence also covers a 70-digit magnitude and a
negative subpicture whose literal `#` characters are apostrophe-quoted.

The former `unsupported/FXXP1009/FXXP1009` initialization frontier falls from
ten cases to two. Five additional cases initialize. The execution-error case
Microsoft `BVTs_bvt015` now reaches the required runtime error rather than
failing during compilation, and expected-error accounting moves one case from
initialization to execution. Microsoft `XSLTFunctions__Pattern-separator` and
three sibling named-format cases clear formatting semantics and reach the
independent bounded ISO-8859-1 serializer's explicit non-ASCII boundary. The
byte-oriented corpus lane moves 15 other cases from string-serialization
failure into comparison; five become exact and one exposes a visible formatting
mismatch rather than a false execution failure. Other named-format suites expose
later formatting limits, while two Microsoft number cases expose their already
independent non-UTF-8 source-document boundary.

The conserved totals are 2,139 initialized cases, 996 initialization failures,
2,060 successful executions, 79 execution failures, 1,917 exact XML-semantic
matches, and 56 mismatches. The strict compatibility lower bound is
`1,917 / 3,173 = 60.42%`.

## Boundaries

- Runtime format names select immutable compiled declarations; they do not
  create resource lookup, mutable global state, or a host callback.
- Static namespace bindings are retained only for QName semantics and are not
  treated as result namespaces.
- Decimal-string arithmetic is confined to finite lexical formatting. General
  XPath arithmetic continues to use its existing typed exact-rational plan.
- Non-UTF-8 source decoding and serialization remain explicit parser and
  serializer boundaries. The ISO-8859-1 serializer still admits only ASCII
  result characters; the harness change does not widen that contract.
- The remaining formatting failures are visible; no permissive fallback or
  partial result is substituted.

## Reproduction

```powershell
cargo test -p fastxslt --all-features format_number_experiment::tests
cargo test -p fastxslt --all-features xslt10_format_number_resolves_a_variable_named_decimal_format
cargo test -p fastxslt --all-features bounded_byte_transform_preserves_the_selected_ascii_compatible_encoding
cargo test -p fastxslt --all-features oasis_actual_decoder_admits_declared_iso_8859_1_bytes
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier unsupported/FXXP1009/FXXP1009
./scripts/measure-oasis-xslt10.ps1 -TraceCase XSLTFunctions__
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
