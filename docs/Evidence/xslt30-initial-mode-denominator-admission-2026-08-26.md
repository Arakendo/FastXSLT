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
| `initial-mode-001` | `inimode` | XML | Native pass: typed constructed integer sequence over `1 to 10` |
| `initial-mode-002` | `inimode` | `XTDE0045` | Native pass: `mode="#all"` does not declare arbitrary initial modes |
| `initial-mode-003` | `inimode` | `XTDE0050` | Native pass: absent required global parameter reports expected error |
| `initial-mode-004` | `flobble` | XML | Native pass: local expanded-QName and tunnel parameters plus mixed node/atomic sequence |
| `initial-mode-005` | `b` | XML | `FXST1015`: element-bearing global sequence constructor |

The denominator is five discovered and selected: four native passes, one
explicit engine gap, and none excluded, harness-unsupported, failed, or lost.

## Claim boundary

This evidence establishes corpus ownership, entry-metadata preservation,
expected-error identity, reproducible first-gap classification, and the narrow
initial-mode parameter semantics exercised by case 004, plus the required
global-parameter failure exercised by case 003. It does not establish general
template-parameter defaults/types, tunnel propagation across template
application, multi-mode declarations, `#all` semantics, temporary trees,
general sequence constructors, general typed conversion, or general `#all`
dispatch behavior.

The first implementation decision should be the host-neutral invocation shape:
an initial mode is standards-defined entry state, not a special ASP.NET or CLI
execution path. The current cases also show why adding the entry enum alone
would not create a pass; each case has deeper stylesheet or XPath pressure that
must remain independently classified.

## Subsequent entry-seam evidence

The private runtime now admits a principal-source plus initial-mode entry,
rejects an unavailable compiled mode with standards code `XTDE0045`, and executes a
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

## Subsequent required-global evidence

Case 003 now preserves `xsl:output/@indent`, compiles a required global
`xsl:param`, recognizes `inimode` from the stylesheet's matched-template mode,
and reports `XTDE0050` when the invocation supplies no value even though the
parameter is unused. The error occurs before template evaluation and before
serialization. Indentation remains an explicit serialization boundary:
successful work requesting indentation reports `FXSR1003` rather than silently
emitting unindented bytes.

## Subsequent unavailable-mode evidence

Case 002 now compiles `mode="#all"` as template declaration metadata while
keeping it distinct from a named mode made available for initial invocation.
Requesting `inimode` therefore returns the expected `XTDE0045` during request
admission. This does not claim general `#all` dispatch; it proves the narrower
and crucial negative rule that `#all` is not a wildcard declaration of every
possible initial-mode name.

## Subsequent typed-sequence evidence

Case 001 now compiles its `xs:integer *` local variable constructor, evaluates
the bounded integer range `1 to 10`, retains ten typed atomic values in
invocation-local sequence state, and applies `xsl:value-of/@separator` without
preformatting the sequence into one string. Each retained range item first
charges the XPath-operation budget; a nine-operation control fails before the
tenth item is retained.

This is evidence for one typed range-construction shape. It does not establish
general `xsl:for-each`, arbitrary sequence constructors, node atomization,
sequence-type conversion, or a complete XPath range operator.
