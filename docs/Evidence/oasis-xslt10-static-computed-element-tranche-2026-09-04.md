# OASIS XSLT 1.0 Static Computed Element Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Archive SHA-256 | `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5` |
| Baseline | 366 definite unchanged XML passes |
| Result | 386 definite unchanged XML passes |
| Disposition | Shared modern primitive implemented; broader `xsl:element` remains open |

The subsequent
[location-path copy-of tranche](oasis-xslt10-location-path-copy-of-tranche-2026-09-04.md)
raises the running definite-pass lower bound to 395.
The later
[static element namespace tranche](oasis-xslt10-static-element-namespace-tranche-2026-09-04.md)
admits literal namespace URIs and raises the running bound to 452.

## Outcome

FastXSLT now compiles and executes the first bounded `xsl:element` slice through
the same result-construction instruction and runtime used by literal result
elements. Static unprefixed NCNames construct no-namespace elements. Static
prefixed QNames resolve their in-scope stylesheet namespace at compile time and
retain the required result namespace binding.

The complete local OASIS sweep moved as follows:

| Observation | Initial sweep | This tranche | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 506 | 540 | +34 |
| Engine initialization failed | 2,629 | 2,595 | -34 |
| Execution succeeded | 410 | 436 | +26 |
| Execution failed | 96 | 104 | +8 |
| Definite unchanged XML passes | 366 | 386 | +20 |
| XML comparison mismatches | 20 | 25 | +5 |
| Comparator unsupported | 14 | 15 | +1 |

The strict standard-operation lower bound is therefore **386 / 2,742 =
14.08%**. Against the complete 3,173-case catalog it is **12.17%**. These are
compatibility measurements, not conformance claims. Cases newly reaching a
later failure or comparison outcome remain visible rather than being credited
as passes.

Unchanged `Lotus/lre_lre14#1` is a direct corpus sentinel for the admitted
shape. Its stylesheet constructs `all` with `xsl:element`, executes a nested
`xsl:for-each`, and produces `<out><all>XYZ</all></out>` through production
compilation, execution, serialization, and comparison.

## Architectural conservation

- The compiler selects the static name and namespace once. Execution does not
  branch on stylesheet version or reparse the QName.
- The new form lowers to the existing element-construction operation. It does
  not introduce a legacy backend, a second result model, or an alternate
  runtime path.
- A private constructor-origin marker lets semantic inspection distinguish a
  computed element from a literal result element without exposing the private
  instruction representation.
- Child sequence construction, computed attributes, result-node work charging,
  cancellation, namespace fixup, diagnostics, and serialization continue
  through their existing shared machinery.
- Focused tests cover static unprefixed and prefixed QNames, namespace retention,
  nested execution, and explicit rejection of unadmitted forms.

## Deliberately open forms

This initial tranche did not admit an `xsl:element` `namespace` attribute, a
dynamic attribute value template in `name` or `namespace`, or
`use-attribute-sets`. The later static-namespace tranche admits literal URI
values; dynamic name/namespace AVTs and attribute sets remain structured
unsupported outcomes.

The mismatch review performed before this implementation also confirmed that
the archival suite needs an owned infoset/canonical comparison policy. Several
of the existing 20 mismatches and 14 comparator gaps involve DTD-bearing output,
legacy encodings, indentation, HTML serialization, or reference-output
conventions. FastXSLT must not tune engine semantics to those bytes before the
comparison obligation is classified.

## Next work

Continue the shared-primitive campaign in dependency order. A bounded
node-selecting `xsl:copy-of` location-path slice was selected next and is
recorded separately. Broader `xsl:element` namespace/AVT behavior should land
only with explicit QName, namespace, and attribute-value-template semantics
rather than as a corpus-specific shortcut.
