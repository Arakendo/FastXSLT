# OASIS literal-document path binding — 2026-10-17

## Pressure and scope

Unchanged OASIS CD04 `Lotus/idkey_idkey51#1` first failed while compiling a
local variable selecting `document('idkey49a.xml')//doc/body/text()[1]`.
The existing local literal-document binding admitted only the document root.
This slice implements the path binding under
[ADR-0022](../ADR/ADR-0022-explicit-retained-owners-for-runtime-source-node-values.md),
not general cross-document identity or ordering.

The compiled binding optionally retains a static location path. It uses the
existing document-path compiler, sealed resolver, effective whitespace owner,
charged XPath evaluator and retained-source frame. Compiled-capacity accounting
includes the path. Root-only bindings retain their existing representation.
The shared effective-owner helper also serves existing variable-path selection;
no alternate executor, new authority, public type or processor default is added.

Selection retention is charged before extending the retained vector. The frame
is updated only after preparation and complete selection succeed. Whitespace
visibility is applied before retaining nodes, and the owner survives aliasing
and a subsequent principal-source focus switch.

## Replay and next blocker

The complete 3,173-case modern-preprocessing/declaration-driven-baseline replay
remains **2,501 exact matches (78.82%)**, with 14 mismatches and 423 separately
credited expected errors. Initialization remains 2,702 and successful execution
2,653. No fixture, expected output, exclusion or profile selection changed.

`idkey51` now reaches its next compile failure, `FXXP1002`, at stylesheet span
837..932: comparison of generated identities selected through literal and
source-derived `document()` calls. This is concrete prerequisite removal, not
an additional corpus pass. The case additionally exercises text, attributes,
comments, repeated resource resolution and a retained variable in identity
comparisons; admitting only its first comparison would not finish it.

Do not infer general identity support from the new carrier. Existing unsupported
guards for consumers that still require the current effective source remain.
In particular, this tranche does not promote cross-owner template arguments,
count/copy consumers or generated-identity comparisons. A probe of those
combinations correctly hit `FXRT1017`; the focused success tests exercise
already owner-aware scalar and iteration consumers instead.

The refreshed frontier also confirmed that format-number picture failures have
prior standards-based dispositions, and that the BVT encoding references are
not simply BOM-less UTF-16 files. Neither is a justified shortcut to 80%.
The target still needs **38 additional exact matches**.

## Validation

Focused compiled tests cover descendant/text selection, an empty selection,
aliasing, source switching, effective whitespace, COW/complete frame clones,
exact and one-less XPath work budgets, cancellation, sealed-resource denial and
fresh execution after failure.

Full verification passed: formatting, Clippy, 2,054 core tests (64 ignored),
22 native tests (3 ignored), 4 WASM tests, 18 worker tests, documentation,
1,998 Markdown files checked for local links, and pinned corpus integrity.
The official `wasm32-unknown-unknown` target check and `git diff --check` passed.

```powershell
cargo test -p fastxslt --all-features literal_document_path_variable
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier '/'
./scripts/verify.ps1
cargo check -p fastxslt-wasm-workbench --target wasm32-unknown-unknown
```

## Follow-up: generated-ID eligibility bypass

