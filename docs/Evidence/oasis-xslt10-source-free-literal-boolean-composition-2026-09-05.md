# OASIS XSLT 1.0 Source-Free Literal Boolean Composition

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Source checkpoint | `12678da` plus the measured implementation |
| Corpus | OASIS XSLT/XPath 1.0 Committee Draft 04 local archive |
| Corpus SHA-256 | `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5` |
| Status | Complete; three unchanged standard cases promoted to definite passes |
| Governing review | [AR-0019](../Architectural%20Reviews/AR-0019-xslt10-compatibility-profile-on-modern-core.md) |

## Shared semantic slice

The typed source-free boolean evaluator already supported `and`, `or`,
short-circuiting, and effective boolean values for singleton string and numeric
literals. Its compiler activation predicate previously recognized only
expressions containing a named boolean function. Consequently, compositions
such as `'foo' and 'fop'` fell through to location-path parsing.

The predicate now also recognizes top-level whitespace-delimited `and` and `or`
operators and routes them into the existing parser. It does not recognize a
path name such as `child::and`, and unsupported compositions still fail in the
typed parser rather than becoming paths. The implementation is shared by XSLT
1.0 and modern stylesheets; a version 3.0 lifecycle sentinel produces
`true|true|false` for the three measured shapes.

## Corpus result

The complete local sweep changed only the intended frontier:

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,054 | 1,057 | +3 |
| Initialization failures | 2,081 | 2,078 | -3 |
| Executed successfully | 853 | 856 | +3 |
| XML comparison passes | 770 | 773 | +3 |
| XML comparison mismatches | 50 | 50 | 0 |
| Execution failures | 201 | 201 | 0 |

The promoted unchanged cases are:

- `Lotus/boolean_boolean20#1`: two non-empty strings joined by `and`;
- `Lotus/boolean_boolean23#1`: the non-empty strings `'1'` and `'0'` joined by
  `and`;
- `Lotus/boolean_boolean27#1`: numeric zero and the empty string joined by `or`.

The strict standard-operation lower bound becomes 773 of 2,742 cases, or
28.19%. This remains compatibility evidence rather than a conformance claim.

