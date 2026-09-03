# FastXSLT Roadmap

The roadmap is ordered by executable semantic evidence, not by a desire to fill
every conceptual module. Dates are intentionally absent until requirements and
standards scope are decided.

## Current critical path

The first
[adversarial engine review](../Reviews/adversarial-engine-review-2026-08-30.md)
found a misleading-green corpus overlay and six reproducible semantic
counterexamples. The
[first correctness tranche](../Evidence/adversarial-review-first-correctness-tranche-2026-08-30.md)
now parses the private/output overlays through strict typed records, binds pass
assertions to the same case identity, normalizes every XPath path step, restores
temporary-tree shallow-copy and focus parity, rejects unadmitted `xml:space`
semantics explicitly, retains effective namespace bindings when copying an
isolated descendant, and rejects forward/cyclic globals as unsupported before
execution. The subsequent
[worker control-frame tranche](../Evidence/aspnet-worker-control-frame-serialization-2026-08-31.md)
serializes each bounded cancellation command across write and flush; a
byte-fragmenting 10,000-pair stress recovered all 20,000 frames exactly once,
while the live worker retained correlated cancellation and process reuse. The
native boundary now preflights every result/failure envelope against its
existing 1 MiB bound and publishes engine plus creation outcome atomically, as
recorded in the
[native ownership repair](../Evidence/native-outcome-bounds-and-atomic-creation-publication-2026-08-31.md).
The next adversarial tranche measures registry retention under
[AR-0017](../Architectural%20Reviews/AR-0017-native-handle-registry-retention-and-abandonment.md)
without prematurely selecting an aggregate quota. The
[first abandonment measurement](../Evidence/native-registry-abandonment-measurement-2026-08-31.md)
now covers 100,000 controls and bounded outcomes: all logical releases succeed,
but empty maps and the allocator retain material capacity/working set. The
[first live-use probe](../Evidence/native-registry-live-use-high-water-2026-08-31.md)
separately covers two overlapping ×4 `for-004` generations plus eight controls
and 128 delayed outcomes, reaching 144 handles and roughly 828 KiB above
baseline. Broader consumer requirements, rather than the 100,000-handle abuse
shape, must now drive comparison of count, byte, host-domain, and isolation
policies. The first
[ASP.NET registry-pressure matrix](../Evidence/aspnet-native-registry-pressure-calibration-2026-08-31.md)
adds read-only scalar observation and real `SafeHandle` retention across
concurrency 1/4/8/16/32, two/three generations, and 16–256 delayed valid
`for-004` results. All rows reclaimed logical handles immediately; process
memory remained noisy after release. The companion
[registry-burst trace](../Evidence/aspnet-native-registry-burst-pressure-2026-08-31.md)
now holds eight transforms at a real first-charge barrier, retains and validates
128 structured failures, and retains eight 900 KB semantic results. All native
ownership returns immediately to baseline, while decoded managed strings and
allocator/OS pages reclaim independently. The first
[candidate-policy replay](../Evidence/native-registry-candidate-policy-replay-2026-08-31.md)
keeps count ceilings as cheap abuse protection but shows exact aggregate
outcome bytes are needed to distinguish tiny diagnostic bursts from bounded
near-limit results. A
[32-promotion replacement trace](../Evidence/aspnet-native-sustained-generation-replacement-2026-08-31.md)
now keeps two old eight-engine generations live while new requests begin,
preserves both retired and promoted semantics, holds the predicted 25-engine
high-water after overlap fills, and records replacement/request p50/p95/p99.
The
[exhaustion-delivery comparison](../Evidence/native-registry-exhaustion-delivery-comparison-2026-08-31.md)
nominates a versioned tagged scalar result over a special structured sentinel:
it consumes no registry capacity, preserves the normal handle namespace, and
adds no foreign writable pointer. No tag or ABI behavior is admitted yet.
The first
[large prepared-engine trace](../Evidence/aspnet-native-large-prepared-engine-pressure-2026-08-31.md)
now retains three ×16 generations over a 5,000-item input. Its stable roughly
6 MB private-byte delta per engine for that shape—versus roughly 98 KiB in the
tiny probe—falsifies both raw input bytes and handle count as general memory
estimates. The subsequent
[prepared-engine estimator calibration](../Evidence/prepared-engine-retention-estimator-calibration-2026-08-31.md)
now composes private prepared-XDM and recursively owned compiled-state
capacities across the exact standards workload and seven generated source- and
stylesheet-heavy shapes. It covers 90.94%
through 99.97% of production-like live requested bytes without crossing the
ABI or claiming allocator-exact accounting. A
[30-second natural reclamation trace](../Evidence/aspnet-native-extended-reclamation-observation-2026-08-31.md)
then found more than 98% of the peak process-memory deltas gone before its first
post-disposal sample and process memory near baseline at ten seconds. It closes
the longer-window observation without inventing a portable half-life.
Candidate replay using the engine estimate is complete.
[ADR-0016](../ADR/ADR-0016-host-configured-native-registry-admission.md) now
accepts AR-0017's mechanism without inventing production thresholds: the host
must supply one immutable process-wide hybrid count/accounted-byte policy before
native handle admission. Versioned tagged scalar statuses report exhaustion
without consuming outcome capacity, no valid handle is evicted, and the limits
explicitly do not claim a whole-process memory cap. The
[native admission implementation](../Evidence/native-host-configured-registry-admission-2026-08-31.md)
now proves exact boundary/release behavior, concurrent last-slot admission, and
real managed tagged-status mapping through ABI version 3. Consumer-selected
production values and deployment guidance remain future host evidence; isolated
workers remain the hard-reclamation profile. The
[template-candidate fanout probe](../Evidence/template-candidate-fanout-and-cancellation-gap-2026-08-31.md)
has already confirmed exact `nodes × templates` growth, including 33,024
candidate checks in the largest local sweep. A distinct candidate domain now
charges and observes cancellation before every source and temporary-tree
candidate, closing Finding 7 while retaining an uncharged test oracle. The
[document-rooted match-path probe](../Evidence/document-rooted-match-path-reevaluation-2026-08-31.md)
also confirms exact `(items + 1)^2` reference visits through width 256.
[ADR-0013](../ADR/ADR-0013-invocation-owned-document-rooted-match-membership.md)
now admits the safe bounded invocation-owned membership after reducing that
width from 66,049 to 514 charged visits with differential, fallback, memory,
cancellation, and concurrent-ownership evidence. Broader indexes remain private
AR-0013 experiments. The
[global-frame clone probe](../Evidence/named-template-global-frame-cloning-2026-08-31.md)
confirmed material allocation and latency growth through 256 globals and eight
named calls. [ADR-0014](../ADR/ADR-0014-invocation-owned-copy-on-write-atomic-frames.md)
now admits private invocation-owned safe copy-on-write frames after eliminating
all eight complete clones in that workload, while preserving the complete-clone
oracle. The [prepared-XDM anatomy](../Evidence/prepared-xdm-byte-anatomy-2026-08-31.md)
also closes Finding 12's measurement half: node records dominate its repetitive
3,002-node shape, followed by resource identities and relationships, so no XDM
representation change is yet justified. Finding 6 is now closed through the
accepted policy, focused implementation evidence, and managed-boundary smoke.

Completing the 31-case QT3 mixed deep-equal group raised
`deep_equal_experiment.rs` to 1,054 lines and exposed independent atomic and
node responsibilities. The mandatory
[deep-equal source-unit cohesion review](../Evidence/deep-equal-source-unit-cohesion-review-2026-08-27.md)
is discharged by a private 516-line atomic parsing/value/comparison owner; the
711-line parent retains function recognition, node semantics, diagnostics, and
invocation work accounting. A separate 96-line boolean-composition owner now
adds only identity, `not(...)`, and boolean-equality projections above the
function. Representation-local tests need neither XDM nor execution control,
while the prior complete denominator and exact early-mismatch charges remain
conservation evidence.

ADR-0004's first mandatory 2,000-line review has fired. The retained
[runtime and compiler decomposition review](../Evidence/runtime-and-compiler-decomposition-review-2026-08-27.md)
moves general runtime contract tests out as a preparatory navigation checkpoint
and then separates private transform-set admission/result correlation from the
invocation engine. A second semantic extraction gives dynamic `xsl:value-of`
and its XPath adapters a one-way private owner. The runtime composition owner
also delegates invocation-local globals, variable frames, parameter binding,
source-context requirements, and temporary-tree preparation to a 207-line
context owner. Structured runtime failures and admitted-resource compilation
now have 92-line and 60-line one-way owners. The runtime core fell from 2,431 to
990 lines; its 304-line transform-set and 267-line value children call, but do
not own, the remaining invocation semantics.
The stylesheet compiler's first module dependency crossed its retained
decomposition trigger. The resulting
[stylesheet-module assembly review](../Evidence/stylesheet-module-assembly-decomposition-review-2026-08-28.md)
leaves a 1,135-line single-document compiler, a 775-line instruction compiler,
a 176-line module assembly owner, and an 80-line cross-template validation
owner. Module discovery/merge and whole-program validation now have explicit
one-way children without creating a public compiler graph or importing host
authority into compiler semantics. The remaining mutually recursive
sequence/template core is retained rather than split behind callbacks or a
broad context.
Template priority then fired the runtime/compiler review's named reopening
trigger. The resulting
[template selection and pattern decomposition review](../Evidence/template-priority-selection-and-pattern-decomposition-review-2026-08-29.md)
extracts a 111-line source-XDM selector for modes, pattern applicability,
retained-priority comparison, and source-order ties. The runtime core falls from
1,009 to 919 lines. A separate 117-line compiler child owns pattern
normalization and priority parsing; the 1,213-line parent retains cohesive
single-document/template assembly and integration tests. Both boundaries are
private and one-way; temporary-tree selection remains representation-local.
Subsequent output, static-context, and mode campaigns raised the single-document
compiler to 1,450 lines. The renewed
[stylesheet output compilation review](../Evidence/stylesheet-output-compilation-decomposition-review-2026-08-29.md)
extracts the independently coherent `xsl:output` lexical and settings policy
into a 138-line private owner. The 1,325-line parent continues to own
single-document/template composition, while output evolution no longer shares
that source unit and module assembly receives identical default settings.
Computed-attribute work then raised the instruction compiler above 1,000 lines
while template invocation already supplied an independent responsibility
trigger. The resulting
[instruction-compiler decomposition review](../Evidence/instruction-compiler-template-invocation-decomposition-review-2026-08-29.md)
extracts a 289-line owner for apply/call invocation, arguments, selections, and
modes. The 854-line parent retains sequence composition and delegates through
five narrow entry points; the separate 125-line computed-attribute owner retains
its static construction policy.
The completed choose campaign subsequently raised the instruction compiler to
2,050 lines and fired ADR-0004's mandatory review threshold. The
[conditional-expression decomposition](../Evidence/instruction-compiler-conditional-expression-decomposition-review-2026-09-03.md)
extracts balanced branch parsing, recursive conditional structure, typed-path
operands, and schema-prefix validation into a 255-line private child. The
1,813-line parent retains sequence-constructor and expression-family dispatch;
the extraction changes neither compiled plans nor corpus dispositions.
The shared XSLT30 corpus test unit then reached 1,117 lines while owning both
template-dispatch and XPath path-expression campaigns. Its
[test-owner decomposition review](../Evidence/xslt30-corpus-test-owner-decomposition-review-2026-08-29.md)
extracts a self-contained 342-line path corpus adapter and leaves an 866-line
template-dispatch owner. Both exercise the same production runtime directly;
neither depends on sibling test internals or changes corpus dispositions.

FastXSLT has accepted its staged-modern semantic direction and passes the
complete XSLT30 `template`, `path`, and `expr/for` test-set denominators. It also
executes the complete four-case QT3 `Axes002` group through a
stylesheet-independent XPath seam and the complete two-case XSLT30
`fn/deep-equal` denominator through attribute/comment node comparison.
The adjacent complete three-case QT3 `Axes001` group now executes through the
same seam with `child::*` selecting element children across namespaces while
excluding other child node kinds and retaining exact node-visit charging.
The complete four-case `Axes003` group adds typed `child::node()` selection of
text, element, comment, and processing-instruction children in document order;
the three adjacent groups now contribute eleven direct XPath passes.
The adjacent complete `Axes004` through `Axes006` groups add all eleven
abbreviated child-axis equivalents without a second evaluator path; explicit
and abbreviated syntax now contribute 22 direct XPath passes over the same
typed child steps.
Complete `Axes007` through `Axes011` add 15 explicit and abbreviated attribute
steps while preserving attributes outside the child sequence and namespace
declarations outside the attribute sequence. The direct axis seam now has 37
passes. Its [cohesion review](../Evidence/path-location-step-cohesion-review-2026-08-28.md)
retains one production location-path owner for this tranche, then requires a
  private invariant-test extraction and child-only terminology correction before
  another axis family is added. That
  [checkpoint](../Evidence/path-location-owner-decomposition-checkpoint-2026-08-28.md)
  is complete: the production owner is 527 lines, its 561-line private test
  child preserves the invariant suite, and all callers use location-path
  terminology.
QT3 `Axes012` now adds the root-only path `/` through a typed document-node
origin, including insignificant operand whitespace and an exact one-visit
charge from a non-document context. The direct location-path seam now has 38
passes without claiming general absolute paths.
Complete `Axes013` through `Axes019` add typed parent name tests, the parent
axis's element principal node kind, document-node-capable `parent::node()`, and
the equivalent `..` abbreviation. Their two absolute child paths extend the
document origin deliberately; the direct seam now has 45 passes.
The selected early self-axis tranche adds `Axes020`, `Axes021`, `Axes023`,
`Axes027`, and both `Axes030` cases. Typed self steps preserve element,
attribute, and text-node context identity; a typed child `text()` kind test
reaches the latter without widening other axes. The direct seam now has 51
passes. Numbering gaps remain unselected rather than being counted as
unsupported or passed.
Complete `Axes031` through `Axes033` add explicit descendant any-element,
unnamespaced named-element, and any-node tests. The typed step reuses the
existing charged document-order traversal without double charging during
filtering, and the direct seam now has 63 passes.
Complete `Axes034` through `Axes037` add typed descendant-or-self element,
named, and node tests. Nested input contexts retain one result per XDM identity
in document order while every repeated traversal remains charged; the direct
seam now has 73 passes.
Selected `Axes041` and `Axes043` close descendant-or-self evidence over
attribute and text contexts. Complete `Axes044` through `Axes049` verify
explicit and abbreviated absolute child element/name/node forms against the
same typed document origin, including seven mixed top-level nodes. No new
engine path was required; the direct seam now has 88 passes.
Complete `Axes055` through `Axes061` compose the typed document origin with
self, descendant, and descendant-or-self steps. The 19 cases required no
absolute-only engine implementation and distinguish 58 `TopMany` descendants
from 59 descendant-or-self nodes; the direct seam now has 107 passes.
Complete `Axes062` through `Axes067` verify that explicit `//child::` and
abbreviated `//` child forms lower to the same typed descendant origin and
step. One traversal remains charged once per visited node, and the direct seam
now has 125 passes. Leading attribute steps remain deliberately unsupported
pending correct descendant-or-self-element expansion.
Complete `Axes068` through `Axes071` now perform that expansion: the absolute
leading descendant origin resets to the document, visits descendant element
contexts, then applies explicit or abbreviated attribute steps while excluding
namespace declarations. Traversed nodes and inspected attributes are charged
separately; the direct seam now has 137 passes.
Complete `Axes072` and `Axes073` make that leading expansion axis-aware for
self steps. `//self::node()` retains the document node while `//self::*`
applies the element-principal node kind after the same charged expansion; the
direct seam now has 141 passes.
Complete `Axes074` through `Axes079` lower one isolated internal `//` to the
existing typed descendant-or-self node step before explicit or abbreviated
child steps. Overlapping nested contexts retain unique document-ordered
results without erasing attributable repeated traversal work; the direct seam
now has 164 passes.
Complete `Axes080` through `Axes083` plus `Axes084-1` through `Axes084-4`
compose the same internal expansion with explicit and abbreviated attribute
steps. No production widening was required; the direct seam now has 181 passes.
`Axes084-5` remains visibly outside selection because it introduces
`normalize-space()` predicate semantics rather than another axis form.
Before that next semantic addition, the 992-line path invariant-test owner was
split at its demonstrated syntax-versus-evaluation seam under ADR-0004. The
result is a 112-line syntax/diagnostic owner and an 891-line
navigation/predicate/work-accounting owner; production semantics are unchanged.
`Axes084-5` now executes as the deliberately narrow next slice: a final text
step can carry a typed zero-argument `normalize-space()` effective-boolean-value
predicate. The pinned 37,212-byte Northwind source is admitted under explicit
64 KiB and 16,384-event limits, returns the expected 827 nodes, and brings the
direct seam to 182 passes.
Seven unchanged AxisStep negative cases now distinguish unambiguous static path
syntax errors from syntactically valid but unsupported forms. Trailing empty
steps, a bare descendant separator, an unknown axis, and incomplete QNames
retain `XPST0003` plus their source locations, bringing the AxisStep seam to
189 passes without requiring a dynamic source document.
The complete five-case QT3 `fn-deep-equalint2args` group also executes through
source-free checked `xs:int` comparison, and the adjacent five-case
`fn-deep-equalintg2args` group extends that evidence to the checked `i128`
subset of arbitrary-precision `xs:integer`. The complete five-case
`fn-deep-equaldec2args` group adds exact normalized decimal comparison within a
checked `i128` coefficient boundary, without using binary floating point. The
complete five-case `fn-deep-equallng2args` group now also passes through a
range-checked signed 64-bit `xs:long` constructor. The adjacent five-case
`fn-deep-equalusht2args` group adds a range-checked `xs:unsignedShort`
constructor with explicit lower- and upper-bound rejection controls. The
complete five-case `fn-deep-equalnint2args` group adds a checked-`i128`
`xs:negativeInteger` subset with executable rejection of zero and positive
values. Its adjacent five-case `fn-deep-equalpint2args` group adds the mirrored
checked-`i128` `xs:positiveInteger` subset with zero and negative values
rejected. The next five-case `fn-deep-equalulng2args` group adds an exact
checked-`u64` `xs:unsignedLong` path with controls at both real value-space
boundaries. The complete five-case `fn-deep-equalnpi2args` group adds a
checked-`i128` `xs:nonPositiveInteger` subset whose inclusive zero boundary is
verified independently. The adjacent five-case `fn-deep-equalnni2args` group
adds its checked-`i128` `xs:nonNegativeInteger` mirror, also with an independently
verified inclusive zero boundary. The complete five-case
`fn-deep-equalsht2args` group adds an exact checked-`i16` `xs:short` path with
controls immediately inside and outside both fixed boundaries. The complete
31-case `fn-deep-equal-mix-args-*` group now adds ordered integer/string sequences,
singleton parentheses, and empty-sequence forms through a depth-aware argument
splitter, plus admitted URI/string and exact integer/decimal comparisons; the
same tranche now covers numeric promotion through float/double, infinities,
paired NaN, and the four boolean lexical forms plus boolean functions. Its
final cases retain validated date/time type identity against equal lexical
strings, while broader calendar lexical forms remain outside the slice.
The adjacent complete five-case `fn-deep-equalflt2args` and five-case
`fn-deep-equaldbl2args` groups now independently verify equal and unequal finite
boundary fixtures through retained IEEE `f32`/`f64` values and exact singleton
sequence work charges.
The first three `K-SeqDeepEqualFunc` cases now preserve `XPST0017` as an
XPath-owned static arity identity while stylesheet compilation translates it
to private `FXXP0005 / invalid`; valid three-argument collation calls remain
distinctly unsupported.
The independently named K2 cases 8 and 9 now exercise the same structured arity
identity without admitting their surrounding clock- and XQuery-shaped cases.
The next explicit K-family tranche executes case 6 under the exact standard
codepoint collation URI and cases 8 through 11 through paired float/double NaN
semantics. Unknown and empty collation arguments remain unsupported rather than
being optimized around.
Cases 64 and 65 now add the standard HTML ASCII case-insensitive collation for
ordered atomic string sequences, retaining exact early-exit charging while
leaving host-defined, locale, Unicode-folding, and node-collation behavior open.
The following nine-case tranche executes case 7, cases 12 through 16, and cases
18 through 20 through one shared compiler/runtime/direct boolean-composition
owner. Corpus pressure also corrected quoted integer/decimal constructors and
three-item parenthesized sequence parsing without moving those responsibilities
out of the atomic child.
The next 18-case K-family tranche executes cases 25 through 31 and 36 through
46 through those same owners. It verifies ordered equality, first/second/third
early mismatch, string placement, empty-item flattening, and unequal-length
short-circuit work without admitting QName, binary, or `index-of` semantics.
Cases 47 through 51 now complete the adjacent direct unequal-length forms with
one length-decision charge apiece. Cases 52 through 55 discharge the separate
range/`reverse` checkpoint through bounded literal folding with a 1,024-item
retained-result ceiling; dynamic range execution remains unclaimed.
The two remaining QName-shaped K-family cases, 17 and 21, now retain and compare
expanded-name identity through the atomic owner, including prefix-insensitive
equality controls. Cases 22 through 24 then add decoded hex/base64 binary values
with lexical rejection controls and exact early exits. `index-of` remains the
only subsequent gap in this K-family range; cases 32 through 35 now close it
through explicitly compile-time-folded literal searches, without claiming a
dynamic runtime function.
`for-004` closes its family through
bound-variable attribute paths, checked exact-decimal multiplication and
aggregation, and the single required two-decimal formatting picture. The
complete 28-case `expr/data-manipulation` denominator now passes through native
execution.

