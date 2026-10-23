# Processor-edition seam investigation — 2026-10-16

Status: evidence and experiment proposal; no implementation change

Inputs: [AR-0031](../../../Architectural%20Reviews/AR-0031-compilation-scoped-processor-edition-profiles.md), [2026-10-16 handoff](../../../Checkpoints/2026-10-16-vscode-agent-handoff.md)

Corpus: locally acquired OASIS XSLT/XPath Conformance Test Suite CD04; archive SHA-256 66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5

## Scope and evidence limits

The original investigation below is historical. The following post-acceptance
replay updates the observed baseline without changing engine behavior.

### Post-acceptance corpus checkpoint — 2026-10-16

After ADR-0025 acceptance, fresh release-mode full replays preserve modern
2,501 exact / 14 mismatches and strict-preprocessing 2,498 exact / 17 mismatches,
with 423 expected-error credits in each lane. Strict preprocessing still does
not implement the accepted processor policy. All 3,173 cases remain accounted.

The modern remainder beyond exact matches and expected-error credits reconciles
as follows; categories are existing harness dispositions, not new exclusions:

| Disposition | Cases |
| --- | ---: |
| Ordinary initialization failures | 83 |
| Ordinary execution failures | 11 |
| Output mismatches | 14 |
| Unsupported XML comparator | 4 |
| Non-XML comparison not credited | 6 |
| Host parser policy exclusions | 24 |
| Host collation policy exclusions | 5 |
| Serialization layout policy exclusions | 18 |
| XSLT 1.0 discretionary policy exclusions | 5 |
| Legacy processor profile exclusion | 1 |
| Unusable reference results | 70 |
| Expected-error infrastructure exclusions | 3 |
| Unusable error expectations | 5 |
| **Total outside the two credit categories** | **249** |

Thus `2501 + 423 + 249 = 3173`. This is a ledger reconciliation, not 92.15%
conformance. The 94 ordinary phase failures include invalid inputs, missing
dependencies and deliberate boundaries as well as implementable gaps. Neither
94 nor 249 is a promised pass tranche. The output-exact percentage must not be
extrapolated as if all 672 remaining entries wanted successful output.

The fresh fallback trace reproduces initialization `FXST1006` at `xsl:foo` for
all six ordinary version-2.0 cases (91818–91821, 91823–91824). The expected-error
91822/91825 cases currently fail at the same unsupported boundary; their existing
error credit does not prove edition-correct error phase. 91817 executes and
91826 still rejects top-level fallback. Those controls must remain visible.

Next implementation target: make explicit processor policy reach the actual
compilation entry and shared sequence lowering for this family, with coherent
preprocessing and qualified artifacts. Do not just widen the version interval
or promote the limited reference callback. Start with 91818, then replay the
full ten-case family and both baselines as integration gates permit. Six ordinary
cases are a measured affected cluster, not a predicted gain or enough to reach
80%. Local scope, error timing and introspection must remain coherent under
ADR-0025; no public selector or production default change is implied.

Reproduction (all completed successfully):

```powershell
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Fallback__' -TraceFrontier '/'
./scripts/measure-oasis-xslt10.ps1
./scripts/measure-oasis-xslt10.ps1 -PreprocessingProfile strict-xslt10
```

This checkpoint and contributor-guidance update change no Rust or upstream
fixtures. No new corpus credit is claimed.

### Original investigation scope

This investigation reads the proposed review, handoff, unchanged catalog and
stylesheets, implementation owners, and prior evidence. It does not run tests
or a corpus replay. It changes no production behavior, fixture, AR/ADR, or
public API. The inventory records catalog operation and fixture shape; it does
not claim new case outcomes.

The handoff's last complete replay is dated 2026-10-15: modern was 2,501 /
3,173 exact, strict preprocessing was 2,498 / 3,173 exact, and each profile
credited 423 / 431 expected-error cases. AR-0031 separately records an ordinary
Microsoft/Fallback__91818#1 trace ending in FXST1006. No other member of this
fallback family was replayed for this investigation.

## Observed catalog inventory

The CD04 catalog contains ten Microsoft/Fallback cases with IDs 91817–91826.
Catalog scenario operation divides them this way:

| Catalog operation | Case identities | Count | Fixture shape |
| --- | --- | ---: | --- |
| Ordinary (standard) | Microsoft/Fallback__91817#1, Microsoft/Fallback__91818#1, Microsoft/Fallback__91819#1, Microsoft/Fallback__91820#1, Microsoft/Fallback__91821#1, Microsoft/Fallback__91823#1, Microsoft/Fallback__91824#1 | 7 | 91817 declares 1.0; 91818–91824 declare 2.0 and exercise unknown xsl:foo with fallback content. |
| Expected execution error (execution-error) | Microsoft/Fallback__91822#1, Microsoft/Fallback__91825#1, Microsoft/Fallback__91826#1 | 3 | 91822 declares 2.0 and references unbound $var in fallback content; 91825 declares 2.0 and places xsl:fallback beside, not inside, xsl:foo; 91826 declares 1.0 and is a fallback-placement control. |

