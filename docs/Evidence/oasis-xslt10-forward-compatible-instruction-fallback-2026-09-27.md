# OASIS XSLT 1.0 Forward-Compatible Instruction Fallback -- 2026-09-27

Date: 2026-09-27  
Status: Verified implementation and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

Six unchanged Microsoft `ForwardComp` cases exercised elements that XSLT 1.0
does not allow as instructions: an unknown `xsl:foo` and the top-level-only
`xsl:include` used inside template content. FastXSLT rejected every occurrence
during compilation, including instructions in templates that were never
instantiated and instructions with an `xsl:fallback` child.

XSLT 1.0 section 2.5 requires forward-compatible instruction errors to be
deferred until instantiation. When instantiated, the processor performs the
standard fallback behavior; an instruction without fallback remains an error.

## Change

The XSLT 1.x compatibility compiler now distinguishes XSLT 1.0 template
elements from future or misplaced XSLT-namespace elements. In the bounded
forward-compatible interval:

- an instruction with one or more `xsl:fallback` children compiles only the
  fallback sequence constructors;
- other instruction content is ignored without validation or retention; and
- an instruction without fallback compiles to a small private deferred-failure
  plan carrying only its local name and source location.

Executing that plan returns structured invalid `XTDE1450`. Merely compiling or
retaining it does not fail, so an uninstantiated template remains harmless.
Supported stylesheet versions retain the existing strict instruction compiler.

The new private plan participates in compiled-state retention accounting,
semantic inspection, named-template validation, cancellation/work charging,
and ordinary source-location diagnostics. It adds no resource access or
runtime version branch.

## Corpus result

Four unchanged standard-operation cases now compare exactly:

- `Microsoft/ForwardComp__91842#1` executes `xsl:foo` fallback content;
- `Microsoft/ForwardComp__91843#1` retains but never instantiates `xsl:foo`;
- `Microsoft/ForwardComp__91845#1` executes fallback for a misplaced
  `xsl:include`; and
- `Microsoft/ForwardComp__91846#1` retains but never instantiates that
  misplaced instruction.

Two unchanged expected-error cases now fail during execution rather than
initialization:

- `Microsoft/ForwardComp__91841#1`; and
- `Microsoft/ForwardComp__91844#1`.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,402 | 2,408 | +6 |
| Executed successfully | 2,355 | 2,359 | +4 |
| Initialization failures | 768 | 762 | -6 |
| Execution failures | 47 | 49 | +2 |
| Expected errors observed during initialization | 390 | 388 | -2 |
| Expected errors observed during execution | 33 | 35 | +2 |
| Exact expected-result matches | 2,195 | 2,199 | +4 |
| Visible mismatches | 9 | 9 | 0 |

The conservative all-catalog exact-match ratio is now
`2,199 / 3,173 = 69.30%`. Expected-error credit remains 423 / 431 and
comparator gaps remain 52.

## Boundaries

- This is XSLT 1.x forward-compatible behavior selected during compilation.
- It does not make unknown instructions executable or extend the XSLT
  namespace.
- Only standard `xsl:fallback` children contribute semantics; sibling content
  is ignored.
- A missing fallback is a deferred transformation error, never silent success.
- The existing aggregate work budget charges the deferred instruction before
  it can fail.
- The result is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Verification

- Focused compiler tests prove fallback-only lowering and deferred failure
  retention.
- The four unchanged standard-operation cases compare exactly.
- The two unchanged error cases produce execution-time `XTDE1450`.
- The complete 3,173-case catalog sweep produced the counters above.

## Normative reference

- [XSLT 1.0 section 2.5, Forwards-Compatible Processing](https://www.w3.org/TR/xslt-10/#forwards)