Pinned XSLT30 `conflict-resolution-0101` now starts standards-driven
`insn/apply-templates` evidence. Built-in document dispatch reaches the exact
`doc` rule, then the exact `foo` pattern beats element-wildcard and any-node
fallbacks. Its previously exposed `xsl:text` gap now lowers explicit character
content to the existing owned text instruction with whitespace-preservation and
invalid-element controls. This is one selected case, not a denominator claim.
Paired XSLT 3.0 cases `conflict-resolution-0102c` and `0104c` correct the next
selector boundary: element-wildcard and any-node patterns tie at the same
default priority, so reversing their stylesheet order reverses the selected
last-declared rule. Exact-name and path priority bands remain distinct. These
three selected cases still do not establish the complete 50-case denominator.
`conflict-resolution-0106` then retains bounded signed-integer explicit
priorities in compiled template state: priority `10` selects the `doc` rule over
the otherwise matching priority-`1` node test. Its selected attribute remains
outside the child-axis `node()` pattern and reaches the corrected built-in
attribute string-value rule. Fractional/general priority remains unclaimed.
`conflict-resolution-0107` conserves the compiled higher default priority of a
non-simple `doc/foo` pattern. Paired XSLT 3.0 cases `0108c` and `0110c` add only
the exact unnamespaced `foo[@test]` presence pattern and prove last-declared
selection against the equal-priority path in both source orders. Each inspected
attribute is charged; general predicate expressions remain unsupported.
`conflict-resolution-0112` adds only exact `//*`: compilation specializes it as
element applicability with retained non-simple priority, so it beats exact-name
and node-test rules without general descendant navigation in the dispatch hot
path. Named and arbitrary descendant patterns remain unsupported.
`conflict-resolution-0201` adds the exact unnamespaced
`element[@attribute='literal']` form. It retains the literal and non-simple
priority in compiled state, charges inspected attributes, and distinguishes two
sibling elements by attribute string value. Other comparisons and general
predicate expressions remain unsupported. Its case-local environment also
proves the harness does not require every upstream source to use a shared named
environment.
`conflict-resolution-0401c` resolves prefixed exact-name and namespace-wildcard
patterns against retained stylesheet bindings. Both rules carry explicit
priority `5`, and XSLT 3.0 source-order selection chooses the later wildcard.
`conflict-resolution-1701` subsequently widens the retained priority domain to
exact millionths and adds the complementary `*:NCName` pattern. Namespace and
local-name wildcards now retain their implicit quarter-step priority without
floating-point comparison, while bounded explicit priorities retain up to six
decimal places. More precise values and the XSLT 2.0 recover/error variants
remain outside these tranches.
`conflict-resolution-1801` maps the exact `element()` kind test to the same
typed any-element applicability and `-0.5` priority already used by `*`.
Its exact `name(.)` value operation is admitted only for unnamespaced context
nodes; namespaced lexical-QName reconstruction remains explicitly unsupported.
`conflict-resolution-1601` retains the direct single-root execution path until
root rules actually compete, then migrates them into typed document-node
selection with `/` default priority `-0.5`, bounded explicit priorities, and
declaration-order tie-breaking. Cross-module root conflicts remain unsupported.
`conflict-resolution-1602` and `1603` add typed exact-name and wildcard
`document-node(element(...))` patterns. Their default priorities remain exact
at `0` and `-0.5`; document-element inspection is locally charged, and
distinct-priority duplicate rules are admitted to compete within one module.
`conflict-resolution-1201` retains parameter-free `xsl:next-match` and the
current compiled-template index in its private runtime frame. It walks the
strictly lower-ranked applicable rules `5 → 4 → 3 → 2`, then invokes the
built-in fallback. Equal-rank semantics are admitted separately; cross-module
precedence stays out.
`conflict-resolution-1202c` then applies the selected XSLT 3.0 use-last rule to
two equal-priority wildcard templates and retains declaration rank across
`xsl:next-match`, producing `(3b)(3a)`. Its `xsl:fallback` content stays inert
because next-match is supported. Older recover/error profiles and mode controls
remain out.
`conflict-resolution-1205` carries one non-tunnel atomic parameter from
`xsl:apply-templates` through that entire next-match chain. Compiled argument
expressions remain separate from invocation-local values. Its final variable
attribute value template also admits a separately retained and charged
unnamespaced result attribute without general AVT or namespace widening.
`conflict-resolution-0601` retains a global integer default for invocation-time
pattern evaluation. Its typed `*[@id=$p]` predicate performs the required
untyped-attribute-to-integer comparison, while element `xsl:copy` preserves the
source expanded name and namespace declarations, constructs one leading static
attribute, and executes its body without implicitly copying source attributes
or children. General pattern expressions and general attribute construction
remain out.
`conflict-resolution-0501` and `0502` then normalize two equivalent standard
surface forms—one using `current()`, one a quantified range variable—into one
typed same-named-child pattern operation. Charged runtime inspection stays in
the admitted unnamespaced lexical-name domain; namespaced candidates fail
explicitly rather than being approximated through expanded-name equality.
`conflict-resolution-0503` preserves the different focus semantics of a
multi-step current pattern in a separate typed same-named-parent relation. The
final candidate supplies `current()` while its parent supplies the predicate
context; parent inspection is charged and remains in the same explicit
unnamespaced lexical-name domain.
`conflict-resolution-1501` adds a filtered position to that relation: the
same-named parent must be second among element siblings sharing the final
candidate's lexical name. The runtime charges parent and sibling inspection.
Its two upstream XPath assertions are conserved verbatim and interpreted by a
bounded case-specific XML oracle, without claiming general assertion XPath.
`conflict-resolution-1101` retains an attribute-free local temporary tree in
the invocation variable frame and applies templates through its conceptual
document root. Built-in document/element descent preserves the supplied
non-tunnel parameter, overriding the matched template's compiled integer
default. Neither the temporary tree nor parameter value enters reusable state.
`conflict-resolution-1102` retains temporary document and element focus while
dispatching in mode `m`. Its `xsl:apply-imports` instruction has no imported
module or lower-precedence user rule, so it falls through to the built-in
document rule while preserving the current mode and non-tunnel parameter.
Import precedence and lower-precedence user-template selection remain outside
the admitted slice.
`apply-templates-001` and `002` then exercise the opposite boundary: a
statically known integer range supplies the `xsl:for-each` focus, so both
default apply-templates selection and explicit `select="."` violate the
required node-sequence type. Compilation reports located, structured
`XTTE0510` failures without claiming general `xsl:for-each` execution or broad
static type inference.
`conflict-resolution-1001` specializes the two relative
`planche/{section}/*[@type=$type]` expressions into typed parent steps plus a
wildcard attribute-equality filter. The empty global string default produces
the upstream empty-section result. A supplemental `type="enfant"` invocation
forces both matching rules and bounded current-element `xsl:copy-of` execution,
retaining selected attributes, descendants, and text without runtime XPath
reparsing.
`conflict-resolution-0901` then conserves the existing typed path machinery
across template selection: `//b` supplies six candidates in document order,
while retained `doc/a/b` and `doc/z/b` patterns distinguish their parent chains
and produce `111222`. No lexical pattern parsing enters the dispatch loop.
`conflict-resolution-0701` now resolves an inherited template-local
`xpath-default-namespace` into expanded names for simple unprefixed element
patterns and child selections. The prefixed child rule proves equivalent
expanded-name matching. Multi-step default-namespaced paths fail explicitly
rather than falling through to local-name-only semantics.
`conflict-resolution-0702` places the same static-context control on a literal
result element using the XSLT namespace. Compilation applies it to the
descendant selection without manufacturing a result attribute, while the
stylesheet's retained `u` binding remains serialized on `out`. Narrow
unnamespaced literal attributes are admitted separately by `1205`; other XSLT
control attributes remain unsupported.
`conflict-resolution-0703` propagates a stylesheet-wide
`xpath-default-namespace` into simple element patterns and child selection,
while preserving the XSLT rule that unprefixed attribute names remain in no
namespace. The qualified element path and unqualified `@test` dispatch produce
the pinned `foo"true"` result without broadening general path support.
`conflict-resolution-0801` retains the active mode across a named-template call
and resolves `mode="#current"` before redispatching the document node. Distinct
mode-qualified `/` rules now participate in normal typed template selection;
the namespace-insensitive root path remains valid under a stylesheet-wide
default element namespace. The result is the pinned `[a][b]` sequence.
`conflict-resolution-0802` lets one template participate in modes `a`, `b`, and
`#default`; explicit default dispatch maps to the unnamed mode, and `#current`
preserves all three contexts through the shared named template. Its `//bar`
selection is a typed, default-namespaced descendant-name operation that charges
each inspected node and preserves document order. It does not widen general
default-namespaced path support.
The complete pinned apply-templates test set is now conserved as an ordered
50-case denominator with 50 principal stylesheets, one secondary stylesheet,
41 XML assertions, eight error assertions, and one compound assertion.
Forty-nine cases have explicit passing overrides. The sole remainder,
`conflict-resolution-1402`, retains its native `schema_aware` dependency and is
excluded by ADR-0007 rather than mislabeled as an engine failure. The adjacent
16-case include denominator likewise retains 14 passes and explicitly excludes
`include-0101` and `include-0102`: their immutable upstream bytes require
external DTD/entity processing and DTD-typed ID selection, respectively, while
the current XML boundary denies those capabilities. Both denominators now have
complete explicit dispositions with no default not-run cases. This corrects
the earlier provisional counts without turning inventory into a conformance
percentage or preprocessing upstream bytes in the harness.

The next coherent mode tranche executes `mode-0101` through `mode-0104`,
`mode-0201` through `mode-0701`, and `mode-0901` through `mode-1001`. Explicit
and omitted mode selection remain
isolated, built-in rules preserve an active explicit mode, nested omitted mode
returns deliberately to the unnamed mode, and comment, processing-instruction,
node, and attribute selections dispatch through typed moded rules. The compiler
now correctly admits one template carrying both `name` and `match` into both
indexes instead of dropping one identity. Equivalent namespace prefixes and
admitted NCName punctuation also retain exact expanded mode identity. The
complete mode denominator now records 16 passes and 153 visible default not-run
cases.

The following initial-mode and `#all` tranche adds `mode-1101` through
`mode-1104` and `mode-1201` through `mode-1204`. Suite-native initial-mode
entry now flows through the shared transform request, one exact quoted-string
parameter shape reaches invocation-local global override, and `#all` competes
by ordinary priority before `xsl:next-match` retains the active explicit mode.
The complete mode denominator then reaches 25 passes and 144 visible default
not-run cases through `mode-1105`. Its exact inline `/doc` environment now
selects the named document element as invocation-owned initial context before
mode dispatch. Ordinary initial-mode document entry remains unchanged, and the
private exact-name entry does not select a public context-selector API.

The native multiple-match mode tranche adds `mode-0801a` through `mode-0801c`.
The XSLT 1.0/2.0 recover dependency and XSLT 3.0 default both use the private
later-equal-rank path, while the suite's error dependency reaches structured
`XTDE0540` and satisfies its native `XTRE0540` pattern. The common stylesheet's
document-rooted `/sss//*` pattern reuses the typed location-path evaluator from
the document node rather than acquiring a parallel pattern engine. The mode
denominator now records 28 passes and 141 visible default not-run cases.

The adjacent declaration-validation tranche adds `mode-0803`, `mode-0805`, and
`mode-0806`. Warning-disabled `no`, `false`, and `0` values validate without
retained runtime state, while invalid mixed-case `Yes` reports native static
error `XTSE0020`. Warning-enabled cases remain visibly not run because a
successful result alone cannot establish the required warning event. The mode
denominator now records 31 passes and 138 visible default not-run cases.

The next numerically adjacent case, `mode-1301`, was deliberately not selected
as a narrow traversal patch. Its `xsl:strip-space elements="*"` declaration
creates stylesheet-dependent source semantics over an immutable prepared XDM
document that must remain reusable by other stylesheets. Parser-time stripping
would violate AR-0008, prepared-tree mutation would violate AR-0009, and
filtering only built-in template traversal would let XPath and other semantic
consumers observe a different tree. [AR-0016](../Architectural%20Reviews/AR-0016-stylesheet-dependent-source-views-and-whitespace-stripping.md)
therefore incubated a complete safe derived-document reference and a private
immutable visibility-view experiment. At that checkpoint the mode denominator
recorded 31 passes and 138 visible default not-run cases; later evidence below
records the completed reference slice.

Independent of that view decision, the static mode-validation tranche selects
`mode-1444` and `mode-1447`. The dedicated declaration compiler validates
invalid mixed-case `warning-on-no-match="Yes"` and `typed="No"` as native
`XTSE0020` before unrelated unsupported `on-no-match`, warning-delivery, or
schema-aware behavior is considered. The mode denominator now records 33
passes and 136 visible default not-run cases; at that checkpoint `mode-1301`
remained among the latter.
[Evidence](../Evidence/xslt30-mode-static-boolean-validation-tranche-2026-08-30.md)

The following static visibility tranche selects `mode-1507` through
`mode-1509`. The same declaration owner reports native `XTSE0020` when an
unnamed mode is public or final and when a named mode is abstract, before the
cases' unrelated unsupported template expressions are compiled. Otherwise
valid visibility semantics remain unsupported. The mode denominator advances
to 36 passes and 133 visible default not-run cases.
[Evidence](../Evidence/xslt30-mode-visibility-validation-tranche-2026-08-30.md)

The complete native streaming-dependent mode subset is now explicitly
classified rather than left inside the generic not-run remainder. All 26 cases
whose catalog metadata declares `feature="streaming"` are excluded by the
current ADR-0007 profile and remain unexecuted. The executable inventory checks
the native dependency for every identity, including `mode-0014`, whose static
error does not become a pass while its required feature is excluded. The mode
denominator therefore records 36 passes, 26 profile exclusions, and 107 visible
default not-run cases.
[Evidence](../Evidence/xslt30-mode-streaming-profile-exclusions-2026-08-30.md)

The same metadata-first accounting now identifies all 18 mode cases whose
native test supplies a principal package artifact. ADR-0007 excludes packages,
so each is explicitly profile-excluded and unexecuted; the inventory verifies
the `<test><package>` shape for every identity rather than relying on the
`mode-17` numbering. The mode denominator now records 36 passes, 44 profile
exclusions, and 89 visible default not-run cases.
[Evidence](../Evidence/xslt30-mode-package-profile-exclusions-2026-08-30.md)

The independent same-precedence declaration case `mode-1502` now reports
native static error `XTSE0545`. A private compiler prepass resolves both named
mode declarations and detects their conflicting explicit `on-no-match` values
before runtime mode policy or the unrelated template body is considered. The
slice does not merge includes/imports or retain executable `on-no-match`
behavior. The mode denominator advances to 37 passes, 44 profile exclusions,
and 88 visible default not-run cases.
[Evidence](../Evidence/xslt30-mode-same-precedence-conflict-2026-08-30.md)

The adjacent positive case `mode-1501` now executes its native `#all` and
`#current` recursion from initial mode `baz`. The private `xsl:copy` path copies
the exercised document, element, processing-instruction, and text contexts
through existing bounded result construction; the mode-specific `foo` rule
still replaces only that element. Attribute and comment copy contexts remain
outside this slice. The mode denominator advances to 38 passes, 44 profile
exclusions, and 87 visible default not-run cases.
[Evidence](../Evidence/xslt30-mode-all-current-node-copy-2026-08-30.md)

The independent structural case `mode-1108` now reports native static error
`XTSE0260` for a nonempty `xsl:mode`. The stylesheet-wide declaration prepass
detects the meaningful child before source-ordered compilation reaches the
case's unrelated accumulator declarations, preserving a concrete invalid
outcome instead of degrading into unsupported accumulator behavior. The mode
denominator advances to 39 passes, 44 profile exclusions, and 86 visible
default not-run cases.
[Evidence](../Evidence/xslt30-mode-nonempty-declaration-2026-08-30.md)

The later same-precedence composition case `mode-1904` now reports native
static error `XTSE0545` when two declarations of expanded mode `X` specify
different explicit visibility values. The declaration prepass owns the
conflict independently of executable visibility semantics; include/import
composition and higher-precedence override behavior remain unselected. The
mode denominator advances to 40 passes, 44 profile exclusions, and 85 visible
default not-run cases.
[Evidence](../Evidence/xslt30-mode-same-precedence-visibility-conflict-2026-08-30.md)

AR-0016's first safe reference slice now executes unchanged `mode-1301`.
Compilation retains only exact `xsl:strip-space elements="*"`; each stripping
invocation clones the complete prepared document and filters cloned element
child relationships while retaining original visible `NodeId`, locations, and
payload. XPath, template selection, built-ins, values, and copying all receive
that one effective document. Controls prove budget/cancellation behavior,
indirect string-value changes, and sequential preserving/stripping reuse of one
prepared source without mutation. The optimized visibility view, broader
parity/concurrency controls, and measurements remain open. The mode denominator
advances to 41 passes, 44 profile exclusions, and 84 visible default not-run
cases. [Evidence](../Evidence/ar-0016-source-access-inventory-and-safe-reference-2026-08-30.md)

The next AR-0016 slice replaced the executable full clone with a private,
invocation-owned visibility view while retaining the clone as a differential
oracle. Shared immutable node storage preserves prepared identity and the view
retains only affected child sequences. Differential testing found and closed a
direct physical-child read in string-value recursion; full runtime results,
unchanged `mode-1301`, and 100 concurrent preserving/stripping repetitions now
agree. A preliminary 500-item release probe measured 4.86-times lower median
invocation time and about 141-times less attributable additional capacity than
the clone. Follow-up controls now prove effective child positions, focus size,
source element/text copying, and concurrent old/new stylesheet-generation
overlap. A descendant `node()` control also proves stripped text is absent
before focus positions and size are assigned; no sibling axis is currently
implemented. The later five-shape decision matrix measured 2.74x to 8.35x
lower total time and 2.53x to 8.87x higher four-thread throughput versus the
clone. On a 6,003-node source, allocator-requested peak bytes fell from
3,214,912 to 32,408. ADR-0012 now accepts the invocation-owned view for exact
strip-all semantics without admitting a cache, public abstraction, or broader
whitespace declarations.
[Prototype evidence](../Evidence/ar-0016-visibility-view-prototype-2026-08-30.md)
[Decision evidence](../Evidence/ar-0016-decision-measurement-matrix-2026-08-30.md)

The adjacent native `mode-1439` case now retains a named `typed="yes"` mode
requirement and reports dynamic `XTTE3100` with stylesheet provenance when its
untyped suite source enters that mode. The error occurs before the case's
otherwise unsupported shallow-copy fallback, so no schema type or partial
`on-no-match` behavior is fabricated. The adapter also admits the suite's
explicit 9,001-byte external source into the same sealed memory snapshot. The
streaming-dependent sibling `mode-1438` remains profile-excluded. The mode
denominator advances to 42 passes, 44 profile exclusions, and 83 visible
default not-run cases.
[Evidence](../Evidence/xslt30-mode-1439-typed-untyped-error-2026-08-30.md)

The unchanged `mode-1431` case now retains the unnamed mode's
`on-no-match="fail"` policy in compiled state and reports dynamic `XTDE0555`
when dispatch reaches the first unmatched source text node. The policy is
checked only after normal template selection fails, so matching document and
element templates remain authoritative. The diagnostic retains request identity
and the mode-declaration location. That slice left other `on-no-match`
policies unsupported; bounded shallow-copy is admitted separately below while
the remaining policies stay open. The mode denominator advanced to 43 passes, 44 profile
exclusions, and 82 visible default not-run cases.
[Evidence](../Evidence/xslt30-mode-1431-fail-on-no-match-error-2026-08-30.md)

The companion unchanged `mode-1423` case now supplies the positive control for
that policy: explicit document, element, and text rules cover every visited
node, so fail-on-no-match remains dormant and the complete result is produced.
Its roughly 9 KiB expected document is admitted through a case-specific 16 KiB
serialization ceiling rather than weakening the engine's bounded-output
contract. The mode denominator advances to 44 passes, 44 profile exclusions,
and 81 visible default not-run cases.
[Evidence](../Evidence/xslt30-mode-1423-fail-on-no-match-success-control-2026-08-30.md)

The paired unchanged `mode-1445` and `mode-1446` cases now execute a bounded
`on-no-match="shallow-copy"` policy while proving the whitespace-padded
`typed=" false "` and numeric `typed="0"` forms remain inert. Normal template
selection still runs first; the built-in policy copies the document traversal,
elements, namespaces, attributes, text, and processing instructions required by
the native source. At this checkpoint comment results, standalone attribute
results, and attribute-template interception remained explicit structured
unsupported outcomes instead of becoming partial output; later slices admit
those paths independently. The mode denominator advanced to 46 passes, 44
profile exclusions, and 79 visible default not-run cases.
[Evidence](../Evidence/xslt30-mode-1445-1446-bounded-shallow-copy-2026-08-30.md)

The coherent `mode-1601` through `mode-1606` tranche now resolves inherited
`default-mode` on literal result elements and `xsl:template`. Template-local
defaults determine both the rule's implicit mode and descendant unmoded
instructions; `#unnamed` retains the unnamed representation, qualified values
retain expanded identity, and explicit instruction modes still win. All six
unchanged XML assertions pass, advancing the mode denominator to 52 passes, 44
profile exclusions, and 73 visible default not-run cases.
[Evidence](../Evidence/xslt30-mode-1601-1606-default-mode-inheritance-2026-08-31.md)

The following `mode-1610` through `mode-1615` tranche extends the same static
context through `xsl:if`, explicit instruction-level shadowing, and the
stylesheet's default initial mode. The boolean evaluator now admits a bounded
location-path effective-boolean-value operation for the native `//@test`
guards; the compiler retains `#unnamed` alongside named template modes and
resolves the stylesheet default once into the compiled program. All six
unchanged XML assertions pass. The mode denominator advances to 58 passes, 44
profile exclusions, and 67 visible default not-run cases.
[Evidence](../Evidence/xslt30-mode-1610-1615-default-mode-scope-and-entry-2026-08-31.md)

