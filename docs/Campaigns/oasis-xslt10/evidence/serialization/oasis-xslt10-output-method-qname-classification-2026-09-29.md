# OASIS XSLT 1.0 Output-Method QName Classification -- 2026-09-29

Date: 2026-09-29  
Status: Verified diagnostic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Finding

The output compiler classified every method other than the built-in XML, HTML,
XHTML, and text methods as unsupported. That conflated two distinct cases:

- an empty or no-namespace custom method is not an admitted output-method QName;
- a namespace-qualified method is a valid extension point whose serializer is
  outside FastXSLT's current implementation.

XSLT 1.0 defines the extension form as `qname-but-not-ncname`; a prefixed QName
identifies a processor-specific output method. A bare custom NCName therefore
cannot be treated as a valid but merely unavailable extension method.

## Repair

Compilation now resolves every non-built-in output method as an expanded QName.
Malformed QNames remain invalid `XTSE0020`; a custom method with a null namespace
is invalid `XTSE1570`; and a namespace-qualified extension method remains
explicitly unsupported `FXST1004`.

A focused test conserves all three classifications. No extension serializer,
fallback method, or host callback is admitted.

## Corpus result

- `Microsoft/Output__77926#1` now reports invalid `XTSE0020` for its empty
  method.
- `Microsoft/Output__77966#1` now reports invalid `XTSE1570` for bare `foo`.
- Namespace-qualified standard cases remain unsupported rather than being
  mislabeled invalid.

Both corrected cases already received expected-error credit, so the conserved
totals do not move: 2,261 / 3,173 exact matches (71.26%), 2,441 initialized
cases, 2,392 successful executions, and 423 / 431 expected-error credits.

## Boundaries

- This is diagnostic classification, not support for extension output methods.
- FastXSLT does not invoke ambient serializers or grant new host authority.
- This is compatibility evidence from a locally acquired archive, not a formal
  XSLT 1.0 conformance claim.

## Verification

```powershell
cargo test -p fastxslt output_method_distinguishes_invalid_null_namespace_from_unsupported_extension
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Output__77926#1'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Output__77966#1'
./scripts/verify.ps1
```

## Standards source

- [XSLT 1.0, 16 Output](https://www.w3.org/TR/xslt-10/#output)
