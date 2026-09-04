# OASIS XSLT 1.0 Qualified Attribute Path Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 657 definite unchanged XML passes; 931 initialized cases |
| Result | 662 definite unchanged XML passes; 937 initialized cases |
| Disposition | Shared expanded-name path mechanics; not a conformance claim |

## Change

The typed path model now represents an attribute name test by expanded name,
alongside its existing expanded-name child step. The narrow qualified-path
parser can therefore resolve and lower paths such as
`doc/p:item/@q:attribute` without confusing the attribute marker with the QName
prefix. Runtime matching compares the retained expanded name and continues to
charge normal path navigation.

This is shared XPath/XDM mechanics rather than XSLT 1.0 compatibility behavior.
It does not reconstruct lexical namespace prefixes for `fn:name`, broaden the
path parser beyond its admitted shapes, or introduce a second evaluator.

## Unchanged cases

Five cases move directly from initialization failure to definite XML passes:

- `Lotus/string_string30#1` and `string31#1` exercise `local-name()` and
  `namespace-uri()` over qualified attribute paths;
- `Lotus/string_string35#1` exercises an empty qualified attribute selection;
- `Lotus/namespace_namespace09#1` exercises a selected qualified attribute's
  local name;
- `Lotus/namespace_namespace11#1` exercises namespace URI selection across
  qualified child and attribute paths.

`Lotus/string_string36#1` now initializes and reaches the deliberate
`FXRT1008` boundary because `fn:name` requires a retained lexical prefix for
the selected namespaced attribute. It remains visibly uncredited.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 931 | 937 | +6 |
| Execution succeeded | 738 | 743 | +5 |
| XML comparison passes | 657 | 662 | +5 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 19 | 19 | 0 |
| Execution failures | 193 | 194 | +1 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **662 / 2,742 = 24.14%** of standard-operation
cases and **662 / 3,173 = 20.86%** of the complete catalog.

## Verification

A first-party path test resolves distinct element and attribute prefixes and
selects only the matching namespaced attribute. The complete local OASIS sweep
confirms the five intended passes, keeps the lexical-prefix boundary visible,
and introduces no mismatch, comparator gap, expected-error leak, or panic.
