# OASIS XSLT 1.0 Conditional Static Introspection

Date: 2026-09-19  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Finding

FastXSLT already folded namespace-aware `system-property()`,
`function-available()`, and `element-available()` calls used as values. The
same calls were not admitted as `xsl:if` or `xsl:when` tests, so two otherwise
executable cases stopped at the generic conditional-expression frontier.

## Repair

- Reuse the existing namespace-aware introspection owner when compiling XSLT
  1.0 conditional tests.
- Fold direct introspection effective boolean values.
- Fold bounded `contains(system-property(...), 'literal')` tests.
- Fold exact numeric comparisons of a static system-property result using
  `=`, `!=`, `<`, `<=`, `>`, or `>=`.
- Lower the result to the ordinary constant boolean instruction plan; no
  invocation-time reflection, ambient host inspection, or alternate evaluator
  is introduced.

Focused tests cover namespace-aware direct availability, version comparison,
and vendor-URL containment. A production transform exercises the forms through
`xsl:if` and `xsl:choose`.

## Corpus result

Two unchanged cases move through compilation and execution:

- initialized: 2,031 -> 2,033;
- successfully executed: 1,930 -> 1,932;
- exact XML-semantic matches: unchanged at 1,803;
- visible XML mismatches: 53 -> 54.

The mismatch increase is intentional and informative. The Lotus
`extend_extend03#1` reference expects `document()` to be reported available,
but FastXSLT does not yet provide that function and therefore reports `false`.
The processor-information case similarly contains implementation-specific
expectations. Executing introspection must reveal FastXSLT's actual admitted
capabilities and identity; impersonating the reference processor would turn a
corpus number into a false product claim.

## Boundary conclusion

This admits compile-time introspection in conditional position. It does not
admit `document()`, extension discovery, dynamic introspection arguments, or a
host capability callback. Capability answers remain tied to the implemented
engine surface.
