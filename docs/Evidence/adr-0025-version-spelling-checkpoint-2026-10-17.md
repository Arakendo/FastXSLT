# ADR-0025 legacy version spelling checkpoint

Date: 2026-10-17. Private integration evidence; no production promotion.

The matched-template lane now accepts equivalent unsigned numeric spellings
of its existing 1/2/3/4 witnesses, including `1`, `01.00`, `2.` and `0004.000`.
Root and literal-result local declarations use the same bounded interpretation.
Modern reference behavior and ordinary compilation remain unchanged.

[XSLT 1.0 forwards-compatible processing](https://www.w3.org/TR/1999/REC-xslt-19991116#forwards)
uses declared numeric values; the literal-result version attribute refers to
the [XPath Number production](https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-Number).
This tranche admits only exact alternate spellings of the existing witnesses,
without floating-point rounding. It does not settle general numeric comparison,
whitespace handling, or error classification for invalid version declarations.

Two regressions exercise root/local spelling combinations, nested policy use,
and rejection of nonwitness spellings with original resource provenance.
The guard deliberately includes valid-but-unimplemented values such as zero,
fractions and five, alongside malformed spellings. Unsupported is an experiment
boundary, not a claim that these values are invalid XSLT.

The remaining general-number seam is concrete: shared root validation currently
requires a positive decimal and allows a leading plus. It cannot serve as the
legacy Number validator unchanged. Extending beyond these witnesses requires
policy-aware validation, not rewriting stylesheet bytes or mapping every value
onto the `Two` witness. Modern local rules and general expression error timing
also remain integration gates under accepted ADR-0025; no new decision requested.

Focused validation: 15 processor-policy tests pass, including unchanged ten-case
OASIS replay (seven XML comparisons, two compilation errors, one execution error).
No corpus credit is added. Last full corpus baseline remains 2,501/3,173 modern
and 2,498/3,173 strict-preprocessing exact matches, with 423 expected-error
credits reported separately in each lane.

Repository verification passed: formatting, Clippy, 2,041 core tests (64
ignored), 22 native tests (3 ignored), 4 WASM tests, 18 worker tests,
documentation, local links and conformance inventory checks. The official
`wasm32-unknown-unknown` target check and `git diff --check` also passed.

## Follow-up: policy-aware numeric validation

The shared top-level compiler now accepts a private version-validation callback.
Ordinary callers still supply the original positive-decimal validator; the
test-only selected legacy lane supplies its Number validator. No source bytes
are rewritten, and the modern reference retains its existing witness gate.

The legacy context now represents normal versus forward processing directly,
not a synthetic `Two` version. Zero, fractions and larger numeric values enter
forward processing; numeric equality to one disables it. The implementation
uses the [XPath 1.0 double-precision number model](https://www.w3.org/TR/1999/REC-xpath-19991116/#numbers),
with lexical grammar checked before conversion. Tests include rounding to one,
root/local scope swaps, actual root-zero execution, and ordinary compiler
controls proving zero still fails and `+1.0` still passes baseline validation.
Non-admitted lexical forms remain explicit Unsupported outcomes in this private
lane; full invalid-input diagnostics and surrounding-whitespace treatment are
not claimed complete. Other top-level declarations still require edition-aware
integration, and no general graph or public profile support is inferred.

Follow-up validation: 2,042 core tests passed (64 ignored), plus the unchanged
native/WASM/worker suites. Clippy initially flagged exact float comparison;
the narrow allowance documents the specified equality test rather than using
an epsilon that would change semantics. Fresh full OASIS replays retain modern
2,501 exact / 14 mismatches and strict-preprocessing 2,498 exact / 17 mismatches,
both over 3,173 with 423 expected-error credits separate. Official WASM check
passed. There is no numerator change or promotion.
All repository verification gates completed successfully; the focused run also
passed all 16 tests, including the unchanged ten-case private corpus replay.

## Modern literal-result local scope

The matched integration now retains the modern declared-version witness in its
local context. Literal-result `xsl:version` uses that context to choose the
admitted value-of conversion through the existing shared expression compiler:
1.0 selects first-node conversion, while the modern witnesses select sequence
conversion. Root 1.0 supplies the inherited context too; it does not change the
processor stamp or preprocessing profile. This follows the
[effective-version rules](https://www.w3.org/TR/xslt-30/#backwards) while admitting
only the existing bounded expression shapes and version witnesses.

Two new execution tests pin nested and sibling restoration, propagation through
a static computed-element constructor, root-1.0 inheritance, and a same-input
legacy control. Modern output joins two text nodes; compatibility selects the
first. Legacy conversion stays first-node even inside a modern-looking local
declaration. No new evaluator or stylesheet rewrite is introduced.

Instruction-local version attributes, modern lexical expansion, general
expression behavior, unknown-instruction integration and introspection remain
guarded. This slice is not a complete modern compatibility implementation and
earns no baseline corpus credit.

Validation: all 18 focused tests passed, including the unchanged ten-case
fallback replay. Full repository verification passed with 2,044 core tests
(64 ignored), unchanged adapter suites, formatting, Clippy, documentation,
links and corpus inventories. Official WASM check and diff whitespace check
passed. Full OASIS totals are the preceding tranche's verified baseline, not
a new full replay for this test-only change.

## Modern instruction-local value-of scope

The private integration now admits unqualified `version` on modern `xsl:value-of`
for the existing four witnesses. It consumes the child's effective version,
rather than discarding it and lowering the instruction under its parent's mode.
The version affects the instruction itself and does not escape into siblings.
This follows the modern [standard attribute namespace rules](https://www.w3.org/TR/xslt-30/#standard-attributes).

Shared value-of construction was separated from attribute validation: ordinary
callers retain their existing validator; this private caller explicitly admits
only `select` and the already-validated modern `version`. Construction, expression
lowering, provenance and execution remain shared. This does not rewrite source
attributes or widen production attribute acceptance.

Two regressions pin both conversion directions, sibling restoration, immutable
processor qualification and legacy refusal. Negative controls retain provenance
for prefixed/duplicate-form version attributes, extra attributes, separators,
disable-output-escaping and unadmitted lexical forms. All 20 focused tests pass,
including the unchanged ten-case fallback replay. Other instruction/container
version declarations remain guarded; no corpus credit or production promotion.

Full verification passed: 2,046 core tests (64 ignored), unchanged adapter suites,
formatting, Clippy, documentation, links and corpus inventories. The official
WASM target and diff whitespace checks passed too. Full OASIS totals remain the
previously recorded baseline; only the focused unchanged corpus replay was
rerun for this tranche.

## Static computed-element container scope

The next private slice admits modern unqualified `version` on static-name
`xsl:element`. The existing shared constructor receives an already validated
attribute set and the profile-aware body callback; ordinary compilation retains
its original validator. Local context is applied on body entry, nested declarations
override it, and siblings retain their parent's context.

Two regressions prove nested modern/1.0 conversion, sibling restoration, legacy
refusal, and continued rejection of dynamic names, namespace overrides, prefixed
version attributes and attribute sets. All 22 focused tests passed, including
unchanged fallback corpus replay. No production/default change or corpus credit.
The concrete risk removed is loss of local version context at a shared container
body callback. Further container permutations are not the next priority;
reassess general expression error timing before expanding this bounded lane.

Full repository verification passed with 2,048 core tests (64 ignored), unchanged
adapter suites, formatting, Clippy, docs, links and corpus inventories. Official
WASM and diff whitespace checks passed. The full OASIS baseline was not rerun
for this tranche; the unchanged ten-case focused replay was.

## Bounded path-syntax error timing

The selected legacy lane now defers typed `XPST0003` syntax failures from the
shared location-path parser for a bounded unqualified simple-path lexical
domain in value-of. It does not catch instruction/compiler error codes generally.
General expressions must not be classified as invalid merely because a path
parser does not implement them; numeric expressions, calls, predicates and
qualified names are excluded from this classification. Parser Unsupported
outcomes remain unsupported. Unbound variables remain static errors.

Attribute and child-content validation precede this deferral. Since the admitted
value-of evaluates its complete select expression when reached, the existing
shared deferred-failure instruction is a valid evaluation boundary for this
slice. It is not a strategy for short-circuited subexpressions or function calls.
Legacy non-forward and admitted modern non-forward contexts retain static
syntax failure; modern forward error handling remains explicitly guarded.

The requirement comes from [XSLT 1.0 forward processing](https://www.w3.org/TR/1999/REC-xslt-19991116#forwards).
Two focused tests pin dormant/reached/reused programs with original provenance,
modern static behavior, and unsupported-versus-invalid controls. All 24 focused
tests passed, including the unchanged ten-case corpus replay. The first full
gate caught duplicate empty match arms; they were combined without semantic
change. No production behavior or corpus numerator changes are claimed.

Final verification passed: 2,050 core tests (64 ignored), unchanged adapter
suites, formatting, Clippy, docs, links and corpus inventories. WASM target
and diff whitespace checks passed. The 1,048-line matched integration now warrants
an ADR-0004 review before further growth; its inline test matrix is a distinct
private extraction candidate. Full OASIS totals remain the prior full replay.

## Checkpointed regression extraction

ADR-0004 review: the 1,048-line integration combined profile-aware lowering
with a substantial independent regression matrix. Move that matrix to private
`matched_integration_tests.rs`, preserving the `tests` child module name and
all assertions, fixture selection and helper behavior. After formatting the
implementation is 453 lines and the test owner 577 lines. The directory has
seven direct files, below the directory-density review threshold.

Dependency direction remains tests to private implementation; no visibility,
runtime state, policy or semantic ownership moves. No new API, ABI, unsafe
surface or runtime allocation/dispatch is introduced. This is a structural
checkpoint, not a semantic fix or performance claim. Compilation-time pressure
did not trigger this extraction and no build-speed improvement is claimed.

Conservation passed: all 24 focused tests including original corpus replay,
2,050 core tests (64 ignored), unchanged adapter suites, complete repository
verification, official WASM target and diff whitespace checks. No semantic
changes were combined with the extraction. The next investigation must separate
instruction reachability from actual function-call evaluation; the existing
whole-instruction deferred failure is not proof of short-circuit call timing.

## Direct function-call arity timing

The private selected lane now reuses the existing system-property arity
classifier for a complete direct select call with zero or only string-literal
arguments. Argument splitting and literal recognition are shared; there is no
second function catalog or broad compiler-error catch. Legacy forward mode
defers XPST0017 to reached execution; legacy normal and modern non-forward
contexts retain static failure. Modern forward mode remains guarded.

This intentionally admits no successful introspection, processor-version claim,
general unknown-function handling, variable argument, nested call or Boolean
short circuit. Those shapes must not be collapsed into an instruction-level
failure. Two tests pin dormant/reached/reuse/provenance, static controls and
continued unsupported classification for those other shapes. All 26 focused
tests passed, including the unchanged ten-case fallback replay. No corpus
numerator or production default changes.

Full verification passed: 2,052 core tests (64 ignored), unchanged adapter
suites, formatting, Clippy, docs, links and corpus inventories; official WASM
and diff whitespace checks passed. Full OASIS totals remain the previous full
replay, not newly measured counts for this private slice.
