# R0 P04/P05 expression and introspection boundary audit

Date: 2026-10-23. Source: `b4c92004` plus the pending R0 P06 repair and this
test-only boundary matrix. Tranche 2 objective: map the existing admitted
expression owners and establish what must remain refused, not expand grammar.

## Result

No new semantic owner or feature is required by this inspection. The current
reference deliberately admits numeric processor-version reporting in selected
value/test positions, while valid availability queries and other properties
remain unsupported. A new six-owner matrix proves that distinction across
both legacy-normal and legacy-forward version contexts. No capability table
was added and no unsupported expression was admitted to improve a numerator.

This is closure evidence for the existing private boundary, not whole-gate
closure for a non-test profile that has not yet been migrated. P04/P05 remain
in progress; migration must preserve the routes below and replay their witnesses.

## P04 finite owner map

Owners are under `compile/instruction_compiler/processor_edition_reference/`
unless otherwise stated. An unsupported expression is not an invalid admitted
expression; lack of implementation must not become a deferred standards error.

| Expression-bearing owner | Existing boundary / error behavior | Witness or next migration check |
| --- | --- | --- |
| `matched_integration::value_of_with_options` | Guards literal/path/value families before shared lowering; selected numeric reporting is handled explicitly. Only narrowly classified path syntax and literal property-call arity failures use legacy-forward deferral. General compiler failures are not caught. | `selected_value_of_options_preserve_scope_error_timing_and_guards`, original ForwardComp replay; preserve the admission-before-lowering order. |
| `conditional_integration` / `boolean_call_timing` | Parses the complete bounded Boolean shape, resolves unbound variables over both operands, then applies edition-sensitive arity timing. Shared Boolean IR short-circuits at execution; every conditional body is compiled. | `processor_policy_conditionals_keep_lexical_errors_static_in_skipped_branches`, `processor_policy_conditionals_pin_literal_multiargument_call_boundaries`, numeric-QName timing tests. |
| `matched_integration::local_variable` | Selects are bounded strings, names and attributes; text and single-value constructors use existing guarded paths. Unsupported general selects do not enter a generic expression compiler. | Existing binding/scope tests plus the six-owner introspection refusal matrix. Preserve constructor child-version handling. |
| `validate_selected_global_binding` / `lower_selected_global` | Bounded literal/string conversion and the already-admitted sealed document-global shape. General introspection/arithmetic select does not enter global lowering. | Constant-global admission/refusal and module-graph global-visibility tests; preserve sealed resource ownership and the fixed grammar. |
| `validate_literal_parameters` | Parameter defaults retain their explicit eligibility; not a second unrestricted expression path. | Existing named-template/default tests and the new parameter-owner positive/refusal controls. |
| `template_arguments` | Shared invocation lowering only after argument/name/select/content admission. Unbound argument variables remain static even in dormant bodies; admitted argument content uses selected value lowering. | `argument_binding_errors_remain_static_in_dormant_bodies`, `comparison_binding_errors_remain_static_across_short_circuit`, `value_argument_keeps_unintegrated_constructor_boundaries`. |
| Literal-result attributes | Constants and the already-admitted whole literal/variable-concat AVT only; selected variable validation precedes shared literal construction. | Existing literal-concat scope tests. Arbitrary AVTs remain outside R0; do not implement them to populate a timing matrix. |
| Computed element names / attribute sets | Names are fixed or the admitted attribute-read form; namespace strings and attribute-set names are guarded. Attribute-set members remain static text constructors. | `selected_computed_name_admission_does_not_open_general_avts`, computed-name/control and static-set tests. No generic expression timing obligation is created for a refused AVT. |
| `selected_keys` | Declaration match/use and lookup key/value grammar are bounded. Variable lookup arguments validate bindings before shared lookup lowering. | Existing key-lookup match, count, focus and control tests. Dynamic key names and arbitrary use expressions remain refused. |
| Selected match predicates, child iteration/copy, number and apply selection | Admission functions restrict the expressions before shared compilers run; they do not inherit arbitrary baseline capability. | Existing match-predicate, source-copy, positional number and template-argument controls. Preserve each eligibility function during migration, rather than invoking the baseline compiler as fallback. |

