# OASIS XSLT 1.0 Static-Error Honesty

Date: 2026-09-23  
Status: Local implementation and compatibility evidence

## Question

Can FastXSLT reduce silent acceptance and unsupported/invalid confusion in the
OASIS XSLT 1.0 error cases without converting questionable archival metadata
into engine behavior?

## Method

- Reject attributes in the XSLT namespace on `xsl:stylesheet` with `XTSE0090`.
  XSLT-defined elements may ignore foreign namespaced attributes, but an
  unrecognized attribute in the XSLT namespace is a static error.
- Classify empty and malformed template mode names as `XTSE0550`, not an
  unsupported engine capability.
- Enforce the XSLT 1.0 single-QName mode shape while preserving XSLT 3.0 mode
  lists and their reserved tokens.
- Reject duplicate modern mode tokens and `#all` combined with another token.
- Classify forbidden meaningful children of `xsl:apply-templates`,
  `xsl:apply-imports`, and `xsl:call-template` as `XTSE0010`, not as an
  unsupported private compiler slice.
- Reject an XSLT instruction used as the document element of a purported
  simplified stylesheet as `XTSE0010`. Simplified syntax requires a literal
  result element; this is invalid stylesheet structure, not an unimplemented
  module form.
- Inspect all seven prior expected-error/unexpected-success cases before
  changing semantics. Several use apparently stale or contradictory catalog
  expectations, so they remain visible rather than being made to fail merely
  to improve a counter.

The normative references used for the classification are the
[XSLT 1.0 stylesheet rules](https://www.w3.org/TR/xslt-10/) and the
[XSLT 3.0 mode-list rule](https://www.w3.org/TR/xslt-30/#modes), including
`XTSE0090` and `XTSE0550`.

## Result

`Microsoft/AttributeSets__91101#1` now reports initialization failure
`XTSE0090` at `xsl:use-attribute-sets` on the stylesheet root. The
expected-error/unexpected-success count falls from seven to six, and observed
expected initialization errors rise from 393 to 394.

All ten former `unsupported/FXST1012` mode cases now report
`invalid/XTSE0550`. This changes classification rather than executable
coverage: the cases were already rejected, but are no longer misrepresented as
missing engine functionality.

All eight former `unsupported/FXST1014` template-invocation child cases now
report `invalid/XTSE0010`. Each catalog entry expects an error, and inspection
confirmed forbidden literal text, `xsl:text`, nested `xsl:template`,
`xsl:sort`, or other non-parameter content. The shared `XTSE0010` frontier
therefore rises from 21 to 29 while `FXST1014` disappears. This is also a
classification correction rather than a positive-pass increase.

The two former `unsupported/FXST1022` cases now report `invalid/XTSE0010`
because their document elements are `xsl:template` instructions rather than
literal result elements. Both catalog entries already expected an error, so
this corrects the diagnostic category without changing any measurement count.

The complete sweep initializes 2,170 cases, reports 965 initialization
failures, executes 2,114 successfully, and reports 56 execution failures.
Exact XML-semantic matches remain 1,968, with 65 mismatches. The strict
compatibility lower bound remains `1,968 / 3,173 = 62.02%`.

## Boundaries

- No OASIS case is credited merely because it fails.
- The six remaining unexpected successes are not assumed to be engine defects;
  each requires a standards-backed reason before FastXSLT behavior changes.
- This tranche does not select a general recovery policy for XSLT 1.0 errors.
- Modern mode-list support remains available and is tested independently.
- The archive remains locally acquired and is not redistributed.

## Reproduction

```powershell
cargo test -p fastxslt --all-features rejects_forbidden_mode_and_malformed_extension_prefixes_on_stylesheet_root
cargo test -p fastxslt --all-features rejects_invalid_template_mode_lists_without_treating_them_as_unsupported
cargo test -p fastxslt --all-features rejects_invalid_template_invocation_children_as_static_errors
cargo test -p fastxslt --all-features rejects_an_xslt_instruction_as_a_simplified_stylesheet_root
./scripts/measure-oasis-xslt10.ps1 -TraceCase Microsoft/AttributeSets__91101#1
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier invalid/XTSE0550/XTSE0550
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier invalid/XTSE0010/XTSE0010
./scripts/verify.ps1
```
