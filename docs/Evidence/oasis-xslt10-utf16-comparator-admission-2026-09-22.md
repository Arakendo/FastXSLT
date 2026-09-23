# OASIS XSLT 1.0 UTF-16 Comparator Admission

Date: 2026-09-22  
Status: Local harness evidence

## Question

Can the local OASIS comparator classify the bounded UTF-16 result bytes that
FastXSLT already serializes, instead of leaving those executions opaque?

## Method

- Preserve the engine's existing BOM-marked UTF-16BE serializer and byte
  limits.
- Share a BOM-aware UTF-16 decoder between expected-result and actual-result
  comparison, with explicit odd-length and invalid-surrogate failures.
- Add focused UTF-16BE, UTF-16LE, non-ASCII, and malformed-byte regressions.
- Rerun the unchanged 3,173-case OASIS catalog.

## Result

All nine formerly undecodable successful results now receive an explicit
disposition. Seven compare XML-semantically, one (`Microsoft/Output__77939#1`)
is a visible whitespace mismatch, and one expected-error case
(`Microsoft/Output__78176#1`) is now visibly an unexpected success.

The complete sweep still initializes 2,153 cases and executes 2,097
successfully. Exact matches rise from 1,946 to 1,953, mismatches rise from 61
to 62, and expected-error unexpected successes rise from six to seven. The
strict compatibility lower bound is `1,953 / 3,173 = 61.55%`.

## Boundaries

- This is comparator admission, not a new engine encoding or serialization
  claim.
- Only BOM-marked UTF-16 actual bytes are admitted; the comparator does not
  guess byte order.
- Truncated or invalid UTF-16 remains a named comparator frontier rather than
  lossy text.
- The result is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features oasis_actual_decoder_admits_bom_marked_utf16_bytes
./scripts/measure-oasis-xslt10.ps1 -TraceCase Microsoft/Output__77939#1
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
