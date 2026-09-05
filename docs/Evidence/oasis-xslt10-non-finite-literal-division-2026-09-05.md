# OASIS XSLT 1.0 Non-Finite Literal Division

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Source checkpoint | `f20ded3` plus the measured implementation |
| Corpus | OASIS XSLT/XPath 1.0 Committee Draft 04 local archive |
| Corpus SHA-256 | `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5` |
| Status | Complete; three doubt-annotated expected-result matches promoted |
| Governing review | [AR-0019](../Architectural%20Reviews/AR-0019-xslt10-compatibility-profile-on-modern-core.md) |

## Compatibility seam

XPath 1.0 arithmetic uses IEEE-style non-finite results for numeric division by
zero. XPath 3.1 gives untyped numeric literals such as `0` the decimal type, for
which division by zero is an error. Treating the two versions identically would
therefore make one standards profile wrong.

The compiler now recognizes only source-free division of two finite decimal
literals by a literal numeric zero while compiling an XSLT 1.0 stylesheet. It
lowers the result directly to the existing literal-string or typed-boolean
representation:

- positive divided by zero becomes `Infinity`;
- negative divided by zero becomes `-Infinity`;
- zero divided by zero becomes `NaN`;
- the XPath 1.0 effective boolean values are true for infinities and false for
  `NaN`.

The compatibility decision is made at compilation. It does not add a version
branch to runtime evaluation, broaden exact rational arithmetic, or admit the
same decimal division under version 3.0. A lifecycle regression proves all five
forms together and proves the equivalent modern stylesheet is rejected.

## Corpus result

The complete 3,173-case sweep changed only the intended frontier:

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,057 | 1,060 | +3 |
| Initialization failures | 2,078 | 2,075 | -3 |
| Executed successfully | 856 | 859 | +3 |
| XML comparison passes | 773 | 776 | +3 |
| XML comparison mismatches | 50 | 50 | 0 |
| Execution failures | 201 | 201 | 0 |

The promoted unchanged cases are:

- `Lotus/boolean_boolean37#1`: `boolean(1 div 0)` produces `true`;
- `Lotus/boolean_boolean38#1`: `0 div 0` serializes as `NaN`;
- `Lotus/boolean_boolean39#1`: `boolean(0 div 0)` produces `false`.

All three identities occur in the suite's doubts file; `boolean39` also carries
an explicit gray-area choice. FastXSLT therefore reports them as matching the
suite's expected results while retaining the qualifications rather than calling
them unqualified conformance evidence.

The measured standard-operation ratio becomes 776 of 2,742 cases, or 28.30%.
This remains compatibility evidence rather than an XSLT 1.0 conformance claim.