This map names the current owners, not a claim that every XPath feature or
combination is proved. No new generic expression parser, error-catching policy
or edition rule was selected. Any concrete failure of an admitted combination
must be repaired or returned under the existing obligations, not reclassified
as unsupported to claim closure.

## P05 reporting and availability map

| Surface | R0 private behavior |
| --- | --- |
| `system-property('prefix:version')` with prefix bound to the XSLT namespace | Numeric `1.0` leaf for selected legacy-normal and legacy-forward; independent of stylesheet-declared version. Value-of performs text conversion; numeric comparisons retain numeric semantics. |
| QName aliases / other namespace / unbound prefix | The shared QName parser resolves aliases; the version folder does not accept a different namespace or invent a value for an unresolved name. Existing malformed-call witnesses retain their explicit error behavior. |
| `element-available()` / `function-available()` with valid arguments | Unsupported in this selected integration. They do not inherit the baseline availability table and are not folded to false merely because the reference lacks entries. |
| Other valid `system-property()` queries | Unsupported; no invented vendor, URL or empty-string answer. |
| Invalid/arity calls already admitted by timing probes | Preserve existing selected error timing and original provenance; refusal of valid unimplemented queries must not erase these admitted error witnesses. |
| Modern reporting / general typed bindings / arbitrary arithmetic around version | Not admitted by this legacy reporting slice; no public support claim or selector is created. |

`selected_numeric_version_leaf_uses_qnames_and_numeric_comparisons`,
`selected_version_property_is_numeric_and_independent_of_declared_version`,
`selected_version_composition_preserves_deferred_and_static_errors`, and
`selected_numeric_reporting_preserves_shared_controls_and_cleanup` supply the
positive/type/timing/control evidence.

The new
`selected_introspection_refusals_do_not_turn_into_availability_answers` covers:

- six owners: value-of, condition, local variable, global variable, parameter
  default and named-template argument;
- two declared versions: 1.0 and 2.0 under the selected legacy processor;
- three valid but unimplemented queries: function availability for `document`,
  element availability for `xsl:sequence`, and the vendor property;
- a successfully compiled positive control for each owner/version pair;
- all 36 negative cells remain Unsupported with original resource provenance.

These are regression cells, not corpus cases or additional promotion gates.

## Reproduction

```powershell
$env:FASTXSLT_OASIS_XSLT10_ROOT = (Resolve-Path .workbench/oasis-xslt10/extracted-full/testsuite/TESTS).Path
cargo test -p fastxslt --all-features selected_introspection_refusals
cargo test -p fastxslt --all-features processor_edition_reference -- --include-ignored
cargo test -p fastxslt --all-features selected_policy_ -- --include-ignored
./scripts/verify.ps1
cargo build -p fastxslt-wasm-workbench --target wasm32-unknown-unknown
```

Local logs: `.workbench/r0-p04-p05-{boundaries,probes,replay,verify,wasm}.log`.
The targeted matrix, 229 processor-policy probes and 24 selected replay/control
tests pass. Repository verification passes formatting, Clippy, 2,291 workspace
tests (2,247 core + 22 native + 4 WASM + 18 worker), docs/links, unsafe surface
and pinned conformance checks. The official WASM build passes. Incremental
cache hard-link fallback warnings did not prevent compilation or validation.

## Progress and next boundary

No corpus gain is claimed; the prior production baseline remains 2,504/3,173
(78.92%). There is no new provisional blocker or scope change. P04/P05's next
closure action is preserving and rerunning this finite owner map through P01's
non-test entry, not implementing availability functions or arbitrary expressions.
The next planned tranche is P08's full private selected-profile measurement,
with unsupported families left visible in the conserved denominator. Review
after that third tranche; do not expand the grammar in response to its score.
