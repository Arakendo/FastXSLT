# Adversarial Review: First Correctness Tranche

| Field | Value |
| --- | --- |
| Date | 2026-08-30 |
| Trigger | `docs/Reviews/adversarial-engine-review-2026-08-30.md` |
| Scope | Findings 1-5, 8, and 9 |
| Outcome | Corpus integrity repaired; five silent semantic defects corrected; one unadmitted semantic family rejected explicitly |

## Corpus integrity

The review reproduced a green workspace test run over an invalid
`private-slice-v0.toml`. The file contained one missing rationale and one
duplicate rationale key, while corpus execution tests searched unrelated
strings globally.

The repaired test boundary now:

- parses the complete private slice and output denominator with `toml` into
  strict `serde` records;
- rejects duplicate and unknown fields, missing required fields, empty
  rationales, duplicate suite-native identities, unknown dispositions, and
  incoherent selection/execution pairs;
- conserves the output denominator's seven assertion-family counts to exactly
  232 cases;
- binds an execution assertion to the same typed `(set_file, case_name)` record;
  and
- proves that changing `template-001` to `engine-unsupported` fails its pass
  requirement even while other records remain passed.

The misplaced recover rationale now belongs to `conflict-resolution-1202a`;
`conflict-resolution-1202b` retains only its error-policy rationale.

`serde 1.0.229`, `toml 0.8.23`, and their newly locked transitive packages are
test-only dependencies. Their registry manifests declare `MIT OR Apache-2.0`,
except `winnow 0.7.15`, which declares `MIT`; these terms are compatible with
the MIT library and no dependency enters the published runtime artifact.

## Semantic repairs

| Finding | Implemented boundary | Regression evidence |
| --- | --- | --- |
| 2 -- path duplicate/order normalization | Every location-path step now sorts by XDM document-order rank and removes repeated `NodeId` values, independent of axis spelling. | `/r/a/..` returns one `r`; the same selection invokes its template once. |
| 3 -- temporary `xsl:copy` | Temporary focus now reaches the ordinary compiled shallow-copy instruction path. Constructed attributes and the compiled body execute; descendants are not copied implicitly. | An outer temporary copy emits a constructed attribute and body-selected child; an empty inner copy excludes its `lost` descendant. |
| 4 -- `xml:space` data loss | The exact strip-all profile rejects any source containing an `xml:space` declaration as `FXRT1014 / unsupported`. Broader inheritance/default semantics remain unadmitted under ADR-0012. | Both `preserve` and nested `default` declarations fail before a visibility view is constructed. |
| 5 -- namespace copy fixup | Source element copies retain the source node's effective in-scope bindings assembled from its ancestor lineage. | Isolated prefixed descendants serialize under both `xsl:copy-of` and shallow `xsl:copy`. |
| 8 -- temporary focus | Temporary path and built-in selections carry their real position and size through template and `next-match` execution. | Two selected temporary siblings observe `1/2` and `2/2`. |
| 9 -- forward globals | Same-module forward and cyclic variable-default dependencies fail during compilation as `FXST1044 / unsupported`; admitted backward dependencies remain unchanged. | Forward, cyclic, and backward controls execute at the compiler boundary. |

The complete engine test suite passed after Findings 1-4 and 8 were repaired;
the final workspace gate for this tranche is recorded with the commit that
closes the checkpoint.

## Remaining review work

- Finding 6 requires measurement and review of a native registry quota and
  ownership policy before changing process-wide behavior.
- Finding 7 requires template-candidate and cancellation-gap instrumentation
  before selecting budget units or an index.
- Finding 10 requires serialized worker control writes plus a partial-write and
  concurrent-frame stress test.
- Findings 11 and 12 remain explicitly performance hypotheses under AR-0013;
  no cache, frame representation, interning strategy, or index is selected.
- Broader source-versus-temporary, namespace, whitespace, and axis-pair
  differential matrices remain useful follow-up beyond these minimal
  counterexamples.
