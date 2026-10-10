# XSLT 1.0 concat sort keys and runtime decomposition

Date: 2026-10-04

## Result

Principal-source XSLT 1.0 sort keys now admit the existing bounded typed
`concat()` plan. They reuse the shared lexical evaluator and stable sort
typing, without constructing temporary result text. This is a composition
extension, not a general expression evaluator or a new sorting algorithm.

Before changing behavior, the complete sort evaluator was extracted into
the private runtime `sort_keys` owner. Full verification and OASIS replay
passed at that behavior-preserving checkpoint with unchanged counts.

## Semantics and controls

The initial checkpoint reuses the existing concat operand grammar and argument
bound without nested substring operands. The follow-up below admits a narrow
constant-bound substring operand, not arbitrary function composition. Compatibility
selection remains stylesheet-derived. The modern sort grammar is unchanged.
The boxed expression and its owned operands enter compiled-retention accounting.

Keys are lexical values, not transformation output: evaluating them must not
consume result-text budgets. XPath charges and cancellation remain in the
shared evaluator. Missing-variable diagnostics retain the sort location.
Secondary-document focus reports explicit `FXRT0007` rather than reinterpreting
principal-local variable IDs against another document. Temporary and qualified
namespace sorting are not widened by this plan variant.

Four hand-authored runtime regressions cover first-node conversion, empty
keys, stable ties, visibility-view versus complete-reference parity, unchanged
result-text charges, exact XPath-operation limits, cancellation at a real
charge, failure recovery, and secondary-document rejection. Compiler routing
tests retain quoted-colon diagnostics and the modern unsupported boundary.
These fixtures are executable evidence, not additional corpus passes.

## Corpus replay

The unchanged local OASIS CD04 archive is identified by SHA-256
`66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`.

```powershell
./scripts/measure-oasis-xslt10.ps1 -TraceCase sort36
```

Replay conserves 2,491 / 3,173 exact XML comparisons (78.51%), 2,692 initialized
cases, 2,643 successful executions, 478 initialization failures, 49 execution
failures, 14 XML mismatches, four comparator gaps, and 423 / 431 expected-error
credits. No fixture, expected result, overlay selection, or comparison rule
changed. Another 48 exact cases are required for 80%.

`Lotus/sort_sort36#1` remains initialization-unsupported with `FXXP1001`.
Its first blocker is now the nested substring operand whose length composes
boolean conversion and string length. Do not credit the improved blocker or
replace it with a fixture-specific sort shortcut. Continue through shared
expression primitives, or choose another ordinary frontier case.

## Cohesion review

Under ADR-0004, the runtime composition root decreases from 7,479 to 6,684
lines. The new 852-line private sort owner contains key evaluation, controls,
collation, typing, and comparison. It imports shared charged path, variable,
key and lexical evaluators; it does not acquire resources, own invocation
state, dispatch constructors, or expose public types. The 569-line compiler
sort owner separates scalar/path fallback from compatibility composition.

Retain the still-large runtime root for this bounded slice, with decomposition
required before another broad responsibility expansion. Its directory has
14 direct files, triggering a grouping review: keep the cohesive sort owner
beside existing private semantic owners for now rather than introducing a
one-file folder. Revisit grouping when a related family develops. Runtime
regressions reside in a separate concat-sort test owner; no production
test-only dependency or alternate evaluator is introduced.

## Validation

All four focused concat-sort regressions pass. Complete `scripts/verify.ps1`
passes formatting, strict workspace Clippy, 1,589 core tests (57 manual probes
ignored), 22 native adapter tests (three ignored), four WASM adapter tests,
18 worker tests, documentation, Markdown links, provenance and inventories.
The official all-features WASM target check and full OASIS replay also pass.
No performance improvement is claimed. Windows incremental-cache hard-link
warnings fall back to copying without relaxing a gate. Work remains
uncommitted alongside the preceding corpus tranche.

## Constant bound substring composition

A follow-up admits `concat(substring(path, start[, length]), ...)` in XSLT 1.0
through the existing typed path-substring compiler. Start and length retain
the existing finite numeric literal grammar. Dynamic bounds and nested
string-producing source operands remain unsupported; modern compilation,
key-use concatenation and restricted global initialization are not widened.

