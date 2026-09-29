# OASIS XSLT 1.0 Bounded HTML Comparison

Date: 2026-09-28  
Status: Verified local-harness evidence; not an engine or public comparator contract  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

The conserved sweep had 54 successfully executed cases whose result could not
be classified by the XML document/fragment comparator. Many selected the XSLT
1.0 HTML output method and differed from XML in ordinary lexical ways: ASCII
case-insensitive element and attribute names, omitted end slashes on void
elements, boolean or unquoted attributes, raw script/style text, and HTML-style
processing-instruction termination.

Those cases could not receive pass credit merely because they looked
plausible, but leaving all of them as unparseable also hid real matches and
real serialization mismatches.

## Change

The local OASIS measurement harness now applies a private bounded lexical
normalizer only when the compiled output method is HTML. It:

- folds markup names to ASCII lowercase while preserving attribute values;
- converts the finite HTML 4 void-element set to XML empty-element syntax;
- quotes unquoted attributes and gives boolean attributes a deterministic
  value on both sides of the comparison;
- folds only the encoding token in an HTML Content-Type `meta` value to ASCII
  lowercase, reflecting the case-insensitive encoding-name comparison without
  changing unrelated attribute values;
- escapes raw script/style text for XML parsing; and
- converts HTML processing-instruction termination to XML syntax.

The normalized text is then parsed by the existing bounded XML adapter and
compared by the existing expanded-name/tree comparator. The engine result tree,
serializer, expected corpus bytes, and non-HTML comparison paths are unchanged.

## Conserved result

The former 54-case comparator frontier is now partitioned as follows:

| Disposition | Cases |
| --- | ---: |
| Exact semantic comparison pass | 19 |
| Visible comparison mismatch | 15 |
| Still unsupported by the comparator | 20 |
| Total | 54 |

The strict exact-match lower bound rises from 2,229 to **2,248 / 3,173
(70.85%)**. Initialization and execution remain 2,440 and 2,392. Expected-error
credit remains 423 / 431.

The 15 mismatches receive no credit. The remaining 20 gaps include HTML named
character references, optional-tag/tree-recovery behavior, non-HTML malformed
fragments, and four reference encodings not admitted by the current decoder.

## Boundaries

- This is not an HTML parser, HTML DOM, or HTML conformance claim.
- It does not make HTML output byte-exact or conceal serializer defects.
- It does not decode the full HTML named-character-reference vocabulary or
  infer optional element closures.
- It is compiled only for the local test/workbench measurement configuration.
- An unnormalizable or unparseable result remains explicitly unsupported; a
  parsed unequal result remains an explicit mismatch.

## Verification

```powershell
cargo test -p fastxslt --all-features oasis_html_comparator -- --nocapture
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
