# OASIS XSLT 1.0 Variable-Name Sequence Predicates

Date: 2026-09-23  
Status: Verified semantic and compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the bounded XSLT 1.0 outer-current name machinery compose with a lexical
QName held in an invocation variable while preserving document order, focus,
and the existing modern XPath boundary?

## Implemented slice

Yes. XSLT 1.0 compilation now recognizes three related typed forms:

- `/descendant::*[position() < $position and name() = $name]`;
- `//*[name() = $name]/*`; and
- `//*[name() = $name]/*[position() < $position and
  name() = name(current())]`.

The plans retain only variable identities. Execution uses the existing XSLT
1.0 numeric and string conversions, charged document-order traversal, lexical
QName comparison, and outer-current focus. The modern static context continues
to reject these private compatibility forms.

## Corpus result

Unchanged `Microsoft/Miscellaneous__84425#1` advances from initialization
rejection to successful execution. It remains a visible XML mismatch and gains
no pass credit. The case is listed in the suite's own `doubts.xml`: for the two
`AAA` source elements, the stylesheet expression selects children `BBB`, `EEE`,
and `CCC`, which FastXSLT reports, while the legacy expected file reports only
`BBB`.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,199 | 2,200 | +1 |
| Executed successfully | 2,146 | 2,147 | +1 |
| Expected-result XML matches | 1,999 | 1,999 | 0 |
| XML comparison mismatches | 63 | 64 | +1 |
| Doubt-annotated mismatches | 6 | 7 | +1 |
| Execution failures | 53 | 53 | 0 |
| Comparator unsupported | 75 | 75 | 0 |

The strict complete-catalog lower bound remains
`1,999 / 3,173 = 63.00%`. Generic `FXXP1001` initialization failures fall from
15 to 14. The mismatch is retained rather than changing correct execution to
imitate a questionable legacy expectation.

## Boundaries

- These are typed XSLT 1.0 compatibility plans, not a general XPath predicate
  evaluator.
- Variable values use existing invocation-local conversion and ownership.
- Lexical QName comparison retains prefixes as required by `name()`.
- No namespace-axis, DTD, resource, or public API surface is introduced.
- The archive remains locally acquired and is not redistributed.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_prior_descendant
cargo test -p fastxslt --all-features xslt10_variable_named_parent_children
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Miscellaneous__84425#1'
./scripts/verify.ps1
```
