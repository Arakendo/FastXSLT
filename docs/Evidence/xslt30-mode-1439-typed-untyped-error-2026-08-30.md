# XSLT30 `mode-1439` Typed-Mode Untyped-Source Error

Date: 2026-08-30

## Scope

This slice admits the unchanged pinned XSLT30 `mode-1439` case. It retains one
named `xsl:mode typed="yes"` requirement and reports the native dynamic error
`XTTE3100` when that mode is invoked with FastXSLT's untyped source XDM. It does
not claim schema-aware XDM, typed-node execution, or the declaration's
otherwise unreachable `on-no-match="shallow-copy"` behavior.

## Compilation and execution

The mode declaration owner already validates XSLT 3.0 boolean lexicals and the
closed `on-no-match` value vocabulary. For `typed="yes"`, compilation now
retains the expanded mode name and stylesheet source location instead of
classifying all typed-mode pressure as generically unsupported.

Initial-mode admission recognizes that retained declaration even when the mode
has no executable template. Execution then checks the source type requirement
before template selection or built-in fallback. Because the current XDM is
deliberately untyped, the invocation returns structured `XTTE3100`, category
`invalid`, the request identity, and the original mode-declaration location.
No shallow-copy behavior executes and no schema type is fabricated.

The native case uses the external 9,001-byte `mode-14.xml` source. The mode
adapter now admits either inline catalog content or an explicitly named suite
file into the same sealed in-memory snapshot, with a 16 KiB per-entry and 24
KiB aggregate ceiling. The file is read and closed during test setup; engine
compilation and execution remain memory-resident.

## Accounting

The complete 169-case mode denominator now records:

- 42 passed;
- 0 engine-unsupported;
- 44 profile-excluded; and
- 83 visible default not-run cases.

Across the 11 conserved XSLT30 denominators, the total is now 236 passed, 3
engine-unsupported, 49 profile-excluded, and 243 visible not-run cases out of
531.

## Boundaries retained

- `mode-1438` remains profile-excluded because its native dependency requires
  streaming; its similar expected error does not erase that dependency.
- `typed="no"`, `false`, and `0` remain inert lexical forms; invalid mixed-case
  values continue to report static `XTSE0020`.
- Successful typed-mode execution requires a future schema-aware review and
  evidence. This slice establishes only the required failure for an untyped
  initial node.
- General `on-no-match` execution remains unsupported.
