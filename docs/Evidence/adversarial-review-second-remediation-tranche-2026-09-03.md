# Second Adversarial Review: Boundedness, Binding, and Ownership Tranche

Date: 2026-09-03

## Scope

This tranche addresses Findings 1, 2, 3, and 6 from the
[second adversarial engineering review](../Reviews/adversarial-engine-review-2026-09-03.md).
It does not dispose the character-map scaling, QT3 production-path parity, or
source-unit reopening findings.

## Atomic range control

`xsl:apply-templates` over a static atomic integer range no longer collects the
complete range into a `Vec<i64>`. Execution:

1. returns immediately for a descending empty range;
2. calculates the inclusive focus size with checked arithmetic and host-width
   conversion;
3. iterates without range-sized retained storage; and
4. charges `XPathOperation` before each atomic item reaches template dispatch.

A production compiler/runtime regression uses `1 to 1000000000` with a zero
XPath-operation budget. It returns a structured limit carrying the request and
work-domain identities before retaining the hostile span.

## Lexical binding identity

Runtime local bindings now have one active value kind. Binding an atomic,
atomic sequence, source-node sequence, or temporary tree clears competing
representations and records that same-name global fallback is no longer
permitted.

The review's exact counterexample—a global atomic `$value` and a supplied local
source-node `$value`—produces the source node's string value through both the
shared copy-on-write frame and the complete-clone oracle. This repairs the
confirmed cross-kind shadow leak without making the private runtime value model
public.

## Native creation ownership

An engine remains tied to its creation outcome until the caller takes that
outcome. Releasing an untaken outcome now removes the associated engine and its
known-capacity accounting charge. A concurrent take/release test establishes
one linearization point: either take transfers the live engine handle or release
reclaims it, never both and never neither. Direct engine release is rejected
while the creation outcome still owns the unpublished handle.

## Worker admission

The isolated worker now uses a capacity-one synchronous channel for reader,
supervisor, and completion events. The reader cannot continue decoding an
unbounded sequence while the supervisor is occupied. A focused nonblocking
test confirms that a second decoded event cannot occupy the full queue.

This is bounded in-process transport admission, not a claim that a cooperating
parent, operating-system pipe, allocator, or entire worker process has a fixed
memory footprint.

## Verification

- The hostile atomic-range production-path regression passes.
- Shared and complete-clone parameter-shadowing regressions pass.
- Native release-before-take and concurrent take/release regressions pass.
- The worker queue backpressure regression passes.
- Strict all-target/all-feature workspace Clippy passes.
- The complete repository verification gate passed: formatting, strict Clippy,
  workspace tests, documentation, Markdown links, and corpus integrity checks
  are green.
