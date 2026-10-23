# XSLT30 `castable-002` Typed Local Values

Date: 2026-08-26

## Native case

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/expr/castable/_castable-test-set.xml`
- Case: `castable-002`
- Dependency: `XSLT20+`
- Environment/source: `castbl01` / `castbl01.xml`
- Stylesheet: `castable-002.xsl`
- Assertion: file-backed `castable-002.out`

The unmodified stylesheet declares 12 local variables. Each select expression
atomizes a source element and explicitly casts it to its corresponding built-in
type. The stylesheet then checks every typed value against `xs:string` and
`xs:untypedAtomic`, producing 24 true results. FastXSLT matches the upstream
file-backed XML assertion exactly.

## Runtime value and scope behavior

The private XDM-owned atomic value retains two distinct facts:

```text
runtime atomic value = built-in type identity + retained lexical content
```

XPath `cast as` validates the atomized untyped source using the lexical rules
established for `castable-001`, then constructs an XDM-owned typed value. It
does not erase type identity by storing only a Rust string. Conversely, this
slice does not choose a final physical representation for integers, decimals,
floating-point values,
durations, dates, or times.

Local `xsl:variable` bindings live in an invocation-local frame. A binding is
visible to following instructions and nested sequence construction, while a
nested sequence receives an isolated frame copy. Duplicate bindings in the same
compiled sequence are rejected rather than silently overwritten. Existing
named-template string parameters use the same private value container without
changing their prior behavior.

Twelve casts and 24 variable castability checks consume the XPath-operation
domain independently. Source navigation and atomization retain their existing
XPath-node-visit and XDM-string-value charges.

## Conservation

At this checkpoint, the complete nine-case denominator remained seven selected
and two schema-aware profile exclusions. Selected execution advanced to two
passes, two engine-unsupported cases, and three harness-unsupported cases. No
case failed or disappeared. The later
[`castable-003` evidence](xslt30-castable-003-numeric-conversion-matrix-2026-08-26.md)
advances that same conserved denominator.

## Claim boundary

At this checkpoint, the evidence did not establish a public atomic-value API,
canonical lexical serialization, arithmetic over typed values, cross-numeric
conversion, precision or range behavior, general variable select expressions,
global variables, variable sequences, external parameters, or general static
type checking. Variable castability was intentionally limited to the two native
targets required here; the later `castable-003` slice adds a narrow conversion
matrix instead of guessing compatibility from retained strings.
