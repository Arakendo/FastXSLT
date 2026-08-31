# XSLT30 `mode-1445`/`mode-1446` Bounded Shallow-Copy

Date: 2026-08-30

## Scope

This slice admits the unchanged pinned XSLT30 `mode-1445` and `mode-1446`
cases. Both invoke named mode `s` with `on-no-match="shallow-copy"`; the first
declares whitespace-padded `typed=" false "` and the second declares
`typed="0"`. Both typed forms are inert and both cases compare against the
unchanged `mode-1411.out` XML result.

## Compiled policy

Mode compilation now retains one closed internal policy value rather than a
fail-specific flag. The admitted values are `fail` and `shallow-copy`, together
with the optional expanded mode name and declaration location. Typed-mode
requirements remain separate static state, so `typed="yes"` continues to reject
an untyped initial node before fallback behavior can run.

Normal template selection remains authoritative. Runtime consults the retained
policy only when no user template matches the current node and mode. Named
shallow-copy modes also participate in initial-mode existence checks even when
the stylesheet contains no explicit templates.

## Bounded execution behavior

For the admitted source shape, the policy:

- traverses document children in focus order;
- constructs element copies with their expanded names and namespace bindings;
- copies source attributes when no attribute template intercepts them;
- applies the same active mode recursively to child nodes;
- copies text and processing instructions; and
- charges XSLT instruction, template-selection, result-node, and result-text
  work at their existing ownership points.

The shared native output is roughly 9 KiB, so the adapter uses the same explicit
16 KiB case ceiling introduced for the adjacent large mode result. Compilation
and execution remain memory-resident over the sealed source/stylesheet
snapshot.

## Explicit non-admissions

The current semantic result tree cannot represent comments or standalone
attribute result items. It also cannot yet route an attribute-template result
back into the attribute phase of an element constructor. Focused negative tests
therefore require structured unsupported failures with source locations for:

- a comment reached by shallow-copy (`FXRT1012`); and
- an attribute intercepted by a matching template (`FXRT1013`).

These outcomes prevent plausible partial XML from being mistaken for supported
shallow-copy semantics. Deep copy, deep/shallow skip, and text-only-copy remain
unsupported.

## Accounting

The complete 169-case mode denominator now records:

- 46 passed;
- 0 engine-unsupported;
- 44 profile-excluded; and
- 79 visible default not-run cases.

Across the 11 conserved XSLT30 denominators, the total is now 240 passed, 3
engine-unsupported, 49 profile-excluded, and 239 visible not-run cases out of
531.
