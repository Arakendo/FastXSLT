# QT3 Document-Aware Effective-Boolean-Value Tranche

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- Ten unchanged cases from the complete `fn/not.xml` and `fn/boolean.xml`
  denominators.
- The catalog-owned `atomic` and `auction` context documents, imported into a
  bounded sealed resource snapshot before parsing.

## Method

A private document-aware EBV operation composes the existing atomic-value and
location-path evaluators. It retains XPath item order across parenthesized
mixed sequences and applies the standard boundary exercised by these cases:

- an empty sequence has effective boolean value false;
- a non-empty sequence whose first item is a node has effective boolean value
  true;
- a supported singleton atomic value uses its atomic EBV;
- an atomic-first sequence with another item reports `FORG0006`.

The path evaluator also admits the standard child name test `*:local`, matching
the local name independently of namespace. The new `string(false())` path
projects the boolean to the non-empty string `false`; it therefore has EBV
true rather than the boolean value false.

The executable cases are:

- `fn-not-22`, `fn-not-23`, `fn-not-28`, and `fn-not-29`;
- `boolean-001`, `boolean-002`, `boolean-003`, `boolean-004`, `boolean-008`,
  and `boolean-009`.

Every successful path evaluates the unchanged native assertion. Both invalid
mixed sequences verify the native `FORG0006` code. XPath operation work is
charged in every case, and document navigation retains its existing node-visit
charges.

## Result

| Test set | Added passes | Current passes | Profile excluded | Visible default not run |
| --- | ---: | ---: | ---: | ---: |
| `fn/not.xml` | 4 | 78 | 3 | 2 |
| `fn/boolean.xml` | 6 | 128 | 5 | 10 |
| **Combined** | **10** | **206** | **8** | **12** |

The audited QT3 denominator remains exactly 1,000 cases and now contains 757
passes, 191 profile exclusions, and 52 visible default not-run cases.

## Boundary

This tranche does not admit invocation-clock functions, general sequence
operations such as `remove`, higher-order function items, maps, arrays, or
general FLWOR evaluation. The context documents are harness inputs, not ambient
filesystem authority, and no source bytes enter compiled static state.
