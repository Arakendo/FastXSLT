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
