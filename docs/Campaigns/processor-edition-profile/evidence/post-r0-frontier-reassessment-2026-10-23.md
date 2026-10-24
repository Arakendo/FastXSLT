# Post-R0 corpus frontier reassessment

Subsequent disposition: the maintainer approved the proposed comment slice.
Its [implementation and replay result](post-r0-selected-comments-2026-10-23.md)
is recorded separately; the proposal and initial observations below remain intact.

Date: 2026-10-23. Source: committed R0 `b5935c8e`. This is an investigation
and proposed next slice, not a revision of the closed R0 scope or an admission.

## Replays and constraints

Fresh release replays through the unchanged measurement script retain:

- Ordinary declaration-driven / modern preprocessing: **2,504/3,173 exact**,
  16 output mismatches, 78 ordinary initialization failures and 11 ordinary
  execution failures. Expected-error outcomes remain separate.
- Promoted selected legacy / strict preprocessing: **358/3,173 exact**.
  The R0 promotion reports and manifest were not overwritten.

Tracing the ordinary XPath and execution frontiers confirms deferred boundaries:
`idkey49`, `DocumentFunctionNoArg`, `DocumentInUnion` and `Bug76984` need
mixed-document selection/union behavior; `expression02` and `bvt089` need
unparsed-entity semantics; `PerfRepro2` retains the documented aggregate/extension
boundary. These are not permission to introduce ordering, entity authority or
extension support. The [earlier mismatch audit](../../oasis-xslt10/evidence/documents-and-resources/oasis-xslt10-literal-document-path-binding-2026-10-17.md#2026-10-18-mismatch-and-numeric-composition-audit)
already explains why several sorting/attribute-set mismatches must not be fixed
by weakening whitespace comparison.

## Selected-profile next-slice evidence

Selected `FXAR0031` ordinary initialization traces include 2,220 generic
policy-propagation refusals and 108 bounded-module attribute refusals. Those
messages alone are not a useful feature-yield estimate. A read-only exploratory
mapping of diagnostic source spans to uniquely named extracted fixture files
found these repeated first-refusal shapes:

| Reported shape | Mapped cases | Interpretation |
| --- | ---: | --- |
| Descendant `for-each` selecting `//center` | 82 | Path admission plus downstream bodies; not 82 promised gains |
| Absolute-path `copy-of` selecting `/xslTutorial` | 64 | Selection/construction integration; downstream guards remain |
| `xsl:comment` start tag | 32 | Existing shared constructor, selected instruction not admitted |

This mapping is exploratory, not a conserved classification: ambiguous filenames,
unmatched diagnostics and unusable raw spans were omitted, not reclassified in
the corpus. An initial mapping attempt produced invalid-length errors and was
discarded; the rerun used explicit regex captures and validated span bounds.
No corpus denominator or outcome was changed. Counts describe first refusal,
not eventual exact output.

Direct case trace confirms `Lotus/message_message11#1` refuses at the
`xsl:comment` start tag (source span 453..466). Its comment body contains a
child-path value inside a message. The ordinary compiler already owns
`compile_comment`; selected matched integration has no admitted comment arm.

## Proposed bounded follow-up — approval required

Recommend selected `xsl:comment` integration first, because it can reuse an
existing result constructor and has a concrete unchanged witness. This would
be separately scoped post-R0 breadth, not reopening the nine completed gates.

1. Route comment content through selected-policy body lowering and the existing
   shared comment construction/runtime. Do not call baseline body lowering.
2. Pin normal/forward local version behavior, rejected attributes, reached and
   dormant expression failures, comment content conversion, diagnostic provenance,
   budget/cancellation and reuse. Keep every other current guard intact.
3. Replay `message11` and the affected comment cases, then the full fixed selected
   and ordinary profiles. Record actual gains and subsequent refusals separately;
   retain R0's report. Run repository and official WASM gates after Rust changes.

Stop after this slice and report the outcome. Do not add PI constructors, new
path grammar, output encodings or broader graph support because another case
then requests them. Any ownership or error-timing issue returns for review.

The R0 ceiling does not currently include comments, so this record requests
maintainer approval before implementation. No engine changes were made during
this reassessment. No new AR/ADR, public selector or default is proposed.

## Reproduction

```powershell
./scripts/measure-oasis-xslt10.ps1
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier FXXP1001
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier FXRT
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier invalid/
./scripts/measure-oasis-xslt10.ps1 -ProcessorProfile selected-legacy -PreprocessingProfile strict-xslt10 -TraceFrontier FXAR0031
./scripts/measure-oasis-xslt10.ps1 -ProcessorProfile selected-legacy -PreprocessingProfile strict-xslt10 -TraceCase message_message11
```

## After comments: proposed named element-copy paths

Subsequent disposition: the maintainer approved this slice. The
[implementation report](post-r0-selected-copy-paths-2026-10-23.md) records its
completion; the proposal below retains its original inspection-time wording.

The next reassessment, after the approved comment slice, retains 393 selected
exact and 2,504 ordinary exact cases. No additional engine edits were made.

Named witness: `Microsoft/Copying_Copy-of_CopyOfFromDocumentRoot#1`. Its unchanged
stylesheet copies `/xslTutorial`. The selected entry refuses the instruction at
source span 119..154 with `Unsupported / FXAR0031`; ordinary shared lowering
reaches the existing XML comparison. The catalog identity, not the stylesheet's
different filename, is required for `-TraceCase CopyOfFromDocumentRoot`.

`selected_child_copy` currently requires one ASCII NCName. It then delegates to
`compile_copy_of`, requires a `CopyOfLocationPath` result and explicitly supplies
selected legacy recovery. The shared compiler already owns rooted location
paths. This identifies an admission gap, not a missing copy runtime.

Proposed next slice, pending approval:

- Admit nonempty slash-separated ASCII child names, optionally with a single
  leading slash, only for selected `xsl:copy-of`.
- Retain the shared path compiler and `CopyOfLocationPath` result check. No
  baseline body lowering or automatic expression admission.
- Exclude `//`, `/` alone, wildcards, prefixes, predicates, attributes, namespace
  axes, variables, functions, unions and `document()` from this slice.
- Pin relative versus root selection under non-root focus, multiple sibling
  matches and order, copied subtree namespaces/attributes, empty selection,
  effective stripped relationships, failure provenance, controls and reuse.
- Replay the named witness and both complete 3,173-case profiles, run repository
  and WASM gates, then stop. The earlier 64 repeated `/xslTutorial` refusal
  observations are a leverage clue, not a promised gain; other guards may follow.

This is narrower than a PI-constructor slice (which also needs target/name and
content error-timing treatment), and narrower than descendant iteration (which
combines selection with body/focus behavior). Neither alternative is admitted.

One exploratory trace pipeline terminated its Rust writer early by selecting
only the first output rows, causing Windows broken-pipe errors. Those runs were
discarded. Subsequent traces collected the complete command output before
filtering and completed successfully; this was not an engine semantic failure.

R0 and the comment slice remain complete. Approval of the next slice is requested;
no new obligation, architectural review, default or public selector is proposed.
