# ADR-0023: Compilation Scoped Stylesheet Preprocessing Profiles

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-10-08 |
| Related review | [AR-0029](../Architectural%20Reviews/AR-0029-stylesheet-preprocessing-edition-and-legacy-whitespace.md) |
| Related decisions | ADR-0004, ADR-0007, ADR-0012 |
| Supersedes | None |

## Context

AR-0029 demonstrates an observable stylesheet-text distinction between strict
XSLT 1.0 and modern preprocessing. A local-version candidate gained one
Microsoft comparison but lost four Lotus comparisons; it was withdrawn.
An independent `XslCompiledTransform` probe supports the strict behavior while
also disagreeing with those archived Lotus expectations. Neither an archival
numerator nor a local compatibility helper can select the compilation edition.

ADR-0007 retains the modern standards reference. This decision adds an explicit
edition-sensitive preprocessing boundary, not another engine or a declaration
of complete XSLT 1.0 support.

## Decision

FastXSLT selects an explicit **strict XSLT 1.0** or **modern** stylesheet-
preprocessing profile once at the compilation/package graph boundary. Selection
is initially private. These labels describe semantics, not a selected Rust type,
public API, ABI or wire representation.

The chosen profile applies to the principal stylesheet and every transitive
include/import module. Embedded and literal-result stylesheets use that same
profile, preserving relevant XML ancestry. Principal-module version heuristics
and local `xsl:version` declarations do not select or switch preprocessing.
Local version context continues to govern its legitimate expression and static
compatibility rules independently. Here, package scope means the compilation
graph; it does not admit `xsl:package` or `xsl:use-package` support.

Strict preprocessing applies the stylesheet text-node whitespace rules of
[XSLT 1.0 section 3.4](https://www.w3.org/TR/xslt.html#strip), including
`xsl:text` and effective `xml:space`. Ignoring comments/PIs must not first merge
a whitespace-only fragment into neighboring non-whitespace text and thereby
change that fragment's stripping decision.

Modern preprocessing follows
[XSLT 3.0 section 4.3](https://www.w3.org/TR/xslt-30/#strip-stylesheet), including
comment/PI removal and adjacent-text merging before whitespace stripping, and
the applicable instruction-specific stripping rules. The selected direction
does not imply that every modern rule is already implemented.

Both profiles feed the same compiler, lowering and runtime. Shared sequence
compilation and static/temporary constructors must consume consistently
preprocessed stylesheet content rather than choosing their own edition.
Compiled artifacts are profile-confined and must not be reused as though
compiled under another profile. This does not select a cache or physical layout.

XML adaptation retains the information and original provenance required by
either profile. Diagnostics, text charges, cancellation and bounded compilation
must survive preprocessing; merging does not authorize fabricated source
locations. Prepared source whitespace views remain governed by ADR-0012.
No parser-side global stripping, mutable prepared XDM, new acquisition authority
or adapter-specific semantics is admitted.

## Integration and corpus accounting

Acceptance selects the boundary; implementation remains pending. Existing
entry points retain current modern preprocessing until explicit profile
integration passes its gates. No public selector or default production profile
is selected here.

The OASIS XSLT 1.0 campaign will exercise strict preprocessing after integration;
modern-suite testing will exercise modern preprocessing. Reports must identify
the profile and preserve original suite expectations, case identities and
denominators. A modern-baseline comparison may remain separately visible but
cannot be substituted for strict-profile results or cherry-picked per case.

If strict behavior disagrees with an archival expectation, retain the mismatch
in original exact accounting. Where independent evidence warrants it, record
an archival-oracle disagreement separately. A corrected or edition-specific
oracle never replaces the original exact numerator. Acceptance itself earns
no additional pass and does not predict the next replay's numerator.

## Alternatives not selected

- Retain modern preprocessing as the only long-term compatibility behavior:
  insufficient for an eventual meaningful strict XSLT 1.0 profile.
- Infer preprocessing from principal or local version: conflates graph edition
  with expression compatibility and fails the reviewed local-context experiment.
- Introduce a legacy compiler/runtime: unnecessary duplication of semantics.
- Normalize expected whitespace or dispatch by fixture: hides observable
  differences and compromises corpus accounting.

## Implementation gates

- Thread explicit private selection through the whole compilation graph,
  reviewing ADR-0004 ownership pressure before adding responsibilities.
- Test comment/PI-separated fragments, `xsl:text`, inherited/reset `xml:space`,
  and instruction-specific stripping independently under both profiles.
- Prove mixed-version include/import, embedded and literal-result stylesheet
  inheritance; audit shared and static/temporary constructor paths.
- Preserve original diagnostic provenance, exact text-budget boundaries,
  cancellation, immutable resources and source-view behavior.
- Replay unchanged OASIS accounting under strict mode and pinned modern cases
  under modern mode; retain differential controls and profile-labelled reports.
- Run repository Rust, documentation, conformance and official WASM gates after
  implementation. Review public profile/facade selection separately.

## Consequences and non-decisions

Archival exact coverage may fall while strict semantics improve. This is an
acceptable outcome, not grounds for weakening the profile. One compilation
cannot switch preprocessing edition between modules or instructions.

This decision does not establish full 1.0 conformance, choose a public edition
API, change local expression compatibility, expand resource/DTD authority,
select a second backend or make the compiler's representation public.

## Acceptance history

- 2026-10-08 -- Maintainer selected explicit strict/modern compilation-graph
  profiles, private initially, with unchanged archival comparison rules.
  Architecture accepted; runtime/compiler integration remains pending.
