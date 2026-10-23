# OASIS XSLT 1.0 `xml:lang` Expanded-Attribute Path

Date: 2026-09-20  
Status: Local compatibility evidence

## Question

Can the shared location-path evaluator select the predeclared `xml:lang`
expanded attribute through an ancestor-or-self predicate and following
attribute step without treating the lexical prefix as an unqualified name?

## Method

- Admit exact `@xml:lang` and `attribute::xml:lang` name tests as expanded
  attributes in `http://www.w3.org/XML/1998/namespace`.
- Apply the same expanded-name identity to attribute-presence predicates and
  result attribute steps.
- Preserve charged ancestor and attribute traversal, document-order
  normalization, first-node XSLT 1.0 conversion, and cancellation observation.
- Add a focused source with inherited and nearer `xml:lang` declarations, then
  run unchanged Lotus `expression_expression06` from the hash-verified local
  OASIS XSLT 1.0 CD04 archive.

## Result

The focused path selects both expanded attributes in document order. The
unchanged corpus case initializes, executes, and exactly matches its expected
XML result.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,119 | 2,120 | +1 |
| Initialization failures | 1,016 | 1,015 | -1 |
| Executed successfully | 2,030 | 2,031 | +1 |
| Execution failures | 89 | 89 | 0 |
| Exact XML-semantic matches | 1,899 | 1,900 | +1 |
| XML comparison mismatches | 55 | 55 | 0 |

The strict compatibility lower bound is now `1,900 / 3,173 = 59.88%`.
The generic `FXXP1001` initialization frontier falls from 27 to 26 cases.
This is local compatibility evidence against a non-redistributed archival
suite, not a broad conformance claim.

## Boundaries

- Only the standard predeclared `xml:lang` expanded name is added to the shared
  unqualified path grammar in this slice.
- General lexical QName resolution inside arbitrary path predicates still
  belongs to namespace-aware compilation and remains unsupported here.
- No prefix string is used as semantic identity, and no namespace is inferred
  from source spelling at execution.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xml_lang_attribute_steps_preserve_expanded_name_and_document_order
./scripts/measure-oasis-xslt10.ps1 -TraceCase expression_expression06
./scripts/verify.ps1
```
