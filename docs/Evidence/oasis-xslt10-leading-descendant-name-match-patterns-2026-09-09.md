# OASIS XSLT 1.0 leading descendant name match patterns -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the existing typed location-path and document-rooted membership machinery
implement the XSLT 1.0 `match="//name"` abbreviation without admitting general
descendant patterns?

## Implemented slice

The template-pattern compiler now admits only a leading descendant path with
one unqualified named child step and no predicate. It retains the path form,
rather than normalizing it to a bare name test, so the XSLT path-pattern
default priority remains distinct from the bare-QName priority.

The runtime treats both `/`-rooted and leading-`//` path selections as
document-rooted membership. The bounded invocation-owned membership cache from
ADR-0013 can therefore avoid reevaluating the whole document for every
candidate node. The complete charged path evaluator remains the fallback and
differential oracle.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,624 | 1,632 | +8 |
| Executed successfully | 1,453 | 1,461 | +8 |
| Expected-result XML matches | 1,332 | 1,337 | +5 |
| XML comparison mismatches | 94 | 97 | +3 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |
| Generic unsupported match-pattern frontier | 85 | 59 | -26 |

The three newly visible mismatches are not counted as passes:

- `Microsoft/AVTs__77599#1` reaches an existing fragment/namespace
  serialization difference;
- `Microsoft/Namespace__77700#2` reaches an XML-declaration/output-selection
  difference; and
- `Microsoft/Namespace__84473#1` reaches existing stylesheet whitespace
  handling differences.

Targeted traces show that the intended descendant templates were selected in
all three cases. Their downstream differences remain explicit rather than
being attributed to this match-pattern slice.

The strict standard-operation lower bound is now
`1,337 / 2,742 = 48.76%`; the conservative all-catalog ratio is
`1,337 / 3,173 = 42.14%`. These remain local compatibility measurements, not
an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit predicates after the descendant name, multiple
steps, namespace-qualified names, a default XPath namespace, descendant
wildcards beyond the previously admitted exact `//*` form, or a second match
backend. A compiler regression keeps `//foo[@id='1']` unsupported.

## Verification

- A compiler test proves `//foo` retains the typed path representation and
  path-pattern priority while `//foo[@id='1']` remains unsupported.
- A focused runtime test proves nested named descendants are each selected
  exactly once through document-rooted membership.
- The complete 3,173-case local measurement produced the counters above.
- Targeted traces classified every newly visible mismatch.
