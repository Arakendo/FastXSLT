# FastXSLT Roadmap

The roadmap is ordered by executable semantic evidence, not by a desire to fill
every conceptual module. Dates are intentionally absent until requirements and
standards scope are decided.

## Current critical path

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
three selected cases still do not establish the complete 52-case denominator.
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

The current order of work is:

1. obtain representative consumer transforms, input/result distributions,
   concurrency, deployment targets, trust model, and latency/throughput budgets;
2. execute those workloads through both candidates and add dedicated cold-load,
   native-retention, transport-attribution, and sustained-load evidence;
3. decide whether the measured low-latency and containment candidates become
   supported profiles, then stabilize only their shared lifecycle plus any
   deliberately distinct guarantee surfaces.

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
  second tranche: admit only the standard codepoint collation URI and verify
  paired NaN across both float/double argument orders. Keep unknown and empty
  collations unsupported and leave outer XPath operator cases unselected.
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
- [x] Extend that declaration lane through `output-0110a`, `output-0110b`, and
  `output-0148` through `output-0148b`, accepting whitespace-normalized XSLT
  3.0 boolean lexicals without widening XSLT 2.0 beyond `yes`/`no`.
- [x] Execute `output-0166` with retained UTF-8 and no-BOM metadata, rejecting
  non-UTF-8 encodings and BOM emission until a byte result lane owns those
  semantics.
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
  boundary that keeps `node()` from matching attributes; keep fractional and
  arbitrary-precision priority, duplicate-pattern resolution, and root-pattern
  priority outside the admitted slice.
- [x] Execute `conflict-resolution-0107`, `0108c`, and `0110c` through retained
  non-simple default priority. Add only the exact unnamespaced
  `element[@attribute]` presence pattern, charge inspected attributes, and prove
  source-order selection against an equal-priority path in both directions;
  keep general predicate expressions unsupported.
- [x] Execute `conflict-resolution-0112` through an exact compiled `//*`
  specialization whose element applicability retains non-simple default
  priority. Keep `//QName`, arbitrary descendant patterns, union patterns, and
  the general pattern grammar unsupported.
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
      cycle failure. The private production profile remains depth 1, two
      modules, and 1 MiB rather than selecting public limits.
    - [x] Add a workbench-only explicit dependency/denial input and prove
      `missing-resource` and `denied` remain distinct structured categories
      through the Rust facade plus native and isolated-worker failure envelopes.
    - [x] Admit ADR-0011 and carry one bounded stylesheet dependency plus an
      independent denial flag through both .NET initialization protocols.
      Execute an included module, preserve missing/denied diagnostics, reject
      malformed framing, and advance the native workbench ABI to version 2.
      General dependency collections and resolver profiles remain under AR-0014.
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
- [ ] Start with three bounded, explicitly supplied probes: phase-attributed
  Rust allocation/retention, XPath sequence length/item-kind histograms, and
  prepared-XDM byte anatomy. Treat name duplication, refcount/synchronization,
  dispatch/navigation fan-out, and scratch-capacity behavior as follow-ups
  nominated by evidence rather than simultaneous instrumentation projects.
- [ ] Prototype one measured hypothesis at a time behind private safe-Rust
  owners; preserve reference semantics and diagnostics through differential
  verification.
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
