# Peer Review: AR-0016 Stylesheet-Dependent Source Views

| Field | Value |
| --- | --- |
| Date | 2026-08-30 |
| Reviewer | Monday |
| Subject | AR-0016 stylesheet-dependent source views and whitespace stripping |
| Outcome | Retain Incubating disposition; strengthen identity, relationship, positional, string-value, concurrency, and representation experiments |

## Review assessment

The review agrees that `mode-1301` exposes semantic ownership rather than a
local traversal defect. Whitespace stripping belongs to stylesheet-dependent
source semantics: XML parsing cannot own it, reusable prepared XDM cannot be
mutated by it, and built-in template traversal cannot apply a private version
that disagrees with XPath, copying, or string values.

The proposed ordering remains appropriate:

```text
immutable prepared XDM + compiled whitespace policy
                         |
                         v
              effective source semantics
                    /          \
       safe derived reference   visibility-view candidate
```

The safe reference supplies a differential oracle. A compact view or mask may
later improve execution, but representation intuition is not evidence and does
not select that optimization.

## Strengthened invariants

- A visible source node retains the prepared document's semantic identity under
  both stripping and preserving policies. A reference representation may use a
  mapping internally, but it cannot expose a newly minted identity for the same
  visible source node.
- Every relationship and sequence exposed through the effective document acts
  as though stripped nodes are absent. This includes children, descendants,
  sibling relations where implemented, focus position and size, and therefore
  `position()` and `last()` results.
- Element and document string values exclude stripped text even when no XPath
  step explicitly selects that text. The effect must remain consistent in
  comparisons, predicates, `xsl:value-of`, copying, and diagnostics that expose
  semantic values.
- One prepared source can concurrently serve a stripping stylesheet and a
  preserving stylesheet without visibility, relationship, identity, or cache
  cross-talk. The invariant continues across old/new generation overlap.

## Representation pressure

A precomputed invocation-owned visibility form is a plausible middle point
between cloning a complete effective tree and reevaluating whitespace rules on
every navigation step. Dense stable node identifiers could make a compact mask
possible, but the review does not select a bitset, cache, node layout, or view
lifetime.

The experiment should compare at least:

1. complete safe derived reference;
2. visibility computed once for an invocation and consumed cheaply; and
3. lazy rule checks only if measurement shows that construction cost warrants
   the repeated hot-path work.

Any retention beyond an invocation reopens AR-0009's generation, cache,
eviction, and memory-attribution questions. Any compact physical form remains
subject to AR-0013 and safe-reference differential verification.

## Disposition

Keep AR-0016 **Incubating**. Its current prohibitions and reference-first
direction remain sound. Add explicit proof obligations for visible-node
identity, effective relationships and positions, indirect string-value effects,
and concurrent strip/preserve execution before selecting a representation or
admitting `mode-1301`.
