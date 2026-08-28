# Private Qualified Snapshot Resolution

| Field | Value |
| --- | --- |
| Date | 2026-08-28 |
| Boundary | Private stylesheet acquisition from one sealed resource snapshot |
| Resolution form | Exact qualified absolute logical identity |
| Authority | Snapshot admission plus an explicit deny set |
| Limit | Fixed charged resolution-attempt count |
| Outcome | Qualified admitted identities resolve; denied, missing, invalid, relative, fragment, and exhausted-limit outcomes remain distinct |

## Executed boundary

The private stylesheet compiler now acquires stylesheet bytes through a bounded
snapshot resolver rather than reading the snapshot directly. Every lookup is
charged before validation or resource access. The resolver accepts an exact
qualified identity only when its leading URI scheme has bounded syntactic
shape, consults an explicit deny set before snapshot membership, and returns
bytes only from the sealed snapshot.

Focused tests establish that:

- exact admitted URN and URL-shaped logical identities resolve;
- a URL-shaped identity is only an inert snapshot key and does not authorize a
  network request;
- denied identities remain denied whether or not their bytes were admitted, so
  denial does not reveal snapshot membership;
- a qualified but unadmitted identity is missing;
- relative references, Windows path shapes, fragments, whitespace-bearing
  references, and empty references do not fall back to ambient authority;
- every attempt, including an invalid or missing attempt, consumes the fixed
  attempt budget; and
- the production compiler rejects valid stylesheet bytes admitted under the
  unqualified identity `stylesheet.xsl` as `FXRS1001 / unsupported`.

The compiler seam is also exercised with an explicitly supplied private
resolver. It returns the same `FXRS0003 / denied` category for an admitted and
an unadmitted denied identity. After one missing lookup consumes a one-attempt
budget, a second lookup for admitted bytes returns `FXRS0006 / limit` before
snapshot access.

The private runtime mapping also preserves separate structured categories for
missing resource, denied authority, invalid identity, unsupported resolution
shape, and exhausted limit. Hosts do not need to parse the display detail to
distinguish those outcomes.

## Claim boundary

The qualification check recognizes bounded URI-scheme syntax; it is not a
claim of complete RFC 3986 parsing, normalization, or equivalence. This slice
does not admit relative/base-URI resolution, fragments, catalogs, redirect or
alias rules, a live resolver, filesystem or network access, or language
features such as `doc()`, `document()`, `unparsed-text()`, `xsl:include`, and
`xsl:import`.

The one-attempt compiler policy covers acquisition of the principal stylesheet
only. Broader per-invocation and per-compilation resolution budgets, resolver
ownership, catalog behavior, base identity, recursive dependency detection,
and host disclosure policy remain open. Therefore the roadmap's complete
URI/resource-resolution and execution-limits item remains unfinished.
