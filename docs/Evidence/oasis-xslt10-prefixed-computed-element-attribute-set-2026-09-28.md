# OASIS XSLT 1.0 Prefixed Computed-Element Attribute Set -- 2026-09-28

Date: 2026-09-28  
Status: Verified compatibility and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

Three unchanged Microsoft cases put `xsl:use-attribute-sets` on an
`xsl:element` instruction. The standard instruction attribute is unqualified,
so FastXSLT correctly rejected the namespaced attribute. The suite's own
`doubts.xml` marks all three cases as gray-area
`xsl-prefixed-attrib-on-xsl-instruction` cases and selects the `ignore`
behavior. Their expected results therefore omit the named attribute set from
the computed element.

## Change

The XSLT 1.0 compatibility compiler tolerates and ignores exactly
`xsl:element/@xsl:use-attribute-sets`. It does not resolve or apply the named
set. The standard unqualified `use-attribute-sets` spelling continues through
the existing attribute-set linker and runtime application path.

The rule is compile-selected and limited to an actual XSLT `element`
instruction under XSLT 1.0 compatibility. It does not generally ignore
XSLT-namespaced attributes, weaken modern static contexts, or treat the
prefixed spelling as standards-conforming syntax.

## Corpus result

Unchanged `Microsoft/AttributeSets__91080#1`,
`Microsoft/AttributeSets__91081#1`, and
`Microsoft/AttributeSets__91083#1` initialize, execute, and compare exactly
under the suite-declared gray-area behavior.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,427 | 2,430 | +3 |
| Executed successfully | 2,377 | 2,380 | +3 |
| Initialization failures | 743 | 740 | -3 |
| Execution failures | 50 | 50 | 0 |
| Exact XML expected-result matches | 2,216 | 2,219 | +3 |
| XSLT-Result-Tree exact matches | 1,591 | 1,594 | +3 |

The conservative exact-match ratio is `2,219 / 3,173 = 69.93%`.
Expected-error credit remains 423 / 431, comparator gaps remain 53, and XML
mismatches remain nine. The cases retain their doubt metadata; this evidence
does not convert the suite's chosen recovery behavior into an XSLT 1.0
conformance claim.

## Verification

A focused runtime test proves that the prefixed spelling is ignored while the
existing focused test continues to prove that the unqualified spelling applies
the set. The full corpus sweep then verifies the three unchanged archival
cases.

```powershell
cargo test -p fastxslt --all-features xslt10_prefixed_computed_element_attribute_set_attribute_is_ignored
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/AttributeSets__91083#1'
./scripts/verify.ps1
```

## Normative references

- [XSLT 1.0 section 7.1.2, Creating Elements with `xsl:element`](https://www.w3.org/TR/1999/REC-xslt-19991116#creating-elements-with-xsl-element)
- [XSLT 1.0 section 7.1.4, Named Attribute Sets](https://www.w3.org/TR/1999/REC-xslt-19991116#attribute-sets)