The adjacent `mode-1607` through `mode-1609` trio now carries inherited
`default-mode` through an ordinary source-node `xsl:for-each`. A private
source-node iteration instruction evaluates its selection under the existing
XPath budget, establishes node/position/size focus for every selected item,
and preserves the current mode and template identity for its sequence
constructor. Named, unnamed, and qualified defaults all pass unchanged. The
mode denominator advances to 61 passes, 44 profile exclusions, and 64 visible
default not-run cases.
[Evidence](../Evidence/xslt30-mode-1607-1609-source-for-each-default-mode-2026-08-31.md)

The unchanged `mode-1616` and `mode-1617` cases now preserve each stylesheet
module's own `default-mode` static context across `xsl:include`. The suite
adapter admits the catalog's principal and secondary stylesheet resources into
one sealed snapshot; the included module compiles before composition, so its
unmoded template retains mode `a` without inheriting the principal module's
root settings. Both native assertions pass. The mode denominator advances to
63 passes, 44 profile exclusions, and 62 visible default not-run cases.
[Evidence](../Evidence/xslt30-mode-1616-1617-included-default-mode-2026-08-31.md)

The final adjacent default-mode pair, `mode-1618` and `mode-1619`, now carries
nested static-context choices across element-to-attribute focus. The template
pattern compiler admits standard wildcard attribute tests (`@*` and
`attribute()`), while named attribute selection reuses the existing controlled
source navigation. Template-level mode `a` and instruction-level override mode
`b` each reach the intended wildcard attribute rule. Both unchanged assertions
pass. The mode denominator advances to 65 passes, 44 profile exclusions, and 60
visible default not-run cases.
[Evidence](../Evidence/xslt30-mode-1618-1619-attribute-focus-default-mode-2026-08-31.md)

The unchanged `mode-1411` and `mode-1415` cases now exercise bounded
`on-no-match="shallow-copy"` across the complete external 9 KB mode source.
The first relies entirely on named-mode built-in copying with `typed="no"`;
the second composes unnamed shallow-copy with explicit upper-case and empty
text/element overrides. Both complete file-backed XML results pass under the
existing 16 KiB result ceiling. Attribute-template interception in adjacent
`mode-1413` remains visibly separate. The mode denominator advances to 70
passes, 45 profile exclusions, and 54 visible default not-run cases.
[Evidence](../Evidence/xslt30-mode-1411-1415-large-shallow-copy-2026-08-31.md)

The next non-streaming mode-policy tranche executes unchanged `mode-1417`,
`mode-1419`, and `mode-1421`. Shallow-skip traverses unmatched document and
element children while dropping unmatched text, attributes, comments, and
processing instructions; it participates after `xsl:next-match` and composes
with a separate shallow-copy mode. A temporary-tree control preserves the same
skip behavior, and one-step union alternatives retain exact-name rather than
path default priority. The mode denominator advances to 73 passes, 45 profile
exclusions, and 51 visible default not-run cases. `mode-1413` remains separate
result-attribute pressure rather than receiving an arithmetic-only shortcut.
[Evidence](../Evidence/xslt30-mode-shallow-skip-2026-08-31.md)

The unchanged `mode-1433`, `mode-1434`, and `mode-1435` cases now conserve
named, explicitly unnamed, and namespace-qualified stylesheet default modes
over the complete native source. Default-mode lexical whitespace is normalized,
explicit `mode="#unnamed"` dispatches through the unnamed mode, and independently
prefixed QNames compare by expanded identity. Their shared
`v | chapter/text()` rule is lowered only after proving its alternatives
disjoint, preserving exact-name and path default priorities independently;
potentially overlapping unions remain unsupported. The mode denominator
advances to 76 passes, 45 profile exclusions, and 48 visible default not-run
cases.
[Evidence](../Evidence/xslt30-mode-default-mode-variants-2026-08-31.md)

The output denominator now executes unchanged `output-0186` and `output-0187`
through native serialization error `SEPM0009`. Both string and byte entry
points reject internally inconsistent XML serialization parameters before
emitting bytes. The non-1.0 version in `output-0187` is retained only on this
deterministically failing parameter path; general XML 1.1 serialization remains
explicitly unsupported. The output denominator advances to 74 passes and 158
visible default not-run cases.
[Evidence](../Evidence/xslt30-output-sepm0009-2026-08-31.md)

The adjacent unchanged `output-0182` and `output-0183` cases now report native
serialization error `SEPM0004`. Standalone and external-document-type settings
require exactly one top-level result element; FastXSLT checks the semantic
result before emitting declarations, document types, or bytes while preserving
fragment serialization when neither property is requested. The output
denominator advances to 76 passes and 156 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-sepm0004-2026-09-01.md)

The unchanged `output-0185` case now retains its requested output encoding
through stylesheet compilation and reports native serialization error
`SESU0007` only when the unavailable encoding is materialized. This keeps
declarative output metadata separate from serializer implementation
capability, preserves the bounded ISO-8859-1 byte lane, and does not claim a
general encoding provider. The output denominator advances to 77 passes and
155 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-sesu0007-2026-09-01.md)

The same native encoding boundary now admits unchanged `output-0178` and
`output-0180` through their explicit error-or-recovery alternatives. XHTML and
XML requests for `XXX-xx` report `SESU0007`; FastXSLT does not pretend to
recover with a different encoding. The output denominator advances to 84
passes and 148 visible default not-run cases.

The unchanged `output-0189`, `output-0190`, `output-0192`, and `output-0193`
cases apply the same ownership boundary to unavailable normalization forms.
XML, XHTML, and text output retain the requested form through compilation and
report native serialization error `SESU0011` before emitting result content;
`output-0193` admits that code through its native error alternative. Unicode
normalization and the independently unsupported HTML output case remain
unadmitted. The output denominator advances to 81 passes and 151 visible
default not-run cases.
[Evidence](../Evidence/xslt30-output-sesu0011-2026-09-01.md)

The intervening unchanged `output-0188` case now retains
`undeclare-prefixes=yes` as compiled serialization metadata and reports native
`SEPM0010` when combined with XML output version 1.0. The check is shared by
the bounded string and byte entry points without implying XML 1.1 support. The
output denominator advances to 82 passes and 150 visible default not-run
cases.
[Evidence](../Evidence/xslt30-output-sepm0010-2026-09-01.md)

Eight unchanged invalid-boolean cases, `output-0280` through `output-0283a`,
now extend the compiler-owned `XTSE0020` tranche across indent, declaration
omission, standalone, and prefix undeclaration. Each preserves its native
XSLT-version-specific lexical rules, invalid category, and stylesheet
location. Adjacent `output-0284` validates its XML public identifier before an
unsupported XML 1.1 request can obscure the same native static-error
alternative. The output denominator advances to 93 passes and 139 visible
default not-run cases.
[Evidence](../Evidence/xslt30-output-invalid-boolean-tranche-2026-09-01.md)

The unchanged `output-0285` case now treats `xml:space` on `xsl:output` as an
ignored XML-namespaced control instead of an unsupported unqualified output
property. The allowance is local to that one expanded attribute name and does
not weaken other instruction vocabularies. The output denominator advances to
94 passes and 138 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-xml-space-control-2026-09-01.md)

The unchanged `output-0501` case now reports native `XTSE0010` for a missing
required character-map name before reaching the explicitly unsupported
character-map execution boundary. Named maps remain unsupported rather than
receiving partial replacement semantics. The output denominator advances to
95 passes and 137 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-character-map-static-boundary-2026-09-01.md)

The unchanged `output-0201` case now supplies the first complete character-map
execution slice: one named unprefixed map is resolved by unnamed XML output and
replaces `$` with the raw `€` mapping through the bounded serializer. Mapping
state remains immutable and stylesheet-derived; composition, precedence,
named outputs, CDATA interaction, and non-XML methods remain unsupported. The
output denominator advances to 96 passes and 136 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-first-character-map-2026-09-01.md)

The unchanged `output-0202` and `output-0203` cases now compose one directly
referenced unprefixed character map and apply local mappings over inherited
mappings for the same character. The slice works with explicit XML output and
with XML method inference; longer chains, multiple or QName references,
imports, declaration precedence, and cycles remain unsupported. The output
denominator advances to 98 passes and 134 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-direct-character-map-composition-2026-09-01.md)

The unchanged `output-0303` case now combines three unprefixed character maps
and applies their replacements to an exact file-backed text serialization.
Mapped replacement bytes use the same bounded sink, while QName identity,
imports, named outputs, result documents, and method-specific HTML/XHTML
admission remain separate work. The output denominator advances to 99 passes
and 133 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-multiple-character-maps-text-2026-09-01.md)

The unchanged `output-0205` case now resolves character-map declaration and
reference QNames by expanded name, proving that different lexical prefixes
bound to the same namespace identify the same map. Unbound prefixes and unknown
expanded names remain distinct static failures; imports, repeated references,
longer chains, cycles, and precedence remain separate work. The output
denominator advances to 100 passes and 132 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-character-map-qname-identity-2026-09-01.md)

The unchanged `output-0206` case now resolves an ordered three-item character-
map reference list containing the same expanded name under different prefixes.
Repeated references merge idempotently into immutable runtime lookup state;
longer dependency chains, cycles, imports, and declaration precedence remain
unsupported. The output denominator advances to 101 passes and 131 visible
default not-run cases.
[Evidence](../Evidence/xslt30-output-repeated-character-map-references-2026-09-01.md)

The unchanged `output-0301` case now applies three character maps through the
XHTML serializer and satisfies both pinned `all-of` patterns with a bounded
structural/content comparator. Raw mapped replacement strings are not escaped
a second time; HTML method behavior, CDATA interaction, imports, named outputs,
and result documents remain separate work. The output denominator advances to
102 passes and 130 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-xhtml-character-maps-2026-09-01.md)

The unchanged `output-0204` and `output-0207` cases now preserve character-map
declarations through sealed stylesheet import composition, resolve principal
output references after the complete package is assembled, and select the
higher-precedence principal declaration for a duplicate expanded name. The
output harness now admits catalog-declared secondary stylesheets by RFC-resolved
snapshot identity. Imported output declarations, arbitrary map dependency
graphs, named outputs, and result documents remain separate work. The output
denominator advances to 104 passes and 128 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-imported-character-maps-2026-09-01.md)

The unchanged `output-0302` case now applies three character maps to a narrowly
guarded null-namespace `html/body/p` result and exactly satisfies its pinned
optional-DOCTYPE pattern without emitting an XML declaration. General HTML
output remains compiler-unsupported, and the runtime rejects result shapes
outside this bounded profile; void/raw-text elements, URI behavior, metadata,
doctypes, and general HTML escaping remain separate work. The output denominator
advances to 105 passes and 127 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-bounded-html-character-maps-2026-09-01.md)

The unchanged `output-0309` and `output-0310` cases now prove ordered
multi-map XML serialization and the required CDATA exclusion: ordinary text is
mapped, while text selected for CDATA remains unchanged. Both paths retain the
semantic result tree and charge bytes through the bounded serializer. Named
outputs, result documents, imported output declarations, and arbitrary map
dependency graphs remain separate work. The output denominator advances to 107
passes and 125 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-xml-character-map-list-and-cdata-2026-09-01.md)

The unchanged `output-0305` case now merges two same-precedence unnamed output
declarations, concatenates their character-map lists in declaration order, and
accepts repeated method/encoding/indent properties only when compiled values
are identical. Conflicting scalar properties remain explicitly unsupported;
named outputs and result documents remain separate work. The output denominator
advances to 108 passes and 124 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-character-map-declaration-merge-2026-09-01.md)

The unchanged `output-0306` case now composes character-map declarations and
principal output-map lists across a three-level sealed import chain. The
principal `format1` definition overrides both imported definitions, while the
intermediate module supplies `format2`; final map resolution occurs only after
the complete package is assembled. This admits exactly one nested leaf import,
not arbitrary dependency graphs. The output denominator advances to 109 passes
and 123 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-nested-import-character-maps-2026-09-01.md)

The unchanged `output-0311` case now emits a paired XML public/system doctype
for the actual document-element name and chooses independent quote delimiters
that preserve an apostrophe in the public identifier and a quotation mark in
the system identifier. All three pinned serialization patterns pass; XHTML
doctype name restrictions and the explicit both-quote rejection remain intact.
The output denominator advances to 110 passes and 122 visible default not-run
cases.
[Evidence](../Evidence/xslt30-output-xml-doctype-identifiers-2026-09-01.md)

The unchanged `output-0230` case now reports its native `XTSE0020` for the
nonnumeric `html-version="five"` value. Valid positive-decimal lexicals are
recognized independently; non-five positive versions remain explicitly
unsupported as `FXST1049`.
The output denominator advances to 111 passes and 121 visible default not-run
cases.
[Evidence](../Evidence/xslt30-output-html-version-static-boundary-2026-09-01.md)

Seven unchanged XHTML 5 controls—`output-0208` through `output-0210` and
`output-0212` through `output-0215`—now retain version-five metadata and emit an
automatic doctype only for an XHTML `html` document element, preserving its
case. XHTML `body` and alien-namespace `html` roots prove the negative boundary;
prefix normalization and broader XHTML 5 rules remain separate. The output
denominator advances to 118 passes and 114 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-bounded-xhtml5-doctype-2026-09-01.md)

The unchanged `output-0211` case now normalizes authored XHTML element prefixes
to the default XHTML namespace under the bounded XHTML 5 serializer while
retaining the automatic doctype. This does not admit general namespace fixup,
SVG/MathML handling, or foreign-attribute rewriting. The output denominator
advances to 119 passes and 113 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-xhtml5-prefix-normalization-2026-09-01.md)

Eight unchanged XHTML 5 empty-element cases—`output-0216` through
`output-0223`—now distinguish void from non-void names across the XHTML and
no-namespace forms, including attributes and prefixed input. The tranche also
completes the modern void-name set and collapses surrounding whitespace in the
output-method token without losing diagnostic provenance. The output
denominator advances to 127 passes and 105 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-xhtml5-empty-elements-2026-09-01.md)

Four unchanged XHTML 5 doctype cases—`output-0227` through `output-0229` and
`output-0231`—now conserve paired, system-only, and public-only external
identifier behavior across the admitted `5`, `5.0`, and `+5.0` spellings. The
public-only form emits the XHTML 5 short doctype; invalid `output-0230` remains
its separate native-error control. The output denominator advances to 131
passes and 101 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-xhtml5-explicit-doctypes-2026-09-01.md)

Three unchanged XHTML 5 namespace cases—`output-0224` through `output-0226`—now
normalize XHTML, SVG, and MathML element names to their required default
namespaces across default, prefixed, and mixed authored forms. Prefix bindings
for those three special namespaces no longer leak into normalized descendants;
arbitrary foreign namespaces remain outside the rule. The output denominator
advances to 134 passes and 98 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-xhtml5-svg-mathml-namespaces-2026-09-01.md)

The unchanged XML `output-0234` case now constructs a bounded static result
comment and preserves root comment/processing-instruction order before placing
the doctype immediately before the document element. Comment nodes and bytes
participate in invocation work accounting; computed comment content and HTML
`output-0233` remain outside this slice. The output denominator advances to 135
passes and 97 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-root-misc-doctype-order-2026-09-01.md)

Three unchanged HTML negative cases—`output-0184`, `output-0191`, and
`output-0194`—now distinguish recognition of the standard HTML method from
successful serializer capability. Unsupported encoding, normalization, and
explicit version report native `SESU0007`, `SESU0011`, and `SESU0013` before
the bounded HTML result-shape gate. General HTML output remains unadmitted. The
output denominator advances to 138 passes and 94 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-html-parameter-errors-2026-09-01.md)

The unchanged HTML negative case `output-0196` now reports native `SERE0015`
when result processing-instruction data contains `>`. The check traverses the
bounded result tree before private HTML shape selection, preserving the
standards diagnostic without admitting successful general HTML output. The
output denominator advances to 139 passes and 93 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-html-processing-instruction-error-2026-09-01.md)

The unchanged HTML 5 `output-0233` case now emits its automatic doctype after a
root comment and immediately before the document element. The successful HTML
slice remains restricted to one no-namespace document element and the exact
`html`, `head`, `title`, `body`, and `p` vocabulary without attributes or
namespace nodes; general HTML serialization remains unadmitted. The output
denominator advances to 140 passes and 92 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-html5-doctype-placement-2026-09-01.md)

The unchanged `output-0312` case now retains explicit output-property presence
across compilation and permits a lower-precedence unnamed output declaration
only when the principal declaration explicitly shadows every imported
property. Empty principal doctype identifiers suppress the imported identifiers
and emit the exact `<a><b/></a>` result. Partial cross-precedence merging, named
outputs, and `xsl:result-document` remain separate. The output denominator
advances to 141 passes and 91 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-imported-empty-doctype-override-2026-09-01.md)

The unchanged HTML 5 `output-0601` case now imports its file-backed source into
the sealed snapshot, copies document child elements through the exact charged
`xsl:copy-of select="*"` slice, and serializes all sixteen standard void
elements without end tags. The successful HTML shape remains attribute-free;
URI escaping, arbitrary elements, and raw-text behavior remain separate. The
output denominator advances to 142 passes and 90 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-html5-void-elements-2026-09-01.md)

The environment-driven HTML 5 `output-0602a` through `0602c` cases now inherit
their upstream stylesheet and file-backed source through the named environment.
SVG and MathML element prefixes normalize to default namespaces while the
unrelated `NamespaceN` prefix remains intact. The result vocabulary and
attributes remain explicitly bounded; namespaced attributes and general HTML
namespace fixup remain separate. The output denominator advances to 145 passes
and 87 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-html5-element-namespace-normalization-2026-09-01.md)

The adjacent `output-0603a` through `0603c` cases now preserve SVG-, MathML-,
and unrelated foreign qualified attributes with the exact prefix bindings they
consume. Element-only known prefixes still normalize away. The accepted
attribute expanded names remain a fixed non-URI set rather than general HTML
attribute or namespace support. The output denominator advances to 148 passes
and 84 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-html5-attribute-namespaces-2026-09-01.md)

The unchanged `output-0604` case now applies its compiled HTML 5 character map
to both text and an unnamespaced attribute after copying the source document
element. A separate validator admits only the exact `doc/a/@value` result shape,
so this evidence does not widen the ordinary bounded HTML 5 vocabulary. The
output denominator advances to 149 passes and 83 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-html5-character-map-2026-09-01.md)

The explicit-version HTML 5 `output-0195b` case now serializes its C1 control as
a hexadecimal numeric character reference through an exact `doc` result shape.
The sibling HTML 4 `output-0195` case is now explicitly excluded by profile;
`output-0195a` remains visible because it depends on an environment-supplied
default HTML version rather than an explicit stylesheet property. The output
denominator advances to 150 passes, 1 profile exclusion, and 81 visible default
not-run cases.
[Evidence](../Evidence/xslt30-output-html-version-control-characters-2026-09-01.md)

Four source-free XHTML cases—`output-0102d`, `output-0102f`, `output-0103d`,
and `output-0103f`—now enter through their named initial templates and preserve
non-URI attribute values with URI escaping both enabled and disabled. The
output harness no longer invents a principal source for source-free entries.
The compiled property remains deliberately inert: URI-valued attributes stay
outside selection until output settings retain the boolean and the serializer
owns normalization plus percent encoding. The output denominator advances to
154 passes, 1 profile exclusion, and 77 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-source-free-xhtml-non-uri-attributes-2026-09-01.md)

The adjacent `output-0102e` and `output-0103e` cases now emit a C1 control in
the non-URI XHTML `accesskey` attribute as a hexadecimal numeric reference,
independently of whether URI escaping is enabled. This is shared
XML-compatible attribute escaping, not URI percent encoding. The output
denominator advances to 156 passes and 75 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-xhtml-c1-attributes-2026-09-01.md)

The six source-free XHTML `href` cases `output-0102a` through `0102c` and
`output-0103a` through `0103c` now retain `escape-uri-attributes` in compiled
output settings. Enabled/default escaping percent-encodes non-ASCII UTF-8 bytes
without double-encoding existing ASCII percent sequences; disabled escaping
keeps non-ASCII content under ordinary XML-compatible attribute rules. The
recognized URI vocabulary is still exactly XHTML `href`, and the HTML case that
requires NFC normalization remains separate. The output denominator advances
to 162 passes and 69 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-xhtml-uri-attributes-2026-09-01.md)

Seven HTML content-type cases—`output-0123`, `output-0124` through `0124b`, and
`output-0125` through `0125b`—now exercise the enabled-by-default rule plus the
XSLT 2.0 and 3.0 true/false lexical variants. Injection is serializer-owned and
uses HTML void-element syntax; disabling the property leaves the text-only
`HTML/HEAD/BODY` semantic result untouched. The validator admits only that
bounded shape. The output denominator advances to 169 passes and 62 visible
default not-run cases.
[Evidence](../Evidence/xslt30-output-html-content-type-meta-2026-09-01.md)

The unchanged `output-0157` and `output-0158` cases now replace an existing
case-varied HTML Content-Type meta rather than adding a duplicate. Both an
authored UTF-16 charset and additional `version` parameter are discarded in
favor of the serializer-owned `text/html; charset=UTF-8` value. Admission
remains limited to one exact two-attribute meta in the bounded head shape. The
output denominator advances to 171 passes and 60 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-html-content-type-replacement-2026-09-01.md)

The unchanged `output-0161` case now composes HTML output with the retained
`normalization-form="none"` setting and verifies the exact decomposed
`41 CC 81` UTF-8 sequence. This extends the existing XML, XHTML, and text
non-normalizing evidence without admitting NFC normalization; `output-0164`
still needs NFC before URI percent encoding. The output denominator advances to
172 passes and 59 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-html-normalization-none-2026-09-01.md)