The version-2.0 processor-edition seam is an eight-case subset:

- Ordinary: 91818, 91819, 91820, 91821, 91823, 91824 (6).
- Expected execution error: 91822, 91825 (2).

The 1.0 cases 91817 (ordinary) and 91826 (expected error) are controls.
Within the 2.0 subset, fixtures cover one fallback, multiple fallbacks,
fallback sequence content, lexical variable use, nested unknown instructions,
an execution-time XPath failure, and fallback outside its unknown instruction.
All ten remain in the 3,173-case denominator. The immutable archive and local
extracted stylesheets are under .workbench/oasis-xslt10; neither was modified.

## Observed implementation ownership

| Concern | Current owner and behavior | Seam implication |
| --- | --- | --- |
| Declared-version validation and principal compilation | compile/golden_stylesheet_experiment.rs owns validate_declared_version, is_xslt10_forward_compatible_version, and the principal compiler; compile/stylesheet_module_compiler.rs validates simplified stylesheets. | Version syntax validation is not processor selection. No selected processor edition currently enters the compiler. |
| 1.0 forward-compatible interval | is_xslt10_forward_compatible_version accepts parsed values strictly between 1.0 and 2.0. | It cannot express a 1.0 processor treating stylesheet version 2.0 as future. |
| Local compatibility / forward processing | compile/instruction_compiler.rs owns uses_xslt10_compatibility and uses_xslt10_forward_compatible_processing; they walk ancestry and inspect local xsl:version / root version. A separate is_forward_compatible_processing in that file tests version > 3.0. | Rules are inferred from stylesheet declarations through multiple predicates; they do not receive processor identity. |
| Instruction recognition and lowering | compile/instruction_compiler.rs routes sequence elements, maintains a fixed is_xslt10_template_element list, and dispatches known instructions through compile_xslt_instruction. | Shared implementation knowledge must be separated from each edition's recognized vocabulary. |
| Fallback and deferred failures | The same module's compile_xslt10_forward_compatible_instruction gathers direct xsl:fallback children, lowers those sequences, or stores deferred XTDE1450 when none exists. Extension-element fallback is compiled separately. | Unknown XSLT instructions, extensions, and known-but-unimplemented instructions need distinct paths. |
| Expression compatibility | The instruction compiler feeds uses_xslt10_compatibility into expression/conditional compilation and defers a bounded set of expression failures in its forward-compatible path. | Changing fallback alone would leave expression semantics and error timing on declaration heuristics. |
| Availability and processor reporting | compile/instruction_compiler/xslt10_static_introspection_compiler.rs folds system-property, function-available, and element-available using hard-coded function/instruction lists. xsl:version currently folds to "1" when the stylesheet is deemed 1.0-compatible and "3.0" otherwise. | These answers follow stylesheet declaration today, not selected processor identity or a unified capability policy. |
| Graph compilation and preprocessing | runtime/resource_compiler.rs loads the dependency graph and chooses graph compilation; test-only runtime/resource_compiler/stylesheet_graph_context.rs recursively carries StylesheetCompilationContext through admitted include/import shapes. That context is specifically a preprocessing stamp under ADR-0023, not processor identity. | Select processor semantics once for the complete graph, without repurposing the preprocessing selector. |

Source: [version/compiler owner](../../../../crates/fastxslt/src/compile/golden_stylesheet_experiment.rs), [instruction compiler](../../../../crates/fastxslt/src/compile/instruction_compiler.rs), [static introspection compiler](../../../../crates/fastxslt/src/compile/instruction_compiler/xslt10_static_introspection_compiler.rs), [resource compiler](../../../../crates/fastxslt/src/runtime/resource_compiler.rs), [graph context](../../../../crates/fastxslt/src/runtime/resource_compiler/stylesheet_graph_context.rs), [preprocessing context](../../../../crates/fastxslt/src/compile/stylesheet_preprocessing.rs).

## Standards-backed behavior matrix

The comparison selects either an XSLT 1.0 processor or a modern XSLT 3.0
processor. Both receive a principal stylesheet declaring version="2.0".
Processor selection and stylesheet version have separate roles.

