# OASIS XSLT 1.0 Local Forward-Compatible Version

Date: 2026-09-29  
Status: Verified implementation and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

The unchanged `Microsoft/BVTs_bvt034#1` case is an XSLT 1.0 stylesheet that
places `xsl:version="2.0"` on a literal result element. Its descendants must
therefore use XSLT 1.0 forward-compatible processing. Two unknown `xsl:new`
instructions each contain `xsl:fallback`, but FastXSLT rejected the first
unknown instruction during compilation.

One `xsl:new` also carries `xsl:version="1.0"`. That attribute does not select
the version of an XSLT instruction in this context, so it must not cancel the
literal ancestor's forward-compatible mode.

## Change

The compiler now recognizes a nearest `xsl:version` on a literal result
ancestor while compiling an exact-version-1.0 stylesheet. A numeric value
greater than `1.0` selects the existing private forward-compatible instruction
path for that subtree. `xsl:version` attributes on XSLT instruction elements do
not override that literal-result control.

The same local mode ignores an invalid optional
`xsl:extension-element-prefixes` value rather than raising its ordinary strict
XSLT 1.0 static error. A focused paired regression proves that the identical
invalid value remains `XTSE1430` outside forward-compatible mode. The broader
unchanged `Microsoft/BVTs_bvt022#1` case still stops earlier at a separate
strict `#default` declaration and receives no pass credit from this behavior.

This does not reinterpret a stylesheet whose root declares XSLT 2.0 or 3.0 as
an XSLT 1.0 processor input. Those supported modern roots retain their existing
strict modern compilation behavior. The previously admitted bounded XSLT 1.x
root interval is also unchanged.

## Corpus result

`Microsoft/BVTs_bvt034#1` now compiles both unknown instructions to their
fallback sequence constructors, executes, and compares exactly.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,443 | 2,444 | +1 |
| Executed successfully | 2,393 | 2,394 | +1 |
| Initialization failures | 727 | 726 | -1 |
| Execution failures | 50 | 50 | 0 |
| Exact expected-result matches | 2,262 | 2,263 | +1 |
| Visible mismatches | 0 | 0 | 0 |
| Comparator gaps | 0 | 0 | 0 |

The conservative all-catalog exact-match ratio is now
`2,263 / 3,173 = 71.32%`. Expected-error credit remains 423 / 431.

## Boundaries

- The outer stylesheet must declare exactly XSLT 1.0.
- Only a literal result element's `xsl:version` selects this local mode.
- Invalid optional extension-prefix controls are ignored only inside that
  selected local mode; strict XSLT 1.0 validation is unchanged.
- Unknown instructions still execute only standard `xsl:fallback` children;
  other content remains ignored.
- Full XSLT 2.0/3.0 roots retain the modern engine semantics and are not
  treated as legacy-processor-profile requests.
- The change introduces no runtime version branch, resource access, or public
  processor-profile control.
- This is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt xslt10_literal_result_version_selects_local_forward_compatible_fallback
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/BVTs_bvt034#1'
./scripts/verify.ps1
```

## Normative reference

- [XSLT 1.0 section 2.5, Forwards-Compatible Processing](https://www.w3.org/TR/xslt-10/#forwards)
