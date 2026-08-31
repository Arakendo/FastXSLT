# XSLT30 `mode-1423` Fail-on-No-Match Success Control

Date: 2026-08-30

## Scope

This slice admits the unchanged pinned XSLT30 `mode-1423` case as the positive
control for `on-no-match="fail"`. Unlike `mode-1431`, every node visited by the
stylesheet has an explicit matching template, so execution must complete
without raising `XTDE0555`.

## Result

The principal document rule begins traversal, the wildcard element rule copies
each element and continues traversal, and the text rule copies each text value.
Template selection therefore never reaches the built-in fallback boundary
where the retained fail policy is enforced. The complete native result compares
as equivalent XML.

This distinction is intentional: the fail policy is neither an eager
invocation rejection nor a replacement for ordinary template selection. It is
consulted only when no user rule matches the active node and mode.

The native expected result is 8,997 bytes. The adapter gives this case a 16 KiB
serialization ceiling and parses comparison results under a 4,096-event,
depth-16 limit. These are explicit harness bounds; the engine's output limit
remains host-supplied. Source and stylesheet bytes are still admitted into the
sealed snapshot before memory-resident compilation and execution.

## Accounting

The complete 169-case mode denominator now records:

- 44 passed;
- 0 engine-unsupported;
- 44 profile-excluded; and
- 81 visible default not-run cases.

Across the 11 conserved XSLT30 denominators, the total is now 238 passed, 3
engine-unsupported, 49 profile-excluded, and 241 visible not-run cases out of
531.

## Boundaries retained

- This case adds no new built-in policy or instruction semantics.
- `XTDE0555` remains observable when an unmatched node is actually reached, as
  proved independently by unchanged case `mode-1431`.
- The larger comparator and serialization limits are bounded test-adapter
  settings, not public API defaults or performance claims.
