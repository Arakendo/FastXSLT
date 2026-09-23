# OASIS XSLT 1.0 Message Static Validation

Date: 2026-09-23  
Status: Local implementation and compatibility evidence

## Question

Can FastXSLT classify malformed XSLT 1.0 `xsl:message` instructions before
reporting the still-unsupported runtime message semantics, without weakening
the explicit unsupported boundary for valid messages?

## Inventory and method

The measured instruction frontier contained 27 cases:

- 23 valid ordinary message cases: the 15 Lotus `message01` through
  `message15` cases, Lotus `impincl18`, Microsoft `Errors_err060`, and six
  Microsoft message cases covering absent, `no`, and `yes` termination plus an
  XML-fragment content constructor;
- two invalid `terminate` lexical cases, Microsoft `91760` and `91761`; and
- two unknown-attribute cases, Microsoft `91762` and `91763`.

The compiler now recognizes `xsl:message` as a distinct instruction frontier
under XSLT 1.0 compatibility. It permits only the unqualified `terminate`
attribute, validates its literal value as exactly `yes` or `no`, and then
retains `FXST1006` for valid message execution until AR-0023's bounded runtime
observation contract exists. The validation is gated by the inherited XSLT 1.0
static context, so it does not freeze XSLT 2.0/3.0 message syntax.

Focused tests cover absent, `no`, and `yes` values; empty, whitespace-padded,
boolean-looking, and arbitrary invalid values; and an unknown unqualified
attribute.

## Result

The valid-message frontier falls from 27 to 23 cases. Invalid literal values
are now classified as `invalid/XTSE0020`, and unknown attributes as
`unsupported/FXST1009`, before any runtime-message claim is made.

The complete 3,173-case sweep remains stable: 2,195 cases initialize, 2,138
execute successfully, 1,988 compare exactly, 66 mismatch, and 75 remain at an
unsupported comparison boundary. The strict compatibility lower bound remains
`1,988 / 3,173 = 62.65%`.

## Boundaries

- Valid `xsl:message` remains explicitly unsupported; authored observations
  are not discarded to gain corpus passes.
- No message type, callback, logging hook, retained collection, limit, or wire
  representation is selected by this tranche.
- Top-level `xsl:message` remains a separate invalid/top-level declaration
  question, and a message nested in an unsupported global temporary-tree
  constructor remains behind that earlier boundary.
- The archive remains locally acquired and is not redistributed.

## Reproduction

```powershell
cargo test -p fastxslt validates_xslt10_message_before_reporting_unsupported_execution
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier 'unsupported/FXST1006/unsupported XSLT instruction: xsl:message'
./scripts/verify.ps1
```