| Surface | Selected XSLT 1.0 processor; stylesheet says 2.0 | Selected modern XSLT 3.0 processor; stylesheet says 2.0 |
| --- | --- | --- |
| Version relationship | Version 2.0 is unsupported by the 1.0 processor and invokes XSLT 1.0 forward-compatible processing (§2.5). | Version 2.0 is earlier than the selected processor and invokes applicable backwards-compatibility rules, not future-version processing. |
| Unknown XSLT instruction with direct fallback | An unknown instruction reached under forward-compatible processing uses its xsl:fallback sequence. Without fallback, the error is deferred until execution as required by the 1.0 rules. | Version 2.0 is not future to a 3.0 processor. An unknown XSLT instruction remains an error; fallback does not make it a supported instruction. |
| Fallback shape | xsl:fallback is meaningful as a child of the unknown instruction (or extension element). A sibling is not a recovery arm for xsl:foo. | Same structural distinction; the 1.0 forward-compatible path does not apply. |
| Known newer instruction, e.g. xsl:sequence | Outside the 1.0 vocabulary, so forward-compatible unknown-instruction rules apply. | In the modern vocabulary. If implemented, compile it as known; if not implemented, report an engine capability limitation rather than treating it as unknown to admit fallback. |
| Expressions and deferred errors | Use 1.0/XPath 1.0 semantics plus the specific forward-compatible deferral rules when expressions are evaluated. | Use the modern processor's expression semantics plus the compatibility behavior applicable to version 2.0; the declaration does not switch processor identity or the expression parser to 1.0. |
| element-available() / function-available() | Reflect facilities available to the selected 1.0 processor; modern-only names are not exposed as 1.0 facilities. | Reflect modern processor availability and actual usable engine capability. Answers must agree with what compilation admits. |
| system-property('xsl:version') | Reports processor version 1.0, independent of the stylesheet's 2.0 declaration. | Reports processor version 3.0, independent of the stylesheet's 2.0 declaration. |
| Include/import graph | One processor selection applies to principal and transitive modules; local declarations govern only the version-sensitive rules defined by 1.0. | One modern selection applies throughout the graph; local declarations do not switch processor identity. |

Normative references: [XSLT 1.0 §2.5, Forwards-Compatible Processing](https://www.w3.org/TR/xslt-10/#forwards), [§14.1, Extension Elements](https://www.w3.org/TR/xslt-10/#extension), and [§12.4, additional functions](https://www.w3.org/TR/xslt-10/#misc-func); [XSLT 3.0 §§3.9–3.10, Backwards-/Forwards-Compatible Processing](https://www.w3.org/TR/xslt-30/#forwards). XSLT 3.0 includes modern instruction vocabulary such as xsl:sequence; its version rules distinguish stylesheet versions earlier than the selected processor from versions later than it.

These standards define processor behavior, not FastXSLT's present conformance.
“Available” must reflect usable implemented capability. The engine must not
report a modern feature available when it cannot execute it, and an engine
limitation must not change the edition's recognition rule.

## Proposed smallest private experiment

**Recommendation:** add a private/test-only processor-edition input at the
existing compilation boundary and run two differential lanes through the same
compiler/runtime. Keep it separate from StylesheetPreprocessingProfile and
StylesheetCompilationContext, which are preprocessing owners under ADR-0023.
Do not add a public selector, alter the production default, or add a second
engine.

1. Introduce one private edition-policy owner (for example,
   compile/processor_edition.rs) with explicit 1.0 and modern choices. It
   owns edition-relative version interpretation, forward-compatible
   processing, recognized instruction/function sets, and processor-version
   reporting. Keep edition recognition separate from engine implementation
   capability; availability combines both. Local xsl:version remains scoped
   input to this policy and cannot override processor identity.
2. Add a cfg(test) compilation entry point or test-only field on existing
   private compile limits. Select once before graph compilation and pass the
   immutable choice through root, recursive include/import, and special
   assembly paths. Do not use the preprocessing selector as a proxy. Ordinary
   production compilation stays on its current path.
3. First differential slice: unchanged OASIS 91817–91826, retaining 91817 and
   91826 as 1.0 controls. Record operation and selected processor for each
   result; predict no gain before running.
4. Focused private tests: direct/multiple/nested fallback; no-fallback
   deferred error in dormant and reached branches; sibling fallback error;
   fallback variable scope and expression error timing; modern-known
   xsl:sequence versus 1.0 unknown handling; availability and xsl:version;
   local version changes that do not change processor identity; and
   principal/include/import modules with mixed declarations under one
   selected edition.
5. Compare structured diagnostics and error phase, cancellation/work bounds,
   and compiled-artifact profile isolation. Then replay the unchanged fallback
   subset and complete conserved OASIS denominator under each coherent preset.
   Preserve the existing modern and strict-preprocessing baselines separately.

Cross-combinations of processor semantics and preprocessing may later serve as
private diagnostic controls; they are not needed for the initial proposal and
are not supported product modes.

### Likely decomposition review

instruction_compiler.rs is 3,411 lines and
golden_stylesheet_experiment.rs is 3,116 lines. Both exceed ADR-0004's
2,000-line explicit decomposition-review threshold. Version predicates and
instruction recognition cross these owners, so adding more scattered local
checks would deepen coupling. Review a private, named processor-edition policy
seam and route current call sites through it. Keep lowering and introspection
folding in their current owners unless review finds a cohesive recognition /
capability table that can move without broad mutable context or cycles. Do not
split crates or make broad refactoring a prerequisite for the probe.

## Disposition and next decision boundary

The evidence supports a concrete, bounded private experiment; it makes no
architecture choice or production change. Maintainer review is needed before
any non-test compiler path adopts processor-edition semantics. Public
naming/defaults, exact modern edition reporting, policy for known-but-
unimplemented modern features, and any future compiled-artifact reuse key
remain deferred.

No tests or corpus command were run for this document-only investigation.
