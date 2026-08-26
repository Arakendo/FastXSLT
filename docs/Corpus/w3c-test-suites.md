# W3C Test Suite Provenance

| Field | Value |
| --- | --- |
| Admitted | 2026-08-25 |
| Source pressure | Match the core conformance submodules used by the TS XSLT peer project |
| Local ownership | Immutable upstream submodules; FastXSLT owns only its harness and overlays |
| Conformance claim | None |

## Pinned sources

| Suite | Upstream | Local path | Revision |
| --- | --- | --- | --- |
| QT3 | `https://github.com/w3c/qt3tests` | `vendor/qt3tests` | `83993587711dbd5c18ed846385ec37d079d6e492` |
| XSLT 3.0 test suite | `https://github.com/w3c/xslt30-test` | `vendor/xslt30-test` | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |

These are the exact revisions observed in `F:\LocalSource\TS XSLT` on
2026-08-25. Pinning the same inputs permits later harness and result comparisons
without assuming the two engines support the same standards surface.

## Acquisition and integrity

Initialize a clone with:

```text
git submodule update --init --recursive
./scripts/check-conformance-sources.ps1
./scripts/inventory-conformance-sources.ps1
./scripts/inventory-xslt30-case-metadata.ps1
```

The check requires both catalog roots, verifies each submodule HEAD against the
table above, and rejects local modifications. Git object identity supplies the
content pin; the submodule URL supplies reproducible acquisition.

The parent repository records only each submodule URL and pinned gitlink; it
does not commit the W3C repositories' individual files. `.gitmodules` uses
`ignore = all` so ordinary parent-repository status and commits do not include
submodule worktree noise. The explicit conformance-source check remains the
authority for detecting a missing, moved, or locally modified suite.

The inventory command securely walks the root catalogs and their directly
referenced test-set documents without resolving DTDs or external resources. At
the admitted revisions it discovers 31,821 QT3 cases in 428 test sets and
14,600 XSLT30 cases in 234 test sets. The retained method, result, and
limitations are in the [catalog inventory evidence](../Evidence/w3c-suite-catalog-inventory-2026-08-25.md).
The XSLT30 metadata inventory additionally records dependency, environment,
stylesheet-reference, and assertion-family pressure across every pinned case;
its retained results and limitations are in the
[case-metadata evidence](../Evidence/xslt30-case-metadata-inventory-2026-08-25.md).

Do not edit upstream catalogs, environments, sources, assertions, or expected
results. FastXSLT-owned selection manifests, expected unsupported
classifications, harness corrections, and issue references belong outside
`vendor/`. The initial selection manifest is
`corpus/overlays/xslt30/private-slice-v0.toml`; it names upstream case identity
without copying suite content.

## Licensing boundary

FastXSLT's MIT license covers FastXSLT-authored code and documentation. It does
not relicense either W3C suite or third-party material nested inside a suite.

The pinned QT3 guide states that the suite is available under the W3C test-suite
license terms and links the W3C legal policy. W3C's current
[test-suite licensing explanation](https://www.w3.org/copyright/test-suites-licenses/)
describes distinct terms for development use and for unmodified tests used to
support performance or conformance claims. The
[W3C test-suite license](https://www.w3.org/copyright/test-suite-license-2023/)
requires preservation of source, copyright, and status notices and restricts
modification under that license.

The XSLT30 repository includes test material and nested third-party fixtures
with their own `COPYING`, `LICENSE`, and `NOTICE` files. Preserve the complete
submodule and its notices. Before distributing a subset, modified case, bundled
archive, crate package, or public conformance report, perform a focused license
and trademark review for that exact use.

The submodules are development/test inputs and are not dependencies linked into
the FastXSLT library artifact.

## Standards and result authority

QT3 covers XPath and XQuery editions and optional features beyond whatever
FastXSLT initially selects. The XSLT30 suite likewise includes dependencies and
features that may remain outside the accepted profile. Presence in `vendor/`
does not make a case supported and does not choose AR-0001's standards decision.

Any published report must record:

- these exact suite revisions;
- the accepted FastXSLT standards profile and feature configuration;
- selection and exclusion rules;
- selected, excluded, unsupported, passed, failed, and harness-error counts;
- unmodified upstream case identity;
- FastXSLT revision, toolchain, target, and harness revision; and
- any reference processor and version used for differential evidence.

## Update procedure

1. Open a focused plan or evidence record naming the reason to update.
2. Fetch upstream without editing the current suite worktree.
3. Review upstream commits, catalog/schema changes, nested license changes, and
   case-count effects between the old and proposed revisions.
4. Run the old and new revisions through the same harness and classify result
   movement.
5. Update the gitlink, this revision table, the verification script, and any
   selection manifest in one reviewable change.
6. Retain prior published result records against their original suite revisions.

An upstream default-branch movement alone is not a reason to update.
