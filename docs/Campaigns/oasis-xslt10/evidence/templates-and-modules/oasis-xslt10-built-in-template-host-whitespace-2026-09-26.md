# OASIS XSLT 1.0 Built-In-Template Host Whitespace

- Date: 2026-09-26
- Status: Verified non-pass corpus classification
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Microsoft case `BVTs_bvt056` supplies an otherwise empty XSLT 1.0 stylesheet:
it declares output properties but no template rules and no whitespace-stripping
declaration. Transformation therefore proceeds entirely through the standard
built-in template rules.

FastXSLT preserves and emits every source text node, including whitespace-only
nodes between elements. The catalog-selected result omits those nodes while
retaining indentation that is part of mixed content, such as the whitespace
around `publication`, `details`, and story text. That shape is consistent with
a historical host parser configured to discard whitespace-only source nodes
before the transformation. It is not behavior established by the stylesheet.

## Disposition

The case receives the exact `host-parser-policy-excluded` disposition. FastXSLT
does not add ambient parser whitespace removal to the built-in template rules.
The case remains visible in the conserved 3,173-case denominator and receives
no pass credit.

## Measurement effect

The unchanged sweep remains at 2,342 initialized cases, 2,291 successful
executions, and 2,126 exact XML comparisons out of 3,173 (67.00%). Visible XML
mismatches fall from 12 to 11, and host-parser-policy exclusions rise from 20
to 21.

## Verification

```powershell
cargo test -p fastxslt --all-features oasis_host_parser_whitespace_policy_exclusion_is_exact_and_bounded
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/BVTs_bvt056#1'
./scripts/verify.ps1
```
