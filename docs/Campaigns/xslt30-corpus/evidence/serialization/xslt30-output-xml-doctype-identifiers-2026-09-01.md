# XSLT30 XML DOCTYPE identifiers -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 case `output-0311` and satisfies all
three pinned `serialization-matches` assertions. The XML output contains its
declaration, an empty `a` document element, and this doctype:

```xml
<!DOCTYPE a PUBLIC "ABC'DEF" 'ABC"DEF'>
```

The public identifier is double-quoted because its value contains an apostrophe;
the system identifier is single-quoted because its value contains a quotation
mark. Both retained values reach serialization unchanged.

## Bounded serializer change

The existing doctype serializer now derives the lexical name of the sole XML
document element from its retained namespace bindings instead of requiring an
XHTML `html` element. XHTML still requires the XHTML `html` expanded name.
External identifiers containing both quote forms remain explicitly unsupported
as `FXSR1008`, and the existing document-shape, byte-budget, and XML parameter
validation still runs before emission.

For this admitted XML doctype path, an empty document element uses its
empty-element tag. Other established XML and XHTML byte shapes are unchanged.

## Denominator movement

The complete output denominator moves from 109 to 110 passes and from 123 to
122 visible default not-run cases. Across the eleven conserved XSLT30
denominators, the total moves from 307 to 308 passes, with 3 engine unsupported
cases, 50 profile exclusions, and 170 visible default not-run cases.
