# ADR-0013: Invocation-Owned Document-Rooted Match Membership

- Status: Accepted
- Date: 2026-08-31
- Related reviews: AR-0009, AR-0010, AR-0013
- Related ADRs: ADR-0002, ADR-0004, ADR-0007, ADR-0012
- Related evidence: `docs/Evidence/document-rooted-match-path-reevaluation-2026-08-31.md` and adversarial review Finding 11
- Supersedes: None

## Context

The reference matched-template selector evaluates an absolute location-path
pattern from the document node whenever that template is tested for a candidate
node. A broad dispatch over `n` matching siblings therefore reevaluates the same
stylesheet-derived path `n` times. The measured shape charged `(n + 1)^2` XPath
node visits, reached 66,049 visits at width 256, and allocated repeated temporary
path results. Accounting remained honest, but useful work scaled poorly.

A safe prototype builds one membership bitset on the first test of each
document-rooted pattern, then performs constant-time membership checks for later
candidates in the same invocation. At width 256 it reduced path evaluations
from 256 to 1, node visits from 66,049 to 514, total allocator-requested bytes
from 2,851,120 to 236,160, and local median execution from 991.8 us to 88.1 us.
Peak requested bytes rose only from 62,208 to 62,528 while the membership itself
retained 40 deterministic bytes.

## Decision

Use a private invocation-owned membership cache for compiled document-rooted
match paths.

The cache must:

- key membership by the compiled matched-template index within the current
  `StylesheetProgram`, never by URI text, content hash, host path, or global
  identity;
- build through the existing charged and cancellable location-path evaluator;
- retain a word-backed safe-Rust bitset indexed by the prepared document's
  stable private `NodeId`;
- preserve template ranking, modes, diagnostics, document order, source
  identity, and result semantics;
- remain local to one invocation and drop before the invocation returns;
- never mutate or extend the prepared XDM, compiled program, resource snapshot,
  worker, or another generation;
- fall back to the complete reference evaluation when its one-megabyte or
  1,024-entry private retention ceiling cannot admit another membership; and
- retain the uncached evaluator as a test-only differential and measurement
  oracle.

The first candidate still pays the complete XPath work and observes its exact
budget/cancellation charge points. Later cache hits avoid work that no longer
occurs; the independently charged template-candidate unit continues to bound
each dispatch decision.

## Non-decisions

This ADR does not admit:

- a cache shared across invocations, sources, snapshots, workers, or
  generations;
- eager construction during source preparation or stylesheet compilation;
- content-addressed document identity;
- caching relative patterns, general XPath expressions, keys, or arbitrary
  selections;
- a public cache, bitset, node-index, or provider type;
- changing the prepared-XDM representation;
- a guarantee that every eligible pattern will be cached; or
- unsafe code.

Broader stylesheet-activated indexes remain AR-0013 experiments and must earn
their own preparation, retention, parity, and host-visible benefit.

## Consequences

Repeated absolute pattern tests become a lean activated path without imposing
cache work on stylesheets that do not contain such patterns. Invocation memory
is deterministically capped and attributable. Exceeding the optimization cap
changes performance only: the safe reference semantics remain available.

Budget consumption can be lower than the reference because repeated path work
is removed. This is an honest work reduction, not a hidden charge. Failure
during first construction remains a normal XPath cancellation or limit outcome
with its existing request and domain identity.

## Validation

- Differentially compare cached and uncached semantic results across the width
  matrix.
- Assert one build plus `n - 1` hits for one `n`-candidate invocation.
- Assert the exact reduced XPath visit count and one-less construction budget
  failure.
- Measure construction/lookup latency, total and peak allocator-requested bytes,
  and deterministic retained membership bytes.
- Run concurrent invocations over one compiled program and prepared source and
  prove each owns an independent cache.
- Reject cache entries beyond both private ceilings without mutation.
- Run the normal FastXSLT verification suite and unchanged corpus cases.

Revisit when representative workloads reverse the benefit, the fallback path
becomes common, a consumer needs cache observability, or any sharing beyond one
invocation is proposed.
