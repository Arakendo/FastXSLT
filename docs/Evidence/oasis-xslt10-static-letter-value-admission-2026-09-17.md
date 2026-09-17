# OASIS XSLT 1.0 Static `letter-value` Admission -- 2026-09-17

Date: 2026-09-17  
Status: Verified bounded semantic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Finding

The admitted `xsl:number` formatter already implements decimal, Latin
alphabetic, and Roman tokens, but rejected every `letter-value` attribute before
checking whether it requested exactly those existing semantics.

## Repair

The compiler now admits static `letter-value="traditional"` for the existing
token set and `letter-value="alphabetic"` when every compiled token is already
Latin alphabetic. It retains explicit `FXST1049` for dynamic values,
non-equivalent Roman reinterpretation, non-Latin numbering, and other
unimplemented numbering semantics. No runtime branch or locale policy is added.

Focused tests cover traditional mixed tokens, alphabetic Latin tokens, decimal
tokens, and rejection of alphabetic reinterpretation of a Roman token.

## Corpus result

Affected unchanged cases advance from attribute rejection to their next honest
non-ASCII or compound-format-token boundary. No case becomes newly executable,
so the exact-result lower bound remains 1,518 and all aggregate counters remain
unchanged. This is capability and frontier evidence, not a conformance pass.

## Verification

The complete sweep continues to conserve all 3,173 identities: 1,866 initialize,
1,667 execute successfully, 1,518 compare exactly, 120 remain visible
mismatches, and 23 reach comparator-unsupported outcomes. Expected-error
accounting remains 403 initialization observations, 21 execution observations,
and four doubt-annotated unexpected successes.