Ordinary substring output and concat now call one shared lexical substring
helper. That helper selects the first path node through effective navigation,
charges XPath work, and applies the existing character-position and rounding
implementation. Only the output consumer appends result text. The boxed
operand and its path storage are included in compiled-retention accounting.

Three additional regressions cover ordinary-output versus concat parity,
fractional rounding, two-argument suffixes, zero-length results, empty source
selection, stable sort composition, visibility-view/reference parity, exact
operation ceilings, cancellation and recovery, and explicit rejection of
unadmitted dynamic/nested forms. The full verification suite passes with
1,592 core tests and the same adapter counts. WASM checking and complete OASIS
replay pass; every corpus counter above remains unchanged. `sort36` still
requires dynamic bound composition, so no pass is credited.

ADR-0004 review retains the 115-line concat compiler and 950-line compatibility
value owner. The 3,538-line value-expression compiler is only called through
its existing substring helper, not expanded with a second grammar. Its
16-file instruction-compiler directory remains organized by explicit feature
owners; this slice adds no sibling and does not justify a cosmetic move.
Revisit a named string-expression subgroup before broader recursive expression
work. No new public contract, authority, runtime carrier, or backend is selected.

## Direct substring sort traversal control

Reviewing the next composition step exposed a resource-control gap in direct
substring sorting. Its path selection was charged, but selected string values
were traversed with an unbounded temporary control. Equivalent concat keys
used the invocation's charged string traversal. A regression reproduced four
reported `XdmStringValueNode` visits on the old direct path versus ten on the
equivalent concat path, despite identical `EABZ` output.

Direct substring sorting now calls the same lexical substring owner as output
and concat. The helper receives the actual source document and context, so
this repair does not reinterpret secondary-document IDs against principal
storage. Traversal uses effective relationships and the supplied invocation
control; internal keys still do not charge result text. This intentionally
increases string-value work charges where previously omitted. It is a bounded
execution repair, not a performance optimization or grammar expansion.

The regression checks equal string-value charges, exact N versus N-minus-one
ceilings, `FXCT0002` with the correct work domain, cancellation during key
traversal, and complete-reference recovery. Full verification passes with
1,593 core tests, the same adapter counts, and the official WASM target check.
Corpus replay retains the counters above; dynamic substring composition is
still unsupported. No new corpus credit is claimed.

The private sort owner is now 846 lines, the compatibility value owner 971,
and the value-evaluator composition root 3,144. Retain these owners under
ADR-0004 for this narrow control repair: the root adds only the typed lexical
forwarder, while traversal stays with its existing semantic owner. No directory,
public surface, authority, prepared representation or mutable state is added.

## Adjacent sort string traversal controls

The adjacent sort review found seven more calls to uncharged source
`string_value`: ordinary paths, unions, key results, numeric positional
selection, the temporary-tree truthiness positional branch, child-name
selection, and substring-after. Before repair, the new regression matrix
failed on the ordinary `name` key because only the four output attribute
string visits were charged. This is a reproduced control defect, not a
hypothesis about performance.

These branches now share a private controlled selected-node conversion. It
uses the actual source document and invocation control, returns an empty
string for an empty selection, and reports the existing structured control
failure. Selection, stable ties, first-node conversion and key typing are
unchanged. This deliberately increases the previously omitted
`XdmStringValueNode` charges; it does not introduce result-text charges or
widen expression grammar.

The seven-expression regression matrix exercises paths, unions, both admitted
numeric positional forms, child-name selection, substring-after and key lookup.
Each runs against the immutable visibility view and complete derivation,
checking exact output, N and N-minus-one string-work ceilings, `FXCT0002` with
the correct work domain, cancellation during key traversal and successful
reuse afterward. The temporary-tree truthiness branch uses the same repaired
conversion but is not separately exercised by this matrix.

The private sort owner is 865 lines and the focused test owner 399. Retain
these cohesive owners under ADR-0004: no new module, directory, state carrier,
public surface, authority or representation is introduced. This control repair
does not claim additional conformance passes or a performance improvement.

