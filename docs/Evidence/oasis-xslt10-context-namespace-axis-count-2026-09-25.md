# OASIS XSLT 1.0 Context Namespace-Axis Count

- Date: 2026-09-25
- Status: Verified compatibility evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Unchanged Lotus `axes129` evaluates `count(namespace::*)` in an attribute value
template while the dynamic focus is each source attribute. XPath 1.0 defines
the namespace axis only for element context nodes, so each count must be zero.
FastXSLT previously rejected the AVT at generic `FXST1031`.

## Implemented slice

The XSLT 1.0 AVT compiler now retains the exact zero-argument context namespace
count as a typed value. Runtime evaluation charges an XPath operation and:

- returns zero for source nodes other than elements;
- counts the element's effective non-empty namespace bindings; and
- includes the implicit `xml` namespace when it is not already represented.

The value requires a source-node focus and does not manufacture namespace nodes
in the private XDM representation. Modern XPath parsing is unchanged.

## Verification and corpus disposition

A focused regression proves both sides of the context rule: an element with
one declared namespace reports that binding plus `xml`, while its unqualified
and qualified attributes each report zero.

Unchanged `Lotus/axes_axes129#1` now initializes, executes, and compares
exactly. The complete sweep reaches **2,322 initialized**, **2,274 successfully
executed**, and **2,108 / 3,173 exact matches (66.44%)**. Initialization
failures fall to 813; execution failures, XML mismatches, and comparator
exclusions remain unchanged. The generic `FXST1031` frontier is reduced to the
separate qualified computed-attribute case.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_namespace_axis_count_is_empty_for_attribute_focus
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/axes_axes129#1'
```
