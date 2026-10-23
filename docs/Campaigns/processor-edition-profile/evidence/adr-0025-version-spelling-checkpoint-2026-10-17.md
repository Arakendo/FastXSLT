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

## 2026-10-18 literal binding and fallback whitespace checkpoint

The unchanged `Microsoft/BVTs_bvt023#1` provides the next whole-case pressure.
Its private selected-legacy compilation initially stopped at explicit UTF-8
output metadata. The bounded lane now admits that metadata through the shared
output owner and literal-string local selects through the existing static
atomic-variable constructor. Numeric/dynamic selects and broader constructors
remain guarded; duplicate names, select-plus-content and sibling-scope leakage
remain errors. This does not promote a production processor profile.

The shared fallback attribute validator now accepts `xml:space="default"` and
`xml:space="preserve"`, and rejects other values with attribute provenance.
Whitespace still comes from canonical stylesheet preprocessing and the shared
sequence executor; no fallback-specific whitespace evaluator was introduced.
This follows [XSLT 1.0 whitespace stripping](https://www.w3.org/TR/1999/REC-xslt-19991116#strip)
and [fallback instantiation](https://www.w3.org/TR/1999/REC-xslt-19991116#fallback).
Focused tests combine nested reset/preserve scopes, multiple fallbacks,
standalone fallback suppression, literal binding reuse and lexical isolation.

The whole archival case does **not** pass yet: the next selected-lane refusal
is the value-of at bytes `618..666`, `(//author)[1]/last-name`. A trial through
the ordinary shared value compiler confirmed that this grouped constant-position
selection followed by a path suffix is unsupported there too. The spelling was
not left admitted by the private wrapper. The new ignored corpus test pins that
actual frontier, not a successful output or an adjusted fixture.

All 29 matched-integration checks pass with the separately acquired corpus,
including the existing ten-case fallback replay and this negative classification.
Next investigate shared grouped positional selection/suffix composition before
expanding unrelated profile eligibility. No exact numerator increase is claimed.

Full validation passes: 2,080 core tests (65 ignored, including the new external
corpus frontier probe), 22 native, 4 WASM and 18 worker tests; formatting, strict
Clippy, documentation, 1,998 Markdown link checks, pinned corpus checks and the
official WASM target. Fresh full sweeps retain modern 2,502 exact / 16 mismatches
and strict preprocessing 2,499 exact / 19 mismatches out of 3,173; both retain
2,705 initialized, 2,656 executed and 423 separate expected-error credits.

## 2026-10-18 grouped literal positions close the BVT integration slice

The preceding negative `bvt023` frontier is now superseded. The shared XSLT 1.0
value compiler accepts a numeric-literal operand in its existing grouped-path
position/suffix instruction. A private typed operand distinguishes variable
names from numeric lexicals; both use the same charged evaluator, effective
source navigation, positional selection and first-node string conversion.
Literal lexicals are borrowed during execution and included in compiled-state
retention accounting. No second evaluator, source ownership rule or public
profile is introduced.

Following [XPath 1.0 filter-expression composition](https://www.w3.org/TR/1999/REC-xpath-19991116/#node-sets),
the position applies to the complete selected sequence before suffix navigation,
not independently to each parent's children. Tests compare numeric variables
and literals at positions 1, 2.0 and 3, plus zero, negative, fractional,
out-of-range and empty-source outcomes. Multiple suffix results retain first-node
conversion. Both whitespace representations, exact/one-less visit budgets,
cancellation and subsequent reuse are exercised. A descendant-abbreviation
suffix remains unsupported instead of being misinterpreted as an absolute path.
These tests do not establish general dynamically typed variable-predicate
semantics; the pre-existing variable route's string coercion warrants a separate
typed-predicate regression rather than serving as an oracle for nonnumeric values.

The private profile admits the bounded descendant-name/numeric-position/child-name
shape through this shared compiler. Unchanged `Microsoft/BVTs_bvt023#1` now
passes the original XML comparator, including nested fallbacks, whitespace and
literal bindings. Its ignored test is now a positive replay. This is additional
selected-profile evidence, not baseline corpus credit or profile promotion.

The first full replay caught a one-case initialization regression in
`Lotus/position_position85#1`: nested grouping in `((ancestor::foo))[1]/@att1`
was passed to the axis parser as `(ancestor::foo)`. The grouped compiler now
uses the existing enclosing-parenthesis normalizer. A focused test verifies
single/nested grouping and document-order selection across the reverse ancestor
axis. The initial 2,501/2,498 exact counts were rejected as a regression checkpoint,
not presented as an acceptable trade for the private BVT gain.

Final validation passes: 2,083 core tests (65 ignored), 22 native, 4 WASM and
18 worker tests, all 29 corpus-enabled matched-integration tests, full repository
verification and the official WASM target. Full debug-harness corpus replays
restore modern 2,502 exact / 16 mismatches and strict preprocessing 2,499 exact /
19 mismatches out of 3,173; both retain 2,705 initialized, 2,656 executed and
423 expected-error credits. No archive or comparator changes were made.

## 2026-10-18 typed grouped-variable predicate correction

The follow-up closes the numeric-coercion defect identified above. In the shared
grouped selection/suffix evaluator, a numeric variable remains positional;
strings, booleans and admitted node selections use their effective boolean
meaning. In particular, the string `'2'` keeps all selected nodes rather than
selecting node two. Completed local/global bindings retain their value kind;
the invocation resolver remains the owner of reads during initialization.

When a true predicate retains several contexts, each suffix is evaluated through
the charged path evaluator. The value consumer retains only the earliest result
node in document order, not an accumulated union or the first context's answer.
This handles a missing first suffix and convergent parent navigation without
inventing cross-document ordering. Numeric-literal behavior is unchanged.

Focused regressions cover numeric versus string values, empty strings, true and
false booleans, populated/empty node selections, local/global bindings and both
whitespace representations. Multi-context controls pin exact/one-less node-visit
budgets, late cancellation and subsequent reuse. A global-constructor probe still
reports `FXRT1004`; this correction does not widen constructor admission or select
a processor profile. Local boolean constants use the already-admitted constant
`contains()` route; admitting local `true()`/`false()` is separate compiler work.

This is a semantic defect correction, not additional corpus credit. Unchanged
full corpus replays retain modern 2,502 exact and strict preprocessing 2,499 exact
out of 3,173, with 2,705 initialized, 2,656 executed and 423 separate expected-error
credits. Resume the next named frontier or ADR-0025 integration gate rather than
extending grouped predicates without another demonstrated failure.

Validation: all repository gates pass with 2,086 core tests (65 ignored), 22
native, 4 WASM and 18 worker tests. The official `wasm32-unknown-unknown` check,
1,998-document Markdown link check and `git diff --check` also pass.

## 2026-10-18 Boolean call-boundary timing

The next bounded slice addresses the specific hole left by direct-call deferral:
`false() and system-property()` must not fail merely because its value-of is
reached, whereas `true() and system-property()` must fail when the zero-argument
call is reached. The rules are [XSLT 1.0 forward-compatible call timing](https://www.w3.org/TR/1999/REC-xslt-19991116#forwards)
and [XPath 1.0 Boolean short circuit](https://www.w3.org/TR/1999/REC-xpath-19991116/#booleans).

The selected-profile compiler lowers bounded `and`, `or`, `not()`, `boolean()`,
grouping, Boolean constants and unqualified child-path truth to the existing
shared Boolean IR. A private deferred-failure operand retains the diagnostic
and original source location. The shared Boolean evaluator encounters that
operand only if ordinary left-to-right short circuit reaches it. Boolean text
output uses the existing choose/text lowering; there is no probe interpreter,
speculative evaluation, or instruction-wide error substituted for the expression.
Ordinary compilation does not produce the new deferred operand.

This slice recognizes only the zero-argument `system-property()` call as the
failure leaf. It does not implement successful processor-version introspection,
unknown functions, argument evaluation, qualified functions, arbitrary expression
grammar or modern forward-compatible error timing. The whole tree must fit the
bounded admission grammar before any deferral is selected. Unsupported siblings,
malformed calls, variables and predicates stay guarded even behind a constant
short circuit. The private byte/depth bounds are 4,096/32, not public limits.
Legacy normal and admitted modern non-forward contexts retain static `XPST0017`,
including for a call on a branch that would otherwise be skipped.

The regression matrix distinguishes dormant templates, skipped operands, reached
operands, nested calls, Boolean precedence and reversed operands. Source-dependent
truth selects different outcomes on repeated uses of the same compiled program,
so the proof is not just constant folding. The shared-runtime control test pins
zero call-operation charges when skipped, one charge before a reached error,
exact/one-less operation budgets, cancellation at that first charge, request and
source-span ownership, failure cleanup and subsequent reuse. The initial test
incorrectly allowed one charge before signaling cancellation; it was corrected
to signal on the first charge, without changing runtime semantics.

ADR-0004 review: bounded call lowering lives in a 116-line named private child,
not in the existing general Boolean compiler. That compiler exposes three
existing lexical helpers only to its parent module; no new public parser or
runtime dependency is introduced. The matched integration is 534 lines and its
test owner 821 lines; its subject directory has eight files. Runtime changes are
the reached-leaf dispatch and retention accounting. The large runtime parent
retains existing decomposition debt, not a new evaluator. A separate runtime
control-test file keeps these assertions out of that parent; the 15-file test
directory remains grouped by semantic responsibility, with no useful multi-file
new subsubject to move at this checkpoint.
Full lint review also moved the existing focus equality/comparison branches to
their position/focus helper and grouped string/location retention in its existing
accounting helper. These are delegation changes with unchanged arguments and
charges. The new IR variant has a precise non-test dead-code expectation because
only selected-profile test lowering constructs it; this does not enable a
production compiler route.

This removes one error-timing risk under ADR-0025, not the entire general-expression
gate. Argument evaluation and successful capability/introspection reconciliation
remain unproved, and production profile promotion remains gated. No baseline
corpus credit, default-policy change or public selector follows from this slice.

Final validation passes: 2,094 core tests (65 ignored), 22 native, 4 WASM and
18 worker tests, formatting, strict all-target/all-feature Clippy, docs, 1,998
Markdown link checks, pinned corpus checks and the official WASM target. The
32 selected-profile tests pass with the separately acquired corpus enabled.
Fresh full corpus replays preserve modern-preprocessing 2,503 exact / 16
mismatches and strict-preprocessing 2,500 exact / 19 mismatches out of 3,173.
Both retain 2,706 initialized, 2,657 executed and 423 separate expected-error
credits. No archive or comparator changes were made. `git diff --check` passes.

## 2026-10-18 Static lexical reads inside deferred-call expressions

The next error-timing risk was hiding an unbound variable behind a skipped call
or Boolean operand. The bounded selected-profile compiler now resolves plain
local variable reads against its existing lexical bindings after admitting the
complete expression, but before selecting call deferral. Both Boolean operands
are checked statically: short circuit governs execution, not lexical scope.
An absent binding remains `XPST0008` with the original location, including in a
dormant template and in legacy-forward, legacy-normal and modern contexts.

Bound reads lower to the existing typed variable effective-Boolean-value IR and
shared evaluator. Empty strings are false; the strings `0` and `false` remain
true. Tests repeat execution, distinguish skipped and reached bad calls, reject
reads before declaration and across sibling fallback scopes, and preserve an
outer binding through fallback and subsequent execution. No new variable store,
shadowing rule, evaluator or production profile route is introduced.

This extends only the bounded Boolean slice's plain local reads. Qualified
variables, arbitrary argument expressions, successful introspection and general
expression admission remain outside it. The earlier statement that variables
remain guarded is superseded only for these statically resolved plain reads.
The named lowering module is now 148 lines and its integration test owner 910;
the responsibility and directory grouping remain unchanged.

Validation: all 35 selected-profile tests pass with the acquired corpus enabled.
The full verification script passes with 2,097 core tests (65 ignored), 22 native,
4 WASM and 18 worker tests, strict Clippy, formatting, documentation, 1,998
Markdown link checks and pinned corpus checks. The official WASM target check
also passes. Fresh full replays preserve modern-preprocessing 2,503 exact / 16
mismatches and strict-preprocessing 2,500 exact / 19 mismatches of 3,173. Both
retain 2,706 initialized, 2,657 executed and 423 separate expected-error credits.
No archive, comparator, production default or baseline credit changed.

## 2026-10-18 identity-function capability reconciliation

Auditing the capability gate found a baseline inconsistency: static
`function-available()` returned false for unprefixed `id`, `key` and `generate-id`,
although their shared compiler/runtime routes already execute. A new compiled
regression first failed with `false|false|false` and skipped all three guarded
operations. This is an implementation defect, not a new recognition policy.

The existing static introspection table now includes those three functions.
The same regression executes a typed-ID name lookup, key count and generated
identity behind the advertised checks. Effective-view and complete-reference
execution agree, including generated identity. The default namespace does not
qualify unprefixed function names. Qualified `p:id`, `xsl:key` and Microsoft
`node-set` remain unavailable; unparsed-entity support remains unadvertised.
This follows the expanded-name/function-library rule in
[XSLT 1.0 section 15](https://www.w3.org/TR/1999/REC-xslt-19991116#fallback).

The correction uses the existing compile-time owner and adds no runtime
dispatcher, registry, extension authority or public selector. It does not
change `system-property('xsl:version')`, promote the selected-processor reference,
or treat every recognized standard function as implemented. Reconciliation of
the complete selected-profile catalog and its executable lowering remains open.

Full unchanged OASIS replays retain modern preprocessing 2,503 exact / 16
mismatches and strict preprocessing 2,500 exact / 19 mismatches, out of 3,173.
Both retain 2,706 initialized, 2,657 executed and 423 separate expected-error
credits. Thus this tranche removes a concrete misleading introspection result,
not another whole-case blocker. No conformance percentage gain is claimed.

ADR-0004 review: the 408-line static introspection owner gains three table
entries; the regression belongs to the existing 242-line ID/name composition
test owner and introduces no directory or dependency boundary. No extraction
is warranted for three entries; existing directory regrouping debt is unchanged.

Validation: the regression fails before the correction and passes afterward.
Full repository verification passes with 2,108 core, 22 native, 4 WASM and
18 worker tests (2,152 total), formatting, strict Clippy, documentation, 1,998
Markdown link checks and pinned corpus checks. The official WASM target and
`git diff --check` also pass. No production processor default or archival
expectation was changed.

## 2026-10-18 Conditional execution under selected processor policy

The next bounded slice admits `xsl:if` and `xsl:choose` in the selected-profile
matched-template integration. It reuses the existing conditional assembler,
typed Boolean expressions and runtime executor. A private routing module owns
profile-aware condition/body lowering; it does not introduce another executor
or promote the test-only processor profile into production.

Conditions admit bounded child paths, Boolean constants/composition, resolved
plain local reads and the already-proved literal `system-property` arity-failure
leaf. Zero arguments and literal-only multiple arguments share that leaf.
Legacy forward-compatible calls fail only when reached; ordinary legacy and
modern contexts retain static arity failure. Dynamic arguments, successful
introspection, qualified paths and broader conditional static context remain
explicitly guarded. Every branch is compiled: a false condition cannot hide an
unbound variable or make a branch-local binding visible to its sibling.

Eight focused regressions cover source-dependent branch selection, first-branch
ordering, lexical isolation, inherited version interpretation, structural
validation, guarded attributes, depth bounds, literal multi-argument calls,
exact/one-less node-visit budgets, cancellation, failure cleanup and reuse.
The selected and ordinary shared paths agree on output and charges for XPath
node visits/operations, XSLT instructions, template candidates, result nodes,
result text bytes and serialized bytes on the common supported fixture.

An additional ignored acquired-corpus replay exercises the unchanged Microsoft
ForwardComp cases: `91849` and `91853` match their archived empty results;
`91850` reaches `XPST0003` and `91854` reaches `XPST0017` with original
provenance. These are selected-profile observations, not additional baseline
exact matches. All 44 matched-integration checks pass with the corpus enabled.

ADR-0004 review: conditional routing is a 60-line private owner and its tests
have their own 389-line owner. The matched-integration parent remains 548 lines;
the processor-reference directory has ten direct files. The runtime test helper
now delegates to a controlled version of the same execution/serialization path,
without adding production state or another lifecycle.

The next named argument gate is `91855`/`91856`, which use
`system-property(123)`. Do not equate a numeric actual argument with a type
error: [XPath 1.0 function calls](https://www.w3.org/TR/1999/REC-xpath-19991116/#section-Function-Calls)
convert arguments to the required type, while
[XSLT 1.0 system-property](https://www.w3.org/TR/1999/REC-xslt-19991116#function-system-property)
requires the resulting string to be a QName. Conversion, QName validity and
edition-sensitive error timing need separate evidence. Arbitrary Rust numeric
parsing is not an XPath literal grammar. No numeric-argument implementation or
new diagnostic contract is selected by this checkpoint.

Final validation passes: 2,116 core tests (66 ignored), 22 native, 4 WASM and
18 worker tests (2,160 total), formatting, strict Clippy, documentation, 1,998
Markdown link checks, pinned corpus checks and the official WASM build-target
check. Fresh complete OASIS replays retain modern preprocessing 2,503 exact /
16 mismatches and strict preprocessing 2,500 exact / 19 mismatches of 3,173.
Both retain 2,706 initialized, 2,657 executed and 423 separate expected-error
credits. Original fixtures, comparators and production defaults are unchanged.

## 2026-10-18 Numeric argument conversion and QName failure timing

The `91855`/`91856` follow-up found an existing baseline defect: the compiler
classified a numeric introspection argument as `XPTY0004` because it was not a
string. XPath 1.0 function conversion instead produces a string, after which
the introspection function requires a QName. The old Rust `f64` recognizer also
misclassified XPath name expressions such as `NaN` and `Infinity` as numeric
arguments. The correction uses the existing XPath numeric-literal recognizer,
not Rust's unrestricted numeric lexical space.

For the bounded literal slice, finite numeric values and negative infinity
cannot produce a valid QName. Positive infinity produces `Infinity`, which is
a valid QName; the classifier therefore leaves that successful-lookup case
guarded instead of fabricating an error. No numeric value formatter, general
argument evaluator, QName parser or processor policy is invented here.

The baseline direct `value-of` route now validates instruction attributes and
content normally, then retains the proven literal-call failure. Boolean
composition uses the existing charged deferred-failure leaf and short-circuit
evaluator. The selected legacy route reuses that Boolean IR for direct and
composed calls; normal legacy and forward-compatible contexts both preserve
dynamic QName failure. Static arity errors in other operands remain static
outside forward compatibility. Modern typed calls remain guarded in the
selected-profile lane rather than inheriting legacy coercion accidentally.

The structured codes are `XTDE1390` for `system-property`, `XTDE1400` for
`function-available`, and `XTDE1440` for `element-available`, using the
[later XSLT error vocabulary](https://www.w3.org/TR/xslt20/#error-summary).
XSLT 1.0 itself does not define these identifiers. This replaces the erroneous
numeric-argument type classification; no new compiler catch-and-defer codes
are added. General dynamic argument evaluation is still not implemented.

Focused tests cover fractional/signed literals, name/exponent exclusions,
positive and negative overflow, dormant/reached calls, local conditional
execution, program reuse, skipped Boolean operands, static arity precedence,
original provenance, exact/one-less operation budgets, cancellation and zero
retained allocations after failure. Invalid `value-of` options/content still
fail compilation rather than being hidden by the selected call error.

The unchanged selected-profile corpus replay now includes `91855` (archival
empty output) and `91856` (reached QName failure), alongside the four earlier
expression-timing cases. This is evidence for the selected profile, not six
new baseline credits.

ADR-0004 review: the numeric classifier stays in the existing static
introspection owner, Boolean lowering in its existing private owner, and
tests in the existing conditional/control test groups. No directory expands.
The 3,391-line instruction compiler retains a narrow validated direct-call
dispatch addition; its decomposition debt is not resolved by this correction.
Moving that clause alone would couple another module to the same validation
and lowering callbacks without separating responsibility. Broader value-of
extraction remains a separate cohesive follow-up, not a new semantic backend.

The 46 selected-profile checks pass with the acquired corpus enabled. The first
complete verification after this correction passes 2,121 core tests (66 ignored)
plus 44 adapter/worker tests, strict Clippy, formatting, documentation, links,
pinned suites and the official WASM target. Full modern/strict OASIS replays
retain 2,503/2,500 exact matches, 16/19 mismatches, 2,706 initialized, 2,657
executed and 423 separate expected-error credits, all on the original 3,173
denominator. The correction improves failure classification and selected-profile
integration, not the archival exact numerator.

## 2026-10-18 Message and fallback availability reconciliation

The adjacent capability audit found two further baseline omissions:
`element-available()` returned false for implemented `xsl:message` and
`xsl:fallback`. The compiled regression first produced `false|false|` and skipped
the guarded operations. Two table entries correct that result; the same test
then requires `true|true|fallback`, exactly one observed message, and `XTMM9000`
when termination is selected. Standalone fallback remains inactive, while an
unknown extension instruction uses its fallback without executing the extension.

The [XSLT 1.0 instruction classification and availability rule](https://www.w3.org/TR/1999/REC-xslt-19991116#fallback)
support both entries. Alias prefixes resolve by namespace identity. Unimplemented
extension names, newer instructions and non-instruction declarations such as
`xsl:param`, `xsl:sort` and `xsl:stylesheet` remain false in this baseline table.
This is not production promotion of ADR-0025, broader extension authority, or
an inference that every recognized instruction is fully implemented.

The change stays within the existing static-introspection owner and its embedded
tests. It creates no new module, directory, runtime dispatch or public contract.

Final combined validation passes 2,122 core tests (66 ignored), 22 native,
4 WASM and 18 worker tests (2,166 total), strict Clippy, formatting,
documentation, 1,998 local Markdown link checks, pinned corpus checks and the
official WASM target. The final complete OASIS replays still retain modern
2,503 exact / 16 mismatches and strict 2,500 exact / 19 mismatches, with 2,706
initialized, 2,657 executed and 423 separate expected-error credits out of 3,173.
No fixture, comparator, exclusion or production profile default changed.

Next work remains the accepted ADR-0025 implementation gates: dynamic argument
evaluation, modern typed-call semantics and selected-profile capability/reporting
reconciliation. The literal numeric correction is not a license to treat all
introspection arguments as constants or report the baseline table as a complete
selected processor profile. No new architectural decision is required for the
completed fixes; public profile promotion remains unearned.

## 2026-10-18 selected top-level declaration recognition

The next-case reassessment found no new dynamic-introspection argument fixture
beyond the completed pair. Whole-case inspection rejects three tempting detours:
`idkey49` requires cross-document union/ranking, `bvt033` combines broad
forward-compatible attribute/error handling, and the remaining format-number
cases include deliberately invalid pictures and archival recovery expectations.
None justifies removing its first guard and claiming a complete slice.

Instead, unchanged `ForwardComp` 91837–91839 expose a concrete selected-policy
gap: unknown top-level XSLT elements are refused before shared compilation.
The existing 91840 top-level fallback case covers the same placement rule.
The new regression first fails with unsupported `FXAR0031`; after the change,
all four original fixtures compare exactly against their original references
in the private selected-legacy/strict-preprocessing lane.

[XSLT 1.0 sections 2.2 and 2.5](https://www.w3.org/TR/1999/REC-xslt-19991116#forwards)
define the recognized top-level declarations and forward-compatible ignoring.
The private admission stage now distinguishes this edition-defined set from
shared compiler knowledge. Unknown declarations, modern `mode`, `function`,
`character-map` and `accumulator`, and misplaced `fallback` are ignored with
their content in legacy forward-compatible mode. Normal legacy mode reports
invalid placement with original provenance. The modern route is unchanged.

Recognized 1.0 imports/includes, globals, keys, whitespace declarations,
decimal formats, aliases and attribute sets remain explicitly unsupported in
this bounded route until their policy propagation is integrated. They are not
made ignorable by an implementation gap. Integrated output/template declarations
still validate their attributes. Foreign/unqualified top-level elements retain
their existing private guard; this is not a complete top-level profile.

Regressions prove ignored content cannot bind a variable or activate a nested
include, message or fallback. They retain XML/XDM preprocessing charges,
exact/one-less node-budget behavior, cancellation, allocation cleanup, fresh
compilation after failure, and execution after the stylesheet document is
dropped. No filesystem resolver is supplied or invoked by the compiler.

ADR-0004 review: the matched-integration owner is 576 lines; the 236-line test
child owns only top-level recognition/placement/control invariants. Its directory now
has 11 direct files and remains a coherent private processor-policy integration
subject. No additional nesting is justified at this checkpoint. Classification
delegates to existing excluded-top-level compilation, with no production runtime,
public API, ABI, compiled representation or authority change.

The 51 focused matched-integration tests, including separately acquired corpus
replays, pass. Full verification passes with 2,126 core, 22 native, 4 WASM and
18 worker tests (2,170 total), 67 core ignored probes, formatting, strict Clippy,
docs, 1,998 Markdown link checks and pinned corpus checks. The official WASM
target passes. Modern and strict OASIS replays remain 2,503/2,500 exact and
16/19 mismatches, both with 2,706 initialized, 2,657 executed and 423 separate
expected-error credits out of 3,173. This private integration earns no baseline
corpus credit and does not promote ADR-0025's processor profile.

## 2026-10-18 ignored output Boolean merge metadata

Review of the next declaration boundary exposed a shared-compiler defect.
An invalid optional `indent` value is already ignored under the existing 1.x
forward-compatible rule, but its lexical presence remained in the output
declaration's specified-property set. Combining it with a valid declaration
therefore failed with unsupported `FXST1018` instead of preserving the valid
setting. Two focused regressions first reproduce the false conflict and the
incorrect specified-property metadata.

The correction removes `indent` and `undeclare-prefixes` from specified metadata
when their existing shared optional-Boolean validator yields no value. It does
not change which versions or lexical values the validator admits, broaden error
recovery, or reinterpret a recognized modern facility as legacy capability.
The second property uses the same preexisting optional-value path; its treatment
here is metadata consistency, not an XSLT 1.0 feature claim.

Compiled tests cover both declaration orders, yes/no values, absent semantic
settings, both exported private property lists, unchanged actual conflicts,
and invalid values under non-forward 1.0/2.0/3.0 declarations. The `indent`
cases execute through the shared runtime and serializer and must equal the
valid-only stylesheet output. There is no whitespace-insensitive comparison.

This is a production shared-compiler fix independent of private profile
promotion. It does not make the broad `bvt033` recovery fixture executable.
The existing 833-line output compiler remains the sole declaration/merge owner;
tests stay in its private child module. No new source file or hot runtime path
is introduced for this fix.

Final combined validation passes 2,129 core tests (67 ignored), 22 native,
4 WASM and 18 worker tests (2,173 total), all 51 selected-policy integration
tests including acquired fixtures, strict Clippy, formatting, documentation,
1,998 Markdown link checks, pinned corpus checks and the official WASM target.
The final complete OASIS replays retain modern 2,503 exact / 16 mismatches and
strict 2,500 exact / 19 mismatches, both with 2,706 initialized, 2,657 executed
and 423 separate expected-error credits out of 3,173. `git diff --check` passes.

Next concrete integration pressure is declaration attribute policy, exemplified
by `ForwardComp` 91847/91848 and the larger `bvt033`: template/output validation
still has declaration-derived compatibility helpers. Thread the selected policy
through the existing owners before admitting broader selected-profile attribute
recovery. Do not rewrite a declared version, filter attributes in a synthetic
tree, or catch arbitrary declaration errors to imitate that propagation. This
remains implementation under ADR-0025, not a request for a new ownership decision.

## 2026-10-18 selected declaration attributes and complete ForwardComp replay

The unchanged `91847` unknown-template-attribute case first fails at the private
`FXAR0031` guard. The selected legacy policy now reaches the existing template
attribute-validation seam. Unknown unqualified attributes are ignored in future
version contexts, without parsing their values as expressions or AVTs; normal
1.0 contexts retain invalid-input diagnostics and original provenance. Known
fields whose readers are not policy-integrated remain explicitly guarded.
In particular, ancestor `default-collation`, version, namespace/static-context,
template name/mode/priority and qualified attributes are not silently ignored.

`91848` pressures optional output Booleans. The existing output compiler now
accepts an explicit private Boolean-policy projection rather than relying on a
rewritten version string. The selected legacy route accepts only `yes`/`no`,
ignores invalid optional `indent` and `omit-xml-declaration` values in forward
contexts, and rejects them in normal contexts. A declaration of `3.0` does not
grant modern Boolean aliases to a selected 1.0 processor. Baseline wrappers
preserve their existing eligibility, including modern aliases and the previous
non-recovering omit-declaration rule.

Unknown output attributes are neither evaluated nor included in specified
property metadata. Known but unintegrated output readers remain guarded; this
is not a general output-recovery implementation. The shared declaration and
merge owners still construct settings. Valid settings survive ignored values in
both declaration orders and both include/import artifact composition directions.
Composed programs execute after stylesheet documents are dropped and equal the
valid-only reference output exactly. These tests exercise existing artifact
composition, not a new graph loader or authority path.

A read-only in-memory `XslCompiledTransform` check corroborates the three narrow
rules: unknown template attributes, unknown output attributes and invalid Boolean
aliases are rejected with a 1.0 declaration, but ignored with 1.5, 2.0 and 3.0
declarations. Resolver access and DTD processing are disabled. This is independent
diagnostic evidence, not a replacement for the standard or archival expectations.

The complete unchanged Microsoft `ForwardComp.xml` catalog is now replayed under
one private selected-legacy/strict-preprocessing profile: **18 cases, 13 exact
output comparisons and 5 expected-error outcomes**. Error outcomes separately
pin `91841`/`91844` to `XTDE1450`, `91850` to `XPST0003`, `91854` to `XPST0017`
and `91856` to `XTDE1390`. These diagnostic labels do not claim XSLT 1.0 defined
the later-edition codes. No original fixture or expectation changes, no per-case
profile switching, and no baseline numerator credit are involved.

Reproduce with `FASTXSLT_OASIS_XSLT10_ROOT` pointing to the acquired `TESTS`
directory:

```text
cargo test -p fastxslt --all-features selected_legacy_replays_complete_forward_comp_catalog -- --ignored --nocapture
```

Next-case reassessment: `bvt033` is still a broad integration case, not the next
two-case Boolean patch. Its unchanged stylesheet also needs root attributes,
imports/includes, attribute sets, decimal formats, keys, whitespace declarations,
named/mode/priority template attributes, instruction-specific optional recovery,
and deferred invalid patterns/expressions. Do not remove its first guard and
describe the case as supported. The next bounded task should select one existing
semantic owner from that list and propagate processor policy through it; complete
profile promotion and truthful reporting remain separate gates.

### Bounded text-instruction follow-up

The `bvt033` `test16` instruction supplies the next concrete pressure:
`xsl:text unknown="" disable-output-escaping="maybe"`. A focused regression
first fails at the selected-policy guard. Text attribute admission now supplies
the existing shared constructor with the selected escaping flag. The character
content reader, malformed-child diagnostic, text IR, result builder and serializer
remain shared. Normal 1.0 rejects unknown attributes and invalid optional values;
forward contexts ignore those values. A future declaration does not suppress a
valid legacy `disable-output-escaping="yes"` flag through a modern version helper.
Baseline validation and ordinary entry points retain their prior behavior.

Tests pin local literal-result version scope, unknown attribute values that are
not evaluated, comment-separated character content, invalid element content,
source release, modern guards and unintegrated static-context/qualified attributes.
Shared execution is differential against the ordinary 1.0 path for both escaping
values, including node visits, operations, instructions, template candidates,
result nodes/text bytes and serialized bytes. Exact/one-less result-text limits,
active cancellation, allocation cleanup and subsequent reuse pass. No new runtime
evaluator, recovery catch, authority or public profile switch is introduced.

An ignored acquired-corpus test lowers the original `test16` node directly and
pins its value, escaping flag and original location. It also asserts that the
whole `bvt033` stylesheet remains unsupported. This is explicitly fragment
evidence, not a case pass or a rewritten archival expectation. `value-of`
attributes, the other instruction families and the larger declaration graph
remain separate integration work; do not infer their admission from text support.

### Cohesion and validation checkpoint

ADR-0004 review: the 3,224-line stylesheet compiler retains shared declaration
composition and template assembly; the change introduces narrow validation/
lowering seams rather than copying either owner. Output semantics remain in the
870-line output compiler. The 3,411-line instruction compiler factors text
validation from its existing content reader/constructor; this is a bounded
sequencing exception to further extraction, not an invitation to add another
instruction family there. No second text traversal or representation is added.
Selected-policy routing remains in its 726-line private integration owner.
Its 1,133-line test child remains the existing matched-body invariant family;
declaration tests occupy their separate 597-line child. The subject directory
still has 11 direct files. Additional nesting would not improve the current
reference/integration ownership split; reconsider if another subject emerges.

The catalog replay validates unique case identities and catalog source/style/
reference filenames before using its bounded acquired-file layout. Duplicate
entries or changed mappings cannot silently stand in for full catalog coverage.
The wider processor-policy family passes all 113 tests, including the separately
acquired replays; the matched-integration subset contributes 62 of these.

The official WASM target passes. Final modern/strict full OASIS replays remain
2,503/2,500 exact and 16/19 mismatches, both with 2,706 initialized, 2,657 executed
and 423 separately reported expected-error credits out of 3,173. No case has
been transferred from the private selected lane into these baseline numerators.
The concrete next sibling is `bvt033`'s `xsl:value-of` attribute policy: retain
its shared expression/error-timing owner while propagating optional escaping
and unknown-attribute treatment. Do not infer this from the text constructor or
claim the full fixture until every remaining owner is integrated and replayed.
No new AR or maintainer disposition is required for that bounded continuation.

Final repository verification passes: 2,138 core tests (69 ignored), 22 native,
4 WASM and 18 worker tests, for 2,182 non-ignored tests total; formatting,
strict Clippy, documentation, 1,998 Markdown link checks, pinned conformance
sources/inventories and `git diff --check` also pass. Build-cache hard-link
warnings fall back to copying; no semantic test or gate is waived.

## 2026-10-18 selected value-of attribute policy

The next named boundary was the original `bvt033` `test17` value-of instruction:
unknown attributes and an invalid optional escaping value precede a literal
`select`. The focused regression first failed with `FXAR0031`. Selected legacy
policy now validates these options before the existing shared value-of
structural validator and typed expression lowering. Ordinary entry points keep
their existing validation path. No runtime evaluator or public profile is added.

The text/value-of policy helper shares only option validation; their recognized
attributes and static-context guards remain distinct. Normal legacy contexts
reject unknown attributes and invalid escaping values; forward contexts ignore
them. Modern selected controls retain their existing guards. Plain string
literals enter the existing typed literal compiler. Boolean call-result text
preserves the validated escaping flag instead of silently resetting it.

Four focused regressions cover declared/local version scope, escaping markup,
unknown attributes, structural failures, eager lexical checks, dormant/reached
call errors, provenance after stylesheet release, and retained static-context
guards. Differential controls reconcile all seven work-charge domains, exact
and one-less result-byte budgets, cancellation, zero retained allocation after
failure and subsequent reuse. Text-only unknown `select`/`separator` attributes
remain ignored in forward contexts; sharing the helper does not widen their
recognition or accidentally impose value-of guards on text.

The acquired-corpus fragment test now lowers both original `test16` and
`test17`, retaining their source locations and exact literal/escaping IR. It
also verifies that the whole stylesheet remains unsupported. These are fragment
checks, not case credit. No archival bytes, expected outputs or comparison
rules changed. Whole-case declaration/static-context integration remains open.

ADR-0004 review: the shared instruction owner is 3,430 lines; its additional
seam separates validated options from the existing structural/expression reader,
without another traversal or representation. The private selected integration
owner is 762 lines and its matched-body test child 1,413 lines. The latter stays
one invariant family; declaration tests remain separate. This bounded change
does not justify another nested folder or a new generic compiler abstraction.

The separate production choose-attribute fix described in the
[archival recovery record](../../oasis-xslt10/evidence/construction/oasis-xslt10-prefixed-computed-element-attribute-set-2026-09-28.md#2026-10-18-choose-attribute-consistency)
earns the only baseline gain in this tranche. The selected-profile work earns
none. Modern/strict replays now report 2,504/2,501 exact, 16/19 mismatches,
2,707 initialized, 2,658 executed and 423 separate expected-error credits out
of 3,173. The official WASM target builds successfully.

Final repository verification passes: 2,144 core tests (69 ignored), 22 native,
4 WASM and 18 worker tests, for 2,188 non-ignored tests total. Formatting, strict
Clippy, documentation, all 1,998 Markdown link checks and pinned conformance
source/inventory checks pass. The test module is placed after production items
as required by Clippy; no lint exemption was added.

## 2026-10-18 selected conditional attribute policy

After reassessing the same sixteen baseline mismatches, the next bounded
integration target was original `bvt033` `test5`/`test12`: unknown attributes
on choose/when/otherwise and if. A focused test first failed with `FXAR0031`
instead of selected-policy behavior. Conditional assembly now accepts a private
attribute-validation callback, alongside its existing condition/body lowerers.
Ordinary compilation supplies the unchanged validators; selected integration
supplies edition-aware validation. Structural checking, required tests, branch
assembly and runtime execution retain one shared owner.

Selected legacy-forward contexts ignore unknown unqualified attributes without
interpreting their values. Normal contexts reject them with `XTSE0090`.
Modern selected controls retain their prior guards. Namespaced attributes and
unpropagated version/default-mode/default-collation/XPath namespace controls
remain unsupported; this does not grant blanket optional-attribute recovery.
Inherited local version context still governs the policy. Static lexical errors
remain visible in untaken bodies, while reached/dormant call failures retain
their established timing and original provenance.

Two new ordinary tests pin these boundaries, stylesheet release and reuse after
failure. Existing differential tests now include forward unknown attributes on
all four conditional elements and reconcile all seven work domains against
ordinary lowering. Exact/one-less instruction budgets, cancellation, allocation
cleanup and reuse also exercise the new attributes. A separate acquired test
lowers the original `test5`/`test12` nodes, checks shared conditional IR and
provenance, and asserts that the complete stylesheet remains unsupported.
It is explicitly fragment evidence, not an archival case pass.

The private processor family passes 120 tests including acquired replays.
ADR-0004 review: the conditional instruction owner is 308 lines, its selected
policy child 85 lines and its cohesive conditional regression child 706 lines.
No second structural traversal, evaluator, retained invocation state, dependency,
public selector or folder is introduced. Baseline validators remain routed
through their original wrappers; production defaults do not change.

Next return to the compilation-root/declaration graph boundary of `bvt033`, or
another named whole case under the remaining ADR-0025 gates. Do not infer full
fixture support from these four original instruction fragments or keep extending
conditional probes without a new failure/risk. No new architectural disposition
is needed for the bounded policy propagation already accepted by ADR-0025.

Final verification for this follow-up passes with 2,146 core tests (70 ignored),
22 native, 4 WASM and 18 worker tests: 2,190 non-ignored workspace tests.
Formatting, strict Clippy, documentation, all 1,998 Markdown link checks and
pinned corpus checks pass. The acquired profile family passes all 120 tests.
Both complete OASIS profiles retain the preceding 2,504/2,501 exact results and
16/19 mismatches, with 2,707 initialized, 2,658 executed and 423 separately
credited expected errors. No full-case credit comes from conditional fragments.
The official `wasm32-unknown-unknown` adapter also builds after the conditional
integration changes; `git diff --check` and the final Markdown link pass are clean.

## 2026-10-18 whole-case reassessment and bvt034 scope

The planned root/declaration continuation uncovered a whole-case limitation in
`bvt033`, not another reason to widen optional-attribute recovery. Its original
`fwdcompat.xsl` contains a bare `xsl:import` after other declarations. The acquired
`TESTS/doubts.xml` entry `BVTs_bvt033` explicitly flags the illegal import
placement; inspection additionally confirms the missing `href`. The catalog
nevertheless requests an XML-output comparison against `bvt033.txt`.

XSLT 1.0 [section 2.5](https://www.w3.org/TR/1999/REC-xslt-19991116#forwards)
requires ignoring unknown attributes and invalid optional values in forward
contexts; it does not waive missing required attributes or structural import
constraints. [Section 2.6.2](https://www.w3.org/TR/1999/REC-xslt-19991116#import)
requires an import reference and imports before other element children.
Consequently, the earlier fragment successes do not establish that completing
profile propagation should produce the archive's full successful output.
Do not use this fixture as a promised exact-output milestone, suppress its
required-attribute error, or change its expectation/accounting. No such recovery
policy is selected. Useful individual semantics remain independently testable.

The adjacent unchanged `Microsoft/BVTs_bvt034#1` is a cleaner complete case.
It enables forward compatibility on a literal-result element, then places
`xsl:version="1.0"` on an unknown XSLT instruction. The selected legacy scope
reader previously refused that attribute before instruction recognition. The
focused regression reproduced `FXAR0031`; the corrected reader preserves
inherited scope for an instruction unknown to the selected 1.0 edition.
Neither spelling of version on that instruction changes processor policy or
literal-result scope. Recognition remains edition-based, including newer
instructions known by the modern implementation.

The unchanged complete `fwdcompat2.xsl`, `data.xml` and `bvt034.txt` now compare
exactly in the **private selected-1.0/strict-preprocessing lane**. The stylesheet
document is dropped before execution. No fixture or comparator is modified.
This does not promote that lane or add a baseline corpus pass.

Focused controls cover qualified/unqualified version spellings, malformed
values on unknown instructions, known-newer `sequence`, refusal outside forward
scope, nested literal-result reset to 1.0, original failure provenance and exact
missing-fallback runtime code `XTDE1450`. The reset's unknown instruction remains
a static invalid failure; ignored version metadata cannot conceal it. Modern
scope handling and recognized legacy instruction guards are unchanged.

ADR-0004 review: the selected integration owner is 773 lines and its existing
matched-body regression child 1,516 lines. The new branch owns local-version
interpretation only; it adds no expression/compiler/backend, graph authority,
runtime state or public surface. The directory remains one processor-policy
subject with 11 direct files. No additional nesting is justified by this slice.

Validation passes: 2,148 core tests (71 ignored), 22 native, 4 WASM and
18 worker tests, totaling 2,192 non-ignored workspace tests; the acquired
processor-policy family passes 123 tests. Formatting, strict Clippy,
documentation, 1,998 Markdown link checks, pinned conformance checks and the
official WASM build pass. Modern/strict full replays retain 2,504/2,501 exact
comparisons out of 3,173. The ordinary unsupported frontier remains 22 cases.
This is a selected-profile parity gain and a whole-case blocker correction,
not an increase in the baseline exact numerator. No new AR, production profile
default, fixture edit, or recovery exception is introduced.

## 2026-10-18 selected numeric processor reporting and ver07

The next whole-case target is unchanged Xalan `ver07`, which tests
`system-property('xsl:version') >= 2.0` before a newer instruction. Selected
1.0 execution must not derive processor identity from the stylesheet's declared
2.0 version. A focused alias/declared-version regression first reproduced the
private `FXAR0031` refusal; the complete original stylesheet/source/expected
triple now compares exactly in the private selected-1.0/strict lane.

XSLT 1.0 [system-property](https://www.w3.org/TR/1999/REC-xslt-19991116#function-system-property)
defines the version property as a number and resolves its QName in the
expression's namespace context. The private integration now supplies numeric
1.0 to shared literal-call/QName parsing, independently of normal/forward local
scope. It deliberately does not reuse the baseline folder's local-compatibility
reporting policy. The shared test-only parsing callback became generic over its
return value; production folding is unchanged.

The admitted consumers are direct value-of stringification, effective-Boolean
use, and six numeric comparisons with a numeric literal on the right, composed
through the existing bounded Boolean compiler and runtime. This is not a new
numeric evaluator or admission of string-valued variables masquerading as
numeric processor properties. Reversed operands, dynamic/string/Boolean
comparison operands, arithmetic, other properties and modern reporting remain
outside this slice. General typed consumers and capability reconciliation still
gate production promotion.

Focused controls cover declared versions 1.0/1.5/2.0/3.0, QName aliases,
misbound/unbound/unqualified names, numeric rounding and negative zero, shared
short-circuit behavior, reached `XPST0017` provenance and eager `XPST0008`
failures even in skipped operands. Numeric value-of uses the ordinary constructor
and matches the literal reference across all seven work domains. Exact/one-less
result-byte budgets, cancellation, zero retained requested allocations after
success/failure, and subsequent reuse are checked. The whole-case replay drops
the stylesheet document before execution and changes no fixture or comparator.

ADR-0004 review: numeric reporting has a dedicated private child owner; Boolean
assembly accepts a bounded static-leaf callback rather than duplicating parsing
or evaluation. The directory now has 12 direct files, all belonging to the
processor-policy investigation/integration family. Additional nesting would not
clarify ownership here. No runtime state, host authority or public API is added.

This is selected-profile evidence only: ordinary corpus totals and processor
defaults are not changed or silently credited by the `ver07` success.

Validation passes at this checkpoint: 2,153 core tests (72 ignored), 22 native,
4 WASM and 18 worker tests, totaling 2,197 non-ignored workspace tests. The
acquired processor-policy family passes 129 tests. Formatting, strict Clippy,
documentation, 1,998 Markdown link checks, pinned conformance checks and the
official WASM build pass. Both complete OASIS replays preserve 2,504/2,501 exact
comparisons for modern/strict preprocessing respectively. Four historical
private guards were updated from the now-admitted version property to the still
unintegrated vendor property; their unsupported-boundary purpose remains tested.

## 2026-10-18 selected message body policy and ver01

Unchanged Xalan `ver01` supplied the next concrete missing owner: selected
message-body compilation. A focused test first reproduced `FXAR0031` on
`xsl:message`. The shared message compiler now accepts a private body-lowering
callback; its ordinary caller still supplies the same sequence compiler and
retains exactly the existing attribute validation and instruction construction.
The selected-1.0 caller supplies its policy-aware sequence owner. It admits only
absent/valid literal `terminate`, leaving invalid optional values, unknown
attributes and modern features unsupported pending their own integration.

The complete original `ver01.xsl`, `ver01.xml` and `ver01.out` now compare
exactly in the private selected-1.0/strict lane. The expected non-terminating
message is recorded independently of result text. No ambient observation,
new message runtime, authority or public sink was introduced.

Focused regressions prove that message bodies preserve newer-instruction
fallback under selected 1.0, numeric processor reporting, lexical variable
bindings and local literal-result version reset. Reached termination records
the message and reports `XTMM9000` with original provenance; skipped termination
does not run. A reached deferred call failure produces `XPST0017` without a
partial message. Shared-reference execution agrees across all seven work
domains. Exhaustion/cancellation during message construction leaves no partial
observation or retained requested allocation, and the compiled program remains
reusable. The acquired processor-policy family passes 134 tests.

Neighboring `ver05` and `ver06` now execute under the same numeric reporting
slice but are **not exact**: their literal stylesheet text says 17.1 and 17.2,
while the original expected files still say 1.1 and 1.2. The replay asserts both
the actual literal text and the unchanged comparator's mismatch disposition.
This is not permission to correct the expected files or silently credit them.
`ver07` remains exact. No baseline corpus percentage changes are inferred.

ADR-0004 review: the 3,438-line instruction owner receives only an eight-line
body-callback seam in its existing message constructor; this retains the
existing owner rather than duplicating it in the profile reference. The
835-line selected composition owner routes policy, while 241 lines of dedicated
message tests own observation/termination/control evidence. The instruction
directory's 24 files already separate expression and instruction families; this
change adds no production sibling or cross-layer dependency. Retain this small
constructor seam here for this bounded checkpoint; revisit extraction if
message validation/content rules expand. The processor-reference directory now
has 13 files; its named message test family is clearer than another directory
level. No evaluator, mutable context, cache, host policy or unsafe surface moves.

The message checkpoint passed the full verification script with 2,157 core tests
(73 ignored), 22 native, 4 WASM and 18 worker tests: 2,201 non-ignored workspace
tests. Final target/corpus conservation is rerun below after the adjacent dispatch
slice; these private successes do not increase baseline coverage.

## 2026-10-18 selected default dispatch and complete version-group replay

The remaining `ver02`–`ver04` whole cases expose a small, shared dispatch path:
empty `apply-templates`, an unprefixed child-name match path, and a direct
attribute value read. `ver03` additionally carries ordinary stylesheet `id`
metadata. A focused dispatch regression reproduced `FXAR0031` before integration.

The private selected legacy route now delegates default invocation to the
existing template-invocation compiler, accepts child-name-only match paths
through the existing match compiler, and lowers `@name` / `./@name` through the
existing explicitly selected value-compatibility mode. The modern witness is
not widened. Empty default invocation cannot lower parameters, sorting, modes
or selection expressions through unpropagated readers; those shapes still
refuse before shared compilation. No template dispatcher or path evaluator is
introduced. Root `id` does not select processor identity or resource authority.

Controls pin explicit template calls containing selected fallback/reporting,
source focus, direct-parent matching and shared default-priority behavior,
independent callee lexical scope, and stylesheet/transform synonym behavior.
All seven runtime work domains agree with the shared baseline on the admitted
dispatch fixture. Exact/one-less text budgets, candidate-scan cancellation,
zero retained requested allocations and post-failure reuse remain intact.

The complete unchanged Xalan `ver` group is now replayed in the **private
selected-1.0/strict lane**: five exact comparisons (`ver01`–`ver04`, `ver07`) and
two explicitly retained archival mismatches (`ver05`, `ver06`, described above).
This is a seven-case local profile checkpoint, not a new conserved production
denominator or an increment to the 3,173-case baseline. Stylesheet documents are
dropped before execution. The profile family passes 140 tests.

ADR-0004 review: the selected composition owner remains below 1,000 lines;
dispatch-specific regressions live in a named 246-line child, not in the
already-large matched-body regression file. The reference directory has 14
files under one profile-integration subject; another directory level is not
needed for this bounded addition. Production dispatch/evaluation owners are
unchanged. The default-invocation guard is named separately so the composition
function does not acquire selection/parameter semantics.

Dispatch-checkpoint verification passes with 2,162 core tests (74 ignored),
22 native, 4 WASM and 18 worker tests: 2,206 non-ignored workspace tests.
Formatting, strict Clippy, documentation, links and pinned conformance checks
pass. Full modern/strict replays retain 2,504/2,501 exact comparisons; the
official WASM adapter builds. The 22 ordinary unsupported cases remain visible.

## 2026-10-18 selected named template body owner

Whole-case review of `namedtemplate04`/`05` identified the next ownership route:
named declarations bypassed the selected matched-template callback and invoked
the ordinary named-body compiler directly. Merely allowing `name` would
therefore have leaked baseline policy into an otherwise selected compilation.
The focused QName-alias/fallback regression first reproduced `FXAR0031`.

Shared top-level assembly now receives an explicit named-declaration callback
alongside its matched-template callback. Ordinary callers still use exactly
their existing named compiler, including its distinct parameter handling. The
selected callback refuses parameters before using shared template/body assembly
with selected validation. Empty `call-template` reuses the shared invocation
compiler after a guard excludes argument children and unpropagated attributes.
QName normalization, duplicate-name detection, reference validation and runtime
invocation are unchanged. This is not a second named-template compiler/runtime.

The unchanged original `namedtemplate04` and `namedtemplate05` triples compare
exactly in the private selected-1.0/strict lane, including qualified-versus-
unqualified template identity. Focused tests additionally pin QName aliases,
selected fallback/reporting inside a named body, shared static diagnostic
code/category parity, callee lexical isolation and a declaration used in both
named and matched roles. Named calls preserve caller focus; matched calls use
the selected node. Both remain on shared execution and all seven work domains
match the baseline reference fixture.

Dormant forward-context call failures remain deferred; reached failures retain
`XPST0017` and source provenance, while normal-context arity failures remain
static. Exact/one-less byte budgets, cancellation, zero retained requested
allocation and reuse are covered. A recursive named call is run only under a
16-instruction budget and must report exhaustion, not execute unboundedly.
Old private guards now retain parameter/mode boundaries instead of asserting
that all named declarations are unintegrated. Modern named declarations remain
outside the private witness's admitted surface.

ADR-0004 review: the 3,227-line shared stylesheet owner adds only the explicit
named callback plumbing; the existing small declaration-routing child owns
where it is used. The selected body helper centralizes its parameter refusal
and policy lowering for named/matched routes. This reduces policy-path coupling
without merging the ordinary named/matched parameter semantics. No runtime
context, authority, cache, public surface or unsafe code is added. The private
reference directory now has 15 named source/test files; retain the cohesive
layout at this checkpoint and reassess invocation-subject grouping before
further parameter/default integration grows it.

Next whole-case target: `namedtemplate01` requires parameter defaults, supplied
arguments and value-of temporary bindings. It must not be described as solved
by this no-argument named-call slice. Imported named declarations, complete
capability reconciliation and production promotion remain separate gates.

Final validation for this continuation passes:

- `scripts/verify.ps1`: 2,168 core tests (75 ignored), 22 native, 4 WASM and
  18 worker tests, totaling **2,212 non-ignored workspace tests**; formatting,
  strict Clippy, documentation, 1,998 Markdown link checks and pinned conformance
  checks pass.
- Acquired `processor_edition_reference -- --include-ignored`: **147 passed**.
- Complete modern/strict OASIS replays: **2,504 / 2,501 exact**, **16 / 19
  mismatches**, 2,707 initialized and **423 expected-error credits separately**,
  each conserving the original 3,173 cases.
- Official `wasm32-unknown-unknown` workbench build and `git diff --check` pass.

No production processor default, public profile API, expectation, fixture,
comparison rule or conformance numerator was changed by these four private
integration slices. No new architectural decision is requested. The roadmap's
completed profile history was condensed into evidence links while retaining the
remaining gates and explicit archival limitations.

## 2026-10-18 literal parameters and temporary arguments

The next unchanged whole-case targets, Xalan `namedtemplate01` and
`namedtemplate02`, now compare exactly through the private selected-1.0 processor
with strict preprocessing. The first combines a source-derived value-of temporary
binding, an explicitly supplied parameter and an omitted parameter's literal
default. The second additionally exercises a dual named/matched declaration,
string-literal arguments and explicit child selection. Original source,
stylesheet, expected bytes and XML comparator remain unchanged. These are
private integration results, not additions to the production baseline numerator.

The shared named-template compiler now accepts attribute-validation and body
lowering callbacks; ordinary callers retain their original callbacks. Its
parameter ordering, duplicate validation and default compiler remain the owners.
Matched declarations retain their separate existing validation route. The
selected eligibility guard admits only literal/empty defaults; shared sequence
traversal excludes parameter declarations and starts with the declared bindings.
Selected calls and applies admit NCName argument names, bound-variable or string
literal values, and unqualified child-name selection paths. Modes, sorts,
expression/instruction defaults and argument constructors remain guarded.

A single selected value-of can lower into the existing
`Xslt10ValueOfTreeVariable` representation. This reuses the selected value policy
and shared temporary-tree runtime rather than invoking an unqualified compiler.
A deferred instruction is deliberately **not** treated as a value expression:
the constructor remains unsupported when its value-of lowers to such an
instruction. This is an explicit remaining integration limit, not a new
static-error rule or a successful fallback. Selected processor-version reporting
inside an admitted temporary value remains `1` even in a version-2 stylesheet.

Focused checks cover supplied/default precedence, caller and selected-node focus,
multiple matched siblings, independent repeated calls, caller binding survival,
callee parameter isolation, duplicate/late parameters, unbound arguments, and
guarded defaults. Both declared-version 1 and 2 selected legacy lanes match all
seven shared runtime work-charge domains against the existing 1.0 compiler.
The temporary-argument probe also covers exact/one-less result-text budgets,
cancellation, zero outstanding measured allocations after failure and successful
reuse after failure. No runtime representation, cache or host boundary changed.

ADR-0004 review: the shared stylesheet owner is 3,264 lines after adding callback
routing; no default algorithm moved or duplicated. Retain this bounded change
because declaration assembly and parameter validation already reside together;
broader parameter-expression work should reassess extraction of that owner.
The selected integration owner is 1,026 lines and the named test owner is 804
lines after the follow-up below. Retain the selected owner for this bounded
tranche: its helpers are policy eligibility and shared lowering routes, not
runtime/default evaluators. Further mode/argument expansion should reassess a
private invocation-policy child rather than indefinitely growing this owner.
The 15-file reference directory does not
grow: parameter tests extend its existing named-invocation subject. No new
subdirectory is justified by this tranche alone.

Follow-up whole-case reassessment completed `namedtemplate03` through bounded
absolute child-name / wildcard match admission, wildcard child selection and
context `name()` / `name(.)` local and argument values. Shared match, variable,
argument and recursive execution owners remain unchanged. A recursive focused
probe preserves all seven work-charge domains, instruction-budget exhaustion,
template-scan cancellation and post-failure reuse under declared versions 1 and
2. `namedtemplate15` (leading underscores) also passes unchanged through the
same parameter machinery without a special name rule.

`namedtemplate14` then justified admitting string-literal `select` defaults
through the same shared default compiler, separately from general expressions.
Focused tests cover both named/matched instantiation and invalid simultaneous
`select` plus content. After the metadata follow-up below, the complete 19-case
named-template group has **8 exact private selected-1.0 comparisons**
(`01`–`06`, `14`, `15`) and **11 explicit
unintegrated refusals**. Dependency helper stylesheets are not additional cases.
The group check also corrected an eligibility diagnostic: valid with-param
content without `select` is unsupported in this slice, not invalid for omitting
an optional attribute.

`namedtemplate06` also passes through bounded single-NCName mode metadata,
signed-32-bit integer priorities and unqualified child-path source arguments.
The priority bound is deliberate: shared legacy binary64 and modern decimal
readers agree exactly for this integer domain; fractional/large priorities stay
guarded until selected policy reaches their numeric interpretation. Named calls
ignore match/mode/priority as required, while default-mode dispatch excludes a
named-mode rule. Focused controls check first-node string conversion of a
multi-node argument, unchanged caller focus, all seven charge domains, minimum
and maximum i32 priority ordering, and rejection of unpropagated mode lists,
special modes and wider priority domains. A separate source-argument owner test
exhausts node-visit work, cancels during selection, reuses the compiled program,
and observes zero outstanding allocations after source/invocation teardown.

Next target `namedtemplate07` requires content arguments and a composed
variable predicate; later cases include globals and module graphs. Keep those
whole-case dependencies visible rather than allowing unprofiled compiler reentry.
No new architectural approval is requested.

Final validation for this parameter/invocation continuation:

- `scripts/verify.ps1`: **2,176 core tests**, 76 ignored; 22 native, 4 WASM and
  18 worker tests, totaling **2,220 non-ignored workspace tests**. Formatting,
  strict Clippy, docs, 1,998 Markdown link checks and pinned conformance pass.
- Acquired private `processor_edition_reference -- --include-ignored`:
  **156 passed**, including the complete named-template classification above.
- Both complete 3,173-case baseline replays remain unchanged: modern **2,504
  exact / 16 mismatches**, strict preprocessing **2,501 exact / 19 mismatches**;
  both initialize 2,707 and retain **423 expected-error credits separately**.
- Official WASM target build, final formatting and `git diff --check` pass.

Six additional unchanged named-template cases are exact in the private selected
lane relative to the previous `04`/`05` checkpoint. No production default,
public profile selector, upstream fixture or baseline conformance count changes.

## 2026-10-18 content arguments predicates and numeric composition

Starting from the unchanged failure in `namedtemplate07`, the selected legacy
lane now admits literal/empty with-param content through the existing argument
compiler and the shared `child[@attribute=string($variable)]` Boolean leaf.
The existing leaf compiler accepts an explicit compatibility value; selected
legacy always supplies true rather than reading the stylesheet's version.
The lexical-variable walk now visits this leaf before runtime short-circuiting.
Untaken `false() and ...` / `true() or ...` branches cannot hide an unbound name.
Instruction-valued argument content remains explicitly unsupported.

The original `namedtemplate07` output compares exactly. Focused controls cover
repeated calls with different values, an empty argument, absent/empty attributes,
namespace-distinct attributes, both declared versions 1 and 2, all seven shared
work-charge domains, selection-budget exhaustion, cancellation, reuse and zero
outstanding observed allocations after failure.

The whole-case reassessment then completed three further original fixtures:

- `namedtemplate13`: document-rooted `//NCName` conditions use the existing
  charged location-path evaluator. A focused non-root-context probe proves the
  lookup sees a sibling outside the current element, preserves all seven charge
  domains, and retains budget/cancellation failure behavior.
- `namedtemplate09`: bounded integer arguments reuse the shared argument IR and
  nested named-call runtime. Admission is signed-32-bit integer lexical syntax;
  leading plus, exponents, fractions and larger values are not silently admitted
  via Rust parsing. Focused checks pin zero/nonzero Boolean conversion and both
  numeric endpoints under selected legacy with version-1 and version-2 stylesheets.
- `namedtemplate08`: `number($variable)` and bounded integer-literal plus variable
  lower through the shared explicit-legacy value compiler. The same value lowering
  feeds temporary value-of bindings. First-node conversion of a multi-node
  argument, whitespace numeric conversion, all seven charge domains, and static
  unbound names in dormant bodies are pinned. Non-finite arithmetic remains
  explicitly unsupported: both shared and selected paths reject the NaN operand
  produced from a nonnumeric parameter with `FXRT1022`, then permit clean reuse.
  No numeric evaluator or alternative parameter runtime was introduced.

The complete 19-case named-template group is conserved at **12 exact private
selected-1.0 comparisons** (`01`–`09`, `13`–`15`) and **7 explicitly unintegrated
refusals** (`10`–`12`, `16`–`19`). Imported helper stylesheets are not extra cases.
These four new private comparisons do not change the production baseline.

ADR-0004 review: shared Boolean/conversion helpers change visibility only; their
algorithms and ordinary call sites are unchanged. Within the private reference,
Boolean dynamic leaves and selected value-mode lowering now have separate named
functions, preserving the existing short-circuit/error ordering without adding
files or runtime state. The selected owner is 1,076 lines; the named test owner
is 1,045 lines. Retain these cohesive units for this checkpoint, but reassess
argument-constructor policy and its tests as a named child before expanding
instruction-valued arguments. The reference directory remains at 15 files.

Next `namedtemplate10` requires instruction-valued with-param bodies, variable-
variable addition/relational tests and additional wildcard path matching.
`namedtemplate11` needs for-each argument construction; `12` needs numbering AVTs;
`16` needs namespace/exclusion propagation; `17`–`19` need selected module graphs.
These are accepted-policy integration tasks, not authorization for unqualified
compiler reentry or a new processor default. No new AR decision is requested.

Validation for this checkpoint: acquired private processor-policy tests pass
**161/161**. Workspace tests pass **2,225** (2,181 core, 22 native, 4 WASM,
18 worker; 79 ignored probes remain separate). The official
`wasm32-unknown-unknown` adapter build passes. Full unchanged OASIS replay retains
**2,504/3,173 exact modern** and **2,501/3,173 exact strict-preprocessing**;
both initialize 2,707 and separately record 423 expected-error credits.
Modern/strict mismatches remain 16/19. Neither profile selects the private
processor-edition reference for this baseline measurement.

## 2026-10-22 bounded iteration arguments

The unchanged `namedtemplate11` now compares exactly through the private selected
legacy processor path. It supplies an argument constructed by iterating child
elements and appending each string value. The shared
`ForEachPathStringContent` representation and executor already own this meaning;
the integration adds only a narrow pre-lowering eligibility guard.

The guard permits one `for-each`, an unqualified child path, and one
`value-of select="."`. It rejects version overrides, escaping controls,
predicates, other value expressions, multiple instructions and known-newer
instructions. The underlying leaf does not select expression semantics from a
stylesheet version. No callback, alternate executor or public surface was added.

Focused tests cover version-1 and version-2 declarations under the selected
legacy policy, nested element string values, document order, repeated parameter
reads, empty supplied content overriding a default, and namespace-sensitive
selection. Seven work domains match the shared reference. Budget exhaustion,
cancellation and post-failure reuse remain pinned. An apply-templates test proves
argument construction uses caller focus, not the selected target's children.

The 19-case group is now **13 exact private comparisons** (`01`–`09`, `11`,
`13`–`15`) and **6 explicitly unintegrated** (`10`, `12`, `16`–`19`). This is
not a production coverage credit. `namedtemplate10` remains the next broader
instruction-argument, recursive arithmetic and match-pattern integration task.

ADR-0004 review: retain the selected owner (1,107 lines) and named-call test owner
(1,181 lines) for this narrow leaf. The additional helper owns eligibility only;
semantic construction remains in the existing shared compiler/runtime. The
reference directory stays at 15 files. General instruction-valued argument
lowering remains the trigger to extract argument policy and its tests into a
cohesive child subject, rather than growing this guard into a second compiler.

Validation: `scripts/verify.ps1` passes, including **2,228 workspace tests**
(2,184 core, 22 native, 4 WASM, 18 worker), Clippy, formatting, documentation,
Markdown links and pinned conformance checks. Acquired private processor-policy
tests pass **164/164**, including the full named-template group. The official
WASM target builds. Unchanged full OASIS replays retain modern **2,504/3,173**
and strict-preprocessing **2,501/3,173** exact comparisons; both initialize
2,707, with 423 separately recorded expected-error credits and 16/19 mismatches.

Reassessment of `namedtemplate10`: the stylesheet requires the absolute wildcard
match `/doc/*`, single-value-of argument constructors (`@repeat`, `$stop`, and
`$start + $step`), and the variable comparison `$start < $stop`. It is not merely
another for-each argument. Selected-policy propagation and eager lexical binding
checks must reach those value constructors; calling the unqualified content
compiler on arbitrary instruction bodies would bypass the accepted boundary.
No ownership decision is newly required, but that broader integration has not
been implemented or credited in this checkpoint.

## 2026-10-22 recursive value arguments

The next bounded integration now replays unchanged `namedtemplate10` exactly.
The conserved named-template group is **14 exact private comparisons**
(`01`–`11`, `13`–`15`) and **5 explicit unintegrated refusals** (`12`, `16`–`19`).
No production default or baseline numerator changes.

The shared named-call compiler has a private content-lowering hook; its ordinary
entry point supplies the unchanged content compiler. Selected argument lowering
admits a single value-of over an unqualified attribute, a bound variable or
addition of two bound variables. The selected path checks both addition operands
and emits the existing `Xslt10Content` argument representation through explicit
legacy value lowering. General instructions, deferred constructors, escaping and
version overrides remain refused. Runtime argument ownership is unchanged.

The selected path also admits absolute child-path matches ending in `/*`, context
`name()` output, and variable-to-variable numeric relations. These use existing
shared owners. Relational variable checks visit both operands before runtime
short-circuiting; recognized syntax does not bypass lexical resolution.

Focused tests pin version-1 and version-2 stylesheets under the selected legacy
policy; recursive caller focus; all seven work-domain counters against the shared
reference; instruction exhaustion, cancellation and reuse; unbound variables in
dormant constructor bodies and short-circuited comparisons; and rejected general
constructor forms. Relational tests prove existential node-set comparison when
only a later node satisfies the relation, plus empty and nonnumeric cases.
Non-finite arithmetic remains the existing explicitly unsupported boundary.

ADR-0004 review: selected argument admission/content lowering moved to the private
`template_arguments` subject (194-line implementation, 176-line focused tests).
The parent is 1,023 lines and retains sequence orchestration, local-version
context and selected value lowering. The child has no runtime state, acquisition,
module linking or public surface. Shared invocation remains cohesive at 1,686
lines; the hook separates content policy without duplicating argument validation.
The reference directory stays at 15 direct files, with the new argument subject
below it. Literal-result eligibility/body assembly is a named parent helper.
The extraction creates no crate, ABI, unsafe surface or alternate evaluator.

Next candidates are `namedtemplate12` numbering AVTs, `16` namespace/exclusion
propagation, and `17`–`19` selected module graphs. General constructor expansion
is not implied by this single-value slice. No new ownership decision is requested.

Validation: full `scripts/verify.ps1` passes with **2,233 workspace tests**
(2,189 core, 22 native, 4 WASM, 18 worker; ignored probes remain separate).
Acquired private processor-policy tests pass **169/169**. The official WASM
adapter builds. Full unchanged OASIS replays preserve modern **2,504/3,173**
and strict-preprocessing **2,501/3,173** exact comparisons, 2,707 initialized
cases, 423 separate expected-error credits and 16/19 mismatches respectively.

Next-case inspection confirms `namedtemplate12` reaches a further policy seam:
the shared numbering compiler currently derives its compatibility flag and some
AVT eligibility from local stylesheet version. A selected-profile call must not
merely reuse that entry point and relabel the finished instruction. Threading
explicit selected compatibility through the admitted numbering-format owner is
the next bounded implementation task under ADR-0025, not a new architectural
decision or permission to enable all numbering options.

## 2026-10-22 number format integration and exclusion blocker

Unchanged `namedtemplate12` now compares exactly. The private selected legacy
named-template group is **15 exact / 4 explicitly unintegrated**: `01`–`15`
are exact; `16`–`19` remain refused. These are not new production credits.

Shared numbering compilation now separates format compatibility from its ordinary
local-version lookup. A test-only format-only entry point validates the narrow
attribute surface before sharing the same compiler. The selected owner supplies
legacy compatibility, then checks any format-variable name against lexical
bindings. No post-compilation relabeling, new formatter or runtime is introduced.
Other numbering attributes remain guarded because their policy propagation is
not established by this slice.

Focused tests pin temporary content parameter conversion under selected legacy
with both version-1 and version-2 declarations, default sibling counting,
alphabetic formatting, all seven work-domain counters against the shared path,
instruction-budget failure, cancellation and reuse. An unbound variable remains
static in a dormant numbering instruction. Value, count/from, levels, language,
letter-value, grouping and version overrides remain explicitly unintegrated.

ADR-0004 review: retain the 687-line cohesive shared numbering compiler, 1,049-line
selected orchestration owner and 1,309-line named-template test owner. The new
entry point owns only format/default-count eligibility and delegates shared
compilation. No new files, state owners, public APIs or unsafe surface are added.

### Confirmed next-case blocker: namespace exclusion aliases

Historical checkpoint: repaired by the [namespace exclusion follow-up](#2026-10-22-namespace-exclusion-repair)
below. The manual probe is no longer ignored and no test skip is now required.

Inspection before `namedtemplate16` admission found a shared semantic defect.
With `p` and `q` both bound to `urn:excluded`, `exclude-result-prefixes="p"`
currently emits `<out xmlns:q="urn:excluded"></out>`. XSLT 1.0 specifies the
excluded namespace through the URI bound at the declaration, so an alternate
prefix for that URI must not defeat exclusion. See
[XSLT 1.0 section 7.1.1](https://www.w3.org/TR/1999/REC-xslt-19991116#literal-result-element).

The failing regression is retained as an explicitly ignored manual probe:

```text
cargo test -p fastxslt --all-features shared_namespace_exclusion_must_remove_other_prefixes_for_the_same_uri -- --ignored
```

Observed: **0 passed / 1 failed**, showing the leaked `q` declaration. This is a
known failing semantic assertion, not a successful conformance test. Ordinary
verification ignores it; the acquired private integration sweep must explicitly
skip this named defect probe until it is repaired. No corpus case is excluded or
reclassified by that test-run choice.

Stop namespace-profile expansion here. Repair the shared namespace selection
owner with URI qualification at the declaring element, alias-prefix and prefix
rebinding controls, and preservation of namespaces required for result names.
Audit default-prefix and extension exclusions and replay both complete OASIS
profiles before admitting the selected namespace surface. The existing ownership
decisions suffice; no new AR decision or maintainer approval is requested merely
for this semantic repair. Do not mark `namedtemplate16` supported just because
its particular fixture lists both aliases and therefore avoids the defect.

Validation of the completed numbering slice: `scripts/verify.ps1` passes with
**2,235 workspace tests** (2,191 core, 22 native, 4 WASM, 18 worker). There are
77 ignored core probes, including the newly documented failing exclusion probe,
and 3 ignored native probes. The acquired private sweep passes **171 tests**
with `--include-ignored --skip shared_namespace_exclusion_must_remove_other_prefixes_for_the_same_uri`.
This does not claim that every manual probe passes. The official WASM target
builds. Full unchanged OASIS replays retain modern **2,504/3,173**, strict
preprocessing **2,501/3,173**, 2,707 initialized, 423 separate expected-error
credits and 16/19 mismatches respectively.

## 2026-10-22 namespace exclusion repair

The shared compiler now resolves explicit excluded/extension namespace prefixes
to their URIs at the declaration, not to lexical prefix spellings at the literal
result. It also marks nearer bindings as seen before filtering, including empty
bindings; excluded or undeclared bindings cannot expose a shadowed ancestor.
Required element and attribute namespace bindings remain retained.

The formerly ignored alias regression now passes as an ordinary test. Focused
controls cover alias prefixes, rebinding between declaration and result element,
local exclusions, default undeclaration, extension-namespace aliases, and required
element/attribute prefixes. Existing modern `#all` behavior is unchanged; this
tranche does not claim a new modern exclusion profile.

After the shared repair, selected legacy compilation admits a root
`exclude-result-prefixes` list of bound NCNames through the shared validator.
Both version-1 and version-2 declarations preserve the selected behavior. Unbound
prefixes retain `XTSE0808`; `#all`, `#default`, nested controls and extension
declarations remain unintegrated. Result namespace exclusion does not remove
stylesheet namespace bindings used to resolve named-template QNames.

Unchanged `namedtemplate16` now compares exactly: **16/19 private exact**, with
`17`–`19` still explicitly refused for module-graph integration. No original
fixture or expectation changed, and no private-profile credit is added to the
production numerator.

ADR-0004: extracted static literal-result namespace selection into the private
126-line `result_namespace_compiler` owner. Its two responsibilities are
declaration-scoped exclusion collection and effective-binding selection; inputs
are immutable document/element identities and output is a namespace vector. The
3,346-line parent delegates through its existing entry point. Imports are explicit;
the child has no runtime state, cache, host authority, public/ABI or unsafe surface.
The compiler directory now has 25 direct files. Retain the existing subject-named
compiler owners for this bounded repair; do not manufacture a new directory solely
for one namespace file. Reassess a broader static-result-construction subdirectory
when multiple related owners need substantive changes. Full source-unit debt is
not claimed resolved by this extraction.

Next inspect graph qualification/import precedence for `namedtemplate17`–`19`.
The earlier single-root-module experiment is not sufficient permission to call
an unprofiled module linker from the selected matched-template path.

Validation: `scripts/verify.ps1` passes with **2,239 workspace tests** (2,195 core,
22 native, 4 WASM, 18 worker; 76 core and 3 native ignored probes remain separate).
The acquired private processor-policy suite passes **175/175 with no defect skip**.
The official WASM target builds. Full OASIS baselines remain modern
**2,504/3,173 exact** and strict preprocessing **2,501/3,173 exact**, with 2,707
initialized, 423 separately recorded expected-error credits and 16/19 mismatches.
The repaired minimal regression demonstrates a real semantic fix even though
these archived exact-output numerators do not change.

## 2026-10-22 selected single-import integration

The private matched-template integration now lowers a supplied principal and one
imported leaf independently through selected declaration/body policy and strict
preprocessing, then uses the existing shared single-import linker. Neither module
passes through unprofiled lowering. Named-reference validation occurs after
composition: imported reads may resolve principal declarations, principal reads
may resolve imported declarations, and unresolved names still fail linking.

The experiment requires one first-position import whose href exactly matches the
supplied document's logical identity. It performs no acquisition or relative URI
resolution; repeated identities, additional imports/includes and nested edges
remain guarded. A version-2.0 imported body with `xsl:sequence`/fallback proves
selected legacy recognition, and principal override plus document-release tests
pin precedence and retained compiled lifetime. This does not expand production
entry points, defaults, public graph APIs or supported-profile claims.

Unchanged `namedtemplate18` compares exactly. The selected named-template group
is now **17/19 exact**, with `17` (multiple imports) and `19` (nested imports)
still explicitly unintegrated. Baseline modern **2,504/3,173** and strict
preprocessing **2,501/3,173** receive no new credits from this private path.
The acquired processor-policy suite passes **179/179**, including the new corpus
replay; no defect skip is used.

The bounded composition helper and its tests live in a private `single_import`
module rather than growing the selected instruction owner with graph mechanics.
It delegates precedence and policy compatibility to the existing shared linker;
the new module creates no second linker or resolver. The existing directory
cohesion pressure remains open. Next inspect multiple-import composition before
admitting `17`; do not sequentially merge imports if that changes precedence or
validates cross-module references prematurely.

Validation: full `scripts/verify.ps1` passes with **2,242 workspace tests**
(2,198 core, 22 native, 4 WASM, 18 worker; 77 core and 3 native ignored).
The official WASM build and `git diff --check` pass. Both complete OASIS baseline
replays retain the counts above, including 423 separate expected-error credits,
2,707 initialized cases and modern/strict mismatch counts of 16/19.

## 2026-10-22 selected multiple leaf imports

The shared multi-import compiler now delegates composition to a private helper
accepting already-lowered principal and imported programs. The ordinary caller
retains its existing compilation and qualification; the selected-policy caller
lowers every module through selected policy instead. The extracted linker retains
declaration-count validation, policy compatibility checks, reverse import-rank
assignment, merge order and final named-reference validation. It does not perform
sequential pairwise merges or prematurely validate sibling modules.

The private supplied-document experiment admits up to three ordered leaf imports,
with exact logical href matching and no acquisition. Duplicate identities, missing
dependencies, non-leading imports, additional unsupplied edges and nested imports
remain guarded. This bound is experimental, not a supported product limit.

Unchanged `namedtemplate17` compares exactly. A focused sibling-reference test
pins later-import precedence, selected legacy fallback in a version-2.0 dependency,
cross-sibling named reads, cancellation and fresh compilation after cancellation,
and compiled lifetime after source-module release. The selected group is now
**18/19 exact**; only nested `namedtemplate19` remains unintegrated. Production
numerators receive no new credit.

ADR-0004 review: the 1,854-line shared module compiler remains the existing
composition owner; this change factors 24 lines of entry/validation around its
unchanged composition body, without adding a second linker. The private bounded
import experiment is 297 lines including tests. Nested composition must next
separate intermediate assembly from whole-graph reference validation so a module
can legitimately reference declarations supplied by its parent or siblings.

Validation: full `scripts/verify.ps1` passes with **2,243 workspace tests**
(2,199 core, 22 native, 4 WASM, 18 worker; 78 core and 3 native ignored).
All **181 private processor-policy probes** pass with acquired cases included.
The official WASM target builds; full OASIS baseline replays remain modern
**2,504/3,173** and strict preprocessing **2,501/3,173**, with unchanged
initialization, expected-error and mismatch counts. `git diff --check` passes.

## 2026-10-22 selected nested-import completion

Unchanged `namedtemplate19` now compares exactly through a bounded supplied chain
of selected-policy modules. The selected named-template group is **19/19 exact**.
This remains private profile-integration evidence, not an increase in either
production archival numerator or permission to select production defaults.

The shared single-import merger now separates intermediate assembly from final
named-reference validation. Existing complete-program callers still validate
before returning. The chain experiment assembles from the leaf upward, preserving
policy checks and import precedence, then validates the complete program once.
A leaf-to-principal reference regression fails when only the incomplete subtree
is compiled and succeeds when its principal supplies the target. A middle-module
override and selected legacy fallback in a version-2.0 module remain effective;
compiled output also survives release of all module documents.

The private chain accepts two to four explicitly supplied documents, exact logical
href matches, and one leading import per non-leaf module. It rejects repeated
identities/cycles, missing dependencies, excess depth and additional unsupported
edges. It acquires nothing and does not introduce a general graph resolver.
Cancellation remains a control failure; fresh compilation after cancellation
succeeds. The single-module entry point still refuses all three import fixtures
when no graph is supplied, preventing accidental baseline lowering.

Next reassess the unchanged corpus frontier rather than expanding graph shapes
without a named case. General graphs, public selection, production promotion and
the other ADR-0025 gates remain separate unfinished work.

Validation: full `scripts/verify.ps1` passes with **2,245 workspace tests**
(2,201 core, 22 native, 4 WASM, 18 worker; 79 core and 3 native ignored).
All **184 private processor-policy probes** pass with acquired cases included;
the official WASM target builds. The shared module owner is 1,871 lines and the
private import experiment is 469 lines including tests: intermediate assembly
remains in the existing shared composition owner, not a new linker/backend.

## 2026-10-22 include/import reassessment and impincl02

A fresh ordinary-frontier replay found the same known profile gaps, deferred
DTD/document semantics, resource failures and invalid archival cases. It did not
establish another set of safe production percentage gains. Following completion
of the private named-template group, unchanged Xalan `impincl02` supplies the next
accepted integration pressure: the principal includes `f.xsl`, which imports
`g.xsl`. All three modules must retain selected policy and import precedence.

The bounded supplied chain now distinguishes include and import edges and delegates
each to its existing shared composition owner. Named references are still checked
only after the complete chain is assembled. Initial whole-case replay exposed an
additional refusal at the literal `sheet="f"` attribute. The selected literal
owner now admits unqualified constant attributes without braces, which cannot
enter attribute-expression lowering. AVTs, brace escapes, namespaced attributes
and other XSLT literal-result controls remain guarded. No baseline compiler,
resolver, default or public profile changed.

`impincl02` now compares exactly against its unchanged archived output. Focused
checks pin equal-precedence include ordering, duplicate named-template rejection,
legacy handling of a version-2.0 included module, and refusal of AVTs/control
attributes outside the slice. All **187 private policy probes** pass, including
acquired cases. The named-template group remains **19/19**; this is one additional
named private comparison, not a conserved impincl-group denominator or a new
production baseline credit.

Next inspect `impincl01`, whose principal has sibling import/include edges.
Do not generalize the current linear chain into arbitrary graph resolution or
admit expression attributes just because constant attributes now work.

Validation: `scripts/verify.ps1` passes with **2,247 workspace tests** (2,203 core,
22 native, 4 WASM, 18 worker; 80 core and 3 native ignored). The official WASM
target builds; Markdown links and `git diff --check` pass. Fresh full baseline
replays retain modern **2,504/3,173** and strict preprocessing **2,501/3,173**,
with 2,707 initialized, 423 separate expected-error credits and 16/19 mismatches.

## 2026-10-22 sibling import/include and impincl01

The unchanged `impincl01` principal imports `i.xsl` and includes `j.xsl`.
A bounded three-document helper validates the two leading edges and exact supplied
logical identities, lowers all modules under selected policy, merges the included
module at principal precedence, then applies the shared import merger and final
name validation. Missing/misordered identities and additional dependencies remain
guarded. No new resolver, acquisition path or production entry point is introduced.

Whole-case replay then exposed selected-value refusal of `self::node()`, used by
both dependencies. The legacy selected-value guard now admits that exact expression
to the existing shared value compiler/runtime. Differential tests cover document,
element and text focus with empty and nested text sources; there is no new XPath
evaluator. A separate sibling test pins included-over-imported named precedence,
cross-sibling name resolution, version-2.0 legacy fallback, swapped-resource
rejection and compiled lifetime after release of all module documents.

`impincl01` now compares exactly against its unchanged HTML output. All **190
private processor-policy probes** pass, including acquired cases. `impincl01` and
`02` are named private comparisons, not a conserved whole-group denominator or
production baseline credits. Named-template accounting remains **19/19**.

Read-only inspection of `impincl04` identifies the next seam: its relative include
crosses a directory boundary. Do not claim relative-resolution semantics by
assigning the href string to a supplied document identity. Integrate selected
module lowering with the existing host-neutral logical-resource owner instead
of inventing more case-shaped resolver helpers. `impincl03` additionally needs
global parameters; `05`/`06` require branching dependencies and apply-imports or
position-sensitive includes, so they are not promised immediate gains.

Validation: full `scripts/verify.ps1` passes with **2,249 workspace tests**
(2,205 core, 22 native, 4 WASM, 18 worker; 81 core and 3 native ignored).
The official WASM target builds. Full OASIS baseline replays retain modern
**2,504/3,173** and strict preprocessing **2,501/3,173**, with unchanged 2,707
initialized, 423 separate expected-error credits and 16/19 mismatches.

## 2026-10-22 selected policy over sealed resolved modules

`impincl04` supplied the reproducible next blocker: its relative include crosses
a directory boundary. The private selected compiler now consumes documents from
the existing sealed dependency loader instead of pretending an href spelling is
the dependency's canonical identity. The bridge admits a principal/leaf pair or
a bounded linear chain, not arbitrary selected-policy graphs. Every module uses
selected legacy lowering and strict preprocessing, then the existing shared
include/import assembly and final name validation. Resolution and acquisition
remain outside compilation; no production entry point or default is promoted.

Unchanged acquired fixtures now compare exactly as follows:

| Case | Pressure exercised |
| --- | --- |
| `impincl04` | Relative include across a directory boundary |
| `impincl10` | Resolved leaf import and shared precedence |
| `impincl15` | Single-NCName apply-template mode selection across an import |
| `impincl18` | `apply-imports` without imported rules, using built-in behavior |
| `impincl19` | Four resolved modules, three sharing a basename but distinct identities |
| `impincl25` | Nested imports and built-in dispatch followed by normal template selection |
| `impincl28`, `impincl29` | Imported parameter boundaries through shared invocation semantics |

These are **eight additional named exact private comparisons**. Together with
`impincl01`/`02`, ten named impincl cases compare exactly; no complete impincl-group
denominator is claimed. The selected named-template group remains **19/19**.
Original fixtures and expectations are unchanged. Production baseline credits
are not increased by these private results.

The selected legacy invocation owner now admits attribute-free, empty-body
`apply-imports` and delegates the existing compiler/runtime. A named-call control
pins current-template-rule preservation, ordinary compiler parity across work
domains, exact/one-less instruction budgets, cancellation and compiled reuse.
Attributes, text and with-param children remain guarded. `impincl15` additionally
required bounded single-NCName mode selection; special tokens, qualified names,
empty values and lists remain rejected. `impincl19` required context `.` value
conversion, now compared against shared semantics alongside `self::node()` at
document, element and text focus with empty and nested-text sources.

Loader controls retain missing/denied resource and attempt-limit failures. A
mixed-declared-version dependency proves selected legacy lowering rather than
accidental use of the ordinary compiler. Compilation controls share preprocessing
charges across both modules, pin exact/one-less budgets and cancellation, preserve
the original documents and permit a clean retry. Programs execute after loaded
graphs and snapshots have been dropped. The nested replay checks canonical
identity at every edge, preventing equal basenames from aliasing.

For ADR-0004 cohesion, bounded graph compilation moved from the growing
`single_import.rs` test-reference unit into `module_graph/mod.rs`, with its
semantic/control/corpus tests in `module_graph/tests.rs`. The compiler still
accepts XDM documents, not runtime loader types. Resolved-resource tests live
under the loader's own test subdirectory. This reduces the processor-reference
directory to fifteen direct files and separates graph composition from its
evidence without adding a crate, runtime owner, public API or unsafe surface.

Reproduction (with the separately acquired immutable corpus):

```powershell
$env:FASTXSLT_OASIS_XSLT10_ROOT = (Resolve-Path .workbench/oasis-xslt10/extracted-full/testsuite/TESTS).Path
cargo test -p fastxslt --all-features processor_edition_reference -- --include-ignored
cargo test -p fastxslt --all-features selected_policy_resolved -- --include-ignored
```

The first command passes **193** probes; the second passes **9** resolved-graph
tests. Full repository verification passes **2,254 workspace tests** (2,210 core,
22 native, 4 WASM, 18 worker; 88 core and 3 native ignored), plus formatting,
Clippy, documentation, local links and pinned corpus checks. The official WASM
target builds. Fresh full OASIS baseline replays retain
modern **2,504/3,173 (78.92%)** and strict preprocessing **2,501/3,173**,
with 2,707 initialized, 423 separate expected-error credits and 16/19 mismatches.

Next assess branching/position-sensitive composition (`impincl05`/`06`) under
the existing module owner. Read-only inspection also identified globals in
`03`/`11`, a global secondary-document read in `08`, and wider pattern admission
in `16`; these do not inherit support from the resolved-module bridge. No new
architectural decision is requested by this tranche.

## 2026-10-22 OASIS-driven branching and positioned composition

The next work stayed tied to unchanged OASIS cases rather than adding adjacent
features. `impincl05` first required branching imports and then stopped at the
selected `doc/*` guard. Selected child-path admission now permits a terminal
wildcard through the existing shared path/invocation compiler. Focused tests
compare empty/nonempty selections, element order, text/comment exclusion and
work charges against the ordinary compiler under both declared 1.0 and 2.0
with the same selected legacy policy. Predicates, descendant paths, absolute
paths, intermediate wildcards and attribute axes remain outside this guard.

The private resolved-module bridge lowers each import-only module under the
selected policy and passes already-lowered child programs to shared composition.
The existing multi-import assembly was separated from final graph validation,
as already done for the single-import path. Its ordinary wrapper retains the
same finalization and validation behavior. A regression proves a nested branch
can call a named template supplied by the principal, while completing that
branch alone still reports `FXST0014`. No missing-name error is suppressed at
the completed graph boundary.

`impincl06` then exercised includes placed among principal declarations. The
selected bridge delegates merging to the existing source-location-aware include
owner, preserving declaration order and imported subtrees. Controls pin both
include-before-template and include-after-template outcomes. Both include and
multi-import composition reject incompatible baseline artifacts with exact
`FXST1058`; the compiler does not relabel children into the selected profile.

The following original comparisons now pass without further semantic changes:

| Case | Existing behavior exercised by selected integration |
| --- | --- |
| `impincl05`, `impincl07` | Reversed sibling import order, nested imports and apply-imports |
| `impincl06`, `impincl12` | Position-sensitive includes with imported descendants |
| `impincl14` | Apply-imports remains confined to the selected import subtree |
| `impincl20` | Apply-imports uses current-node focus and built-in fallback |
| `impincl21`, `impincl22` | Nested/sibling import precedence and ordinary dispatch |
| `impincl23` | Included import outranks the earlier direct import |

The mixed case reuses the existing mixed-composition algorithm through a new
already-lowered-program entry point; ordinary callers still compile through the
same wrapper. A focused control proves precedence, selected legacy fallback in
a version-2.0 descendant and exact rejection of incompatible artifacts. The
acquired `impincl13` replay separately pins `FXAR0031` at its included stylesheet's
computed-element instruction. That refusal is not a conformance pass and is not
hidden by the successful graph composition cases.

This adds **nine named exact private comparisons**, bringing the named impincl
set to **nineteen**. It is not a conserved whole-group report, a new production
credit or an XSLT 1.0 conformance claim. The named-template private denominator
remains **19/19**. Acquisition remains test-host-owned filesystem import followed
by sealed memory-only resolution; no compiler/runtime ambient authority changed.

Validation: **197 private processor-policy probes** and **14 resolved-graph
tests**, including acquired cases, pass. Full repository verification passes
**2,258 workspace tests** (2,214 core, 22 native, 4 WASM, 18 worker; 93 core and
3 native ignored), formatting, Clippy, documentation, links and corpus checks.
Full baseline replays retain modern **2,504/3,173 (78.92%)** and strict
preprocessing **2,501/3,173**, with 2,707 initialized, 423 separate expected-error
credits and 16/19 mismatches. The official WASM target is checked independently.
That build also passes. `git diff --check` and local Markdown links pass.

ADR-0004 review: the selected graph owner is 371 lines with 885 lines of tests;
the existing shared module-composition owner is 1,908 lines. The shared changes
extract assembly from existing compilation/finalization functions without adding
a new responsibility or copying the precedence algorithm. No additional source
unit or directory split is warranted for this narrow change; reassess before
further growth of the shared owner.

Next-case inspection, not a claim of complete execution classification:

- `impincl13`: computed-element name from a source attribute in its included
  module. This is now an executable unsupported classification, not just inspection.
- `03`/`11`/`24`: selected global declarations and dependent expressions/AVTs.
- `08`: a global secondary-document read in the included module.
- `09`: computed-element and attribute-set integration.
- `16`/`17`: broader patterns and key-related declarations.
- `26`: a deeper import chain beyond the current replay's depth ceiling.
- `27`: explicit `file:` logical identity and copy semantics; it does not grant
  permission for filesystem fallback.

No new AR/ADR decision is needed for the completed slice. Continue removing
concrete selected-profile integration gaps, then replay the full profile-labelled
corpus; do not turn these private witnesses into production numerator gains.

## 2026-10-22 computed attribute name and deeper import replay

The previously pinned `impincl13` blocker is now removed. Its unchanged included
stylesheet constructs an element using `name="{@x}"`, then applies imported
templates inside that element. Selected legacy lowering admits exactly an
unqualified attribute-name AVT and delegates to the existing dynamic-name path
compiler and runtime. The shared constructor assembly was extracted so ordinary
and selected compilation do not create separate evaluators. The selected entry
sets legacy semantics explicitly, including when the stylesheet declares 2.0 or
3.0. It does not consult local version to choose processor identity.

The selected guard still rejects general AVTs, namespaced attribute selections,
predicates, namespace overrides, attribute-set use and computed-attribute children.
Its body still uses selected lowering. Focused differential tests cover valid
unprefixed/prefixed results, missing attributes, malformed/unbound names, complete
failure/provenance text from the test adapter, shared work charges, exact/one-less
result-node limits, cancellation and clean reuse. The modern selected guard is
unchanged. Result QName validation remains owned by the shared runtime.

`impincl13` now compares exactly against its original expected output. The next
low-risk replay, `impincl26`, also compares exactly using the existing recursive
import bridge. Only the corpus test host's explicit depth allowance changes from
two to four; the five-resource/aggregate-byte ceilings remain in place. No
production resource defaults or resolver semantics change. The earlier fixed
two-to-four-document supplied-chain reference remains unchanged.

This adds **two named exact private comparisons**, bringing the named impincl
set to **twenty-one**. It is still not a conserved full-group denominator or a
production coverage increase. The remaining named cases are `03`, `08`, `09`,
`11`, `16`, `17`, `24` and `27`; earlier inspection categories remain visible.
The unchanged `p.xsl` leaf dependency of `impincl03` now has an executable
`FXAR0031` unsupported check at its global parameter declaration. That is a
next-blocker witness, not a pass for `impincl03`. It requires selected global
declaration/expression integration, not a new computed-name extension.

Validation: **200 private processor-policy probes**, **16 resolved-graph tests**
including acquired comparisons and the explicit unsupported witness, and
**2,261 workspace tests** (2,217 core, 22 native, 4 WASM, 18 worker; 95 core and
3 native ignored). Formatting, Clippy, docs, links and pinned corpus checks pass.
The official WASM target builds. Full OASIS replays retain modern
**2,504/3,173 (78.92%)** and strict preprocessing **2,501/3,173**, with 2,707
initialized, 423 separate expected-error credits and 16/19 mismatches.

ADR-0004: focused constructor tests are grouped below the existing integration
test owner instead of adding another direct file to the fifteen-file reference
directory. No new runtime ownership, public API, acquisition authority or unsafe
surface is introduced. No architecture decision is requested by this slice.

## 2026-10-22 complete private impincl denominator

The acquired-suite test `selected_policy_measures_complete_impincl_group` now
classifies every unchanged `impincl01` through `impincl29` under the same private
selected legacy processor / strict-preprocessing pairing. It uses the existing
sealed dependency loader, selected module lowerers and shared runtime, comparing
the original `REF_OUT` files with the existing XML comparator. No per-case profile
selection or expected-output correction is applied.

The conserved result is **29 = 21 exact + 7 selected-unsupported + 1 unadmitted
resource**. Each case's disposition is pinned, rather than asserting only a total.
There are no observed mismatches, execution failures or harness-comparator gaps
in this replay. The seven refusals retain `FXAR0031` and their module provenance:

| Cases | First blocked module / surface |
| --- | --- |
| `03` | `impincl/p.xsl`, global parameter |
| `08` | `impincl-test/impincl08.xsl`, global secondary-document value |
| `09` | principal module, computed element / attribute-set integration |
| `11` | `impincl/fragments/ss5.xsl`, global declaration |
| `16` | `impincl/fragments/imp16all.xsl`, broader pattern |
| `17` | `impincl/fragments/imp17all.xsl`, key declaration |
| `24` | `impincl/fragments/i24imp.xsl`, global declaration |

`impincl27` requests the logical identity `file:fragments/imp27b.xsl`, which is not
admitted by this private replay's canonical HTTPS-only importer. This is **not**
an assertion that the upstream resource is absent. The ordinary OASIS harness
already has an explicit historical URI alias policy; integrating that host-side
policy is separate from changing compiler acquisition authority. No filesystem
or network fallback is added. Its dependency also contains selected copy
semantics that have not yet been replayed through this lane.

The test host recursively admits regular stylesheet files from the two acquired
impincl directories, without following symlinks. Snapshot ceilings are 256
resources, 1 MiB per resource and 16 MiB aggregate; graph ceilings are 32 attempts,
depth 8, 32 modules and 1 MiB aggregate, with XML event/depth limits of
100,000/128. These are private measurement allowances, not production defaults.
Unsupported composition shapes remain explicitly classified harness limitations.

Next-case inspection confirms that `impincl03` is not solved by admitting
`xsl:param` alone. It needs constant global defaults across its module graph,
global-value concatenation and an attribute-value match predicate. The selected
body currently uses a flat local-binding list both for visibility and duplicate
checking. Appending global names to that list would incorrectly reject legal
local shadowing. Separate static global visibility from local declaration checks
before widening the selected guard; retain the shared global runtime.

Reproduce with `FASTXSLT_OASIS_XSLT10_ROOT` pointing to the acquired suite's
`testsuite/TESTS` directory:

```text
cargo test -p fastxslt --all-features selected_policy_measures_complete_impincl_group -- --ignored --nocapture
```

This tranche removes an accounting blind spot, not an engine blocker: the former
21 named successes are now a complete classified 29-case private denominator.
Production coverage remains **2,504/3,173 (78.92%)**, with strict-preprocessing
coverage **2,501/3,173**. No production profile promotion, new architectural
decision or corpus numerator credit is implied.

Validation: the acquired 29-case replay passes. `scripts/verify.ps1` passes
formatting, workspace Clippy/tests, documentation, local Markdown links and
pinned corpus checks. The official `wasm32-unknown-unknown` workbench build and
`git diff --check` also pass. Production OASIS totals above are the unchanged
preceding full-replay checkpoint, not a newly run production measurement.

## 2026-10-22 selected constant global parameter admission

The motivating unchanged `impincl03` leaf `p.xsl` now compiles under selected
legacy policy. The private guard admits only `xsl:param` with a constant string
`select`, limited to direct single-quoted literals or `string()` around a string
literal, and delegates declaration validation/storage to the shared compiler.
No selected global constructor, global variable, secondary-document default,
`required` metadata or general expression is admitted. Ordinary entry points
and production defaults are unchanged.

Focused tests cover declared versions 1.0/2.0/3.0, both quote delimiters inside
`string()`, source-document release, repeated execution, valid local shadowing,
duplicate global/local diagnostics and guarded expressions. A declared but
unintegrated same-module global read is unsupported rather than falsely unbound.
Bare double-quoted defaults remain guarded: the experiment found that the shared
direct-global literal path does not currently lower that form. This limitation
is recorded rather than extending unrelated production expression behavior.

The acquired leaf regression now asserts the compiled `pParam` string default.
It is not a whole-case pass. Full private group replay stays **21 exact + 7
selected-unsupported + 1 unadmitted resource = 29**. `impincl03` now first refuses
the principal value expression at span 411..475; `impincl11` likewise advances
from its included declaration to its principal expression at span 429..493.
Other classifications are unchanged. The next implementation owner is static
global visibility across linked modules, distinct from the local-binding list;
then bounded global-value concatenation can enter shared expression lowering.

Correction to the prior inspection note: the existing matched-template guard
already admits the shared `ElementWithAttributeValue` pattern variant. The
whole-group replay reaches the principal expression in `03`; do not create an
unnecessary separate predicate integration task from the earlier inspection.

This removes a concrete declaration-admission blocker with no corpus numerator
gain. Production coverage stays **2,504/3,173 (78.92%)**. No new AR/ADR decision
is requested; global-read integration remains work under ADR-0025.

Graph regressions also pin shared import override precedence, same-precedence
include rejection (`FXST0029`), successful non-conflicting inclusion and retained
default ownership after the source module is dropped. These are declaration
integration controls, not proof that selected global reads are complete.

Validation: **203 processor-policy probes**, **18 selected-policy loader/replay
tests**, and **2,264 workspace tests** pass (2,220 core, 22 native, 4 WASM,
18 worker; 96 core and 3 native ignored in the ordinary workspace run).
Formatting, Clippy, documentation, local links, pinned corpus checks, the official
WASM build and diff whitespace checks pass. No full production OASIS rerun was
needed for this test-only admission change; the production totals above retain
their preceding measurement provenance.

## 2026-10-22 selected dependency global reads

Unchanged **`impincl03` and `impincl11` now compare exactly**. The complete private
selected-legacy / strict-preprocessing group is **29 = 23 exact + 5
selected-unsupported + 1 unadmitted-resource**. The case-by-case ledger required
explicit updates after each newly exact comparison. Original fixtures and
comparators are unchanged; these are private-profile integration gains, not new
production numerator credits.

Selected lowering now carries separate static local/global name slices. Only
locals seed the shared sequence compiler's mutable lexical-binding list, so a
local declaration may shadow a global without becoming a duplicate. Expression
visibility combines the two without mutating global declarations or runtime
state. Conditional, fallback, literal-result, message and computed-element body
recursion preserve that distinction. Own-module parameter names and names from
already compiled dependency globals are available to the resolved import,
include and mixed-composition entries. The shared linker still owns precedence,
duplicate validation and artifact qualification.

The newly admitted concatenation is deliberately narrow: two or more bare
unqualified variable references. It checks static visibility, then uses shared
legacy value-expression lowering/execution. Unknown names remain static failures,
including in untaken forward-compatible branches. General concatenation grammar,
AVTs, global constructors and newer-edition semantics are not inferred.

Focused tests cover direct global reads, local shadowing within a conditional,
sibling scope restoration, versions 1.0/2.0/3.0, released stylesheet storage and
repeated execution. Shared-reference parity includes output and work charges,
exact/one-less result-text ceilings, cancellation at the first result-text charge
and clean reuse. Existing import/include declaration controls remain in force.

This is **not complete bidirectional graph name binding**. Dependencies are still
compiled before their parents in these private entries; principal-declared
globals are not passed backward into imported template compilation. `impincl24`
still needs that capability, a global variable declaration and general AVT
integration. The other remaining refusals are `08`, `09`, `16` and `17`;
`27` retains the explicit historical URI alias admission gap. Older supplied-graph
helpers and other expression consumers are not silently promoted by this slice.

ADR-0004: private fallback-body routing and variable-concat eligibility are
named helpers within their existing selected-lowering owner. This avoids
lengthening the sequence/value lowering functions beyond the lint threshold,
without introducing another runtime or public abstraction. Production defaults
and **2,504/3,173 (78.92%)** coverage remain unchanged. No new architecture decision
is required for the next accepted integration work.

Validation: **205 processor-policy probes**, **18 selected-policy loader/replay
tests**, and **2,266 workspace tests** pass (2,222 core, 22 native, 4 WASM,
18 worker; 96 core and 3 native ignored in the ordinary workspace run).
Formatting, workspace Clippy, docs, links and pinned corpus verification pass;
the official WASM build and diff whitespace checks pass. Full production OASIS
totals retain their preceding measurement provenance because the changed
semantic entry remains test-only.

## 2026-10-22 complete-pair global visibility and impincl24

Unchanged **`impincl24` now compares exactly**. The private selected-legacy /
strict-preprocessing include/import denominator is **29 = 24 exact + 4
selected-unsupported + 1 unadmitted resource**. `08`, `09`, `16` and `17` remain
unsupported; `27` retains its historical URI alias admission gap. No fixture,
expected output, profile choice or comparator changed.

The existing complete resolved-pair compiler now collects both modules' validated
constant global names before lowering either body. This supplies principal names
to imported templates without deferring static errors or putting values in the
name pass. Shared linking still owns precedence, duplicate checks and artifact
qualification; shared runtime globals still own values. This is bounded pair
integration, not arbitrary graph-wide predeclaration. The group replay selects
this entry by graph shape (one leaf dependency), not by fixture name.

Constant-string global variables reuse the parameter slice's declaration guard.
The selected literal-attribute guard additionally admits exactly a whole
`{concat(string-literal, $NCName)}` AVT with no surrounding text or nested braces.
Its eligibility parser and execution representation are the existing shared
attribute compiler's; value-of admits the same literal/variable concatenation
through shared value lowering. General AVTs and expressions remain unsupported.

Focused tests pin imported reads under declarations 1.0/2.0/3.0, source-owner
release, repeat execution, equal-identity changed-generation isolation, exact /
one-less result-text limits, cancellation and clean recovery. Unbound AVT names
are static errors; surrounding text, reversed arguments, nested functions and
variable-only AVTs remain guarded. The new cohesive tests live below the existing
module-graph owner rather than increasing the flat reference directory.

Next-case inspection confirms the remaining group is heterogeneous: `16` also
requires compound position/value predicates, `17` needs keys, `09` combines
attribute-set declarations with a computed constructor, and `08` reads a
secondary document. These are separate integration slices, not reasons to widen
the newly admitted AVT. Production coverage remains **2,504/3,173 (78.92%)**;
this is one private integration gain and no production promotion.

Validation: **208 processor-policy probes**, **18 selected loader/replay tests**
and **2,269 workspace tests** pass (2,225 core, 22 native, 4 WASM, 18 worker;
96 core and 3 native ignored by the ordinary run). Formatting, Clippy, docs,
local links, pinned corpus checks, official WASM build and diff whitespace checks
pass. Production OASIS totals retain the preceding full-replay provenance.

## 2026-10-22 static attribute sets and impincl09

Unchanged **`impincl09` now compares exactly**. The complete private
selected-legacy / strict-preprocessing group is **29 = 25 exact + 3
selected-unsupported + 1 unadmitted resource**. No expected bytes or comparator
rules changed. `08`, `16` and `17` remain unsupported; `27` still needs explicit
historical URI alias admission in this measurement host.

This slice admits NCName attribute-set declarations containing only static
NCName attributes with text-only values (including empty values). Static NCName
`xsl:element` constructors can supply a literal namespace and a nonempty NCName
set list. Shared declaration compilation, set finalization, computed construction
and execution perform the semantics. The guard rejects dynamic attribute values,
referenced sets inside declarations, namespace AVTs and newly combined options
on the earlier dynamic-name constructor. It does not generalize the set or AVT
subsystem merely because the ordinary compiler knows additional shapes.

Differential tests compare shared-reference output and work charges under
declared versions 1.0/2.0/3.0, including reversed set order, conflicting attribute
precedence, an empty attribute and literal namespace attachment. They pin
missing-set diagnostics, exact/one-less result-text limits, cancellation at a
real charge, source-owner release and reuse. Older unsupported-route tests now
distinguish admitted static namespaces and malformed integrated declarations
from still-guarded dynamic routes.

ADR-0004: the new guard remains a named static declaration helper; focused tests
extend the existing computed-constructor test module. No alternate compiler,
runtime, resolver authority or public profile is introduced. Production coverage
remains **2,504/3,173 (78.92%)**; this is a private integration gain only.

Validation: **210 processor-policy probes**, **18 selected loader/replay tests**
and **2,271 workspace tests** pass (2,227 core, 22 native, 4 WASM, 18 worker;
96 core and 3 native ignored in the ordinary run). Formatting, Clippy, docs,
links, pinned corpus verification, the official WASM build and diff checks pass.
Production OASIS totals retain the preceding full-replay provenance.

The next-case inspection must not mistake declaration admission for whole-case
support: `impincl17` also needs key truth tests, key-selected template dispatch,
source-attribute local bindings and key-count values. `impincl08` additionally
needs relative secondary-resource admission and owner-qualified global document
reads. Both remain complete-case tasks rather than single declaration guards.

## 2026-10-22 child copy and explicit host alias admission

Unchanged **`impincl27` compares exactly**. The complete private selected-legacy /
strict-preprocessing include/import group is now **29 = 26 exact + 3
selected-unsupported** (`08`, `16`, `17`). Original fixtures, expected bytes and
comparison rules are unchanged. Production remains **2,504/3,173 (78.92%)**
under the preceding full-replay provenance; this private result is not a
production numerator increase or promotion of the processor profile.

The measurement host explicitly aliases historical `file:` references to bytes
it already loaded from the acquired stylesheet directories. It reuses the
existing corpus importer's canonical path-containment check, requires the target
to exist in the already-loaded map, rejects ambiguous aliases with different
bytes, and admits every alias through the existing resource count/byte limits.
Discovery does not read additional resource bytes. The test importer retains its
own original-byte map; this is not production prepared-state accounting. The
engine still receives only a sealed snapshot. A negative replay with alias
admission disabled remains `unadmitted-resource` at
`file:fragments/imp27b.xsl`; explicit admission permits the two-import graph.
There is no runtime filesystem fallback or case-selected processor policy.

The selected legacy instruction guard now admits `xsl:copy-of` selecting one
unqualified NCName child element. It delegates selection and copying to the
shared compiler/runtime and explicitly selects legacy recovery, independent of
declared version. The admitted shape cannot select unattached attributes.
Variables, attributes, namespaces, document lookup, wildcards, predicates and
longer paths remain guarded rather than broadening copy semantics speculatively.

Focused differential tests cover declared versions 1.0/2.0/3.0, child subtrees,
namespace/attribute retention, text/comments/PIs, work-charge parity,
exact/one-less result-node limits, cancellation, compiled-owner independence and
clean reuse. Tests occupy a private source-copy module under the existing
matched-integration test owner. No public API or alternate executor was added.

Validation: **212 processor-policy probes**, **19 selected loader/replay tests**
and **2,273 workspace tests** pass (2,229 core, 22 native, 4 WASM, 18 worker;
97 core and 3 native ignored in the ordinary run). Formatting, Clippy, docs,
Markdown links, pinned corpus checks and the official WASM build pass. The first
focused invocation omitted the acquired-corpus environment variable; its 19
fixture-loading failures disappeared when rerun with the required root.

Next complete-case candidates remain secondary-document globals (`08`), compound
value/position predicates (`16`) and key lookup/truth/count/dispatch (`17`).
Their integration must preserve selected policy throughout shared lowering;
declaration admission alone is not completion. No new architectural decision is
requested by this child-copy/host-import tranche.

## 2026-10-22 typed match predicates and impincl16

Unchanged **`impincl16` compares exactly** under the private selected-legacy /
strict-preprocessing lane. The complete include/import denominator is now
**29 = 27 exact + 2 selected-unsupported** (`08`, `17`). The first replay exposed
the intentionally pinned old classification; it was updated only after observing
the exact comparison. No fixtures, original expectations or comparator changed.
Production remains **2,504/3,173 (78.92%)**, with its preceding full-replay
provenance; these selected-policy gains do not change that numerator.

The shared compiler already produces the required match forms. Selected admission
now accepts `//NCName`, named-element attribute presence, wildcard numeric
equality and typed wildcard Boolean predicates with context-number equality,
position comparisons and attribute-string equality, composed with and/or/not.
The guard inspects the shared typed predicate instead of adding another parser
or matcher. Numeric-attribute predicates, named-element Boolean predicates,
general descendant paths and unrelated node kinds remain guarded. The modern
selected route is unchanged.

Focused tests use independently stated outputs and the ordinary 1.0 compiler as
a differential control under declarations 1.0/2.0/3.0. Interspersed text,
comments and PIs pin element-sibling position semantics; the tests also compare
node-visit/instruction/result-text work charges, exact/one-less node-visit
ceilings, cancellation at a real charge, source-stylesheet release and clean
compiled reuse. Negative tests preserve explicit refusal and resource provenance.

Clippy identified module-lowering function-size pressure during validation.
Preflight admission now returns local version and excluded declarations to a
separate private validated-lowering helper. Shared declaration composition and
execution remain unchanged; no lint exemption or second compiler was introduced.
Focused tests live under the existing matched-integration test directory.

Validation: **214 processor-policy probes**, **19 selected loader/replay tests**
and **2,275 workspace tests** pass (2,231 core, 22 native, 4 WASM, 18 worker;
97 core and 3 native ignored in the ordinary run). Formatting, Clippy, docs,
local links, pinned corpus checks and the official WASM target pass.

Next-case inspection: `impincl08` is not just resource admission. Shared
`compile_xslt10_literal_document_global` still gates on local compatibility;
selected integration needs an explicit global-default lowering entry and a
guarded owner-qualified `$variable/child/path` read before admitting it. Its
relative URI must remain anchored to the included module and resolve only from
the supplied snapshot. Required tests include missing/denied resources, changed
snapshot isolation, controls and declared-version independence. `impincl17`
still requires the complete key lookup/truth/count/dispatch composition. Neither
case earns credit from relaxing a declaration guard alone. No new architectural
decision is requested by the completed pattern slice.

## 2026-10-22 selected document globals and impincl08

Unchanged **`impincl08` compares exactly**. The private selected-legacy /
strict-preprocessing include/import group conserves **29 = 28 exact + 1
selected-unsupported** (`17`). Original resources, expected outputs and comparison
rules remain unchanged. Production remains **2,504/3,173 (78.92%)** under its
preceding full-replay provenance; this is private integration, not promotion.

The declaration composition seam now accepts an explicit global-binding lowerer
alongside its template/output/named-template lowerers. Ordinary compilation
supplies its existing global-binding compiler. The selected route admits only
constant strings and global variables selecting one nonempty literal document
root, with no second argument or suffix. Shared binding validation, reference
construction, duplicate handling and module assembly remain authoritative.
The shared literal-document compiler was separated from its ordinary local-version
gate so selected compilation can call it explicitly without pretending that a
future declared version selects a modern processor. This is a private callback,
not a new public contract or global evaluator. The existing cohesive declaration
composition lint allowance now also covers its eight explicit arguments.

Selected value-of admits a bound variable followed by unqualified child-element
steps, with static binding validation and explicit legacy value lowering. It
uses the existing owner-qualified variable path executor. Document parameters,
dynamic/empty references, suffixes in global document expressions and arbitrary
variable paths remain guarded. No source document enters compiled state.

The group measurement host now seals XML resources as well as stylesheet bytes
from the same acquired directories under its existing count/byte limits. It
passes the snapshot to shared execution; admission does not eagerly parse all
documents. Historical aliases remain explicit host policy. The `impincl08`
document is resolved relative to the included module, not the principal module.

Focused tests cover declarations 1.0/2.0/3.0, a wrong-base decoy, a wrong-principal-
source decoy, first-node value conversion, module/source-owner release, changed
snapshot bytes under the same identity, explicit missing/denied outcomes and
clean recovery. The ordinary 1.0 include compiler provides a differential output
and XML-event/XPath/instruction/result-text charge reference. Exact/one-less
XML-event limits and cancellation during secondary parsing preserve controlled
failure without poisoning reuse. New tests have a private document-resource
owner below the existing loader test directory.

Remaining complete-case task: `impincl17` needs key declarations, attribute-backed
local values, key Boolean tests, key-count value-of and key-selected dispatch.
Do not treat declaration admission alone as progress on its exact result.

Validation: **214 processor-policy probes**, **21 selected loader/replay tests**
and **2,277 workspace tests** pass (2,233 core, 22 native, 4 WASM, 18 worker;
97 core and 3 native ignored by the ordinary run). Formatting, Clippy, docs,
local Markdown links, pinned corpus checks, official WASM build and diff
whitespace checks pass. No vendor changes, production profile promotion or
public API changes are included.

## 2026-10-22 bounded keys and complete impincl group

Unchanged **`impincl17` compares exactly**. The private selected-legacy /
strict-preprocessing include/import denominator is now **29 = 29 exact**.
No case is excluded or credited through an expected-error substitute; original
resources, expected outputs and comparison rules remain unchanged. This completes
this private group, not the processor profile. Production remains
**2,504/3,173 (78.92%)** under its preceding full-replay provenance.

The selected key declaration guard admits NCName names, NCName element matches
and unqualified attribute use paths. One cohesive private key-admission owner
validates literal NCName lookup names and attribute/bound-variable lookup values,
then delegates compilation to the existing key compiler. Bare key conditions,
key-count value-of and parameter-free unsorted key dispatch use the existing
Boolean/value/invocation IR and runtime. Attribute-backed local variables retain
source-node values through shared variable compilation, not early string coercion.

Shared apply-templates assembly now accepts an explicit selection lowerer;
ordinary compilation supplies the original selection parser. The private selected
route supplies the admitted key selection directly, so future declared versions
cannot silently select a different interpretation. Modes, sorting and arguments
on this key dispatch remain guarded. No key index, cache ownership, runtime
backend, authority or public API is introduced.

Focused tests compare explicitly stated results and ordinary 1.0 output/work
charges under declarations 1.0/2.0/3.0. They cover duplicate key matches,
missing versus explicitly empty attributes, key truth/count/dispatch composition,
exact/one-less node-visit budgets, cancellation during charged execution, cleanup,
stylesheet-owner release, changed source bytes under the same logical identity
and original-source reuse. Negative tests pin unbound-variable diagnostics and
provenance, guarded declaration families and rejected mode/sort/argument dispatch.
The old top-level test now distinguishes integrated but malformed key declarations
from genuinely unintegrated declaration families.

Clippy's value-of size warning was resolved by extracting bound child-path
admission as a named private helper; no additional size exemption was added.
Key admission lives in its own private subdirectory, while its behavioral tests
join the existing matched-integration test directory. Shared selection assembly
retains validation, parameter handling and output ownership.

Validation: **217 processor-policy probes**, **21 selected loader/replay tests**
and **2,280 workspace tests** pass (2,236 core, 22 native, 4 WASM, 18 worker;
97 core and 3 native ignored in the ordinary run). Formatting, Clippy, docs,
local links, pinned corpus checks, official WASM build and diff checks pass.

Next: stop extending this now-complete group's grammar. Reassess the next
reproducible selected-profile corpus failure against the remaining ADR-0025
production gates. These results do not justify public profile selection,
production default changes or general key/dispatch support beyond the admitted
shapes. No new architectural decision is requested by this completed slice.

## 2026-10-22 selected source whitespace declarations

Reassessment after completing `impincl` selected the unchanged Xalan
`whitespace01`–`07` cases. Before admission, `whitespace01` reproducibly failed
with unsupported `FXAR0031` at its strip-space declaration (bytes 329..364).
All seven now compare exactly in the private selected-legacy / strict-
preprocessing lane, with the comparator's whitespace-preserving option enabled.
These are **seven selected comparisons, not a complete whitespace denominator**.
The separate include/import group remains **29/29 exact**. Neither changes the
production **2,504/3,173 (78.92%)** numerator or its full-replay provenance.

The selected declaration preflight admits nonempty whitespace-separated lists
of ASCII NCName NameTests: unqualified/qualified names, `*` and prefix wildcards.
It delegates prefix resolution, expanded-name matching, declaration validation
and same-module strip/preserve precedence to the existing shared policy builder.
The first five cases justified unqualified declarations; `06`/`07` then supplied
the qualified-name/wildcard pressure. This is policy propagation under ADR-0025,
not new whitespace semantics or a new source representation. ADR-0012 continues
to own invocation-local visibility, preserved node identity, inherited xml:space,
control charging and immutable prepared inputs. Stylesheet preprocessing stays
separate. Existing cross-module composition guards are not weakened.

Focused tests compare selected declarations 1.0/2.0/3.0 with the shared ordinary
1.0 compiler and an explicit whitespace-sensitive expected tree. They cover
effective subtree copy, preserve exceptions, inherited `xml:space="preserve"`
and a nested `default` reset, exact/one-less XDM-node limits, cancellation during
view work, clean reuse and unchanged original input string value. A preserving
program subsequently sees the original whitespace from the same prepared source.
Additional namespace tests use different source/stylesheet prefixes for the same
URI and the same spelling for different URIs, pinning expanded-name ownership.
Unbound names, unadmitted declaration controls and modern selected admission stay
explicit failures. The pre-existing top-level test now treats missing whitespace
attributes as malformed integrated declarations rather than blanket unsupported.

Production-readiness reassessment: completing a private corpus group does not
complete ADR-0025. General expression error timing, truthful capability/
introspection reconciliation and complete graph/artifact qualification still
prevent production promotion. No default, public selector or conformance claim
was selected here. Continue from a named failing case, not by enabling every
shared lowering branch. In particular, the remaining whitespace group includes
stylesheet CDATA/preprocessing behavior (`whitespace09`), which must remain
distinct from this source-whitespace slice and retain original expectations.

Validation: **221 processor-policy probes**, **22 selected loader/replay tests**
and **2,283 workspace tests** pass (2,239 core, 22 native, 4 WASM, 18 worker;
98 core and 3 native ignored in the ordinary run). Formatting, Clippy,
documentation, local links, pinned corpus verification, the official WASM target
and diff checks pass. No vendor files, production defaults or public APIs changed.

## 2026-10-22 complete whitespace group and selected focus

The next replay covers every catalogued Xalan `whitespace` case, not a selected
prefix. The harness checks its 22 pinned identities against the parsed immutable
catalog, rejects duplicate identities, imports source/style/supplemental XML
bytes into a bounded sealed snapshot, and retains whitespace-sensitive XML
comparison. Each case has a pinned disposition; all cases are reported before
classification drift fails the test. No source acquisition occurs in execution.

Initial private selected-legacy / strict-1.0 replay: **14 exact + 2 mismatches +
6 selected-unsupported = 22**. After this tranche: **18 exact + 2 mismatches +
2 selected-unsupported = 22**. Newly exact cases are `12`, `18`, `19`, `20`.
Cases `08`–`11`, `13`, `15`, and `22` were already exact when the whole group was
first replayed; these are newly measured, not implementation gains. In particular,
the CDATA case `09` did not require a preprocessing repair.

The implementation adds only selected-policy admission for shared focus
`position()`/`last()`, unsorted NCName child-path iteration, and constant literal
`xml:space` preserve/default attributes. Shared selection, iteration execution,
lexical frames, result construction and canonical preprocessing remain owners.
This is not a second iteration evaluator. Broader selections, sort/default-mode
controls, dynamic xml:space and other qualified literal attributes stay guarded.

Regressions pin nested iteration position/size and caller-focus restoration,
empty selections, global-to-local shadowing without sibling leakage, declarations
1.0/2.0/3.0 under the same selected legacy processor, shared ordinary-1.0 output
and charge parity, exact/one-less instruction budgets, cancellation and clean
reuse. A first test incorrectly attempted same-template local shadowing; the
selected compiler rejected it with existing `FXST0017`. The fixture was corrected
to legal global-to-local shadowing; duplicate validation was not weakened.
Literal whitespace tests pin inherited preserve, default reset, nested preserve
and retained result attributes independently of source stripping.

Remaining dispositions:

| Cases | Disposition | Next boundary |
| --- | --- | --- |
| `17` | Mismatch | Existing serialization-layout expectation; both ordinary modern preprocessing and selected strict replay differ from the original output. |
| `21` | Mismatch | Comment-separated whitespace: ordinary modern preprocessing matches the archive, selected strict does not. ADR-0023 requires retaining this mismatch, not selecting modern per case. |
| `23` | Selected unsupported | Normalized child-path value in computed attributes and literal AVTs; iteration is no longer the first blocker. |
| `35` | Selected unsupported | `document()` apply selection plus dynamic element construction; supplemental `docws35.xml` is already sealed, so this is not unavailable-resource evidence. |

The two mismatch controls explicitly replay both existing paths against unchanged
expectations with whitespace preservation. `22`'s exact result exercises the
shared same-module strip/preserve recovery; it does not establish a universally
required recovery policy. The production **2,504/3,173 (78.92%)** and strict-
preprocessing **2,501/3,173** baselines retain their preceding full-replay
provenance. Private profile results do not add to either numerator.

ADR-0004 review: the selected module remains a private integration coordinator
(approximately 1,650 lines). Sequence traversal and per-element dispatch now
have named helper boundaries, as does bounded iteration admission. No new
runtime state, public abstraction or sibling-module dependency was introduced.
The existing whitespace test subject remains cohesive; no new directory is
needed for these additions.

Validation: **226 processor-policy probes**, **24 selected-policy replay/control
tests**, and **2,287 workspace tests** pass (2,243 core, 22 native, 4 WASM,
18 worker; 100 core and 3 native ignored in the ordinary run). Formatting,
Clippy, documentation, 1,998-file Markdown-link verification, pinned corpus
verification, the official WASM target and `git diff --check` pass. Logs are
`.workbench/selected-whitespace-next-{probes,replay,verification,wasm}.log`.
No vendor edits, public API changes, default changes or production promotion.

## 2026-10-22 promotion audit and qualification safeguard

Under accepted ADR-0026, the next work is now a finite ledger rather than another
feature-admission slice. [R0](../promotion-ledger.md) proposes
nine obligations traced to ADR-0025, a legacy-only feature ceiling, explicit
deferrals and a three-tranche review boundary. It awaits maintainer scope
approval; no authoritative gate denominator or closure is inferred.

The code audit identified Q01: shared context-taking helpers could lower a
principal through the declaration-driven compiler and then qualify it with a
selected-processor stamp. Equality of parent/child stamps is insufficient when
the principal's lowering did not apply that processor. This was a private-path
qualification risk; no production-profile promotion or observed deployed wrong
output is claimed.

The safeguard checks `require_declaration_driven_lowering` before the five
context-taking include/import compilation entry shapes and the graph-context
dispatcher (including leaf and intermediate requests). A selected request is an
explicit unsupported `FXST1058` with original location, never a `None` that can
fall through to baseline graph compilation. Ordinary preprocessing-only contexts
retain baseline lowering. The mixed internal composition helper also checks the
condition at its own lowering boundary.

Selected integration continues to lower under its selected policy and merge
through shared assembly. Older chain/reference tests now use a narrowly checked
empty dependency shell: root/version validation, exactly one expected edge,
valid href and no edge content, with no principal body permitted. This shell
cannot become a general way to stamp baseline instruction compilation. Existing
cross-processor/preprocessing rejection, chain lifetime and control tests remain.

Validation design was tightened during this work. The initial single-route
`is_err()` probe could pass because its expression was unsupported by baseline
lowering; attempts using processor-version output and a literal `xsl:sequence`
were unsuitable positive controls for that baseline. The final regression uses
an executable text body, requires baseline compilation **and execution** to
succeed for every route, then checks the selected request's exact guard category,
code, detail and provenance. It tests qualification eligibility, not a claimed
semantic output difference. A separate graph test requires explicit rejection
for selected leaf/parent and principal/intermediate calls while baseline succeeds.

No corpus cases are newly credited. Production remains 2,504/3,173 exact and
baseline strict preprocessing 2,501/3,173, using their prior full-replay provenance.
Q01 is recommended for classification within proposed P06, not a tenth mandatory
gate; its repair does not close the whole artifact-qualification obligation.

Final validation: **2,289 workspace tests** (2,245 core, 22 native, 4 WASM,
18 worker), **227 processor-policy probes**, and **24 selected-policy
replay/control tests** pass. The ordinary run retains 100 core and 3 native
ignored tests. Formatting, Clippy, docs, local links, pinned corpus verification,
official WASM build and diff checks pass. Logs are
`.workbench/profile-promotion-{verification,probes,replay,wasm}.log` and
`profile-promotion-qualification-controls.log`. No vendor edits or default/API
changes; work remains uncommitted. Pending next action is R0 maintainer review.
