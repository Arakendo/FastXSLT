# XSLT30 `conflict-resolution-1601` Root-Pattern Priority

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-1601`
- Stylesheet: `conflict-resolution-1601.xsl`
- Source: inline `conflict-resolution-16` environment

## Representation change

A stylesheet with one unmoded `/` template retains the existing direct
single-root execution path. When a second competing unmoded root pattern or an
explicit root priority appears, compilation migrates the direct template into
the ordinary typed matched-template sequence. The migrated `/` rule retains
exact default priority `-0.5`; subsequent rules retain their compiled explicit
priorities and declaration order.

This makes conflict resolution use the same selector as other typed patterns
without imposing selector traversal on the common single-root case. Mode-bound
root patterns continue to use matched-template selection independently. The
single-include compiler still rejects duplicate root rules across modules
because include/import precedence is not admitted by this case.

## Result

| Case | Expected XML | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-1601` | `<out>big</out>` | semantically equal | passed |

The explicit `-0.4` rule wins. The default `/` rule at `-0.5` outranks the
explicit `-0.6` rule but does not outrank `-0.4`.

## Conservation and claim boundary

Existing single-root golden, workbench, output, and host-boundary tests continue
through the direct path. Competing roots use immutable compiled templates and
the existing request-local selector; resource authority, invocation state,
diagnostics, serialization, and host APIs do not move.

This evidence admits competing unmoded `/` rules and bounded explicit priority
within one stylesheet module. Typed `document-node(element(...))` patterns are
evidenced separately by `1602`–`1603`. This record does not admit
include/import/package precedence, generalized duplicate-pattern policy,
ambiguity recovery, or alternate standards-edition conflict behavior.
