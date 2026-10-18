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
