# OASIS XSLT 1.0 wildcard numeric match predicates -- 2026-09-13

Date: 2026-09-13  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can exact integer comparisons against a wildcard element's string value or one
unqualified attribute use explicit XPath 1.0 numeric conversion?

## Implemented slice

The compiler recognizes `*[.=integer]` and `*[@name=integer]` as distinct
private typed patterns. Runtime matching converts the relevant node string value
through the existing XPath number parser rather than comparing lexical text.
The element conversion consumes an XPath-operation charge; attribute matching
charges each inspected attribute plus the conversion of a matching name.
Source and temporary result-tree execution implement the same semantics.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,662 | 1,663 | +1 |
| Executed successfully | 1,491 | 1,492 | +1 |
| Expected-result XML matches | 1,361 | 1,362 | +1 |
| XML comparison mismatches | 103 | 103 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |
| Generic unsupported match-pattern frontier | 26 | 25 | -1 |

The newly executed case matches its unchanged archival expected XML. A second
catalog identity advanced beyond its first numeric predicate but revealed a
later, substantially broader composed predicate in the same stylesheet module;
that identity therefore remains an initialization failure and contributes no
pass.

The strict standard-operation lower bound is now
`1,362 / 2,742 = 49.67%`; the conservative all-catalog ratio is
`1,362 / 3,173 = 42.92%`. These remain local compatibility measurements, not
an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit decimal/exponent pattern literals, relational or
inequality comparisons, arithmetic predicates, variables, boolean composition,
multiple predicates, namespaces, or general XPath expressions. It does not
change template ordering, resource authority, prepared-input ownership, or
invocation state.

## Verification

- Compiler coverage proves both numeric forms retain typed integer operands.
- Runtime coverage uses lexically different but numerically equal element and
  attribute values across source and temporary trees.
- The complete 3,173-case local measurement produced the counters above.