Complete verification passes with 1,594 core tests (57 manual probes ignored),
22 native adapter tests (three ignored), four WASM adapter tests and 18 worker
tests, plus formatting, strict Clippy, documentation, links, provenance and
inventories. The official WASM target check also passes. Complete OASIS replay
retains 2,491 / 3,173 exact XML comparisons (78.51%), 2,692 initialized cases,
2,643 successful executions, 478 initialization failures, 49 execution failures,
14 XML mismatches and four comparator gaps. `sort36` remains explicitly
unsupported at dynamic substring-bound composition. Work remains uncommitted.

## Bounded dynamic substring composition

The next tranche closes unchanged `Lotus/sort_sort36#1` through shared typed
expression primitives, not a fixture recognizer. Its conditional concat key
uses nested substring sources, character lengths, boolean-to-number conversion
and multiplication. Ordinary value output and concat operands now use the same
private compiler and lexical evaluator for these forms. The previous
constant-bound path implementation remains unchanged as the reference path.

The admitted source grammar is ordinary paths, nested `substring()` and
literal-delimiter `substring-after()`. Bounds are finite numeric literals,
lengths of admitted string sources, multiplication, and numeric conversion of
`starts-with(path, literal)` or `contains(., literal)` with bounded `not()`.
Numeric conversion does
not reinterpret arbitrary numeric or node-set operands as booleans. Recursive
string/bound composition is limited to depth 16 and 128 parsed nodes per
operand; the boolean shape check has its own depth-16 bound. Variable bounds,
other arithmetic, arbitrary nested functions, modern grammar and secondary
document concat sorting remain unadmitted.

Path-based `starts-with()` boolean conditions now share controlled first-node
conversion and predicate evaluation with ordinary output. Existing context
`contains()` routing remains intact. An initial full replay correctly exposed
a routing regression: the new boolean path branch intercepted the existing
`starts-with(translate(...), ...)` implementation. Narrowing eligibility to
path operands restores that route; the existing focused translation regression
and complete replay prevent crediting one new pass at the cost of an old one.
Argument splitting also respects nested calls and quoted commas.

The evaluator keeps intermediate strings separate from result construction,
charges source string traversal to the real invocation, and precharges UTF-8
byte scans before length/substring work. Lengths and positions count Unicode
characters, not bytes. Compiled-retention accounting includes every boxed
string/bound node, path, delimiter and boolean operand. Restricted key-use and
global initialization do not acquire the new operands or dependencies.

Five added regressions cover ordinary-output/concat parity, nested sources,
first-node conversion, empty selections and delimiters, fractional rounding,
Unicode character lengths, stable sorting, exact operation/string-work limits,
cancellation and recovery, both whitespace representations, compiler depth and
node guards, arbitrary-number rejection, ordinary boolean conditions, and
continued namespace-focus rejection at compilation. The existing modern and
unsupported sort tests retain their respective profiles.

The complete OASIS replay gains exactly one XML comparison: 2,492 / 3,173
(78.54%). Initialized cases increase to 2,693, successful executions to 2,644,
and initialization failures decrease to 477. Execution failures remain 49,
XML mismatches 14, comparator gaps four, and expected-error credit 423 / 431.
No reference bytes, comparator rules, catalog denominator or policy exclusions
change. The original archive hash and standards edition above remain the
authority for this observation; this is compatibility evidence, not a blanket
conformance claim.

Under ADR-0004, the new private compiler owns only bounded typed string
composition (261 lines), and its 85-line evaluator owns lexical execution.
The runtime composition root remains 6,687 lines and adds only typed dispatch.
Retain that root for this slice with its existing decomposition debt, rather
than adding recursive parsing or new state there. The 3,548-line value compiler
and 3,161-line value evaluator gain narrow forwarding only. Retain the cohesive
1,007-line compatibility owner for the shared predicate helper; review a named
lexical string-function extraction before further expansion. The compiler's
17-file directory merits review: feature owners remain explicit, and a future
string-expression subgroup is a credible seam, but this checkpoint does not
justify moving unrelated owners or adding ceremonial nesting. No new public
contract, host authority, representation, unsafe surface or backend is selected.

Final `scripts/verify.ps1` passes all gates with 1,599 core tests (57 manual
probes ignored), 22 native tests (three ignored), four WASM tests and 18 worker
tests. The official all-features WASM target check, full release OASIS replay,
Markdown links and diff checks pass. Windows incremental-cache hard-link
fallback warnings do not relax any gate. Work remains uncommitted alongside
the preceding corpus tranche. No performance improvement is claimed.
