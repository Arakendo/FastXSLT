# Peer AR-0014 Review: Monday

| Field | Value |
| --- | --- |
| Date | 2026-08-28 |
| Reviewer | Monday |
| Reviewed revision | `2a08449` |
| Subject | AR-0014 resource reference resolution and authority composition |
| Outcome | Retain Incubating; make cache identity and fragment semantics explicit |
| Informs | AR-0014 and the planned sealed-memory `include-0401` experiment |

## Review conclusion

The reviewer found the separation between lexical reference, base identity,
resolved logical identity, catalog/host mapping, acquisition locator, and
immutable admitted bytes to be the central strength of AR-0014. In particular,
the review confirmed that a URI is not authority, a path is not engine identity,
a catalog rewrite is not acquisition, and admitted bytes do not authorize later
host access.

The ownership split also remains appropriate: hosts determine what may be
acquired, while FastXSLT owns the standards-visible meaning of references.
Neither ambient engine access nor independently diverging reference semantics
in every host adapter is acceptable.

## Refinements

The reviewer requested two explicit distinctions.

First, **resolution identity and cache identity are not necessarily the same**.
Catalog and base processing may cause multiple lexical references to resolve to
one logical resource identity, while any future cache may also require snapshot
generation, standards/profile configuration, policy or capability context, and
representation identity. A matching URI string must not become an accidental
cross-generation or cross-authority cache key.

Second, **resource identity and fragment/reference semantics are different**.
`foo.xml#bar` must not automatically be treated by an acquisition layer as a
different blob to fetch from `foo.xml`. FastXSLT's language and reference
semantics must decide how the fragment selects or identifies content after the
resource identity used for acquisition is known. The review does not decide the
rules for embedded stylesheet modules or other fragment-bearing facilities.

## Strengths confirmed

- URL-shaped logical identity does not imply network authority.
- Denial may need to precede snapshot-membership disclosure.
- Static stylesheet dependencies and invocation-time dynamic resources can
  share identity machinery without sharing lifetime or budget policy.
- A live resolver result must not mutate a sealed snapshot invisibly.
- Content hashes cannot replace document identity, authority, or generation.
- Snapshot-first plus optional live authority remains plausible, but lookup
  precedence is observable and therefore cannot be selected casually.

## Recommended next evidence

Retain the Incubating disposition. Exercise relative/base resolution through
sealed resources and a real `xsl:include` case first, then establish cycle,
depth, count, and byte accounting. Live callbacks, network/filesystem policy,
and cache admission should remain outside the initial experiment.

This is a design review, not URI-conformance, cache-correctness, fragment,
resolver-security, or `xsl:include` execution evidence.