Following the complete `idkey51` dependency chain exposed a separate admitted
runtime defect. A template executing against a secondary document could emit
`generate-id(.)`, switch to a retained principal document, and emit that call
again. Both distinct document roots produced `fastxsltPrincipalN0`. The ordinary
value route bypassed the eligibility guard already used by qualified identity
strings. This violates the
[XSLT 1.0 generated-ID rules](https://www.w3.org/TR/1999/REC-xslt-19991116#function-generate-id).

The existing principal-origin guard now also protects ordinary node/root value
strings, concat parts, context/key identity attribute templates, and root-ID
comparison against an atomic variable. Secondary-origin strings remain an
explicit `FXRT0007` unsupported outcome, with source provenance, instead of
silently labeling secondary local IDs as principal IDs. Ordinary same-context
node identity comparisons do not manufacture cross-document strings and retain
their existing implementation. No owner table, URI-derived identity, allocation
address string, ranking rule or new supported cross-document contract is added.

Three compiled regressions pin the original collision's refusal, the affected
string-route matrix under both whitespace representations, and successful
principal-focus ID generation after secondary admission. The last test protects
against an over-broad rule that denies all identity operations once the cache
contains a secondary document. Workspace verification passes with 2,057 core
tests (64 ignored), plus the unchanged adapter and documentation gates.
The follow-up full corpus replay retains all counters reported above; the
official WASM target check and whitespace/diff checks also pass.

The next positive slice must integrate owner-qualified generated strings across
literal-document paths, source-derived document references and retained
variables, with repeated-resolution and empty/first-node behavior. `idkey51`
uses elements, attributes, text and comments, including failure-message value
expressions. Its source-derived reference selects one URI; that does not justify
choosing general cross-document order for multi-document arguments. ADR-0022
already selects the node-identity ownership direction; acceptance alone does
not satisfy this consumer's executable gates.

## Positive ordinary-focus identity integration

The follow-up replaces the temporary refusal for the ordinary-node routes above
with one shared origin-qualified formatter. It finds an active source's origin
in the existing invocation-owned dynamic-document cache, charging each inspected
entry. Dynamic document roots use the existing root consumer's `dN` vocabulary;
ordinary descendants use a distinct document-qualified vocabulary. Principal
IDs retain their established spelling. The numeric dynamic identity already
belongs to the invocation; no new owner registry or public ID format is created.
Cache membership locates the retained origin; neither URI equality nor storage
address is used to manufacture node identity.

Ordinary value/root strings, concat parts, context/key attribute templates and
root-versus-variable comparisons now share that formatter. Existing path
selection and cardinality checks remain shared and unchanged. Principal global
initialization remains principal-confined. Namespace and filtered-union identity
routes retain their explicit secondary-origin guard; this tranche does not
promote them or direct `generate-id(document(...)/path)` expressions.

Compiled tests now prove positive cross-route agreement, distinct local IDs
across equal-byte principal and secondary documents, repeat-resolution reuse,
effective whitespace views, complete frame-clone parity, exact/one-less work
limits, cancellation and concurrent invocation isolation. Reuse is exercised
through the supported bound-document template dispatch, not by broadening the
still-unsupported bound-secondary-document `for-each` route.

The positive integration passes all workspace verification gates: 2,058 core
tests (64 ignored), 22 native tests (3 ignored), 4 WASM tests and 18 worker
tests, plus formatting, Clippy, documentation, local links and pinned corpus
integrity. The official WASM target check and `git diff --check` also pass.

The full unchanged-fixture replay after positive integration remains
2,501 / 3,173 exact (78.82%), 14 mismatches, 2,702 initialized and 2,653
successful executions, with 423 expected-error credits counted separately.
`idkey51` still first fails compilation of its direct document-path generated-ID
comparison (`FXXP1002`, stylesheet span 837..932). This tranche removes the
ordinary secondary-focus identity blocker; it earns no additional corpus case.

## Completed document-path generated-ID slice

The next integration adds a private typed ordinary-source identity operand to
value output and boolean equality/inequality. Literal document paths,
source-node-derived document references and retained ordinary node variables
use the existing sealed resolver, effective document selection and shared
origin-qualified formatter. Selection takes the first node in the admitted
single-document order; empty selections produce an empty string. Compiled
retention accounting includes both operand plans and their locations.

Repeated references resolving to one existing document are admitted. More than
one distinct resolved document remains `FXRT0007`: this does not select a
cross-document ranking. Explicit second document arguments, temporary/namespace
variables and global-constructor admission are not widened by this slice.

Four focused runtime tests cover element/attribute/text/comment selections,
multiple-node first selection, empty selection, repeated source references,
retained variables, equal-byte distinct documents, whitespace representations,
complete-clone frames, exact/one-less work limits, cancellation, denied resource
access and allocation release after success/failure.

### Import-layout correction and replay

The first full replay with engine integration still produced 2,501 exact
matches: `idkey51` compiled but failed `FXRS0001` because the harness placed its
principal input under an invented `source/` URI segment. The archive and catalog
place `idkey51.xml`, `idkey51.xsl`, `idkey49a.xml` and `idkey49c.xml` together.
That relocation made `document(a)` and the literal reference denote different
logical resources, contradicting the test's actual setup.

The first-party import overlay now preserves that reviewed sibling layout for
this case. A focused harness test proves both bases resolve each supplemental
name identically and that unrelated mappings remain unchanged. No fixture bytes,
expected results, comparator, engine URI alias or equal-byte identity rule are
changed. This is a harness correction as well as an engine gain, not an engine
speed or general conformance claim. Auditing the remaining synthetic source-base
mappings is a separate follow-up; do not duplicate this case's resource bytes
under two identities as a substitute for preserving the original layout.

The unchanged `idkey51` XML comparison now reports `<out>Success</out>`.
The full modern-preprocessing/declaration-driven baseline reaches **2,502 /
3,173 exact (78.85%)**, 14 mismatches, 2,703 initialized and 2,654 successful
executions. Expected-error credit remains separately 423. No other exact or
mismatch count changed.

The full strict-preprocessing replay independently reaches **2,499 / 3,173**
exact, 17 mismatches, 2,703 initialized and 2,654 successful executions, with
423 expected-error credits. It also passes `idkey51`; strict preprocessing is
not a claim that the selected processor is the future complete 1.0 profile.

Final workspace verification passes: 2,063 core tests (64 ignored), 22 native
tests (3 ignored), 4 WASM tests and 18 worker tests; formatting, Clippy,
documentation, 1,998 local Markdown files and pinned corpus gates pass. The
official WASM target check also passes. The allocation probe initially used a
cancellation ordinal beyond this fixture's work and did not cancel; it now
derives a late charge from measured work, verifies failure, and reports zero
outstanding measured allocations on both success and cancellation.

### Cohesion review

The new private compiler unit owns only typed generated-source operands and
their comparison dispatch; it delegates path/document grammar to existing
compiler helpers and holds no runtime state or authority. Runtime work stays
in the existing generated-identity evaluator and delegates selection/resolution
to their existing owners. Large parent dispatchers gain only typed dispatch
arms; their existing decomposition debt is not solved by this tranche.
The instruction-compiler folder has 23 direct files: retain its named private
feature modules for this bounded integration rather than introduce a new
directory around a single file. A broader expression-family regrouping remains
a distinct behavior-preserving cleanup. No public surface, crate or backend is
introduced.

## General source-base audit and admission-credit guard

A subsequent diagnostic replay replaced the harness's synthetic principal
`source/` prefix with the catalog-relative principal filename for every case.
This candidate was **withdrawn**, not retained as the corpus baseline. It
reported 2,497 exact, 14 mismatches, 2,698 initialized and 2,649 successful
executions, versus the retained 2,502 / 2,703 / 2,654 baseline. All five lost
exact comparisons were resource-admission collisions, not changed results:

| Ordinary case | Collision exposed by preserved source base |
| --- | --- |
| `Lotus/idkey_idkey04#1` | `t04.dtd` admitted twice |
| `Lotus/select_select59#1` | Principal source also admitted as a supplemental resource |
| `Lotus/select_select68#1` | Same principal/supplemental overlap |
| `Lotus/select_select71#1` | Same principal/supplemental overlap |
| `Microsoft/Text__91137#1` | One archival file is both principal source and stylesheet |

Two expected-error cases, `Microsoft/Elements__89172#1` and
`Microsoft/Sorting__91703#1`, also failed duplicate source/stylesheet admission.
The probe nevertheless retained 423 expected-error credits. That exposed a
separate harness defect: initialization failure was credited even when adapter
admission prevented XML/XSLT validation from starting.

The retained fix routes `FXWB0001` admission arithmetic/limit setup failures
and `FXWB0002` resource-admission failures to
`expected-error-infrastructure-excluded`, with a visible case/code entry.
Neither increments expected-error credit or semantic-initialization observation.
A regression obtains an actual duplicate-identity failure from the workbench,
checks overflow separately, verifies classification does not parse display text,
and conserves the dispositions alongside an ordinary static semantic error.
This is a narrow admission-credit fix, not certification that all remaining
archival error credits match exact standard error codes or phases.

General layout correction needs an explicit import plan distinguishing resource
identity from source/stylesheet/dependency roles. It must remove obsolete
compensating imports, admit each logical resource once, reject conflicting bytes,
and preserve principal/secondary document identity semantics without byte-based
aliasing. Do not weaken the core builder's duplicate-identity rejection merely
to make that import plan easier. No adapter, engine, authority or profile
behavior is changed by the retained reporting fix.

The refreshed ordinary frontier remains 82 initialization and 11 execution
failures. The `Bug76984` candidate needs a multi-document global followed by
nested iteration, not just another literal-document binding. The
`SortExprWithCurrentInsideForEach1` candidate additionally needs cross-owner
sort-key/current-focus handling. Neither is promoted as a cheap isolated path
recognizer; their whole-case dependencies remain visible.

After withdrawal and the accounting fix, full replays retain 2,502 modern /
2,499 strict exact matches, 14 / 17 mismatches, 2,703 initialized, 2,654
successful executions, 423 expected-error credits and 3 expected-error
infrastructure exclusions. The count conservation assertion passes in both
profiles. Full verification passes with 2,064 core tests (64 ignored), 22 native
tests (3 ignored), 4 WASM tests and 18 worker tests, plus formatting, Clippy,
documentation, local links and corpus integrity. `git diff --check` passes.
No new corpus credit is claimed for this reporting tranche.

## Catalog resources versus roles: retained correction

The follow-up replaces the withdrawn attempt with an explicit test-only import
plan. A logical resource may serve as principal source, principal stylesheet,
or dependency without being admitted more than once. Repeated identities must
have exactly equal bytes; conflicts fail with an infrastructure disposition.
Equal bytes at different identities remain different snapshot entries. This
does not coalesce independently prepared document origins or create a cache.

The private corpus constructor seals those unique resources, then invokes the
same compilation/preparation composition as the ordinary workbench constructor.
The ordinary constructor and core resource builder still reject duplicate
identity admissions. No public selector, adapter contract, authority, XML/DTD
policy or runtime node-identity rule changes.

All principal sources now retain their catalog-relative logical location. The
idkey51-only exception and compensating synthetic `source/` aliases are removed;
reviewed supplementary inputs retain their actual sibling/nested locations.
The seven previously colliding sites reach their normal semantic paths again.
Original fixtures, expected results and per-case preprocessing selection are
unchanged.

Focused regressions cover duplicate roles, conflicting bytes, equal bytes at
distinct identities, one stylesheet resource in source/style/dependency roles
under both preprocessing profiles, resource-size rejection and unchanged strict
ordinary adapter admission. The general base regression no longer privileges
one case identity. The shared composition extraction preserves phase controls,
denial policy and immutable preparation.

Modern full replay after alias cleanup retains 2,502 exact, 14 mismatches,
2,703 initialized, 2,654 successfully executed, 423 expected-error credits and
3 expected-error infrastructure exclusions. This tranche removes a concrete
import-fidelity blocker but earns no additional corpus credit.

Strict-preprocessing replay retains 2,499 exact and 17 mismatches, with the
same initialization/execution and expected-error totals. Full verification
passes: 2,066 core tests (64 ignored), 22 native (3 ignored), 4 WASM and
18 worker tests; formatting, Clippy, docs, 1,998 local Markdown files and pinned
corpus integrity pass. The official `wasm32-unknown-unknown` check also passes.
Only the existing Windows incremental-cache hard-link fallback warnings remain.

The refreshed strict frontier still exposes the same semantic boundaries:
`Keys_PerfRepro2` needs general dynamic numeric conversion through nested
translate/string/sum/key and temporary values; `Bug76984` needs multi-document
global iteration; the sort/current case needs cross-owner sort focus. These
are not import failures. No new eligibility shortcut or profile selection is
introduced to turn them into isolated recognizer wins.

## Global literal-call admission safety follow-up

Tracing `Sorting_SortExprWithCurrentInsideForEach1` confirmed that its global
`document('parts.xml')/parts` needs retained selected-node support, not just a
larger literal recognizer. `RuntimeGlobals` currently stores literal document
roots separately; the existing lexical-frame retained-selection representation
cannot simply be substituted without migrating global reads and publication.
ADR-0022/ADR-0024 supply the ownership direction, but this is still implementation
work with its own consumer and control tests, not permission to discard the path.

The investigation found a separate concrete admission defect. The old global
literal recognizer checked only the first and last quote of the whole argument.
The focused regression `select="document(')"` reproduced a compiler panic:
`begin > end (1 > 0) when slicing`. Multiple arguments or compound expressions
could also be mistaken for literal URI bytes instead of being refused by the
bounded initializer.

Global admission now uses the shared quote/depth-aware document-call parser and
requires exactly one literal argument with no trailing path. The shared literal
extractor uses the existing complete string-literal check, preventing compound
expressions from masquerading as literals in local/copy consumers too. Explicit
base arguments remain available only in consumers that already admit them;
the global slice does not acquire that capability. No resource resolver,
runtime global representation or processor-edition policy changes.

Three regressions pin panic-free refusal and expression-site provenance;
preservation of empty references, commas, parentheses, opposite quotes and
Unicode URI text; and shared parsing of a genuine second argument and trailing
path without absorbing either into the URI. The malformed regression was run
before the fix and failed with the reproduced panic, then passed after repair.
The unsupported global path remains unsupported rather than silently selecting
the whole document.

Full modern/strict replays preserve 2,502 / 2,499 exact matches, 15 / 18
mismatches, 2,704 initialized, 2,655 successful executions, 423 expected-error
credits and 3 infrastructure exclusions. Verification passes with 2,071 core
tests (64 ignored), 22 native (3 ignored), 4 WASM and 18 worker tests, plus
formatting, strict Clippy, docs, links, pinned corpus integrity and the official
WASM target check. `git diff --check` passes. This is a compiler safety fix with
no additional corpus credit; all changes remain uncommitted.

## Retained global document-path selection

The next slice admits one literal `document()` argument followed by an already
supported ordinary location path in a legacy global initializer. It retains the
selected nodes with their effective document owner in invocation state, using
the same sealed-resource preparation and charged path selector as lexical
bindings. Publication follows successful initialization only. Compiled state
contains only the static reference/path, including retained-capacity accounting.
No URI fallback, document-identity shortcut or new public contract is introduced.

Ordinary owner-aware string, path, identity, alias and iteration consumers share
the existing retained-source accessor. Effective boolean conversion uses set
emptiness. Principal-only consumers retain their explicit owner guard; reads of
the new value during another global's initialization return `FXRT1004` until
that integration is deliberately implemented. This is not universal support
for secondary-node operations or cross-document ordering.

Two focused regressions prove secondary values despite colliding principal IDs,
empty/nonempty boolean behavior, lexical aliases, attribute paths, generated
identity and iteration; exact/one-less XPath-visit limits, active cancellation,
denied sealed resources, clean reuse after failure and the dependent-initializer
guard. Both principal whitespace representations are exercised; secondary
selection continues using the existing visibility-view selector.

The unchanged `Microsoft/Sorting_SortExprWithCurrentInsideForEach1#1` now gets
past global admission and fails on its first sort key:
`$MyParts/part[@id = (current())/@refid]/@type` (`FXXP1001`). Its concat body
needs the same cross-owner predicate semantics, so the next investigation must
consider a shared typed expression rather than a sort-only special case.

Modern/strict full replays retain 2,502 / 2,499 exact matches, 15 / 18 mismatches,
2,704 initialized, 2,655 successful executions and 423 expected-error credits
with 3 infrastructure exclusions. This tranche removes a real compilation
blocker but earns no additional corpus case. Full verification passes with
2,073 core tests (64 ignored), 22 native (3 ignored), 4 WASM and 18 worker tests,
formatting, strict Clippy, documentation, 1,998 Markdown link checks and pinned
corpus integrity. No new architectural decision is needed for this retained-owner
integration; no production processor-edition promotion is implied.

## Shared current-attribute lookup in sort and concat

The sorting case now compiles through one bounded typed lookup, admitted as a
scalar operand shared by the existing sort/concat evaluator. Its grammar is a
single variable-rooted child step, one attribute equality against `current()/@x`
or `(current())/@x`, and a terminal attribute selection. Names are resolved in
stylesheet context. The current attribute is read from its actual source before
the retained variable owner is navigated. Missing attributes are empty node-sets,
not empty strings; the first selected result attribute wins in document order,
including when an earlier matching element has no result attribute.

The ordinary charged path evaluator still owns navigation. Temporary focus is
explicitly refused, existing secondary-focus sort guards remain, and no mixed
document ranking, cache sharing or resource acquisition is added. Key/global
concat consumers keep their narrower eligibility. Compiled retention accounting
includes the boxed lookup and all static paths.

Whole-case replay next exposed `collection[@name=$MyGroup]`: the text-constructed
global was incorrectly required to be a source node-set. The existing selection
comparison now handles a recognized result-tree fragment through its charged
string value and shared existential source-string comparison. Other unadmitted
value kinds are not silently scalarized.

The unchanged sorting case now executes with the expected nine lookup values
and two-key order. It remains an exact-output mismatch: archived text has leading
indentation tabs absent from the actual result. Modern exact coverage stays
2,502 / 3,173; successful execution rises to 2,656 and initialization to 2,705.
Modern mismatches rise from 15 to 16 as ordinary phase failures fall to 91.
No fixture, comparator or profile is changed to credit this result.

Focused tests cover stable sorting, shared concat values, exact/one-less visit
budgets, active cancellation, namespace aliases, missing versus empty attributes,
duplicate matches and first existing result, concurrent invocations with complete
frame clones, temporary-focus refusal, constructed-global string predicates and
bounded parser refusal/unbound-prefix diagnostics. These are integration proofs,
not additional corpus cases.

Final verification passes: 2,076 core tests (64 ignored), 22 native (3 ignored),
4 WASM and 18 worker tests, formatting, strict Clippy, docs, 1,998 Markdown link
checks, pinned corpus checks and the official WASM target. Strict preprocessing
retains 2,499 exact matches with 19 mismatches and the same 2,705 initialized /
2,656 executed totals. The bounded lookup is admitted only in value-of and sort
consumers; adjacent concat contexts do not gain eligibility implicitly.

## Post-commit whole-frontier reassessment

Replayed the unchanged modern corpus at `8924ba5d` using `-TraceFrontier '/'`
and separately traced the sixteen mismatch cases. The 91 ordinary phase failures
are reported as 23 unsupported, 64 invalid and 4 missing-resource. These are
diagnostic categories, not adjudicated standards outcomes: notably 32 invalid
entries carry `FXXM0001`/`FXXM0002`, including deferred DTD behavior as well as
malformed archival XML. Do not reclassify these cases as passes or exclusions.

Whole-case inspection narrows the next choices:

- `Keys_PerfRepro2` needs composed numeric expressions over key selections and
  temporary node-set values, arithmetic, string/number/translate conversion and
  formatting. Its first formatting error is not just a missing picture rule.
- `idkey49` needs mixed-document unions, positional selection and generated IDs
  together. `DocumentInUnion` needs observable cross-document dispatch ordering;
  `DocumentFunctionNoArg` retains a mixed-owner union in a variable. They must
  not bypass ADR-0022's deferred cross-document ordering/carrier boundary.
- `expression02` and `bvt089` require real unparsed-entity metadata, including
  relative resource identity and secondary documents; returning empty strings
  for unknown entities alone cannot complete either case.
- `numberformat07` explicitly tests a prohibited currency symbol;
  `specialCharInPattern` is catalogued as an invalid-pattern test; `bvt024`
  contains multiple malformed patterns. Do not weaken picture validation to
  manufacture successful archival output.
- Namespace mismatches `namespace32`/`namespace34` declare the matched prefix
  as `http://test` while the source element's namespace is `test`. The empty
  result is not evidence that their namespace-axis predicates failed.
- The sorting, attribute-set, text and ID mismatch traces remain visible;
  matching semantic values do not authorize ignoring significant text or
  replacing the archived expected result.

The useful next implementation campaigns are therefore the already accepted
ADR-0025 profile integration gates, or shared composed expression support—not
another unrelated narrow recognizer. Mixed-document ordering and wider DTD
semantics retain their existing review boundaries. The 80% target still needs
37 original exact comparisons; this reassessment supplies no new credit and
does not establish that 37 safe short fixes exist. No engine or comparator code
was changed in this reassessment.

## 2026-10-18 qualified-function diagnostic boundary

`Microsoft/BVTs_bvt052#1` selects `msxsl:node-set($foo)` in an apply-templates
instruction. The engine does not implement that vendor extension. The shared
qualified-path parser previously treated the entire function call as a malformed
QName and reported `XPST0003`. It now recognizes a leading qualified function
name only to refuse unsupported call syntax (`FXXP1001`); it does not execute
extensions, parse their arguments, or infer function availability. An unbound
function prefix remains `XPST0081`, and malformed QNames remain invalid.
Focused parser and stylesheet tests pin alias-independent refusal and original
instruction provenance. This is diagnostic correction, not corpus credit.

The eleven ISO-2022-JP source failures were also rechecked against the
[existing negative evidence](oasis-xslt10-iso2022jp-source-frontier-2026-09-28.md).
The source still fails at byte 994, at adjacent reset/shift escape sequences.
An independent .NET code-page 50220 decode with exception fallbacks accepts all
10,202 bytes, produces 4,909 UTF-16 code units without replacement characters,
and parses as XML rooted at `xslTutorial`. That establishes decoder disagreement,
not permission to replace the selected decoder contract. The
[WHATWG decoder](https://encoding.spec.whatwg.org/#iso-2022-jp-decoder) has an
explicit consecutive-transition error rule; the segment grammar in
[RFC 1468](https://www.rfc-editor.org/rfc/rfc1468.html) also requires character
content in its single/double-byte segments. No permissive input behavior,
fixture normalization or new encoding policy is admitted.

Validation: full repository verification passes with 2,078 core, 22 native,
4 WASM and 18 worker tests (2,122 total passed; existing ignores unchanged),
plus the official WASM target check. Unchanged corpus replay retains modern
2,502 exact / 16 mismatches and strict preprocessing 2,499 exact / 19 mismatches,
both with 2,705 initialized, 2,656 executed and 423 expected-error credits.
`bvt052` now reports unsupported `FXXP1001` at the original span `466..535`.
The ordinary failure total stays 91; its categories become 24 unsupported,
63 invalid and 4 missing-resource. No new decision is needed for this refusal;
actual extension execution remains outside this slice.

## 2026-10-18 composed string predicates: Miscellaneous 84475

Whole-case inspection selected unchanged `Microsoft/Miscellaneous__84475#1`.
It requires scalar `string(descendant::*)` comparison against a composed
`concat(substring-before(...), ...)`, plus an attribute match predicate comparing
`string-length(self::*)` with constant arithmetic. A read-only
`XslCompiledTransform` probe reproduces the archived element/text result; this
does not replace the original comparator or erase the suite's doubts metadata.

The private XPath predicate owner now admits bounded string literals, first-node
path conversion, concat and substring-before composition, scalar equality and
inequality, and character-length equality/inequality. Nested paths use the
existing charged path evaluator and original source location. Only explicitly
scalar operands enter scalar comparison; bare node-set equality is not rewritten.
The [XPath 1.0 string rules](https://www.w3.org/TR/1999/REC-xpath-19991116/#section-String-Functions)
and [XSLT pattern rules](https://www.w3.org/TR/1999/REC-xslt-19991116#patterns)
govern the behavior, not a fixture-specific recognizer.

This first-node conversion is admitted only through existing XSLT 1.0 expression
compatibility. The ordinary modern path parser remains guarded. This is not
processor-edition selection, an ADR-0025 profile promotion, or new default policy.
Template patterns share the same path representation and runtime, including the
attribute self-axis behavior: `self::*` selects no element when focus is an
attribute. No new template executor is introduced.

Focused tests cover Unicode character counts, first-node conversion, empty and
missing delimiters, absent paths, quoted commas/operators, boolean composition,
attribute focus, union-pattern dispatch and repeated transformations. Controls
exercise exact/one-less path/string-value visits, operations and text-byte charges; active
cancellation and allocation observation prove cleanup and reuse. Variables,
qualified names, resource calls, nested path predicates and broad numeric
arithmetic remain guarded. Small integer arithmetic is admitted only inside an
exact floating-point integer envelope, not as a general XPath numeric evaluator.
The source string consumer uses the shared string-value visitor, charging each
text chunk before copying it. Function nesting, concat arity and the constant
folder's input length have explicit private admission bounds.

ADR-0004 review: the path dispatcher and boolean predicate owner remain roughly
3,100 and 2,100 lines and retain decomposition debt. New composition parsing,
evaluation and accounting live in a named private child module with separate
tests, rather than enlarging either parent with another function family. The
49-file XPath root gains no new direct files; the five-file `path_experiment`
subject directory remains navigable. Parent changes are routing, qualification,
source-location threading and one exhaustive variant. A broad reorganization
is deferred to a separate behavior-preserving checkpoint; no new crate, public
type, runtime state carrier, authority or cache is introduced.

Full unchanged corpus replay confirms **2,503 / 3,173 exact (78.88%)** under
modern preprocessing and **2,500 / 3,173** under strict preprocessing. Both gain
exactly this one case, with 2,706 initialized and 2,657 successfully executed.
Mismatches remain 16/19 respectively; 423 expected-error credits remain separate.
No archive, comparator, resource mapping or profile-per-case choice changed.
The ordinary phase-failure total falls from 91 to 90. Another 36 original exact
comparisons are required for 80%; this slice does not estimate their effort.

Final validation: formatting, strict workspace/all-target/all-feature Clippy,
2,090 core tests (65 ignored), 22 native, 4 WASM and 18 worker tests, docs,
1,998 Markdown link checks, pinned conformance checks and the official WASM
target all pass. Both corpus profiles were rerun after the pre-copy byte charges
were added and retained the counts above. `git diff --check` passes.

## 2026-10-18 next-case and comparator reassessment

A fresh ordinary-failure trace after `84475` still contains 90 cases: 23
unsupported, 63 invalid and 4 missing-resource. Read-only whole-case inspection
of `Microsoft/XSLTFunctions_Bug76984#1` finds two URI-bearing source attributes,
`a.xml` and `b.xml`, selected by `document(root/foo/@doc)`. Its nested iteration
observes positions across the returned documents. This is not just admission of
another document-call spelling: it requires mixed-owner values and an ordering
rule. ADR-0022 deliberately does not select general cross-document ranking.
Do not use source-attribute order, URI sorting or allocation order implicitly.

The four unsupported XML comparisons were checked separately from engine
failures:

- `Lotus/output_output22#1` remains the documented EBCDIC-reference decoding
  boundary after the permitted UTF-8 output fallback. No new encoding provider
  was admitted.
- `Microsoft/BVTs_bvt018#1` and `bvt019#1` request `iso-8859-5` and `koi`
  respectively, but their archived results are identical 440-byte files
  (SHA-256 `CCDAA6273B82C90761A734408E5BF646FDDC0E52A7F404BAA1C925116BDCA7C6`).
  They contain literal question marks where the stylesheet constructs Japanese
  and supplementary characters, and byte `DA` for the authored U+00DA. These
  observations suggest lossy archival conversion, not proof of either requested
  encoding. Do not guess a decoder, replace missing characters or credit an
  inferred corrected reference.
- `Microsoft/Elements__78347#1` constructs two Japanese element names. Its
  reference instead contains `<??????????????? xmlns="bar" />` and
  `<????? xmlns="bar" />`, which are not those names or a parseable XML result.
  The original comparator refusal remains visible.

No reference, comparator, exclusion or engine behavior changes in this
reassessment. The existing four comparator gaps remain four; this investigation
does not silently reclassify them into the unusable-reference category.

Both full debug-harness replays retain 2,503 modern-preprocessing exact matches
and 2,500 strict-preprocessing exact matches. The current modern ledger is
`2503 exact + 423 expected-error credits + 90 ordinary phase failures + 16
mismatches + 10 comparison gaps + 123 policy/reference exclusions + 8
expected-error infrastructure/expectation exclusions = 3173`.
Strict preprocessing has three fewer exact matches and three more mismatches;
it is still not the selected legacy processor profile.

Next implementation work should resume ADR-0025's expression error-timing gate:
the private matched lane already proves direct calls, but nested and
short-circuited calls must fail at their actual evaluation boundary rather than
being replaced with a whole-instruction deferred failure. Keep unsupported
shapes guarded until shared lowering/runtime behavior is proved. This is an
accepted implementation obligation, not a request for another architectural
decision or permission to promote the reference. The seven private fallback/BVT
successes are not new baseline passes and cannot be added to this numerator.

## 2026-10-18 mismatch and numeric-composition audit

Fresh unchanged modern replays still report 2,503 exact comparisons out of 3,173,
16 mismatches and 90 ordinary phase failures. No engine, fixture, comparator,
profile or disposition was changed during this audit.

The mismatch traces rule out several tempting shortcuts:

- `Attributes_EmptyStringInUseAttribSets` and `AttributeSets__91042` have matching
  element/attribute values; the reference adds whitespace inside the innermost
  empty `l7`. `Namespace_XPath_CopyNamespaceNodeToOutput` likewise differs on
  text inside an otherwise matching empty element. Do not change the namespace
  or attribute-set evaluator to manufacture serializer whitespace.
- `Sorting_SortExprWithCurrentInsideForEach1` has the same nine ordered
  type/name/ID records; the reference prefixes records with two tabs. This is
  no longer an unimplemented current-node lookup or sorting operation.
- `TestIdFuncInComplexStruct` produces the expected ID-selected markers; its
  differences are surrounding whitespace. The three text cases `78274`, `78276`
  and `78309` also retain observable whitespace differences. None permits
  blanket whitespace-insensitive comparison.
- `namespace32`/`namespace34` remain the already documented source-versus-match
  namespace disagreement, not a failed namespace-axis selection.
- `Import__84615`/`Include__77751` have both inter-element formatting and retained
  whitespace in their final span. A serializer-only fix would not establish
  whole-case parity.
- `Output__84025` contradicts its older exact-pass evidence. Both current modules
  request XML output; the actual comment/PI/AVT-rich output materially differs
  from the archived fragment. Its historical evidence now carries an explicit
  current-status correction. The original reference and mismatch remain intact.

Whole-source inspection of `Keys_PerfRepro2` confirms that the numeric frontier
requires a compositional implementation, not widening the format-picture parser.
The actual source contains `DVPSIH` records, so the key-based loops are live.
The stylesheet combines `sum(key(...)/...)`, arithmetic, `string`, `translate`,
numeric coercion and formatting, and also requests temporary `msxsl:node-set`
aggregates. The current format-number owner accepts literals, variables and
bounded paths; arbitrary numeric text reaches its source-free evaluator and is
explicitly unsupported. The existing binary-numeric tree is exact-rational and
explicitly refuses empty operands, zero divisors and nonterminating decimals.
Routing the whole expression through that tree without implementing XPath 1.0
NaN/infinity/conversion behavior would merely exchange one unsupported boundary
for another, or produce incorrect results.

The next executable numerical slice must therefore prove shared typed aggregate
and conversion composition, including empty selections and non-finite values,
before connecting it to formatting. It must not parse dynamic XPath strings at
runtime or substitute a fixture-specific expression evaluator. This is a larger
implementation campaign; it supplies no promised case yield. Accepted ADR-0025
integration remains the other actionable campaign. There is no evidence here of
36 independent safe one-line fixes, and no new ownership decision is requested.

## 2026-10-18 first aggregate-to-format integration

The first production composition slice admits `format-number(sum(path), picture)`
for unqualified relative child-name paths in the existing XSLT 1.0 expression
compatibility mode. A typed aggregate operand delegates to the same charged
`sum_path_lexical` used by ordinary sum expressions, then passes its evaluated
numeric value to the existing formatter. It does not send source-dependent
XPath text to the source-free folder, use exact-rational arithmetic as a
replacement for XPath doubles, or introduce a second evaluator.

The relevant rule is [XPath 1.0 sum](https://www.w3.org/TR/1999/REC-xpath-19991116/#function-sum).
Compiled integration tests exercise multiple selected nodes, negative values,
empty selection, invalid lexical values (including exponent spelling), overflow
to positive/negative infinity and opposite-infinity addition yielding NaN.
The same compiled program is reused across inputs and repeated executions.
Exact/one-less operation limits, active cancellation late in aggregation,
request identity, post-failure reuse and zero outstanding allocation-observation
bytes/count are pinned. A parser/formatter test proves unresolved aggregates
cannot be formatted as literals and that modern, key, variable, qualified and
predicate-bearing aggregate shapes remain guarded.

This is deliberately one composition step, not general numeric-expression
support. `PerfRepro2` still requires key aggregation and outer conversion/
translation composition, plus temporary node-set and arithmetic behavior later
in the stylesheet. Its unchanged replay still reports `FXRT1007` at numeric
conversion. Reassess those shared consumers next; do not extend this operand
into an ad hoc interpreter for that stylesheet.

Both full corpus replays conserve the baseline: modern preprocessing has 2,503
exact / 16 mismatches; strict preprocessing has 2,500 exact / 19 mismatches,
each out of 3,173, with 2,706 initialized, 2,657 executed and 423 separate
expected-error credits. No corpus credit, archive change, comparator relaxation
or processor-profile promotion follows from this tranche.

ADR-0004 review: the 1,611-line format-number owner retains parsing, formatting
and its existing tests; the new typed operand and accessor belong to that
responsibility. The 3,335-line value evaluator gains only delegation to an
existing aggregate owner, not new arithmetic or traversal semantics. Its
decomposition debt remains. Integration tests live in a named 101-line numeric
composition child rather than enlarging the test parent. The runtime test
directory now has 16 direct files; the new subject has one file, so a one-file
subdirectory would not improve grouping. Further numeric composition tests
should stay with that subject. Retained-capacity accounting includes the typed
aggregate path through the existing path accounting.

Validation: all three focused composition tests pass. Full verification passes
with 2,100 core tests (65 ignored), 22 native, 4 WASM and 18 worker tests,
formatting, strict all-target/all-feature Clippy, documentation, 1,998 Markdown
link checks and pinned corpus checks. The official WASM target and
`git diff --check` also pass. Two initial test-harness issues (the serializer's
required arguments and its default XML declaration) were corrected in the
test, without changing production serialization behavior.

## 2026-10-18 shared key aggregation

The next production slice lowers `sum(key(...)/...)` through an explicit typed
key-sum value. The compiler delegates name/value/tail parsing to the existing key
lookup compiler. Runtime selection uses the existing invocation-owned key index
or its uncached reference; aggregation then uses the same double-conversion sum
as ordinary path sums. The sum helper's visibility is widened only to its parent
value evaluator. No second key index, numeric interpreter, resource resolution,
cross-document ordering or new cache lifetime is introduced.

Compiled tests exercise variable key values, multiple selected value children,
duplicate key declarations, unrelated keys, negative/fractional numbers, empty
matches and invalid numeric text. Cached and uncached modes produce identical
results on the same compiled program across changing input documents. Exact and
one-less operation budgets and cancellation after partial work preserve failure
codes and request identity; allocation observation finds no outstanding retained
bytes or allocations after cancellation, and subsequent invocation reuse works.
The previously pinned sum-to-format non-finite cases continue to use this same
aggregate helper.

Whole-case replay still stops `PerfRepro2` at the outer numeric conversion:
its `translate(string(sum(key(...)/...)), ...)` is not yet a compositional typed
scalar operand accepted by format-number. The new direct key-sum value is not
silently substituted for that expression. Next connect aggregate values through
shared string/numeric conversion and translation, preserving NaN and actual
character-translation behavior; do not special-case the literal `NaN` idiom or
rewrite the fixture. Temporary-node-set aggregates and arithmetic remain separate
later consumers. This tranche removes the direct key-aggregation gap, not the
whole-case blocker, and earns no corpus credit.

ADR-0004 review: parent compiler/runtime changes are typed dispatch and delegation
to existing owners; retained-capacity accounting delegates to the existing key
lookup accounting. Tests remain in the numeric-composition child introduced in
the preceding slice. No new source file, public type or architectural owner is
added; existing large-parent decomposition debt remains explicit.

Validation: both focused key-sum tests and the full verification script pass:
2,102 core tests (65 ignored), 22 native, 4 WASM and 18 worker tests; formatting,
strict Clippy, docs, 1,998-document link checks and pinned corpus checks. The
official WASM target also passes. Fresh modern/strict replays retain respectively
2,503/2,500 exact and 16/19 mismatches out of 3,173; both retain 2,706 initialized,
2,657 executed and 423 separate expected-error credits. `git diff --check` passes.

## 2026-10-18 formatted key sums and conversion composition

The bounded composition now connects a typed key sum to the shared formatter,
optionally through `string()` and `translate()`. Translation arguments are
constant strings or finite numeric literals converted to strings. The compiler
uses balanced argument parsing and existing key/decimal-format compilation;
there is no runtime XPath parsing or special treatment of the literal `NaN`.
Bare formatted key sums and explicit string conversion use the same route.

The formatter marks its number operand as supplied by shared evaluation,
instead of treating dynamic expression text as a numeric literal. Named and
default decimal-format finalization includes the new typed value. The existing
key selector, sum evaluator, character translator and formatter retain their
semantic ownership. Translation charges operation work and a conservative
four-bytes-per-input-byte intermediate capacity before allocation; output
retention remains separately charged by the existing result builder.

The typed value distinguishes a numeric sum from a string-converted sum.
String/translation results pass through the shared XPath 1.0 number conversion
before formatting: generated exponent spelling or a leading plus yields NaN,
not Rust's permissive floating-point parse. Numeric infinity remains infinity,
whereas the string `Infinity` coerces to NaN. Four compiled cases pin this
distinction. Conversion also charges work and intermediate text capacity before
allocation. This closes a coercion hole found during review of the initial
composition candidate, before final validation.

Compiled regressions cover cached/reference parity, empty and invalid sums,
numeric translation arguments, duplicate search characters/deletion, named
decimal formats, exact/one-less operation and text-byte limits, active
cancellation, allocation cleanup and reuse. Arithmetic around the sum and
dynamic translation arguments remain explicitly unsupported. Formatter failures
now retain the original expression location, and the opt-in corpus failure trace
prints that existing structured provenance using its bounded escaping helper.

The unchanged `PerfRepro2` replay advances past the earlier key-sum conversions.
Its remaining failure is at bytes **5420..5626** of
`DailyVolumePackageSummary.xslt`: the percentage expression dividing a sum over
`msxsl:node-set($business.PeriodCasesVsLastYear)/node()` by its count. This is
temporary-tree aggregate/division composition, not the newly implemented key-sum
route. It still returns unsupported `FXRT1007`; no partial result or exact pass
is credited. Next assess that shared temporary-value/numeric boundary, retaining
the extension's existing admission guards and non-finite arithmetic semantics.

ADR-0004 review: the new compiler child is 83 lines and owns only this bounded
composition; the parent delegates to it. The compiler subject directory has 24
direct files and retains the previously recorded expression-family regrouping
debt; no one-file wrapper directory is added. Numeric integration tests stay in
their existing 382-line child. Runtime changes delegate to existing semantic
owners. Compiled retention accounts for the lookup, translation constants,
formatter and boxed value; no invocation value enters the compiled artifact.
The initial cancellation test confused charged units with charge-call ordinals;
it was corrected to cancel on the first real charge without changing controls.

Final validation: four focused composition tests pass, as do all verification
gates with 2,106 core tests (65 ignored), 22 native, 4 WASM and 18 worker tests,
strict Clippy, formatting, docs, pinned corpus checks and 1,998 Markdown link
checks. The official WASM target passes. Final full corpus replays retain modern
2,503 exact / 16 mismatches and strict 2,500 exact / 19 mismatches, out of 3,173;
both retain 2,706 initialized, 2,657 executed and 423 expected-error credits.
Both locate the remaining `PerfRepro2` failure at 5420..5626. No denominator,
expected output, exclusion or profile selection was changed.

## 2026-10-18 numeric campaign extension boundary

The next unchanged `PerfRepro2` expression requires more than a missing numeric
operator. `msxsl:node-set()` is bound to `urn:schemas-microsoft-com:xslt`; the
ordinary compiler already explicitly refuses this qualified function in
selection, with both its customary prefix and an alias. There is no existing
admitted temporary-tree extension conversion to reuse. The numeric route must
not silently bypass that boundary or infer extension meaning from a local name.

A focused compiled regression pins the current unsupported numeric outcome for
both direct aggregation and sum/count division, under the Microsoft binding,
an alias, and the same customary prefix bound to an unrelated URI. It verifies
original instruction provenance, request identity, repeat invocation, no partial
result publication, and zero outstanding observed allocations after failure.
This is a guard, not implementation of the extension or a conformance credit.

The fresh modern full replay remains 2,503 exact / 16 mismatches out of 3,173.
The ordinary-frontier replay still identifies 90 failures; it does not reveal a
new ordinary numeric cluster that would justify extending this grammar solely
for percentage pressure. The current archive also includes profile/recovery
differences, invalid inputs, unavailable resources and deliberately deferred
ownership/DTD boundaries. These are not interchangeable implementation defects.

Disposition: park `PerfRepro2` at the extension boundary. Preserve the shared
sum/conversion/formatting work already validated, but do not add an extension
conversion as an incidental arithmetic optimization. Next resume accepted
ADR-0025 integration (argument/error timing and capability/introspection), whose
production gates remain open. An explicit Microsoft-extension scope would be a
separate decision; it is not required to continue that accepted work. The 80%
checkpoint remains 36 unchanged exact comparisons away, with no promised yield
from either route.

Validation: the new six-shape regression passes. Full repository verification
passes with 2,107 core, 22 native, 4 WASM and 18 worker tests (2,151 total),
formatting, strict Clippy, documentation, 1,998 Markdown link checks and pinned
corpus checks. The official WASM target passes. The strict full replay retains
2,500 exact / 19 mismatches, 2,706 initialized, 2,657 executed and 423 separate
expected-error credits. `git diff --check` passes. No production behavior,
fixture, comparison rule, denominator or profile default changed in this
boundary reassessment.