The unchanged `output-0160` case now verifies whitespace conservation for an
attribute-free HTML `html/head/body/p/del/ins` hierarchy under `indent="no"`.
Text containing the newline, tabs, and spaces inside `del` and `ins` survives,
while the whitespace-only stylesheet node between those result instructions
does not appear. A private validator admits only this exact hierarchy rather
than widening FastXSLT to general HTML serialization. The output denominator
advances to 173 passes and 58 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-html-ins-del-whitespace-2026-09-01.md)

The unchanged `output-0162` and `output-0163` cases now exercise HTML through
the UTF-8 byte-result lane with `byte-order-mark="yes"` and `"no"`. The first
result begins with exactly `EF BB BF`; the second begins directly with the
bounded `html/body` serialization. This reuses the charged byte boundary rather
than teaching the string-result API to represent a BOM. The output denominator
advances to 175 passes and 56 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-html-utf8-bom-2026-09-01.md)

The unchanged `output-0154` case now exercises the HTML raw-text rule for one
manually escaped script in an exact `html/head/script/body` hierarchy. Entity
references are resolved while constructing the semantic text, and the HTML
serializer emits that script text without XML escaping; XHTML remains on its
separate XML-compatible path. The output denominator advances to 176 passes and
55 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-html-script-raw-text-2026-09-01.md)

The unchanged `output-0159` case extends that bounded HTML evidence to one
script and style in the head plus one pre/b and textarea in the body. Script and
style text is emitted raw, while all significant newlines, tabs, and spaces in
the preformatted content survive under `indent="no"`. The validator fixes the
exact hierarchy and textarea attributes rather than admitting general HTML.
The output denominator advances to 177 passes and 54 visible default not-run
cases.
[Evidence](../Evidence/xslt30-output-html-preformatted-whitespace-2026-09-01.md)

The unchanged `output-0724` case now executes as an exact source-free HTML 5
`input` result. The serializer emits the empty element as a void tag, preserves
the airplane character in the ordinary `value` attribute, and does not apply
URI percent encoding. The validator admits only the case's `type="text"` and
`value="✈"` attributes. The output denominator advances to 178 passes and 53
visible default not-run cases.
[Evidence](../Evidence/xslt30-output-html5-input-value-2026-09-01.md)

The unchanged `output-0725` and `output-0726` cases now retain
`suppress-indentation="p"` as an expanded name and consult that list at the
serializer's child-indentation decision. The complete long paragraph remains
unbroken under both bounded HTML 5 and XML-compatible XHTML output. A compiler
sentinel additionally distinguishes unprefixed `p` from namespaced `z:p`; this
does not claim general word wrapping. The output denominator advances to 180
passes and 51 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-suppress-indentation-2026-09-01.md)

Seven unchanged normalization and URI-expansion cases now execute through one
exact-pinned UAX #15 implementation. `output-0146`, `output-0167`, and
`output-0169` apply `normalization-form="NFC"` to XHTML, XML, and text output;
`output-0101`, `output-0101a`, `output-0101b`, and `output-0164` normalize URI
values to NFC before percent-encoding. Character-map substitutions remain
outside requested normalization, enabled URI expansion bypasses character
maps, and the HTML validator admits only the unchanged
`html/body/div/a[@href]` result shape. NFD, compatibility forms,
fully-normalized output, and the wider URI-attribute vocabulary remain visible
future work. The dependency review records the exact package, licenses,
transitive graph, Unicode version, and its dependency-owned Hangul unsafe
surface. The output denominator advances to 187 passes, one profile exclusion,
and 44 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-unicode-normalization-and-uri-expansion-2026-09-02.md)

The adjacent `output-0115b` through `output-0115e` cases now execute through a
bounded US-ASCII XHTML CDATA byte profile. Nonrepresentable characters close
the CDATA run, serialize as uppercase hexadecimal references, and reopen it;
the NFD case leaves ASCII `c` inside CDATA and externalizes only the combining
cedilla. CDATA continues to bypass character maps. Expansion bytes are charged
and checked against the final host byte ceiling. Non-ASCII content outside
selected CDATA, arbitrary legacy encodings, NFKC/NFKD, and fully-normalized
output remain outside the slice. The output denominator advances to 191 passes,
one profile exclusion, and 40 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-us-ascii-cdata-normalization-2026-09-02.md)

The unchanged `output-0232` case now executes two static `1 to 5` loops through
a bounded context-independent range instruction and verifies all seven native
serialization patterns. Range items are individually charged; bodies that
would observe the unrepresented atomic context item remain explicitly
unsupported. The case also corrected `suppress-indentation` inheritance:
suppression selected by unprefixed `p` or namespace-qualified `z:p` now remains
active through nested descendant elements rather than restarting indentation
one level below the match. The output denominator advances to 192 passes, one
profile exclusion, and 39 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-static-range-suppress-indentation-2026-09-02.md)

The unchanged `output-0134` case now validates its unused named text output
declaration independently of the unnamed principal XHTML declaration. The
implicit final result consequently uses the principal format and satisfies both
native serialization patterns. This does not admit a named-format table,
duplicate named-declaration merging, named character-map resolution, or
`xsl:result-document`. The output denominator advances to 193 passes, one
profile exclusion, and 38 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-unused-named-declaration-2026-09-02.md)

The unchanged `output-0141`, `output-0141a`, and `output-0141b` cases now
execute the XSLT 2.0 `no` and XSLT 3.0 `false`/numeric-zero serializer-property
lexicals together with an independently constant-folded literal
`escape-html-uri()` computed attribute. Literal URI attributes remain
unescaped, while the function result receives uppercase UTF-8 percent escapes
without NFC normalization, preserving decomposed `a%CC%8A`. Dynamic arguments
and general function dispatch remain outside the slice. The output denominator
advances to 196 passes, one profile exclusion, and 35 visible default not-run
cases.
[Evidence](../Evidence/xslt30-output-escape-html-uri-2026-09-02.md)

The unchanged source-free `output-0723` case now constant-folds its bounded
comment select expression from literal strings plus
`codepoints-to-string(13)`. XML serialization preserves the carriage return
directly and matches the exact native assertion. Generated codepoints are
restricted to XML 1.0 characters, while dynamic operands and general string
function dispatch remain unsupported. The output denominator advances to 197
passes, one profile exclusion, and 34 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-comment-carriage-return-2026-09-02.md)

The unchanged `output-0140` case now executes through a deterministic
BOM-prefixed UTF-16BE byte lane, retaining the requested `UTF-16` declaration
and non-ASCII XHTML text. The public/private string result remains UTF-8-only.
Before this addition, `serialization.rs` reached ADR-0004's 2,000-line
calibration trigger; physical US-ASCII and UTF-16 conversion moved into a
private one-way `byte_encoding` module, returning the semantic serializer to
1,973 lines. The output denominator advances to 198 passes, one profile
exclusion, and 33 visible default not-run cases.
[Evidence](../Evidence/xslt30-output-utf16-byte-lane-2026-09-02.md)

The unchanged source-free `mode-0001`, `mode-0003`, and `mode-0005` cases now
enter their catalog-named initial template without a manufactured source,
materialize one typed parentless comment, processing-instruction, or text node,
and apply native shallow-copy, shallow-skip, and text-only-copy policies. The
mode denominator advances to 79 passes, 45 profile exclusions, and 45 visible
default not-run cases.
[Evidence](../Evidence/xslt30-mode-parentless-comment-pi-text-2026-09-02.md)

The unchanged source-free `mode-0015` case now retains a literal attribute as
an immutable temporary XDM node rather than a child approximation. Exact
attribute-rule dispatch composes with shallow-copy, shallow-skip, and
text-only-copy traversal, including the combined attribute/child focus used by
the first two policies. The mode denominator advances to 80 passes, 45 profile
exclusions, and 44 visible default not-run cases.
[Evidence](../Evidence/xslt30-mode-temporary-element-attribute-policies-2026-09-02.md)

The unchanged source-free `mode-0007` case now materializes one typed
parentless attribute. Shallow-copy attaches it to the surrounding literal
result element through private pending construction state, shallow-skip omits
it, and text-only-copy emits its value. Escaped, late, and duplicate result
attributes remain explicit `XTDE0410` failures. The mode denominator advances
to 81 passes, 45 profile exclusions, and 43 visible default not-run cases.
[Evidence](../Evidence/xslt30-mode-parentless-attribute-policies-2026-09-02.md)

The unchanged source-free `mode-0016` case now preserves expanded names for
unqualified and namespace-qualified temporary attributes, retains the
temporary element's required namespace bindings, and evaluates the exact
`{local-name()}` AVT against temporary attribute focus. The unchanged `@*`
rule and all three built-in policies match the native result. The mode
denominator advances to 82 passes, 45 profile exclusions, and 42 visible
default not-run cases.
[Evidence](../Evidence/xslt30-mode-namespaced-temporary-attributes-2026-09-02.md)

The unchanged `mode-1413` case now applies source attribute templates during
shallow-copy rather than blindly copying or rejecting the intercepted
attribute. The exact `@length` rule emits an incremented replacement attribute,
attribute and child traversal retain one combined focus, source comments are
representable, and the `chtitle` wrapper continues through `xsl:next-match`
into the built-in shallow-copy rule. General standalone computed-attribute
expressions remain outside the bounded slice. The mode denominator advances to
83 passes, 45 profile exclusions, and 41 visible default not-run cases.
[Evidence](../Evidence/xslt30-mode-source-shallow-copy-attribute-override-2026-09-02.md)

The unchanged `mode-1516` and `mode-1517` cases now retain overlapping
equal-priority union alternatives as one compiled template rule. A source
`para` matching both `para[foo]` and `para[text()]` therefore remains
unambiguous under the mode-owned `on-multiple-match="fail"` policy, including
the parenthesized spelling. The policy overrides a host recovery fallback,
while two genuinely distinct equal-rank rules still report `XTDE0540`. The
catalog's named initial-template entry now also receives its supplied source
document as context. General predicate grammar and mixed-priority overlapping
unions remain outside the bounded slice. The mode denominator advances to 85
passes, 45 profile exclusions, and 39 visible default not-run cases.
[Evidence](../Evidence/xslt30-mode-overlapping-union-single-rule-2026-09-02.md)

The unchanged `mode-1901` case now admits its exact imported stylesheet into
the sealed snapshot and executes the imported named initial template under the
principal stylesheet's higher-precedence mode policy. The principal
`on-no-match="fail"` replaces the imported text-only-copy behavior and reports
the native `XTDE0555` outcome when the temporary comment is unmatched. This
does not infer package-level mode overriding, visibility composition, or
accumulator semantics. The mode denominator advances to 86 passes, 45 profile
exclusions, and 38 visible default not-run cases.
[Evidence](../Evidence/xslt30-mode-1901-imported-policy-override-2026-09-02.md)

The unchanged `mode-1107a`, `mode-1107b`, and `mode-1107c` cases are now
explicit streaming-profile exclusions rather than generic defaults. Their
native environments mark the principal source `streaming="true"` and enable
the stylesheet's streamable mode, so ordinary tree execution would not be
streaming-conformance evidence. The ledger now validates either an upstream
streaming feature dependency or streamed-source metadata for every such
exclusion. The mode denominator remains at 86 passes, advances to 48 profile
exclusions, and leaves 35 visible default not-run cases.
[Evidence](../Evidence/xslt30-mode-1107-streaming-profile-classification-2026-09-02.md)

The complete 30-case `decl/strip-space` denominator is now conserved with a
visible default disposition. The unchanged `strip-space-012` case executes
through the ADR-0012 invocation-owned visibility view and matches its native
XML assertion; the other 29 cases remain visibly not run. This admits only
exact `xsl:strip-space elements="*"` and does not infer name tests,
`xsl:preserve-space`, declaration precedence, schema-aware behavior,
`xml:space`, or temporary-tree whitespace rules.
[Evidence](../Evidence/xslt30-strip-space-denominator-and-strip-all-2026-09-02.md)

The complete six-case `misc/built-in-templates` denominator is now conserved.
The unchanged `built-in-templates-0101` and `0102` cases execute through the
normal source, compilation, transform-set, and XML-comparison path, proving
`#current` and `#default` propagation through recursive built-in document and
element rules. The four parameter-typing and schema-annotation cases remain
visible defaults rather than being approximated.
[Evidence](../Evidence/xslt30-built-in-template-mode-propagation-2026-09-02.md)

The unchanged `mode-1902` case now retains the principal stylesheet's private
mode declaration into request admission and reports native `XTDE0045` when the
host selects imported mode X as the initial mode. The prerequisite import path
now inherits only the already-supported `method`, `encoding`, and `indent`
properties from one imported program when the principal leaves them
unspecified. Other output properties, public/final visibility composition,
packages, and imported-conflict recovery remain explicit boundaries. The mode
denominator advances to 87 passes, 48 profile exclusions, and 34 visible
defaults.
[Evidence](../Evidence/xslt30-mode-1902-private-initial-mode-2026-09-02.md)

The unchanged `mode-1905` case now composes visibility at import precedence.
A principal public declaration for unprefixed mode X resolves two conflicting
lower-precedence visibility-only declarations without suppressing their
templates or unrelated validation. Initial-mode dispatch selects the principal
document rule and satisfies `exists(/scout)`, while direct `mode-1904` still
reports `XTSE0545`. This is a bounded property-level rule, not general package
component composition. The mode denominator advances to 88 passes, 48 profile
exclusions, and 33 visible defaults.
[Evidence](../Evidence/xslt30-mode-1905-visibility-precedence-2026-09-02.md)

The complete ten-case `fn/root` denominator is now conserved. Unchanged
`root-0101` admits the exact `root(.)` context-node form and the equivalent
`document-node()` match spelling. Two adjacent cases add empty selections,
the explicit empty sequence, and an element argument while enforcing the
function's zero-or-one cardinality. A fourth case adds typed child
`element()`, `comment()`, `processing-instruction()`, `node()`, and `text()`
tests. A fifth resolves a stylesheet prefix into an expanded child name test
without changing the shared location-path evaluator. A sixth binds source-node
sequences in an invocation-local frame and applies `root($variable)` without
copying or detaching their prepared-document identity. Two more compare an
opaque invocation-local generated identity after element- and document-node
root selection. A ninth gives each materialized temporary tree an
invocation-local document identity and proves descendant nodes retain that
root without relying on variable spelling or allocation addresses. The tenth
resolves a literal `document()` reference against the stylesheet base, admits
only sealed-snapshot bytes, prepares one invocation-owned secondary document,
and proves repeated access retains its identity. The first execution exposed
and repaired a shared XDM defect: document and element string values now exclude
descendant comment and processing-instruction content, while those nodes retain
their own string values when selected directly. All ten cases pass; this closes
the denominator without selecting live acquisition or a public resolver/cache
API.
[Evidence](../Evidence/xslt30-root-context-and-xdm-string-value-2026-09-02.md)

The complete one-case `insn/apply-imports` denominator is now conserved with
its inline source, principal plus two secondary stylesheets, initial-template
entry, and native assertion sealed and validated. The unchanged case now
dispatches `1 to 5` through integer-threshold patterns, preserves atomic focus,
and applies imports only down the current stylesheet level's import ancestry.
It also corrects XSLT 3.0 late-import acceptance and higher-precedence named
template shadowing. The exact two-sibling-leaf topology passes without claiming
general atomic pattern or import-ancestry representation.
[Evidence](../Evidence/xslt30-apply-imports-atomic-focus-denominator-2026-09-02.md)

The complete 55-case `insn/choose` denominator is now conserved before case
selection. Forty-two unchanged XSLT 1.0/2.0 cases execute through the
normal principal-source path: true and false child-existence tests, ordered
`xsl:when` selection, `xsl:otherwise`, empty fall-through, constant numeric and
string equality, effective boolean values for non-empty strings and zero,
positive or negated context string-value comparisons, and relative child or
attribute paths compared to a string literal. The latter composes with source-node iteration
and exact strip-all whitespace visibility. Existing constant numeric evaluation
also supplies `round()` and modulo comparisons across nested conditional
instructions. Present nonmatching and absent attributes both produce false
without manufacturing branch output. Four unchanged negative cases additionally compare `XTSE0010`
for a missing `test`, a late `xsl:when`, and duplicate `xsl:otherwise`
structure. One ordered choose also compares an untyped integer child lexical
against successive integer bounds and stops at the first true branch. Another
case filters a relative child path by its context string value and reads sibling
values through both implicit and explicit-context child paths. A further case
short-circuits `or` across two attribute string comparisons. Seven nested
conditionals additionally compose constant arithmetic and string comparisons
with an exact unqualified `name(..)` test. A 26-way choose iterates
context-relative descendants through `.//*`, compares their unqualified names,
and exercises the native exact root-string assertion. Another nested choice
looks ahead from successive ancestor levels through the first matching element
on the forward `following-sibling` axis. Its sibling uses the same descendant
focus across 26 Unicode-codepoint string-length branches. A local
variable may also bind the current source-template `position()` and retain that
integer for ordered branch comparison; a focused control observes distinct
positions across a two-node `xsl:for-each`. Three further cases preserve
schema-namespace-resolved `xs:string` and `xs:untypedAtomic` global values and
apply string-family effective boolean value to bare variable tests. A further
case constructs two independent temporary document nodes from untyped global
sequence constructors and compares their atomized string values both directly
and through `string()`, without collapsing their distinct node identities into
compiled atomic values. A further
case keeps an empty-sequence global distinct from an empty atomic value
and composes exact `()`, `$variable=()`, and `boolean($variable)` tests without
approximating empty sequences as strings or temporary trees. Two numeric cases
retain literal schema-namespace `xs:integer` and `xs:double` constructors plus
the exact source-dependent `xs:double(path div path)` global form. They
distinguish missing operands, `NaN`, zero, and nonzero numeric effective boolean
values under controlled traversal and charged division. One nested conditional
case additionally preserves whitespace-only stylesheet
text inherited from `xml:space="preserve"` on `xsl:choose`, with invalid
lexicals rejected explicitly. Two namespace-context cases additionally retain
typed boolean globals and apply inherited, instruction-local, and explicitly
reset `xpath-default-namespace` values to exact descendant `count()`
expressions. Their mixed-namespace source makes namespace matching observable.
The descendant/name branch family additionally executes under the first
available member of an explicit default-collation list, applying HTML ASCII
case-insensitive comparison without changing QName identity. Two further cases
use a narrow XPath conditional-expression plan from both
conditional tests and `xsl:value-of`, selecting integer branches through an
exact source `contains()` condition or constant integer comparison and lazily
composing one nested conditional. Five more cases use a distinct typed-path
conditional plan: schema-namespace-resolved integer casts feed equality or
ordering conditions, and only the selected path, division, or nested
conditional branch executes. Dead division-by-zero branches in two unchanged
cases make lazy selection observable. The remaining nine cases now execute a
compile-only classification lane and report structured engine-unsupported
outcomes for typed template contracts, typed parameters and arithmetic AVTs,
QName construction/comparison, UCA collation and variable concatenation,
constructed-tree navigation and dynamic elements, or mixed node-and-atomic
sequences. No case remains under the denominator's default disposition.
[Evidence](../Evidence/xslt30-choose-and-if-initial-denominator-2026-09-02.md)

The complete 42-case `insn/call-template` denominator is now conserved before
selection. Twenty unchanged cases pass. Initial-template entry now covers
unqualified and catalog-resolved qualified identities, an independently bound
global stylesheet parameter and template default, and source-aware copying of
a current document's children. Three correctly resolved absent-name cases
report `XTDE0040`; a reserved-namespace declaration reports `XTSE0080`.
Source-free focus access through the exact `ancestor-or-self::*` copy path
reports `XPDY0002`.
An omitted required initial-template parameter reports `XTDE0700` without
confusing host-supplied stylesheet parameters with template arguments.
Its XSLT 2.0-only `XTDE0060` companion is visibly excluded by profile.
Named calls cover QName and EQName identity, literal and default parameters,
integer and relative source-node `select` arguments, six nested calls, repeated calls under conditional
branches, and principal/import precedence. Catalog-declared secondary modules
remain sealed before compilation. The other 21 cases remain visible defaults
for a public host QName contract, broader parameter values and typing, recursion,
focus, and expression or assertion semantics.
[Evidence](../Evidence/xslt30-call-template-initial-denominator-2026-09-02.md)

## Corpus audit -- 2026-08-30

This audit reconciles the pinned suite catalogs, first-party overlays,
executable Rust adapters, golden fixtures, corpus plans, and retained evidence.
Counts below are repository facts at the pinned revisions; they are not a
conformance percentage or a promise about unselected cases.

### Corpus assets and authority

| Family | Current authority | Current state |
| --- | --- | --- |
| First-party golden | Four reviewed directories under `corpus/golden` | `hello`, `template-dispatch`, `built-in-template-rules`, and `host-owned-two-stage` all execute in normal tests; the staged case proves that produced sibling output is unavailable until the host admits it into a later snapshot. |
| QT3 | Immutable submodule `83993587711dbd5c18ed846385ec37d079d6e492` | 428 test sets and 31,821 cases are structurally inventoried; 408 explicitly selected cases execute through two suite-specific XPath adapters. |
| XSLT30 | Immutable submodule `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` | 234 test sets and 14,600 cases are structurally inventoried; 112 complete test-set denominators plus one separate AVT pressure case have first-party records. |
| W3C XML 20130923 | Hash-recorded ignored local candidate | 2,586 cases were inventoried during candidate review, but no bytes are admitted or redistributed pending rights and acquisition decisions. |
| First-party adversarial | Policy and XML plan only | Focused unit/integration tests exercise limits and cancellation, but there is no separately versioned `corpus/adversarial` family, manifest, or report denominator yet. |
| Performance | Workbench fixtures, ignored release probes, and evidence records | Useful ASP.NET/native/isolated and prepared-state measurements exist, but there is no formal `corpus/performance` manifest with correctness gates and reproducible workload identity. |

