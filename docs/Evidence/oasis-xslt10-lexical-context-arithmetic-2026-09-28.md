# OASIS XPath 1.0 Lexical-Context Arithmetic -- 2026-09-28

Date: 2026-09-28  
Status: Verified semantic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

The unchanged Xalan/Lotus `select30`, `select34`, `select35`, and `select38`
cases exercise XPath 1.0 lexical context rather than four independent
arithmetic features:

- `(* - 4)**` uses the first and final `*` tokens as child-element wildcards
  while the middle `*` is multiplication;
- `25-*` uses `-` as an operator and `*` as the child-element wildcard;
- `54 div*` uses `div` as an operator before that wildcard without intervening
  whitespace; and
- `' 6 '*div` uses `*` as an operator, a string literal converted to a number,
  and `div` as a child-element name.

FastXSLT's typed numeric tree already represented subtraction, multiplication,
division, exact rational literals, and location paths. Its bounded splitter
required whitespace after the named operators and its numeric leaf compiler
did not admit the XPath 1.0 string-to-number rule, so the cases stopped before
execution under the generic location-path frontier.

## Change

The private numeric splitter now recognizes `div` and `mod` when their right
boundary cannot continue an XPath name, does not reinterpret a terminal
wildcard operand as a multiplication operator, and does not reinterpret a
right-hand element named `div` or `mod` as a second operator. The typed numeric
compiler now:

- parses wildcard operands through the existing charged location-path plan;
- converts a quoted finite decimal to the existing exact-rational leaf only
  when XSLT 1.0 compatibility was selected during compilation; and
- retains the modern mode's rejection of the same string-numeric expression.

Execution still uses the ordinary typed tree, source navigation, document-order
selection, work charging, cancellation, and structured diagnostics. There is no
runtime stylesheet-version branch.

## Corpus result

All four unchanged lexical-context cases now execute and compare exactly.
Unchanged `math74` and `math82`, whose operands contain elements literally
named `div` and `mod`, remain exact regression controls. The authoritative
all-catalog counters are:

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,417 | 2,421 | +4 |
| Executed successfully | 2,367 | 2,371 | +4 |
| Initialization failures | 753 | 749 | -4 |
| Execution failures | 50 | 50 | 0 |
| Exact expected-result matches | 2,207 | 2,211 | +4 |
| Visible mismatches | 9 | 9 | 0 |

The conservative all-catalog exact-match ratio is now
`2,211 / 3,173 = 69.68%`. Expected-error credit remains 423 / 431 and
comparator gaps remain 52. The ordinary standard-operation
`location-path-shape:wildcard` frontier is empty; the malformed expected-error
probe remains visibly rejected.

## Boundaries

- This is a lexical-context and typed-operand repair, not a general XPath
  tokenizer or a new expression backend.
- Only finite decimal string literals enter the exact-rational leaf.
- XPath 1.0 string-to-number compatibility is selected during compilation and
  does not alter modern XPath typing.
- The result is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Verification

```powershell
cargo test -p fastxslt --all-features splits_only_one_bounded_top_level_operator
cargo test -p fastxslt --all-features compiles_mixed_path_literal_arithmetic_tree
cargo test -p fastxslt --all-features xslt10_binary_numeric_uses_lexical_context_for_wildcards_and_operator_names
./scripts/measure-oasis-xslt10.ps1
```

## Normative references

- [XPath 1.0 section 3.7, Lexical Structure](https://www.w3.org/TR/1999/REC-xpath-19991116/#exprlex)
- [XPath 1.0 section 3.5, Numbers](https://www.w3.org/TR/1999/REC-xpath-19991116/#numbers)
