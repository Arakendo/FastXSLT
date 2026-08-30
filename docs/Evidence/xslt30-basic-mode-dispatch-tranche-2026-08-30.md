# XSLT30 Basic Mode Dispatch Tranche

| Field | Value |
| --- | --- |
| Date | 2026-08-30 |
| Suite revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Test set | `tests/attr/mode/_mode-test-set.xml` |
| Cases | `mode-0101` through `mode-0108`, `mode-0201` through `mode-0701` |
| Result | 14 selected native passes; 155 visible default not-run cases |

## Added evidence

Ten newly selected cases execute through the existing compiled stylesheet and
runtime path:

- explicit modes select only rules in the same mode;
- omitted modes select the unnamed mode rather than inheriting the caller's
  explicit mode;
- built-in element descent retains an explicit mode;
- a missing user mode still has mode-local built-in behavior;
- comment, processing-instruction, any-node, and attribute selections retain
  their XDM kinds through mode-qualified dispatch; and
- a named-template call preserves current mode while an inner unmoded
  `xsl:apply-templates` deliberately enters the unnamed mode.

`mode-0301` exposed a compiler gap: an `xsl:template` may carry both `name` and
`match`. Compilation previously returned after recording the named identity and
the named-template validator rejected `match`. The corrected path records the
same declaration in both the named and matched indexes, with shared compiled
body semantics. A focused compiler control preserves its name, mode, explicit
priority, and matched identity.

The harness also now reads file-backed `assert-xml` results for this test set;
it does not copy or reinterpret the expected result.

This tranche does not claim `xsl:mode` declaration properties, streaming,
schema awareness, packages, mode typing, or complete mode conformance.