The QT3 and XSLT30 submodules remain development/test inputs outside the MIT
library artifact. Verification checks their exact revisions and clean state,
and suite adapters copy only explicitly required bytes into bounded sealed
snapshots before engine compilation or execution. Upstream files and expected
results remain immutable; selection, corrections, and classifications stay in
first-party overlays.

### Executable standards accounting

The XSLT30 work currently conserves these complete native denominators:

| XSLT30 test set | Total | Passed comparison | Engine unsupported | Excluded by profile | Visible default not run |
| --- | ---: | ---: | ---: | ---: | ---: |
| `decl/template` | 6 | 6 | 0 | 0 | 0 |
| `expr/path` | 10 | 10 | 0 | 0 | 0 |
| `expr/for` | 4 | 4 | 0 | 0 | 0 |
| `expr/castable` | 9 | 4 | 3 | 2 | 0 |
| `expr/data-manipulation` | 28 | 28 | 0 | 0 | 0 |
| `fn/deep-equal` | 2 | 2 | 0 | 0 | 0 |
| `misc/initial-mode` | 5 | 5 | 0 | 0 | 0 |
| `insn/apply-templates` | 50 | 49 | 0 | 1 | 0 |
| `attr/mode` | 169 | 88 | 0 | 48 | 33 |
| `decl/include` | 16 | 14 | 0 | 2 | 0 |
| `decl/output` | 232 | 198 | 0 | 1 | 33 |
| `decl/strip-space` | 30 | 1 | 0 | 0 | 29 |
| `misc/built-in-templates` | 6 | 2 | 0 | 0 | 4 |
| `fn/root` | 10 | 10 | 0 | 0 | 0 |
| `insn/apply-imports` | 1 | 1 | 0 | 0 | 0 |
| `insn/choose` | 55 | 46 | 9 | 0 | 0 |
| `insn/call-template` | 42 | 20 | 0 | 1 | 21 |
| `expr/treat-as` | 4 | 0 | 0 | 4 | 0 |
| `expr/type-expr` | 4 | 0 | 0 | 4 | 0 |
| `expr/type-functions` | 12 | 0 | 0 | 12 | 0 |
| 91 test sets inheriting `feature="streaming"` | 2,746 | 0 | 0 | 2,746 | 0 |
| `insn/evaluate` inheriting `feature="dynamic_evaluation"` | 57 | 0 | 0 | 57 | 0 |
| **Conserved total** | **3,498** | **488** | **12** | **2,878** | **120** |

One additional selected `attr/avt` case remains visibly harness-unsupported
because its compound message/equality assertion is not owned by a comparator.
Across the full XSLT30 suite, 11,101 other cases are catalog-inventoried but do
not yet have individual first-party dispositions. This distinction matters:
catalog discovery conserves the source inventory, while only a complete
test-set overlay conserves a reportable case denominator.

QT3 now has complete parent overlays for the two test sets under active
execution. Typed validation composes explicit selected private-ledger records,
native XQuery-only dependency exclusions, and a visible
`harness-unsupported/not-run` default for every other sibling. Explicit
selection wins over the dependency rule so a case whose individual XPath
expression has been admitted is not hidden by broader upstream metadata:

| QT3 test set | Native cases | Selected and passed | Profile excluded | Visible default not run |
| --- | ---: | ---: | ---: | ---: |
| `prod/AxisStep.xml` | 349 | 224 | 112 | 13 |
| `fn/deep-equal.xml` | 263 | 184 | 67 | 12 |
| **Audited subtotal** | **612** | **408** | **179** | **25** |

The 612-case subtotal is therefore conserved without relabeling the 25
unclassified siblings as engine failures. The 179 exclusions describe the
current XPath-in-XSLT profile and remain overridable by explicit case admission.
AxisStep's 13 remaining defaults are precisely its namespace-node/namespace-axis
frontier. Deep-equal's 12 remaining defaults require UCA or suite-private
collations or invocation clock/timezone semantics. These are named feature
boundaries rather than an invitation to add case-specific successful answers.
The other 31,209 QT3 cases remain
structural catalog inventory only and still lack first-party selection
dispositions.
[Evidence](../Evidence/qt3-axis-deep-equal-conserved-denominators-2026-09-02.md)
[Schema-aware XSLT30 profile evidence](../Evidence/xslt30-schema-aware-expression-profile-denominators-2026-09-03.md)
[Streaming XSLT30 profile evidence](../Evidence/xslt30-streaming-profile-denominators-2026-09-03.md)
[Dynamic-evaluation XSLT30 profile evidence](../Evidence/xslt30-dynamic-evaluation-profile-denominator-2026-09-03.md)

### Verification capabilities already established

- [x] Pin, initialize, integrity-check, and deterministically inventory both
  external Git suites with no missing or duplicate root test-set references.
- [x] Preserve native suite revision, test-set path, case identity,
  dependencies, environments, stylesheets, sources, entry mode/template, and
  assertion shape in suite-specific adapters.
- [x] Keep engine execution memory-resident by importing case resources into
  bounded snapshots and closing harness-owned files before execution.
- [x] Maintain first-party overlays outside immutable upstream content, with
  explicit selection and execution dispositions for admitted slices.
- [x] Compare bounded examples of exact serialized text/bytes, parsed XML,
  assertion fragments, QT3 scalar/node counts, expected static/dynamic errors,
  and permitted alternatives without treating every assertion as byte equality.
- [x] Preserve structured diagnostic identity and location for admitted
  negative cases, including standard errors such as `XTSE0020`, `XTTE0510`,
  and `XTDE0540` where the native case requires them.
- [x] Mechanically conserve private ledger totals across filtering, sharding,
  interruption, retry, merge order, and conflicting observations.
- [x] Keep conformance, adversarial, differential, integration, and performance
  evidence conceptually separate even when one fixture informs another.

### Remaining corpus work

The next work is ordered to improve explainability before maximizing raw pass
count:

1. [x] Close the nearest complete XSLT30 denominators. The sole remaining
   `apply-templates` case is schema-aware and the two remaining `include` cases
   require denied DTD/entity behavior, so all three retain native evidence and
   explicit profile exclusions rather than forced execution.
2. [ ] Continue coherent semantic slices through the 33 visible `output`
   gaps and 33 visible `mode` gaps. The output remainder is now an explicit
   feature frontier: 10 cases require secondary-result ownership, 22 require
   XDM 3.1 maps/arrays plus JSON, adaptive, or serialization-parameter
   semantics, and one depends on a host-supplied default HTML version. Each
   promotion still requires native metadata validation, a focused control, and
   an owned comparator or exact diagnostic—not merely successful stylesheet
   execution.
3. [x] Give `AxisStep.xml` and `fn/deep-equal.xml` complete QT3 overlays so all
   612 parent-set cases receive a selection disposition. The typed verifier
   now conserves 408 selected passes, 179 native-dependency profile exclusions,
   and 25 visible default not-run cases against the immutable upstream case
   identities. Explicit selection takes priority over a dependency rule.
4. [ ] Add complete denominators deliberately, selected by standards and
   implementation pressure rather than easy-case sampling. The remaining
   11,101 XSLT30 and 31,209 QT3 catalog-only cases must stay outside pass/fail
   totals until individually classified.
5. [ ] Replace string-scanned experimental overlays with a validated internal
   loader and derive one immutable run report carrying suite/engine/harness
   revisions, profile, target/toolchain/features, selection reasons, outcomes,
   and conservation totals. The two active QT3 adapters now use a typed private
   ledger/denominator loader. XSLT30 runtime adapters now resolve every use of
   `private-slice-v0.toml` through the typed private-overlay loader instead of
   scanning TOML fragments, including mixed `expr/castable` dispositions and
   all 88 selected mode cases. The `fn/root`, `insn/apply-imports`,
   `insn/choose`, and `insn/call-template` overlays now use the same typed
   denominator schema and exact override identities, including selected,
   profile-excluded, passed, and engine-unsupported dispositions. The
   `decl/strip-space` and `misc/built-in-templates` default and override
   dispositions use that loader as well. The complete `decl/include` and
   `attr/mode` overlays now use exact typed defaults and overrides, while the
   richer typed `decl/output` loader owns its direct case checks. The
   `insn/apply-templates` overlay now has explicit selection rationales and uses
   the same exact typed admission path. Runtime checks for complete mode,
   strip-space, and built-in-template denominators no longer depend on duplicate
   records in the broad private overlay. The unified immutable report remains.
   Do not stabilize this test-only loader as a public API prematurely.
6. [ ] Define fast pull-request, focused semantic, and reproducible full-corpus
   CI tiers. A shard, retry, feature flag, or unavailable corpus must never
   silently shrink the denominator.
7. [ ] Resolve W3C XML suite rights and choose local-only hash-verified
   acquisition or reviewed redistribution; then implement edition/namespace/
   entity classification and execute one nonvalidating XML/Namespaces subset.
8. [ ] Create a first-party adversarial corpus with exact bytes or generators,
   named expected work/structural limits, cancellation points, and no
   conformance or production-budget implication.
9. [ ] Create correctness-gated performance manifests for cold compilation,
   prepared reuse, serialization, concurrency, allocation/retention, and host
   transfer. Existing workbench numbers remain evidence, not benchmark corpus
   defaults.
10. [ ] Reconcile stale corpus summaries in `corpus/README.md`,
    `docs/testing-strategy.md`, and AR-0011 with the executable ledgers before
    using those pages for release reporting.
11. [ ] Review W3C license/trademark terms, subset rules, report language, and
    exact distribution contents before publishing a conformance or
    standards-performance claim.

No full-suite percentage is currently warranted. The next credible reporting
milestone is an immutable internal report in which every case in each claimed
denominator has one explainable selection disposition and every selected case
has one explainable execution/comparison outcome.

The current order of work is:

1. continue standards-driven output, mode, XPath, and expression slices while
   keeping their denominators conserved;
2. complete the two QT3 parent-set overlays and introduce a validated internal
   ledger/report loader when duplicated overlay mechanics justify it;
3. resolve XML-corpus acquisition and begin a distinct adversarial family;
4. obtain representative consumer transforms and budgets, then use them to
   prioritize optional compatibility, formal performance workloads, and the
   supported native/isolated host profiles.

Representative consumer transforms are not a prerequisite for a testable
standards-driven preview. The pinned W3C suites provide executable stylesheets,
sources, dependency metadata, environments, assertions, and expected errors and
can drive incremental implementation now. Consumer examples remain necessary
to prioritize optional compatibility, validate useful workload coverage, choose
host-facing lifecycle details, and make ASP.NET or application-performance
claims.

CR-0001 records Tokimu's Web3D X3D-to-VRML workflow as the first concrete
Rust-native consumer pressure and opens AR-0012 for the supported Rust facade.
The request is deferred while Tokimu likely uses Saxon in the near term, so it
does not displace standards-driven engine work or select a facade. It also does
not yet close the representative-workload item: the authoritative Web3D
invocation, pinned/licensed stylesheet and resources, parameters, trusted
sentinels/output, input distributions, trust model, and performance budgets are
still missing.

ADR-0006 now makes AR-0011's essential ledger invariants binding. Its remaining
reporting, storage, CI, and comparison-family work proceeds when executable
standards slices need it; those deferred choices are not the current release
gate. AR-0005 and AR-0010 remain seam-preservation reviews. Compatibility AR-0006
and streaming AR-0007 are deferred unless a representative case activates one
of their reopening pressures.

## M0 -- Project scaffold

- [x] Buildable Rust workspace with formatting, lint, test, and docs gates.
- [x] Documentation authority and lifecycle established.
- [x] Initial SDD, ADR process, AR process, and testing strategy established.
- [x] Golden corpus layout seeded without claiming executable support.

Exit criterion: a clean checkout can run the local verification script and the
next architectural questions are visible rather than encoded accidentally.

## M1 -- Standards decision and first vertical slice

- [ ] Record representative transform families, input/output shapes, and
  compatibility needs from the first intended consumer before claiming product
  fit or selecting host/performance defaults; this does not block the
  standards-driven preview.
- [x] Close AR-0001 through accepted ADR-0007.
- [x] Select and document the leading private XML parser boundary for the slice.
- [x] Compile one root template, evaluate one path/value expression, and produce
  one result through a private end-to-end engine path.
- [x] Run `corpus/golden/hello` with private structured failure identities.
- [x] Load the golden source and stylesheet through a bounded resource set,
  seal it, and execute the case without engine-owned filesystem access.
- [x] Release import handles before sealing, then replace or remove the original
  fixture files and prove the snapshot still executes identically.
- [x] Add negative cases that distinguish invalid input from unsupported syntax.
- [x] Establish the first private structured boundary failures and reportable semantic
  outcomes from emitted behavior rather than an aspirational error catalog,
  providing evidence for AR-0004.
- [x] Preserve native QT3/XSLT30 case identity, separate selection from
  execution disposition, reject unknown metadata visibly, and conserve report
  denominators through a private AR-0011 experiment.

Exit criterion: the seed transform passes through the intended layers and every
implemented behavior belongs to a named standards slice.

## M2 -- Data model and XPath foundation

- [ ] Define node identity, document order, names, strings, and sequence/value
  behavior needed by the accepted profile.
- [x] Record the navigation and retention capabilities the implemented XPath
  and XSLT slice actually requires; keep tree-specific random-access assumptions
  inside their physical owner, providing evidence for AR-0007.
- [ ] Expand XPath lex/parse/evaluate tests before growing XSLT instructions.
- [x] Admit all ten cases in the complete XSLT30 `path` test set and execute
  `path-001` through `path-010`, including charged axis predicates, per-step
  positions, `last()`, checked constant-integer arithmetic, integer-domain
  `floor()`, and the native complex relative match pattern.
- [x] Execute the complete QT3 `Axes002` named-child-axis group through native
  environments, direct XPath, `fn:count`, charged navigation, and `assert-eq`
  comparison without an XSLT wrapper.
- [x] Execute the complete QT3 `Axes001` element-wildcard group through the same
  native environments and direct XPath seam. Make `child::*` select every
  element child across namespaces, exclude non-element children, and charge
  every examined node without claiming `node()` or namespace wildcards.
- [x] Execute the complete QT3 `Axes003` child-node-test group through a private
  typed name-test representation. Preserve text, element, comment, and
  processing-instruction children in document order, keep attributes outside
  the child sequence, and charge each examined child exactly once.
- [x] Execute complete QT3 `Axes004` through `Axes006` as the abbreviated
  named-element, any-element, and any-child-node equivalents. Prove each pair
  lowers to the same private typed child steps, retaining one evaluator and one
  work-accounting path rather than creating abbreviation-specific semantics.
- [x] Execute complete QT3 `Axes007` through `Axes011` through typed explicit
  and abbreviated attribute steps. Keep attributes outside the child sequence,
  exclude namespace declarations, apply the attribute principal node kind,
  charge examined attributes, and reject unimplemented leading `//@*` rather
  than returning a plausible empty result.
- [x] Before another axis family, execute the path-owner checkpoint: move the
  invariant regression body to a private path-owned test module and rename
  child-only private path terminology without changing semantics, diagnostics,
  charging, or downstream compiler/runtime behavior.
- [x] Execute QT3 `Axes012-1` through an explicit document-node path origin.
  Accept insignificant whitespace around the root-only operand, select the
  document node independently of the current element context, and charge that
  selection once without admitting general absolute paths.
- [x] Execute complete QT3 `Axes013` through `Axes019` with typed parent steps
  and the `..` abbreviation. Distinguish the element principal node kind from
  `node()` when the parent is the document, charge each examined singleton
  parent, and admit only the exact unprefixed absolute child-path semantics
  exercised by the group.
- [x] Execute the six selected QT3 self-axis cases in `Axes020` through
  `Axes030`. Preserve element, attribute, and text-node context identity,
  enforce the self axis's element principal node kind, charge the examined
  context once, and reject unimplemented kind tests on other axes rather than
  returning plausible empty results. Keep the numbering gaps outside the
  admitted denominator.
- [x] Execute complete QT3 `Axes031` through `Axes033` through typed descendant
  steps. Preserve depth-first document order across element, text, comment,
  and processing-instruction nodes, apply the axis's element principal node
  kind, and charge each traversed descendant once rather than charging again
  while filtering.
- [x] Execute complete QT3 `Axes034` through `Axes037` through typed
  descendant-or-self steps. Include the context before its descendants, retain
  mixed node kinds for `node()`, eliminate repeated XDM identities produced by
  overlapping nested contexts, preserve document order, and keep every actual
  traversal visit attributable to the invocation budget.
- [x] Execute selected QT3 `Axes041` and `Axes043` to retain attribute and text
  contexts through `descendant-or-self::node()`, then execute complete
  `Axes044` through `Axes049` through the existing document-node origin. Prove
  explicit and abbreviated absolute child forms share typed steps and retain
  top-level non-element nodes only for `node()`.
- [x] Execute complete QT3 `Axes055` through `Axes061` by composing the existing
  document-node origin with typed self, descendant, and descendant-or-self
  steps. Preserve the document node for self and any-node
  descendant-or-self, exclude it for descendant and element-principal tests,
  and avoid a separate absolute-axis evaluator.
- [x] Execute complete QT3 `Axes062` through `Axes067` through the existing
  leading descendant origin. Prove explicit and abbreviated any-element,
  named-element, and any-node child forms share typed steps, preserve document
  order, and charge each visited node once. Keep leading attribute steps
  unsupported until their distinct expansion is implemented deliberately.
- [x] Execute complete QT3 `Axes068` through `Axes071` with deliberate leading
  descendant attribute expansion. Reset leading `//` to the document node,
  traverse descendant element contexts, inspect their attributes in document
  order, exclude namespace declarations, unify explicit and abbreviated
  attribute forms, and charge traversal plus attribute inspection separately.
- [x] Execute complete QT3 `Axes072` and `Axes073` with an axis-aware leading
  descendant self expansion. Retain the document node for `self::node()`,
  exclude it through the element-principal wildcard, preserve document order,
  and charge each expanded context exactly once.
- [x] Execute complete QT3 `Axes074` through `Axes079` by lowering an isolated
  internal `//` separator to the typed descendant-or-self node step before the
  written child step. Unify explicit and abbreviated child forms, deduplicate
  overlapping result identities in document order, and retain every actual
  traversal visit in work accounting.
- [x] Execute complete QT3 `Axes080` through `Axes083` and selected `Axes084-1`
  through `Axes084-4` by composing internal descendant expansion with typed
  attribute steps. Preserve attribute identity and order, exclude namespace
  declarations, unify explicit and abbreviated forms, and keep `Axes084-5`
  outside the passing denominator pending its distinct predicate semantics.
- [x] Execute QT3 `Axes084-5` as a separate text-predicate slice. Admit only a
  final zero-argument `normalize-space()` predicate over text nodes, use XML
  whitespace and effective-boolean-value semantics, retain exact candidate
  work charging, and import the larger pinned source under explicit limits.
- [x] Execute 25 unchanged AxisStep static-syntax cases: `Axes088`,
  `K2-Axes-5` through `-17`, `-29`, `-34`, `-35`, `-37`, `-46`, `-77`, `-90`,
  `-91`, and `K2-Axes-95` through `-97`. Preserve `XPST0003` and expression locations for malformed
  namespace wildcards, trailing empty steps, a bare descendant separator,
  unknown axes, an invalid axis node test, and incomplete QNames while leaving
  valid unimplemented forms classified as unsupported.
- [x] Execute unchanged `K2-Axes-55` and `K2-Axes-56` through an explicit
  empty-sequence path origin. Preserve the native successful `any-of`
  alternative, produce no nodes for both attribute and child steps, and prove
  the otherwise supplied document receives zero XPath visits without claiming
  general sequence expressions, static typing, or absent-context semantics.
- [x] Execute ten unchanged atomic path/type-error cases through bounded static
  recognition above the node-only path parser. Preserve `XPTY0019` for a
  statically atomic left operand of `/`, `XPTY0020` for an axis applied directly
  to an atomic predicate context, native `any-of` alternatives, and exact
  source locations without claiming a general XPath static type system.
- [x] Execute five unchanged missing-dynamic-context cases through a bounded
  source-free classifier. Preserve `XPDY0002`, native `any-of` alternatives,
  relative reserved-word name tests, and exact expression locations without
  claiming a general sequence evaluator or optimizer short-circuit behavior.
- [x] Admit and execute the complete two-case XSLT30 `fn/deep-equal` denominator
  through positioned descendant attribute/comment selection and charged,
  pairwise node comparison. Preserve distinct XDM identity while comparing
  attribute expanded names and values or comment values; broader node kinds,
  sequences, typed values, and collations remain outside this slice.
- [x] Execute the complete five-case QT3 `fn-deep-equalint2args` group through
  checked `xs:int` constructors and source-free numeric value comparison,
  including both argument orders and the type's lower and upper bounds. Keep
  cross-type promotion, floating-point/NaN rules, arbitrary sequences, and the
  remainder of the 263-case QT3 function test set explicitly unclaimed.
- [x] Execute the complete five-case QT3 `fn-deep-equalintg2args` group through
  checked `i128` values, including its 18-digit lower, middle, and upper
  operands in both orders. Treat this as a bounded `xs:integer` subset rather
  than an arbitrary-precision implementation claim.
- [x] Execute the complete five-case QT3 `fn-deep-equaldec2args` group through
  normalized coefficient-and-scale values with checked `i128` coefficients.
  Preserve exact decimal equality without binary floating point and leave
  arbitrary precision, cross-type promotion, floats, doubles, and NaN outside
  this slice.
- [x] Execute the complete five-case QT3 `fn-deep-equallng2args` group through
  checked signed 64-bit values, with a focused upper-bound control that rejects
  an out-of-range constructor value. Do not infer numeric promotion or general
  constructor support from this group.
