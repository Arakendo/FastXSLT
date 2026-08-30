# XSLT30 `include-0301` Single Import and Repeated Apply-Imports

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/decl/include/_include-test-set.xml`
- Case: `include-0301`
- Principal stylesheet: `include-0301.xsl`
- Imported stylesheet: `include-0301a.xsl`
- Source: inline `<doc/>`

## Execution

The host-facing test admits the source, principal stylesheet, and imported
stylesheet bytes into one bounded resource set and seals it before compilation.
The relative `include-0301a.xsl` reference resolves only against that snapshot.
No filesystem handle, network access, or live resolver survives admission.

Compilation records the imported `doc` rule below the principal `doc` rule by
import precedence. Template priority and declaration order remain independent
rank dimensions. Each of the principal rule's three `xsl:apply-imports`
instructions selects and executes the lower-precedence imported rule afresh,
producing one `foo` element inside each numbered `baz` element.

## Result

| Case | Expected | Actual | Disposition |
| --- | --- | --- | --- |
| `include-0301` | `<bat><baz id="1"><foo/></baz><baz id="2"><foo/></baz><baz id="3"><foo/></baz></bat>` | XML-equivalent result | passed |

The comparator parses both results and compares XDM element, attribute, value,
and child structure, so the serializer's expanded `<foo></foo>` spelling is
correctly equivalent to the suite's empty-element syntax. The conserved
16-case include denominator now has 3 passes and 13 visible not-run
dispositions.

## Claim boundary

This evidence admits one relative `xsl:import`, two matching source-element
rules at distinct import precedence, and repeated parameter-free
`xsl:apply-imports`. It does not admit multiple or nested dependencies,
import/include composition, imported root or named templates, imported global
bindings, apply-imports parameters, or general import-precedence graphs.

The adjacent `include-0202` was initially left unselected because its imported
rule also required computed `xsl:attribute`. That separate feature subsequently
earned a bounded prepared representation and executable corpus evidence; it
does not widen this case's claim boundary.

This case also activates ADR-0011's import revisit pressure. No host framing or
ABI change is required: its already accepted one-dependency operation carries
logical identity and bytes without encoding whether the principal declaration
is `xsl:include` or `xsl:import`. This evidence therefore retains that bounded
framing while leaving dependency collections and general import graphs under
AR-0014.
