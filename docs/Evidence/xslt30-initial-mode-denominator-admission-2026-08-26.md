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

| Case | Initial mode | Expected result | First compiler boundary |
| --- | --- | --- | --- |
| `initial-mode-001` | `inimode` | XML | `FXST1009`: typed constructed variable attributes |
| `initial-mode-002` | `inimode` | `XTDE0045` | `FXST1009`: `xsl:output/@indent` |
| `initial-mode-003` | `inimode` | `XTDE0050` | `FXST1009`: `xsl:output/@indent` |
| `initial-mode-004` | `flobble` | XML | `FXST1011`: mode on the root match pattern |
| `initial-mode-005` | `b` | XML | `FXST1015`: element-bearing global sequence constructor |

The denominator is five discovered and selected: zero pass, five are explicit
engine gaps, and none are excluded, harness-unsupported, failed, or lost.

## Claim boundary

This evidence establishes corpus ownership, entry-metadata preservation,
expected-error identity, and reproducible first-gap classification. It does not
establish an initial-mode engine entry, multi-mode template declarations,
`#all` semantics, required global parameters, invocation/tunnel parameters,
temporary trees, typed sequence constructors, or either expected dynamic error.

The first implementation decision should be the host-neutral invocation shape:
an initial mode is standards-defined entry state, not a special ASP.NET or CLI
execution path. The current cases also show why adding the entry enum alone
would not create a pass; each case has deeper stylesheet or XPath pressure that
must remain independently classified.