- [x] Execute the complete five-case QT3 `fn-deep-equalusht2args` group through
  checked unsigned 16-bit values. Prove the derived type boundary by accepting
  `65535` and rejecting both `-1` and `65536`, without claiming the other
  derived-integer families.
- [x] Execute the complete five-case QT3 `fn-deep-equalnint2args` group through
  checked `i128` values constrained to be strictly below zero. Prove the
  derived type boundary by accepting `-1` and rejecting both `0` and `1`, while
  leaving arbitrary precision and the other derived-integer families
  unclaimed.
- [x] Execute the complete five-case QT3 `fn-deep-equalpint2args` group through
  checked `i128` values constrained to be strictly above zero. Prove the
  derived type boundary by accepting `1` and rejecting both `0` and `-1`, while
  leaving arbitrary precision and the other derived-integer families
  unclaimed.
- [x] Execute the complete five-case QT3 `fn-deep-equalulng2args` group through
  checked unsigned 64-bit values. Supplement its advertised upper fixture by
  accepting the actual `18446744073709551615` boundary and rejecting both `-1`
  and `18446744073709551616`, without inferring cross-type promotion or general
  constructor support.
- [x] Execute the complete five-case QT3 `fn-deep-equalnpi2args` group through
  checked `i128` values constrained at or below zero. Distinguish its inclusive
  boundary by accepting both `-1` and `0` while rejecting `1`, and retain the
  arbitrary-precision and cross-type exclusions.
- [x] Execute the complete five-case QT3 `fn-deep-equalnni2args` group through
  checked `i128` values constrained at or above zero. Distinguish its inclusive
  boundary by accepting both `0` and `1` while rejecting `-1`, and retain the
  arbitrary-precision and cross-type exclusions.
- [x] Execute the complete five-case QT3 `fn-deep-equalsht2args` group through
  checked signed 16-bit values. Accept `-32768` and `32767` while rejecting
  `-32769` and `32768`, without inferring cross-type promotion or general
  constructor support.
- [x] Execute QT3 `fn-deep-equal-mix-args-001` through `-010` as an explicit
  first tranche. Preserve integer-sequence order, string value and case,
  singleton parentheses, the distinction between an empty string and an empty
  sequence, and nested/whitespace empty-sequence forms. Charge length and
  reached item comparisons locally, and leave cases 011 through 031 unselected
  until their URI, promotion, floating-point, boolean, and date/time semantics
  are deliberately admitted.
- [x] Extend the explicit mixed tranche through
  `fn-deep-equal-mix-args-014`. Preserve URI, string, integer, and decimal type
  identity while admitting URI/string value comparison and exact
  integer/decimal equality without binary floating point. Leave cases 015
  through 031 unselected pending their floating-point, boolean, and date/time
  semantics.
- [x] Extend the explicit mixed tranche through
  `fn-deep-equal-mix-args-022`. Retain float and double IEEE values, promote
  integer/decimal/float operands according to the selected comparisons,
  preserve the rounded float value when promoted to double, and implement the
  paired-NaN deep-equal rule. Leave boolean and date/time cases 023 through 031
  unselected.
- [x] Extend the explicit mixed tranche through
  `fn-deep-equal-mix-args-027`. Normalize the four valid XML Schema boolean
  lexical forms and compare them with typed `true()`/`false()` function values,
  while rejecting unrecognized boolean strings. Leave date/time cases 028
  through 031 unselected.
- [x] Complete `fn-deep-equal-mix-args-028` through `-031` and therefore the
  entire 31-case mixed group without denominator loss. Validate the admitted
  whole-second, timezone-free date/time forms and retain their typed identity
  against equal strings. Keep timezones, fractional seconds, 24:00:00, and
  negative or expanded years explicitly outside the claim.
- [x] Discharge the ADR-0004 deep-equal cohesion review by extracting a private
  atomic owner for representations, lexical validation, sequence parsing,
  numeric promotion, and item comparison. Retain recognition, node behavior,
  diagnostics, and exact invocation work charging in the parent, with one-way
  dependency and representation-local tests that need no XDM/control context.
- [x] Execute the complete five-case QT3 `fn-deep-equalflt2args` group through
  retained IEEE `f32` values. Conserve the exact upstream and overlay
  denominators and charge singleton sequence length plus the reached item,
  without claiming the complete XML Schema floating lexical space.
- [x] Execute the complete five-case QT3 `fn-deep-equaldbl2args` group through
  retained IEEE `f64` values under the same denominator and work-accounting
  controls, without inferring general floating arithmetic or lexical coverage.
- [x] Execute QT3 `K-SeqDeepEqualFunc-1` through `-3` as an explicit static
  arity-error tranche. Retain `XPST0017` separately from private
  `FXXP0005 / invalid`, conserve expression/stylesheet locations, and keep a
  valid but unimplemented three-argument collation call classified unsupported.
- [x] Extend the static arity tranche with QT3 `K2-SeqDeepEqualFunc-8` and
  `-9`, retaining their independent native identities and `XPST0017` source
  locations without admitting adjacent invocation-clock or XQuery cases.
- [x] Execute QT3 `K-SeqDeepEqualFunc-6` and `-8` through `-11` as an explicit
  second tranche: admit the standard codepoint collation URI and verify
  paired NaN across both float/double argument orders. Keep unknown and empty
  collations unsupported and leave outer XPath operator cases unselected.
- [x] Execute QT3 `K-SeqDeepEqualFunc-64` and `-65` through the standard HTML
  ASCII case-insensitive collation. Preserve sequence order, ASCII-only case
  folding, and reached-item work charges without admitting host-defined,
  locale, Unicode-folding, function-item, or node-collation behavior.
- [x] Execute QT3 `K-SeqDeepEqualFunc-4` and `-5` through their native `any-of`
  assertions by preserving `FOCH0002` for an unknown literal collation URI and
  `XPTY0004` for an empty third argument. Keep these standard invalid outcomes
  distinct from unimplemented dynamic or host-defined collation semantics.
- [x] Execute QT3 `fn-deep-equal-arrays-1` through `-7` through a bounded safe
  literal-array representation. Preserve the difference between arrays and
  atomic items, array member sequences, nested arrays, and empty-array versus
  one-empty-member semantics; leave maps, node members, array update functions,
  general constructors, and public XDM representation decisions unselected.
- [x] Extend literal-array comparison through `fn-deep-equal-arrays-11`, `-12`,
  and `-14` through `-17`. Add codepoint string members and flattened top-level
  sequence constructors while preserving nested member boundaries and exact
  early-exit charging; keep UCA collation, node members, and update functions
  outside the admitted representation.
- [x] Execute `fn-deep-equal-maps-1` through `-4` plus array cases `-8` and
  `-9` through the renamed private composite-value owner. Compare empty and
  integer-keyed maps independently of entry order, preserve empty-sequence and
  boolean values, reject duplicate literal keys, and retain maps nested inside
  arrays without selecting general map keys or a public XDM representation.
- [x] Execute `fn-deep-equal-maps-5` through `-10` with normalized exact
  finite numerics and explicit NaN same-key behavior. Preserve integer,
  decimal, and exponent-form equality for keys and array values, while keeping
  array member order semantic and declining broader floating arithmetic,
  string keys, node values, or a public hashing strategy.
- [x] Execute `fn-deep-equal-arrays-18` and `fn-deep-equal-maps-15` by folding
  bounded literal `array:put`, `array:remove`, and `map:remove` calls into the
  private composite oracle. Preserve one-based positions, member sequences,
  input immutability, exact recursive charging, and explicit rejection of
  invalid positions, missing keys, or nonliteral operations.
- [x] Execute `cbcl-deep-equal-008` by retaining untyped-atomic identity,
  checked year-month-duration months, and exact literal decimals. Preserve the
  second-item type mismatch and exact early-exit charge without admitting
  implicit untyped casts or general duration arithmetic.
- [x] Execute QT3 `K-SeqDeepEqualFunc-7`, `-12` through `-16`, and `-18`
  through `-20` through a private boolean-composition owner shared by direct
  XPath and stylesheet execution. Preserve exact inner work charges and early
  mismatch, admit quoted integer/decimal constructor lexicals and recursive
  three-item sequence tails, and leave QName/binary/index-of cases unselected.
- [x] Execute QT3 `K-SeqDeepEqualFunc-25` through `-31` and `-36` through
  `-46` as an exact 18-case ordered/empty-sequence tranche. Preserve item order,
  first/second/third early mismatch, empty-item flattening, and length-only
  short-circuit charges while leaving QName, binary, and `index-of` cases
  unselected.
- [x] Execute QT3 `K-SeqDeepEqualFunc-47` through `-51` as the adjacent direct
  unequal-length tail. Prove each case stops after its single length-decision
  charge and leave range/`reverse` compilation to a separate checkpoint.
- [x] Execute QT3 `K-SeqDeepEqualFunc-52` through `-55` by compile-time folding
  source-free literal integer ranges and `reverse`. Bound retained folded
  ranges to 1,024 items, preserve empty descending-range semantics, and leave
  dynamic ranges and runtime range accounting unclaimed.
- [x] Execute QT3 `K-SeqDeepEqualFunc-17` and `-21` through an expanded-name
  QName atomic value with lexical validation and prefix-insensitive equality.
  Admit the required `3e2` double literal, preserve exact early-exit work, and
  leave binary constructors and `index-of` unselected.
- [x] Execute QT3 `K-SeqDeepEqualFunc-22` through `-24` through decoded
  hex/base64 binary atomic values. Validate lexical shape and padding, compare
  retained bytes, and preserve first/second/third-position work charges while
  leaving `index-of` composition unselected.
- [x] Execute QT3 `K-SeqDeepEqualFunc-32` through `-35` by compile-time folding
  their bounded, source-free literal `index-of` calls to ordered one-based
  positions. Charge only the retained deep-equal invocation work and leave
  dynamic/context-dependent search and runtime search budgets unclaimed.
- [x] Execute QT3 `K2-SeqDeepEqualFunc-35` as a narrow string-derived atomic
  comparison. Validate the admitted ASCII `xs:NCName` lexical form and compare
  its value with `xs:string` without claiming the other string-derived types,
  Unicode NCNames, general casting, or schema-aware typed nodes.
- [x] Admit all four XSLT30 `expr/for` cases with their native environments,
  stylesheets, entry metadata, XML assertions, and explicit unsupported
  dispositions before implementing sequence semantics.
- [x] Execute native `for-001` through ordered distinct-value binding,
  comparison/path selection, source-node identity preservation, and
  `xsl:sequence` result construction against its complete upstream assertion.
- [x] Execute source-free native `for-002` through an invocation-local
  initial-template entry, ordered integer bindings/addition, an independently
  bounded XPath-operation domain, and `xsl:value-of` separator semantics.
- [x] Execute native `for-003` with the outer focus preserved across its
  binding, empty-sequence multiplication, and the integer zero result of
  `sum(())`, while refusing non-empty numeric multiplication.
- [x] Execute native `for-004` with bound-variable attribute navigation,
  checked exact-decimal multiplication and aggregation, and only the required
  `'0.00'` formatting picture. The complete four-case denominator now passes.
- [x] Admit the complete nine-case XSLT30 `expr/castable` denominator: seven
  selected cases, two explicit schema-aware profile exclusions, four
  admission-time engine gaps, and three harness gaps.
- [x] Execute native `castable-001` through controlled atomization and owned
  built-in lexical castability, retaining inherited prefixed namespaces on its
  literal result. The selected denominator is one pass, three engine gaps, and
  three harness gaps.
- [x] Execute native `castable-002` through explicit built-in casts and typed
  invocation-local variables. The selected denominator is two passes, two
  engine gaps, and three harness gaps.
- [x] Execute native `castable-003` through an explicit value-aware conversion
  matrix for boolean, integer, decimal, float, and double. The selected
  denominator is three passes, one engine gap, and three harness gaps.
- [x] Execute native `castable-004` through explicit duration-family
  castability and its inline XML assertion. The selected denominator is four
  passes, no engine-classified gaps, and three harness gaps.
- [x] Resolve the source-free standard initial-template entry for
  `castable-007` through `castable-009` to a namespace-canonical compiled
  identity, inventory both compound assertion predicates, and classify all
  three at their actual `xsl:function` engine boundary. The selected denominator
  is four passes, three engine gaps, and no harness gaps.
- [x] Admit all 28 XSLT30 `expr/data-manipulation` cases with their referenced
  inline/file-backed environments and XML assertions, then execute `001`
  through `028` using ordered conditional instructions, checked exact-rational
  predicates, nonnegative `round()`, exact-decimal formatting, and
  invocation-local materialization of top-level variable/parameter text
  defaults and source-derived node sequences. The complete denominator passes;
  host parameter overrides, arbitrary global expressions, forward references,
  and general dependency ordering are not claimed.
- [x] Establish representative diagnostic codes and source spans across XML
  and compiler/XPath phases without claiming a complete public catalog.
- [x] Preserve one compiler/XPath resource and byte span through the private
  host-neutral workbench facade as owned structured fields, proving the caller
  need not parse display detail.
- [x] Carry that optional location through the isolated-worker and native
  workbench envelope shapes and managed decoders, bumping the explicitly
  unstable native ABI to version 1. Prove native serialization exactly and run
  the real ASP.NET diagnostic-parity endpoint with exact location and worker
  reuse assertions.
- [x] Retain parser-owned XML offsets and ranges through prepared-input
  construction and the host-neutral facade. Prove the malformed-source `7..7`
  point location through the real isolated ASP.NET diagnostic endpoint without
  parsing display detail.
- [x] Provide a read-only semantic inspection snapshot for the implemented
  compilation slice without exposing private parser, arena, or IR types,
  providing evidence for AR-0005.
- [x] Import the first licensed, versioned, integrity-checked suite selection.

Exit criterion: a published test report identifies supported, unsupported,
failed, and harness-error cases without an unqualified conformance claim.

## M3 -- Reusable stylesheet engine

- [x] Separate reusable compiled stylesheet state from dynamic transform state
  in the private reference path; its public representation remains unstabilized.
- [ ] Add template selection, built-in rules, parameters, variables, and output
  behavior required by the accepted profile.
- [x] Retain literal `xsl:output/@media-type` as immutable stylesheet-derived
  serialization metadata, include it in bounded semantic inspection, and prove
  the existing golden XML bytes remain unchanged. Keep stable host-visible
  result metadata and broader serialization conformance open.
- [x] Admit the complete 232-case XSLT30 `decl/output` denominator under a
  first-party set-level overlay, conserve seven serialization assertion
  families and all environment/resource references, and seal each case's
  engine inputs into an independently bounded memory snapshot. Default cases
  to harness-unsupported and unexecuted until an explicit comparator-backed
  override earns another disposition.
- [x] Execute XSLT30 `output-0128` through the standard `xsl:transform` synonym,
  retained `include-content-type` metadata, namespace-qualified XML element
  serialization, and a case-specific canonical-LF `assert-serialization`
  comparator.
- [x] Execute adjacent XSLT30 `output-0129` through text-method descendant-text
  concatenation without markup, escaping, an XML declaration, or injected
  content-type metadata.
- [x] Execute XSLT30 `output-0110` and `output-0121` through the bounded
  XML-compatible XHTML lane, proving explicit declaration omission and default
  retention with namespace-preserving exact comparisons.
- [x] Execute `output-0105` and `output-0109` as opposing XHTML root-name
  controls: explicit XHTML serialization of a null-namespace `html`, and an
  empty XHTML-namespaced `html` with paired tags. The output ledger then
  recorded 39 passes and 193 visible harness gaps.
- [x] Execute `output-0106`, `output-0106a`, and `output-0106b` through bounded
  indentation of non-empty element-only child sequences for the XSLT 2.0 and
  3.0 true lexicals. Preserve text-only and mixed content without injected
  whitespace. The output ledger now records 42 passes and 190 visible harness
  gaps.
- [x] Execute `output-0142` through `output-0145` and `output-0151` through
  serializer-owned XHTML Content-Type insertion/replacement. Use explicit or
  default media type, emit UTF-8, replace one existing matching meta without
  mutating the semantic result, and preserve paired tags for non-empty-content
  XHTML elements. The predefined `xml` namespace is also recognized during
  attribute serialization, admitting the XHTML `xml:lang` and apostrophe
  control in `output-0104`. XHTML `script` and `style` content retains
  XML-compatible escaping through `output-0107` and `output-0108`, without
  implying HTML raw-text behavior. XHTML CDATA terminator handling is conserved
  through `output-0114` and `output-0115`, including a split into adjacent
  sections for `]]>`. Bounded XHTML system, public-only, and paired PUBLIC
  DOCTYPE behavior is covered by `output-0111` through `output-0113`. The
  XHTML empty-content list, non-minimized attribute control, paired-body
  control, and expanded-name CDATA control are covered by `output-0116` through
  `output-0117` and `output-0119` through `output-0120`. `output-0118` adds a
  static-target, literal-data processing-instruction result while preserving
  public-only DOCTYPE inertia. Explicit and inferred
  XHTML content-type insertion is covered by `output-0126` and `output-0130`.
  The output adapter now also honors native initial-mode entry for
  `output-0155a` and `output-0155b`, validating but ignoring
  `escape-uri-attributes=yes` under explicit XML output as the standard
  requires. Six invalid boolean controls, `output-0197` through
  `output-0199a`, now preserve their exact native `XTSE0020` alternative,
  invalid category, and source location. The output ledger now records 72
  passes and 160 visible harness gaps.
- [x] Extend that declaration lane through `output-0110a`, `output-0110b`, and
  `output-0148` through `output-0148b`, accepting whitespace-normalized XSLT
  3.0 boolean lexicals without widening XSLT 2.0 beyond `yes`/`no`.
- [x] Execute `output-0166` with retained UTF-8 and no-BOM metadata, rejecting
  non-UTF-8 encodings and BOM emission until a byte result lane owns those
  semantics.
- [x] Execute `output-0165` through the private bounded byte-result lane. Emit
  the exact three-byte UTF-8 byte-order mark before the declared XML result,
  charge it against both result limits and invocation work, and keep the normal
  string lane's BOM rejection intact. Preserve byte output as private evidence,
  not a selected public result contract. The output ledger now records 12
  passes and 220 visible harness gaps.
- [x] Execute adjacent text-method cases `output-0171` and `output-0172` as an
  exact byte pair. Prove the byte-result lane prepends `EF BB BF` only when
  requested and otherwise returns the same five `Hello` bytes, without adding
  a general regex comparator or weakening the string lane. The output ledger
  now records 14 passes and 218 visible harness gaps.
- [x] Execute the six XHTML BOM lexical variants `output-0136` through `0137b`
  through the same private byte lane and XML-compatible XHTML serializer.
  Compare the complete BOM/declaration/namespace/body byte sequence for XSLT
  2.0 `yes`/`no` and XSLT 3.0 `true`/`false`/`1`/`0`, without admitting broader
  XHTML rules or regex assertions. The output ledger now records 20 passes and
  212 visible harness gaps.
- [x] Execute `output-0139` as a non-ASCII UTF-8 byte control over the existing
  XML-compatible XHTML lane. Compare the complete declaration, namespace, and
  body bytes and require `Á` to remain the exact `C3 81` sequence, without
  claiming UTF-16, normalization, or general encoding support. The output
  ledger now records 21 passes and 211 visible harness gaps.
- [x] Execute `output-0168` and `output-0170` by retaining
  `normalization-form="none"` in compiled output metadata and preserving the
  exact decomposed `41 CC 81` UTF-8 bytes for XML and text methods. The later
  exact-pinned Unicode-normalization tranche executes NFC siblings `0167` and
  `0169` without weakening these byte-preserving controls. At this checkpoint,
  the output ledger recorded 23 passes and 209 visible harness gaps.
- [x] Execute `output-0131` through the file-backed branch of its native
  `any-of`, preserving two top-level XHTML elements, authored interstitial text,
  ordering, and namespace fixup. Admit only the exact composite assertion shape
  and keep `output-0173`'s declaration merging, standalone, and CDATA semantics
  as separate work. The output ledger now records 24 passes and 208 visible
  harness gaps.
- [x] Execute standalone cases `output-0149` through `0150b` plus `0152` by
  retaining canonical `yes`, `no`, or `omit` metadata and emitting the exact
  XML declaration for the XML-compatible XHTML lane. Preserve XSLT 2.0 and 3.0
  lexical distinctions and treat this as one prerequisite for `output-0173`,
  not admission of declaration merging or CDATA. The output ledger now records
  31 passes and 201 visible harness gaps.
- [x] Execute `output-0147` by composing `normalization-form="none"` with the
  XML-compatible XHTML byte lane and preserving the exact decomposed
  `41 CC 81` sequence. The later exact-pinned Unicode-normalization tranche
  executes NFC sibling `0146` against its native assertion. At this checkpoint,
  the output ledger recorded 32 passes and 200 visible harness gaps.
- [x] Execute `output-0122` and `output-0173` through bounded unnamed
  output-declaration merging. Allow non-overlapping scalar properties, reject
  repeated scalar properties as `FXST1018`, union expanded
  `cdata-section-elements` names, and serialize selected immediate text as safe
  CDATA while retaining ordinary escaping elsewhere. Keep named declarations
  and import-precedence merging outside the slice. The output ledger now records
  34 passes and 198 visible harness gaps.
- [x] Execute `output-0138` as an expanded-name CDATA selection control across
  unprefixed names, two prefixes bound to one namespace, and a same-local-name
  element in a different default namespace. Check every native `all-of`
  fragment without admitting a general regex comparator. The output ledger now
  records 35 passes and 197 visible harness gaps.
- [x] Execute `output-0153` and `output-0156` as XML-compatible declaration
  controls. Retain and emit only admitted serialization version `1.0`, reject
  other versions as `FXST1021`, and prove `include-content-type="no"` remains
  inert under XML output. The output ledger now records 37 passes and 195
  visible harness gaps.
