# XSLT30 Initial-Mode Denominator Admission

Date: 2026-08-26

## Question

Can FastXSLT conserve the complete XSLT30 `misc/initial-mode` family and carry
its invocation metadata and expected outcomes far enough to identify the first
actual engine boundary for every case?

## Pinned inputs

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/misc/initial-mode/_initial-mode-test-set.xml`
- Cases: `initial-mode-001` through `initial-mode-005`
- Selection: all five selected; no profile exclusion

The admission test parses the pinned catalog through the owned XML/XDM path,
verifies each case identity and standards dependency, preserves the declared
initial-mode name, distinguishes XML assertions from expected-error results,
and reads then closes each stylesheet handle before admitting its bytes to a
bounded sealed snapshot.

## Current disposition

| Case | Initial mode | Expected result | Current disposition |
| --- | --- | --- | --- |
| `initial-mode-001` | `inimode` | XML | `FXST1009`: typed constructed variable attributes |
| `initial-mode-002` | `inimode` | `XTDE0045` | `FXST1009`: `xsl:output/@indent` |
| `initial-mode-003` | `inimode` | `XTDE0050` | `FXST1009`: `xsl:output/@indent` |
| `initial-mode-004` | `flobble` | XML | Native pass: local expanded-QName and tunnel parameters plus mixed node/atomic sequence |
| `initial-mode-005` | `b` | XML | `FXST1015`: element-bearing global sequence constructor |

The denominator is five discovered and selected: one native pass, four explicit
engine gaps, and none excluded, harness-unsupported, failed, or lost.

## Claim boundary

This evidence establishes corpus ownership, entry-metadata preservation,
expected-error identity, reproducible first-gap classification, and the narrow
initial-mode parameter semantics exercised by case 004. It does not establish
general template-parameter defaults/types, tunnel propagation across template
application, multi-mode declarations, `#all` semantics, required global
parameters, temporary trees, general sequence constructors, or either expected
dynamic error.

The first implementation decision should be the host-neutral invocation shape:
an initial mode is standards-defined entry state, not a special ASP.NET or CLI
execution path. The current cases also show why adding the entry enum alone
would not create a pass; each case has deeper stylesheet or XPath pressure that
must remain independently classified.

## Subsequent entry-seam evidence

The private runtime now admits a principal-source plus initial-mode entry,
rejects an unknown compiled mode with structured `FXRT0005`, and executes a
focused root template in a named mode through the same parsing, XDM,
invocation-control, result, and serialization path as principal-source work.
This originally advanced case 004 beyond `FXST1011` to `FXST1006` without
promoting a corpus case.

## Subsequent parameter and sequence evidence

Case 004 now compiles leading local `xsl:param` declarations, canonicalizes
prefixed parameter identity to expanded QName form, distinguishes tunnel from
non-tunnel invocation values, excludes `my` and `xs` result namespaces through
the stylesheet's `exclude-result-prefixes`, and executes the asserted ordered
sequence `*,$a,$my:b`. A mismatch control proves that a non-tunnel value with
the same expanded name does not satisfy the tunnel parameter. The produced
`<doc></doc>` spelling is XML-equivalent to the suite assertion's `<doc/>`.

These mechanisms remain private implementation evidence. They do not publish a
host parameter API or imply support for arbitrary XPath sequence expressions.
