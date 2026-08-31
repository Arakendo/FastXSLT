# XSLT30 `mode-1431` Fail-on-No-Match Error

Date: 2026-08-30

## Scope

This slice admits the unchanged pinned XSLT30 `mode-1431` case. Its unnamed
mode declares `on-no-match="fail"`; execution must report dynamic error
`XTDE0555` when the source traversal reaches a node for which no template rule
matches. The slice does not implement any other `on-no-match` policy.

## Compilation and execution

Compilation retains the fail policy as mode-derived static state together with
the optional expanded mode name and original declaration location. An absent
name represents the unnamed mode; it is not replaced by a filename or another
display identity. Included stylesheet programs carry the same retained policy
through the existing one-way module merge.

Runtime template dispatch continues to select user rules first. Only when
selection produces no rule does the built-in-rule boundary inspect the active
mode's retained policy. A matching fail policy returns structured `XTDE0555`,
category `invalid`, the request identity, and the `xsl:mode` declaration
location instead of running a built-in template.

In the native case, the document and element nodes match explicit templates.
Their recursive application eventually selects a whitespace text node with no
matching rule, which proves the failure occurs at the actual fallback boundary
rather than eagerly at invocation admission. The external suite source is read
and closed by the test adapter, admitted to a bounded sealed snapshot, and then
consumed memory-resident by compilation and execution.

## Accounting

The complete 169-case mode denominator now records:

- 43 passed;
- 0 engine-unsupported;
- 44 profile-excluded; and
- 82 visible default not-run cases.

Across the 11 conserved XSLT30 denominators, the total is now 237 passed, 3
engine-unsupported, 49 profile-excluded, and 242 visible not-run cases out of
531.

## Boundaries retained

- `fail` is the only newly admitted built-in policy. Deep/shallow copy,
  deep/shallow skip, and text-only-copy remain unsupported.
- A fail policy does not preempt a matching user template.
- The compiled policy is stylesheet-derived and immutable; invocation state is
  not retained in the program.
- The case establishes one required dynamic error, not general mode or XSLT 3.0
  conformance.