- [x] Execute `output-0127` through its upstream `all-of` using a harness-owned
  literal-plus-required-whitespace serialization matcher that rejects every
  unadmitted regex operator. Preserve 221 visible harness gaps and make no broad
  claim from the eleven passed cases.
- [x] Execute XSLT30 `template-001` through built-in document dispatch,
  comment-node selection, and an isolated named mode while retaining the other
  four unsupported cases in the six-case denominator.
- [x] Execute XSLT30 `template-002/003` through processing-instruction and
  general child-node tests, retaining mode isolation and exact-pattern
  precedence. Four of the six denominator cases now pass.
- [x] Execute XSLT30 `template-004` through attribute-axis selection and an
  exact attribute pattern without adding attributes to the child axis. Five of
  the six denominator cases now pass.
- [x] Execute XSLT30 `template-005` through statically resolved named templates,
  invocation-local parameters, conditional equality, calls, and bounded
  recursion. The complete six-case denominator now passes.
- [x] Execute selected XSLT30 `conflict-resolution-0101` through built-in
  document dispatch and competing exact-name, element-wildcard, and any-node
  template rules. Add only the required attribute-free `xsl:text` character
  content, preserving explicit whitespace and rejecting element content; do
  not infer complete apply-templates conflict-resolution conformance.
- [x] Execute selected XSLT 3.0 `conflict-resolution-0102c` and `0104c` as a
  paired source-order control. Give element wildcard and any-node patterns equal
  default priority and select the last declaration, while preserving the higher
  exact-name and path bands. Do not claim explicit priority, recovery policy,
  warnings, or older-edition ambiguity behavior.
- [x] Execute `conflict-resolution-0106` through compiled bounded signed-integer
  priority and the built-in attribute string-value rule. Preserve the child-axis
  boundary that keeps `node()` from matching attributes. Bounded fractional
  priority is admitted separately by `1701`; keep arbitrary-precision priority,
  root-pattern priority and legacy equal-rank recovery/error profiles outside
  the admitted slice. Distinct-priority chains are admitted separately by
  `1201`; XSLT 3.0 equal-rank use-last selection is admitted by `1202c`.
- [x] Execute `conflict-resolution-0107`, `0108c`, and `0110c` through retained
  non-simple default priority. Add only the exact unnamespaced
  `element[@attribute]` presence pattern, charge inspected attributes, and prove
  source-order selection against an equal-priority path in both directions;
  keep general predicate expressions unsupported.
- [x] Execute `conflict-resolution-0112` through an exact compiled `//*`
  specialization whose element applicability retains non-simple default
  priority. Keep `//QName`, arbitrary descendant patterns, union patterns, and
  the general pattern grammar unsupported.
- [x] Execute `conflict-resolution-0201` through the exact unnamespaced
  `element[@attribute='literal']` pattern, retaining its literal and non-simple
  priority in compiled state and charging inspected attributes. Support both
  shared and case-local suite environments; keep `!=`, namespaces, general
  comparisons, and arbitrary predicates unsupported.
- [x] Execute XSLT 3.0 `conflict-resolution-0401c` through statically resolved
  prefixed exact-name and namespace-wildcard patterns with equal explicit
  integer priority and last-declared selection. Reject unbound prefixes;
  implicit quarter-step wildcard priority is admitted separately by `1701`.
  Do not infer XSLT 2.0 recovery/error behavior from either case.
- [x] Execute `conflict-resolution-1701` through exact fixed-point fractional
  priority plus the `*:NCName` local-name wildcard, matching multiple source
  namespaces and selecting rules by priority and declaration order. Keep more
  than six fractional places and broader pattern grammar explicitly unsupported.
- [x] Execute `conflict-resolution-1801` by lowering the exact `element()` kind
  test to typed any-element applicability with its `-0.5` default priority.
  Admit `name(.)` only for unnamespaced context nodes until lexical QName prefix
  identity is represented faithfully.
- [x] Execute `conflict-resolution-1601` by retaining the direct path for a lone
  unmoded `/` template and migrating competing root rules into ordinary typed
  document-node selection. Preserve exact `-0.5` default priority, explicit
  bounded priorities, and declaration order; keep cross-module conflicts out.
- [x] Execute `conflict-resolution-1602` and `1603` through typed exact-name and
  wildcard `document-node(element(...))` patterns with charged document-element
  inspection and exact default priorities `0` and `-0.5`. Admit
  distinct-priority duplicate competition; XSLT 3.0 equal-rank selection is
  admitted separately by `1202c`. Keep cross-module precedence out.
- [x] Execute `conflict-resolution-1201` through typed parameter-free
  `xsl:next-match`, retaining private current-template identity and selecting
  successively lower-ranked applicable rules before built-in fallback. Admit
  distinct-priority duplicate shapes; equal-rank XSLT 3.0 continuation and
  parameters are admitted separately. Keep imports out.
- [x] Execute XSLT 3.0 `conflict-resolution-1202c` with use-last selection for
  equal-priority applicable rules and same-rank `xsl:next-match` continuation.
  Recognize but do not execute `xsl:fallback` content on the supported
  instruction; keep legacy recover/error profiles, mode controls, warnings,
  root duplicates, and cross-module precedence out.
- [x] Execute the six adjacent explicit-recovery variants `0102a`, `0104a`,
  `0108a`, `0110a`, `0401a`, and `1202a` only after verifying each case's
  native `on-multiple-match=recover` dependency. Reuse the already-evidenced
  use-last paths without selecting a general XSLT 1.0/2.0 compatibility profile
  or admitting the six corresponding `XTRE0540` error variants. The complete
  apply-templates ledger now records 40 passes and 10 visible not-run cases.
- [x] Execute `conflict-resolution-1204` with its principal and one relative
  import admitted into the sealed snapshot. Preserve import precedence ahead of
  template priority so `xsl:next-match` walks principal priorities
  `5 → 4 → 3 → 2` before the imported priority-`25` rule. Generalize only the
  private corpus adapter to catalog-declared secondary stylesheets. The ledger
  now records 41 passes and 9 visible not-run cases.
- [x] Execute the six explicit error-on-multiple-match variants `0102b`,
  `0104b`, `0108b`, `0110b`, `0401b`, and `1202b` through an invocation-local
  private policy. Report concrete dynamic error `XTDE0540` for ambiguity at the
  highest eligible import precedence and priority, including an ambiguity
  reached through `xsl:next-match`, while proving lower-ranked ties do not
  preempt a unique higher rule. Keep policy selection out of the public and
  host adapters. The apply-templates ledger now records 47 passes and 3 visible
  not-run cases.
- [x] Execute `conflict-resolution-1205` with typed non-tunnel integer and
  atomic-variable `xsl:with-param` arguments on `xsl:apply-templates` and
  `xsl:next-match`. Keep values invocation-local, retain result attributes
  separately from children, and admit only unnamespaced literal values and
  whole-value variable AVTs; keep broader parameter and AVT semantics out.
- [x] Execute `conflict-resolution-0601` with an invocation-time global integer
  default in a typed attribute-variable match predicate. Admit element
  `xsl:copy` with leading static unnamespaced text attributes, preserve source
  expanded names and namespace declarations, and do not implicitly copy source
  attributes or children. Keep general comparisons, pattern expressions,
  computed attributes, and namespace fixup out.
- [x] Execute `conflict-resolution-0501` and `0502` by lowering their exact
  `current()` and quantified surface forms into one typed, charged,
  unnamespaced same-named-child pattern operation. Keep general `current()`,
  quantified expressions, namespace-sensitive lexical QName comparison, and
  the positional variants out.
- [x] Execute `conflict-resolution-0503` through a separate typed, charged
  same-named-parent relation that preserves the final candidate as `current()`
  while the predicate context is its parent. Keep general multi-step current
  patterns and namespaced lexical QName comparison out.
- [x] Execute `conflict-resolution-1501` through a typed filtered-parent
  position relation, distinguishing second among matching-name siblings from
  second element child. Conserve both upstream `all-of` XPath assertions and
  interpret them through a bounded case oracle; keep general positional
  patterns and assertion XPath evaluation out.
- [x] Execute `conflict-resolution-1101` with invocation-local literal
  temporary-tree construction, bounded integer template defaults, and
  non-tunnel parameter preservation through built-in document/element rules.
  Keep general local constructors, temporary-tree navigation, compatibility
  mode, tunnel behavior, modes, and `xsl:apply-imports` out.
- [x] Execute `conflict-resolution-1102` through mode-aware temporary document
  focus and the no-import `xsl:apply-imports` fallback to the built-in rule,
  preserving non-tunnel parameters and current mode. Keep `xsl:import`, import
  precedence, lower-precedence user rules, tunnel parameters, and compatibility
  behavior out.
- [x] Verify `apply-templates-001` and `002` as structured `XTTE0510` outcomes
  by proving their literal integer-range focus cannot supply the required node
  sequence. Keep general `xsl:for-each`, arbitrary static typing, atomic-item
  transformation, and dynamic focus execution out.
- [x] Execute `conflict-resolution-1001` through typed relative parent steps
  ending in a wildcard attribute-equality variable filter. Verify the upstream
  empty global default and a supplemental non-empty invocation that exercises
  filtered matching and bounded current-element copy. Keep arbitrary
  predicates, namespaced specialized steps, and general `xsl:copy-of` out.
- [x] Execute `conflict-resolution-0901` as a conservation case for typed
  leading-descendant selection, document-order candidate delivery, and retained
  multi-step parent/child match paths. Keep general `current()` patterns,
  range-variable predicates, default-namespaced multi-step paths, current-mode
  propagation, and general pattern grammar outside that conservation slice.
- [x] Conserve the complete ordered 50-case XSLT30 apply-templates denominator,
  its 50 principal plus one secondary stylesheet, and its assertion-shape
  counts. Record 34 explicit passes and 16 default not-run dispositions without
  converting unexecuted cases into engine failures or an aggregate conformance
  claim.
- [x] Execute `conflict-resolution-0701` through inherited
  `xpath-default-namespace` for simple unprefixed element patterns and child
  selections, retaining expanded names through runtime comparison. Reject
  broader default-namespaced paths explicitly until the typed path
  representation owns namespace resolution throughout.
- [x] Execute `conflict-resolution-0702` with
  `xsl:xpath-default-namespace` on a literal result element as descendant
  static context only. Preserve required namespace declarations while proving
  the XSLT control attribute never becomes a result attribute; keep ordinary
  literal attributes and other control attributes unsupported.
- [x] Execute `conflict-resolution-0703` with stylesheet-wide
  `xpath-default-namespace` inheritance for simple element patterns and child
  selection while keeping unprefixed attribute selection and matching in no
  namespace. Do not infer general default-namespaced path support.
- [x] Execute `conflict-resolution-0801` by retaining current mode across
  named-template calls and dispatching mode-qualified document-node patterns
  through the ordinary typed selector. Keep `#default` multi-mode declarations,
  mode QNames, and broader mode declarations outside this slice.
- [x] Conserve the complete 169-case XSLT30 `attr/mode` denominator under a
  first-party default-not-run overlay and execute `mode-0105` and `mode-0106`.
  Resolve prefixed mode names through stylesheet namespace context into
  expanded identities while keeping unprefixed modes in no namespace, proving
  `foo:a` and `a` remain distinct. This is the first prerequisite for
  `conflict-resolution-1401`; temporary-tree path navigation, union patterns,
  and temporary-focus `xsl:next-match` remain separate work.
- [x] Execute `mode-0107` by resolving a bare global temporary-tree variable to
  its document node, dispatching its moded document rule, and retaining
  temporary focus for the nested unmoded apply-templates instruction. Unify
  local/global temporary-root selection with lexical shadowing while keeping
  deeper temporary paths, text nodes, and temporary-focus continuation out.
- [x] Execute `mode-0108` through the bounded
  `xsl:for-each select="$temporary-tree"` form, establishing temporary document
  focus for its body while retaining the surrounding current-template and mode
  context. Keep general `xsl:for-each` selection and iteration outside this
  slice.
- [x] Execute `mode-0101` through `mode-0104` and `mode-0201` through
  `mode-0701` as one basic dispatch tranche. Preserve explicit versus unnamed
  mode isolation, active mode through built-in descent, deliberate unnamed
  mode selection when nested `xsl:apply-templates` omits `mode`, and typed
  comment, processing-instruction, node, and attribute dispatch. Admit a
  template carrying both `name` and `match` into both compiled indexes while
  retaining one shared body semantics. The mode ledger now records 14 passes
  and 155 visible default not-run cases.
- [x] Execute `mode-0801a` through `mode-0801c` against their native
  `on-multiple-match` dependencies. Reuse the private recover/error policy,
  evaluate the common document-rooted `/sss//*` pattern from the document node,
  and retain structured request identity and stylesheet location on concrete
  `XTDE0540`. Do not infer `xsl:mode` warning semantics or a public policy from
  this suite-configured slice.
- [x] Execute `mode-0803`, `mode-0805`, and `mode-0806` through a private
  `xsl:mode` declaration validator. Admit only absent or warning-disabled
  `warning-on-multiple-match` values, preserve native `XTSE0020` for invalid
  boolean lexicals, and keep warning-enabled cases unselected until an owned
  structured warning channel can satisfy their `assert-warning` metadata.
- [x] Execute `mode-0901` by resolving distinct prefixes bound to the same
  namespace into one expanded mode identity, and `mode-1001` by retaining a
  leading underscore and internal dot in admitted unprefixed NCNames. Reuse the
  same compiled mode path rather than adding lexical-name dispatch. The mode
  ledger now records 16 passes and 153 visible default not-run cases.
- [x] Execute `mode-1101` through `mode-1104` through native initial-mode X
  entry, preserving `#current` across named-template calls, whitespace-
  normalized multi-mode declarations, `#all` participation, and one exact
  suite-supplied string parameter override.
- [x] Execute `mode-1105` through a private invocation entry that resolves the
  exact suite-supplied `/doc` selection to a named document element after
  bounded source preparation. Begin mode X at that element, not a fabricated
  document, while leaving the existing document initial-mode entry unchanged.
  Keep general initial-context XPath and any public selector representation
  outside this evidence slice.
- [x] Execute `mode-1201` through `mode-1204` by ranking `#all` and
  mode-specific rules through the shared priority model, independent of source
  order. Preserve the active explicit mode when `xsl:next-match` continues from
  the winning `#all` rule. Together with the multiple-match and declaration-
  validation tranches, the mode ledger now records 31 passes and 138 visible
  default not-run cases.
- [x] Resolve [AR-0016](../Architectural%20Reviews/AR-0016-stylesheet-dependent-source-views-and-whitespace-stripping.md)
  far enough to execute `mode-1301` through one complete stylesheet-dependent
  source view. Preserve the reusable source-derived prepared document, make
  XPath and XSLT consumers observe identical stripped-node semantics, and
  retain the safe reference as the semantic oracle and measure it before
  retaining a specialized representation. The unchanged native case now
  passes. The invocation-owned visibility view now has differential, concurrent
  strip/preserve, generation-overlap, source-copy, child-position, and
  descendant-position evidence. The five-shape timing/concurrency matrix and
  allocator-requested retained/peak probe select it through ADR-0012. There is
  no sibling axis in the current language surface; a future one must receive
  the same effective-sequence control when admitted.
- [x] Execute the independent static-error cases `mode-1444` and `mode-1447` by
  validating `warning-on-no-match` and `typed` through the existing XSLT 3.0
  boolean policy before unsupported runtime semantics. Preserve native
  `XTSE0020`, structured invalid classification, and stylesheet location. The
  mode ledger now records 33 passes and 136 visible default not-run cases.
- [x] Execute `mode-1439` as native dynamic error `XTTE3100` by retaining the
  named `typed="yes"` requirement and rejecting the suite's untyped initial
  source before unrelated shallow-copy fallback. Preserve the stylesheet
  declaration location, keep streaming-dependent `mode-1438` excluded, and do
  not infer schema-aware execution.
- [x] Execute `mode-1431` through the unnamed mode's
  `on-no-match="fail"` policy. Apply normal template selection first, report
  native dynamic error `XTDE0555` at the mode declaration when no rule matches,
  and leave every other built-in policy unsupported pending its own semantic
  slice.
- [x] Execute `mode-1423` as the positive fail-on-no-match control. Prove the
  policy does not preempt matching document, element, or text templates and
  retain a bounded case-specific output ceiling for the native result.
- [x] Execute `mode-1445` and `mode-1446` through a bounded shallow-copy
  built-in policy while preserving inert `typed=false/0` semantics. Retain
  ordinary template precedence and, at that checkpoint, explicitly reject
  comment copying, standalone attribute results, and attribute-template
  interception until subsequent result-tree slices own them.
- [x] Execute `mode-1507` through `mode-1509` by validating mode name and
  visibility constraints before unrelated unsupported template expressions.
  Preserve native `XTSE0020` and keep valid visibility behavior private and
  unsupported. The mode ledger now records 36 passes and 133 visible default
  not-run cases.
- [x] Classify all 26 mode cases with the native `feature="streaming"`
  dependency as explicit ADR-0007 profile exclusions. Verify every dependency
  from the pinned catalog and leave their execution not run. The mode ledger
  now records 36 passes, 26 profile exclusions, and 107 visible default not-run
  cases.
- [x] Classify all 18 mode cases with a native principal package artifact as
  explicit ADR-0007 profile exclusions. Verify each `<test><package>` shape
  from the pinned catalog and leave compilation and execution not run. The mode
  ledger now records 36 passes, 44 profile exclusions, and 89 visible default
  not-run cases.
- [x] Execute `mode-1502` as native static error `XTSE0545` by detecting
  conflicting explicit `on-no-match` values for one expanded mode in one
  module/import precedence. Keep runtime policy, include/import composition,
  and other mode properties outside this exact slice. The mode ledger now
  records 37 passes, 44 profile exclusions, and 88 visible default not-run
  cases.
- [x] Execute `mode-1501` through the existing `#all` and `#current` dispatch
  model while extending private `xsl:copy` execution to the source document,
  text, and processing-instruction contexts exercised beside elements. Preserve
  result-node and byte accounting and keep attribute/comment copy semantics out
  of this exact slice. The mode ledger now records 38 passes, 44 profile
  exclusions, and 87 visible default not-run cases.
- [x] Execute `mode-1108` as one of its native alternative static errors by
  rejecting meaningful children in `xsl:mode` as `XTSE0260` during the
  stylesheet-wide mode prepass. Preserve the declaration location and detect
  the invalid structure before unrelated unsupported accumulator declarations;
  do not infer accumulator support. The mode ledger now records 39 passes, 44
  profile exclusions, and 86 visible default not-run cases.
- [x] Execute `mode-1904` as native static error `XTSE0545` by comparing
  explicit visibility values for one expanded mode at one import precedence.
  Keep executable visibility, include/import composition, and higher-precedence
  overrides outside this exact slice. The mode ledger now records 40 passes,
  44 profile exclusions, and 85 visible default not-run cases.
- [x] Execute `mode-1902` as native dynamic error `XTDE0045` by retaining one
  principal private named-mode declaration into request admission. Inherit
  only supported `method`, `encoding`, and `indent` settings from its single
  imported program, preserve the private declaration location, and leave
  public/final/package visibility composition and `mode-1905` unresolved.
- [x] Execute `mode-1905` by deferring only lower-precedence declarations whose
  attributes are exactly one unprefixed mode name and visibility, then applying
  the principal public visibility at higher precedence. Preserve imported
  templates, keep direct `mode-1904` conflict detection, and do not infer
  general package or multi-property component composition.
- [x] Retain non-whitespace text children in the private attribute-free
  temporary-tree representation and preserve mixed element/text order through
  invocation-owned materialization, built-in traversal, result accounting, and
  serialization. Treat this as one prerequisite for `conflict-resolution-1401`,
  not admission of its deeper path, union-pattern, or temporary `next-match`
  semantics.
- [x] Execute the exact qualified temporary path from
  `conflict-resolution-1401`, retaining its variable identity and expanded-name
  steps, traversing from temporary document roots in stored order, and charging
  inspected nodes to XPath work. Keep union-pattern selection and temporary
  `next-match` as the remaining independent blockers for that pinned case.
- [x] Retain the four qualified path alternatives from
  `conflict-resolution-1401` as typed expanded-name patterns, preserve private
  temporary-node parent links, and evaluate the same pattern form for source
  and temporary trees. Rank temporary matches by import precedence and compiled
  priority so the explicit-priority union rule wins over a later exact-name
  fallback. Keep temporary-focus `xsl:next-match` as the final case blocker.
- [x] Execute complete pinned `conflict-resolution-1401` by preserving
  temporary node focus, current mode, and current-template identity through
  `xsl:next-match`. Select the lower-ranked exact-name fallback through the
  shared import-precedence/priority/declaration-order model and retain the
  XHTML `h2` result. The complete apply-templates ledger now records 48 passes
  and 2 visible not-run cases.
- [x] Admit the positional-focus prerequisite for `conflict-resolution-1301`.
  Carry the actual selected sequence position and size into each matched
  template, compile its exact `member[position() &lt; last()]` and
  `member[position() = last()]` boundary predicates over matching element
  siblings, and expose that dynamic focus only through the exact
  `{position()}` and `{last()}` AVT forms. Preserve the case's ISO-8859-1
  serialization request as a separate unsupported byte-output boundary; do not
  mislabel UTF-8 `String` output as another encoding.
- [x] Execute complete pinned `conflict-resolution-1301` through a private
  bounded byte-result lane. Retain the existing string lane's UTF-8-only
  contract, emit the exact ISO-8859-1 declaration for the case's ASCII result,
  include declaration and body bytes in limits/work accounting, and reject
  non-ASCII output explicitly rather than replacing or mislabeling it. The
  complete apply-templates ledger now records 49 passes and one visible
  schema-aware not-run case.
