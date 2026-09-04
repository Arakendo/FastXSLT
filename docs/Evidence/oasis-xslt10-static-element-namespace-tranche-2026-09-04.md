# OASIS XSLT 1.0 Static Element Namespace Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Archive SHA-256 | `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5` |
| Input baseline | 403 definite unchanged XML passes |
| Result | 452 definite unchanged XML passes |
| Disposition | Static `xsl:element` namespace URI admitted; namespace AVTs remain open |

## Outcome

The compiler now resolves a literal `namespace` value on a static
`xsl:element` name. An unprefixed name with a nonempty URI retains a default
result namespace binding; a prefixed name retains that prefix bound to the
specified URI. An empty URI produces an unnamespaced unprefixed name and is
rejected for a prefixed name. Namespace attribute value templates remain a
structured unsupported boundary.

The complete local OASIS sweep moved as follows:

| Observation | Prior tranche | This tranche | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 635 | 696 | +61 |
| Engine initialization failed | 2,500 | 2,439 | -61 |
| Execution succeeded | 461 | 522 | +61 |
| Execution failed | 174 | 174 | 0 |
| Definite unchanged XML passes | 403 | 452 | +49 |
| XML comparison mismatches | 32 | 41 | +9 |
| Comparator unsupported | 15 | 16 | +1 |

Two expected-error cases also move from initialization failure to unexpected
execution success and remain uncredited. The strict standard-operation lower
bound is now **452 / 2,742 = 16.48%**; the complete-catalog lower bound is
**452 / 3,173 = 14.25%**. These remain compatibility measurements rather than
conformance claims.

Unchanged `Lotus/namespace_namespace25#1` exercises nested prefixed computed
elements whose explicit namespace URI changes while retaining the lexical
prefix and passes XML comparison. Focused compiler and runtime tests separately
prove an unprefixed computed element retains and serializes its default result
namespace.

## Architectural conservation

- QName and namespace selection happens once during compilation.
- Execution continues through the existing computed-element marker and shared
  element-construction instruction; there is no runtime version branch.
- The retained binding uses the existing namespace-fixup and serializer path.
- Dynamic name and namespace AVTs, attribute sets, reserved-prefix rules beyond
  the admitted static forms, and broader QName compatibility remain explicit.

## Newly visible obligations

Nine additional cases now reach comparison mismatches and one reaches the
existing comparator boundary. Several concern namespace inheritance/fixup and
must be reviewed against XSLT 1.0 versus modern namespace construction rules;
they are not evidence that the compiler should mimic reference bytes. The
remaining work must distinguish genuine edition-sensitive behavior from
ordinary shared namespace semantics before widening this slice.