- [x] Close the complete apply-templates denominator by preserving
  `conflict-resolution-1402`'s native `schema_aware` feature dependency and
  classifying it `excluded-by-profile` under ADR-0007. Do not erase its typed
  attribute-pattern semantics, execute an untyped approximation, or call the
  absence of schema awareness an engine failure. The denominator now records
  49 passes, one profile exclusion, and no default not-run cases.
- [x] Execute `conflict-resolution-0802` with one template in named modes plus
  `#default`, explicit unnamed-mode dispatch, inherited `#current`, and a typed
  default-namespaced descendant-name selection. Keep QName modes, mode
  declarations/properties, and general default-namespaced paths unsupported.
- [x] Admit the complete five-case XSLT30 `misc/initial-mode` denominator,
  preserving each mode identity and expected error or XML assertion through
  bounded snapshots. A focused host-neutral initial-mode entry executes with an
  admitted principal source and rejects unknown compiled mode identity.
- [x] Add invocation-owned atomic parameter values to the private transform
  request and use them to override matching global `xsl:param` defaults without
  mutating reusable compiled state or leaking values between sibling requests.
- [x] Execute pinned `initial-mode-004` with leading template-local parameters,
  expanded QName identity, tunnel/non-tunnel matching, inherited
  `exclude-result-prefixes`, and its ordered child-node/atomic sequence. The
  complete denominator reached one native pass and four explicit engine gaps;
  general parameter defaults/types and tunnel propagation remain open.
- [x] Execute pinned `initial-mode-003` as its expected `XTDE0050` outcome by
  preserving `xsl:output/@indent`, required global-parameter identity, and mode
  identity from matched templates. Indented serialization remains explicitly
  unsupported rather than being silently ignored. The denominator reached two
  native passes and three explicit engine gaps.
- [x] Execute pinned `initial-mode-002` as its expected `XTDE0045` outcome by
  preserving `mode="#all"` declaration metadata without treating it as a
  wildcard that makes every initial mode available. The denominator is now
  three native passes and two explicit engine gaps.
- [x] Execute pinned `initial-mode-001` through a bounded typed local integer
  sequence over `1 to 10`, preserving ten invocation-local atomic values and
  separator semantics without collapsing the sequence into a preformatted
  string. The denominator is now four native passes and one explicit engine
  gap; general `xsl:for-each` and typed sequence conversion remain open.
- [x] Complete pinned `initial-mode-005` by preserving multiple explicit mode
  names, materializing an attribute-free literal global temporary tree per
  invocation under XDM budgets, selecting `$temp/*` without conflating it with
  the principal source, and copying the selected element through the unnamed
  wildcard template. The full five-case initial-mode denominator now passes;
  general temporary-tree navigation and `xsl:copy` construction remain open.
- [x] Execute pinned `mode-0015` by retaining static literal attributes as
  immutable temporary XDM nodes with element parents. Apply shallow-copy and
  shallow-skip over the combined attribute/child focus, keep text-only-copy on
  child descent, and select the unchanged `@bar` rule in `mode="#all"` without
  treating attributes as children. Dynamic temporary attributes, namespace
  nodes, and standalone attribute results remain outside this slice.
- [x] Execute pinned `mode-0007` through one static typed parentless attribute.
  Carry shallow-copy output as private pending construction state consumed by
  the containing literal result element, preserve shallow-skip omission and
  text-only-copy string value, and reject escaped, late, or duplicate result
  attributes instead of representing them as children.
- [x] Execute pinned `mode-0016` by retaining expanded names for namespaced
  literal attributes and the temporary element's required namespace bindings.
  Evaluate the exact `{local-name()}` AVT from temporary attribute focus and
  dispatch the unchanged `@*` rule without claiming selectable namespace nodes
  or general function-valued AVTs.
- [x] Execute pinned `mode-1413` by applying source attribute templates during
  shallow-copy, carrying generated attributes through private pending
  construction state, preserving combined attribute/child focus, and
  continuing an explicit element wrapper through `xsl:next-match`. Retain the
  exact static `xsl:attribute select=". + 1"` boundary rather than inferring a
  general computed-attribute or arithmetic evaluator.
- [x] Execute pinned `mode-1516` and `mode-1517` by retaining overlapping
  equal-priority union alternatives as one template identity, preserving
  parenthesized spelling, and enforcing mode-owned
  `on-multiple-match="fail"` only across distinct selected rules. Keep exact
  child-presence predicates and supplied-source initial-template context
  bounded rather than claiming general pattern predicates or union grammar.
- [x] Execute pinned `mode-1901` by admitting its exact imported stylesheet
  into the sealed snapshot and applying the principal stylesheet's
  higher-precedence `on-no-match="fail"` policy to the imported named-template
  execution. Preserve the native `XTDE0555` outcome without claiming general
  package, visibility, or accumulator component merging.
- [x] Classify `mode-1107a` through `mode-1107c` as streaming-profile
  exclusions from their native streamed-source and static streamable-mode
  metadata. Validate that metadata in the conserved ledger rather than
  treating tree-evaluator output as XSLT streaming evidence.
- [ ] Establish explicit URI/resource resolution and execution limits.
  - [x] Route private principal-stylesheet acquisition through an exact,
    qualified-identity resolver over one sealed snapshot. Charge a fixed attempt
    limit, distinguish denied/missing/invalid/unsupported/limit outcomes, and
    prove that relative references, fragments, Windows paths, and URL-shaped
    logical identities never create ambient filesystem or network authority.
    Relative/base-URI resolution, catalogs, live resolvers, dependency loading,
    and complete execution-limit composition remain open.
  - [x] Open AR-0014 to keep reference/base semantics, host acquisition
    authority, catalog mapping, dependency closure, live resolution, disclosure,
    and bounded policy separate while executable cases are gathered.
  - [x] Inventory the complete pinned XSLT30 `decl/include` denominator: 16
    cases, 16 principal stylesheet references, 34 repeated secondary references,
    and explicit harness-unsupported dispositions without denominator loss.
  - [x] Execute `include-0401` as the first sealed-memory module case, preserving
    its relative base identity, one secondary simplified stylesheet, global
    variable visibility, bounded acquisition, and exact `assert-xml` result.
    - [x] Establish the private RFC 3986/3987 mechanics prerequisite with an
      exact-pinned, license-reviewed `iri-string` adapter: resolve sibling and
      parent references only into the sealed snapshot, reject WHATWG-only
      serialization fallback, and separate fragment selection from acquisition.
    - [x] Keep the private slice deliberately narrow: one include, one
      simplified secondary module, no fragment selection, no ambient fallback,
      and explicit unsupported outcomes for precedence or duplicate-match
      semantics that have not been admitted.
    - [x] Prepare the complete sealed include graph before semantic compilation
      and account independently for reference depth, module occurrences, and
      aggregate dependency bytes. Detect active-path cycles before retention;
      return no graph or compiled program after any limit, resolution, parse, or
      cycle failure. The private production profile now remains depth 2, five
      module occurrences, and 1 MiB rather than selecting public limits.
    - [x] Add a workbench-only explicit dependency/denial input and prove
      `missing-resource` and `denied` remain distinct structured categories
      through the Rust facade plus native and isolated-worker failure envelopes.
    - [x] Admit ADR-0011 and carry one bounded stylesheet dependency plus an
      independent denial flag through both .NET initialization protocols.
      Execute an included module, preserve missing/denied diagnostics, reject
      malformed framing, and advance the native workbench ABI to version 2.
      General dependency collections and resolver profiles remain under AR-0014.
  - [x] Execute `include-0201` as an independent no-module conservation case:
    `xsl:apply-imports` from a matched source element falls through to the
    built-in element/text rules and produces the exact asserted wrapper. Keep
    import precedence and lower-precedence user-rule selection out. The include
    denominator now records 2 passes and 14 visible not-run dispositions.
  - [x] Execute `include-0301` as the first sealed-memory import case. Retain
    import precedence separately from template priority and declaration order,
    and prove three repeated `xsl:apply-imports` calls independently select the
    lower-precedence rule. Keep nested/multiple imports, imported globals,
    imported named/root templates, import/include composition, and parameters
    outside this slice. The denominator now records 3 passes and 13 visible
    not-run dispositions.
  - [x] Execute `include-0202` through an integer `xsl:with-param` on
    `xsl:apply-imports`, lower-precedence template parameter binding, and one
    leading statically named unnamespaced `xsl:attribute` whose value is a bound
    atomic variable. Compile that exact attribute shape into the prepared
    literal-result plan; keep computed names/namespaces, general sequence
    constructors, late attributes, and broader imported declarations out. The
    denominator now records 4 passes and 12 visible not-run dispositions.
  - [x] Execute `include-0105` through post-assembly named-template validation
    and principal-over-import global binding precedence. Remove the shadowed
    imported text default before invocation materialization, preserve one
    resolved imported named template, and let both root and named templates see
    the same invocation-local global value. Keep duplicate named-template
    precedence, non-text shadowed defaults, dependency ordering, and multiple
    imports out. The denominator now records 5 passes and 11 visible not-run
    dispositions.
  - [x] Execute `include-0601` with one imported simplified stylesheet. Lower
    its implicit template into a lower-precedence document rule, let the
    principal `text()` rule invoke `xsl:apply-imports`, and preserve built-in
    text fallback when no lower-precedence text rule exists. Keep general
    imported root conflicts, modes, output merging, and multiple imports out.
    The denominator now records 6 passes and 10 visible not-run dispositions.
  - [x] Execute `include-0501` through two sibling imports. Assign the later
    import higher precedence, discard the earlier shadowed `$second` global
    parameter default before runtime materialization, and preserve the earlier
    module's unshadowed `$first` default. Keep nested imports, mixed
    include/import assembly, and duplicate named-template precedence out. The
    denominator now records 7 passes and 9 visible not-run dispositions.
  - [x] Execute `include-0103` through an embedded stylesheet selected by the
    simple `#embedded` fragment after fragmentless sealed acquisition. Apply
    inherited `xml:base="x/"` to its one nested include, assemble named template
    `x`, and retain DTD denial. Keep `include-0102` excluded because its ID
    typing depends on a DTD; keep general XPointer and arbitrary nested graphs
    out. The denominator now records 8 passes and 8 visible not-run dispositions.
  - [x] Execute `include-0104` through exactly one leading import followed by
    one include. Preserve included rules at principal precedence, retain the
    imported rule below them, and prove `xsl:apply-imports` from the included
    `one-tag` rule selects its imported counterpart. Keep other mixed, repeated,
    and nested import/include topologies out. The denominator now records 9
    passes and 7 visible not-run dispositions.
  - [x] Execute `include-0701` through two principal includes, each with one
    leaf import, using a sealed file-backed source and expected result. Preserve
    included rules at principal precedence, lower imported leaves, and bounded
    later-rule recovery for the same-precedence `title` conflict. Keep the
    `include-0702*` conflict-policy variants and general graph construction out.
    The denominator now records 10 passes and 6 visible not-run dispositions.
  - [x] Execute the positive `include-0702a` and `include-0702c` variants through
    that same graph while conserving their distinct suite metadata: explicit
    XSLT 1.0/2.0 `on-multiple-match=recover` versus XSLT 3.0+ default recovery.
    Keep `include-0702b` not-run until compilation or invocation can request
    error-on-multiple-match and report `XTRE0540`. The denominator now records
    12 passes and 4 visible not-run dispositions.
  - [x] Execute `include-0801` through two ordered principal imports, each with
    one leaf import. Preserve five precedence strata so chained
    `xsl:apply-imports` selects the later branch and then its own imported leaf.
    All positive cases in the set now execute; the denominator records 13
    passes and 3 explicit non-passes for two DTD-dependent cases and the
    expected multiple-match error case.
  - [x] Execute `include-0702b` through the same sealed five-module graph under
    its explicit `on-multiple-match=error` dependency. Reuse the private
    invocation policy to report concrete `XTDE0540` for the tied
    principal-precedence `title` rules while retaining the graph's four lower-
    and six principal-precedence rules. The denominator now records 14 passes
    and 2 explicit DTD-dependent non-passes.
  - [x] Close the complete include denominator by classifying `include-0101`
    and `include-0102` `excluded-by-profile`. Preserve the first case's external
    DTD and entity reference and the second case's DTD-declared ID fragment
    target as executable metadata guards; do not expand, rewrite, or copy their
    upstream bytes into a harness-owned approximation. The denominator now
    records 14 passes, two profile exclusions, and no default not-run cases.
- [x] Execute a private batch of independent requests with shared compiled stylesheets
  and isolated dynamic contexts; randomize scheduling, correlate results by
  identity, and prove a batch of one matches the convenience API.
- [x] Compare parse-per-invocation with private snapshot/work-generation prepared
  input reuse, reporting retained XDM and peak construction memory separately.
- [x] Measure parse, XDM construction, compilation, compiled/direct execution,
  compiled/prepared execution, compile-per-invocation execution, retained XDM,
  and preparation peak memory over native XSLT30 `for-004` and `castable-004`.
- [x] Execute a two-stage host-owned workflow and prove stage-one results remain
  invisible until explicitly admitted into a stage-two snapshot.
- [ ] Compare file-per-call, preloaded snapshot, warmed filesystem cache, and
  compile-once paths with correctness held constant.
- [ ] Add differential and integration tests against named processors.
  - [x] Establish a small ASP.NET comparison against Microsoft's built-in XSLT
    1.0 processor and a locally acquired, non-distributed SaxonCS-HE 13.0.0
    adapter, preserving exact-stylesheet versus equivalent-workload distinctions.
- [ ] Run an ASP.NET consumer workbench through the selected host boundary,
  reusing compiled stylesheets across requests with explicit cancellation and
  resource policy.
  - [x] Establish the first ASP.NET 8 persistent isolated-worker baseline with
    one-time bounded resource transfer, compile-once/prepared reuse, correlated
    results, structured failures, and one explicit in-flight slot.
  - [x] Exercise deterministic 5-, 50-, and 500-item tiers through a bounded
    four-worker pool, recording throughput, p50/p95/p99 latency, CPU,
    allocation, working-set scope, result size, and comparison-engine caveats.
  - [x] Terminate an acknowledged non-cooperating isolated request without
    poisoning a sibling, decline ambiguous retry, replace only its worker slot,
    and promote/drain explicitly identified snapshot generations.
  - [x] Import and close host files, replace them while an old generation lease
    remains active, and prove old/new requests retain their sealed source
    semantics without engine-owned filesystem access.
  - [x] Carry already-signalled cooperative cancellation into an isolated
    invocation, preserve its exact direct-path diagnostic, and reuse the same
    worker generation; natural active-signal measurement remains open.
  - [x] Route a correlated cancellation while execution is paused at a real
    charge point, ignore an unrelated identity, preserve structured failure and
    worker reuse, and keep the artificial barrier out of latency claims.
  - [x] Sample 25 unpaused 20,000-item cancellation races, conserve cancellation
    and completion outcomes, retain same-worker recovery, and distinguish local
    latency observations from deadline guarantees.
  - [x] Adapt a managed `CancellationToken` without converting cooperative
    cancellation into a hard-stop claim, and preserve a four-case direct versus
    isolated diagnostic matrix for invalid, unsupported, and cancelled work.
  - [x] Exhaust an invocation-local XSLT-instruction budget through the isolated
    host boundary, retain `FXCT0002 / limit`, decline retry or replacement, and
    prove the same compiled/prepared worker remains reusable.
  - [x] Design the proposed workbench-only native ABI safety contract, exact
    unsafe surface, panic quarantine, verification matrix, and removal criteria;
    ADR-0008 now accepts that narrow exception.
  - [x] Execute the first in-process native compile/prepare/reuse path with
    byte-exact output, structured invalid/XML diagnostics, independent-handle
    concurrency, `SafeHandle` disposal, and a three-run warm comparison.
  - [x] Exercise the same deterministic 5-, 50-, and 500-item sources through
    four independent native handles, recording tiered throughput, latency,
    managed-allocation scope, isolated working set, and the limits of whole-host
    native memory observations.
  - [x] Carry pre-dispatch cooperative cancellation and a deterministic
    XSLT-instruction budget through ADR-0009 scalar native controls, preserving
    exact diagnostics and ordinary reuse of the same engine handle without
    claiming active cancellation or hard termination.
  - [x] Fully initialize and atomically promote a changed native engine
    generation, retain old prepared semantics under a lease, drain its retired
    pool on release, and preserve the unsupported-stylesheet diagnostic fields
    asserted by direct and isolated execution.
  - [x] Signal active native cancellation after a real charge through ADR-0010
    Rust-owned control handles, ignore an unrelated handle, conserve two
    unpaused 25-trial managed-token samples, and recover the same engine without
    describing cooperative control as hard termination.
- [ ] Exercise AR-0010's private invocation controls under adversarial work;
  distinguish deterministic budgets, cooperative cancellation, best-effort
  deadlines, panic handling, and process-level hard termination claims.

Exit criterion: representative stylesheets compile once, transform multiple
documents without leaked state, fail through structured diagnostics, and expose
measured end-to-end behavior to at least one non-Rust consumer.

## Later candidates

CLI, streaming implementation or conformance, schema awareness, extension
functions, packages, alternate execution backends, transformation graphs, and
specific parallel executor strategies require their own product evidence and
architectural review. Their presence in this list is not a commitment. WASM now
has stated future consumer pressure and is tracked separately by AR-0015, but it
does not enter the current critical path without a named runtime and workload.

### Deferred WASM embedding profile

AR-0015 preserves a future presealed, memory-resident WASM experiment using the
same semantic engine. It selects no browser, WASI, component, binding, or public
API profile yet.

- [ ] Obtain the consumer's exact WASM runtime, target, deployment/trust model,
  resource graph, workload, memory ceiling, concurrency, and performance needs.
- [ ] Inventory dependency, feature, 32-bit accounting, atomics/threading,
  panic, clock, and platform assumptions for the candidate target.
- [ ] Compile the safe core and execute one bounded no-I/O smoke transform,
  then exercise sealed `include-0401` without ambient acquisition.
- [ ] Prove retained compile/prepared reuse and deterministic generation release
  across calls within one instance; make no same-instance concurrency promise.
- [ ] Differentially verify results and structured diagnostics against direct
  Rust before measuring load, transfer, compile, prepare, warm execution, and
  retained/peak linear memory.
- [ ] Require a later ADR before selecting a supported target, binding surface,
  resolver profile, or target-specific interruption guarantee.

### Prepared representation and data-layout audit

AR-0013 preserves a future investigation into whether FastXSLT can prepare XDM,
compiled plans, indexes, values, sequences, and scratch state more effectively
than straightforward reference structures. It deliberately selects no novel
container or unsafe path before profiles establish a concrete pressure.

- [ ] Capture representative workload shapes, semantic sentinels, reuse,
  concurrency, and memory/latency budgets before judging a representation.
- [ ] Inventory current representation ownership and lifetimes, then measure
  compile, prepare, execute, serialization, allocation, retained memory, and
  locality where observable.
- [ ] Audit Rust-level opportunities such as ownership shape, clone/reference-
  count removal, boxed slices, enum/tag layout, safe arenas, worker-local reuse,
  static dispatch, synchronization traffic, and generated hot paths when a
  profile makes one relevant.
- [ ] Continue the bounded probe set with phase-attributed Rust
  allocation/retention and XPath sequence length/item-kind histograms.
- [x] Measure prepared-XDM byte anatomy by node records, relationships,
  names/namespaces, values, resource identity, occurrence/unique counts, and
  construction allocations; retain interning/layout as unselected candidates.
  Treat name duplication, refcount/synchronization,
  dispatch/navigation fan-out, and scratch-capacity behavior as follow-ups
  nominated by evidence rather than simultaneous instrumentation projects.
- [x] Prototype the first two measured hypotheses behind private safe-Rust
  owners: bounded document-rooted match membership (ADR-0013) and
  invocation-owned copy-on-write atomic frames (ADR-0014). Preserve both
  complete reference paths for differential verification.
- [ ] Record both successful and negative experiments, including preparation
  cost, break-even reuse, retained memory, throughput, tail latency, and host-
  visible behavior.
- [ ] Preserve deterministic retained/peak memory attribution and independent
  generation retirement; identical content does not admit hidden cross-
  generation sharing.
- [ ] Require a separate ADR-0003 exception if evidence eventually points to an
  unsafe implementation; construction-time validation alone is not admission.

### Deferred Tokimu/Web3D consumer workload

CR-0001 remains a future real-world compiler, resource, parameter, fidelity,
Rust-facade, and performance workload. Tokimu's likely near-term use of Saxon
does not make Saxon behavior normative and does not authorize FastXSLT-specific
Web3D semantics.

- [ ] Reopen CR-0001 when Tokimu needs to replace or supplement Saxon and has
  supplied the authoritative Web3D invocation to FastXSLT.
- [ ] Independently acquire and verify known-good immutable Web3D stylesheet
  revision `35289`; record its redistribution terms, complete logical resource
  graph, catalog/base-URI behavior, and required parameters. Revision `40046`
  is a known reproducible fidelity failure and must not become expected data.
- [ ] Admit only licensed representative inputs and independently trusted
  outputs or semantic sentinels. Reuse Tokimu-owned checks for translations,
  indexed topology/coordinates, texture URLs, material colours, and
  interpolator keys/values where licensing permits; keep incomplete revision-
  `40046` output out of expected corpus data.
- [ ] Inventory required standards features and compile to the first explicit
  unsupported frontier, then feed independently justified features into normal
  standards-driven slices.
- [ ] Exercise the AR-0012 Rust facade, bounded execution, structured
  diagnostics, compiled reuse, and in-memory result handling before claiming
  Tokimu compatibility.
- [ ] Benchmark cold compilation, warm execution, preparation, result transfer,
  allocation, and retained memory only after semantic fidelity passes.
