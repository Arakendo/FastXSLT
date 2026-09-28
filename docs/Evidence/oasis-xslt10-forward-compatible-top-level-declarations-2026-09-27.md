# OASIS XSLT 1.0 Forward-Compatible Top-Level Declarations -- 2026-09-27

Date: 2026-09-27  
Status: Verified implementation and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

Four unchanged Microsoft `ForwardComp` cases used an XSLT 1.x future version
and an unknown XSLT-namespace top-level declaration. FastXSLT stopped at
`FXST1002` before compiling the ordinary root template.

XSLT 1.0 section 2.5 requires a top-level element not allowed by XSLT 1.0 to be
ignored with all its content when the containing stylesheet enables
forwards-compatible processing. The declaration's attributes and descendants
must therefore establish neither executable semantics nor compiler errors.

## Change

The single-document stylesheet compiler now recognizes the bounded XSLT 1.x
forward-compatible interval: a declared version greater than `1.0` and less
than `2.0`. An otherwise unknown XSLT-namespace top-level declaration is
ignored with its complete subtree in that interval.

Supported version declarations remain strict. The same unknown top-level
declaration under `version="1.0"`, `version="2.0"`, or `version="3.0"` still
returns explicit `FXST1002`. Known top-level declarations continue through
their existing typed compilers before this fallback is considered.

A focused regression embeds an invalid nested template inside the ignored
future declaration and proves that the nested content does not leak into the
compiled program. A companion regression preserves strict rejection for each
supported version.

## Corpus result

The following unchanged cases now compile, execute, and compare exactly:

- `Microsoft/ForwardComp__91837#1`;
- `Microsoft/ForwardComp__91838#1`;
- `Microsoft/ForwardComp__91839#1`; and
- `Microsoft/ForwardComp__91840#1`.

The `version="1.0"` negative sentinel
`Microsoft/Namespace__77658#1` remains an explicit `FXST1002` initialization
failure.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,398 | 2,402 | +4 |
| Executed successfully | 2,351 | 2,355 | +4 |
| Initialization failures | 772 | 768 | -4 |
| Execution failures | 47 | 47 | 0 |
| Exact expected-result matches | 2,191 | 2,195 | +4 |
| Visible mismatches | 9 | 9 | 0 |

The conservative all-catalog exact-match ratio is now
`2,195 / 3,173 = 69.18%`. Expected-error credit remains 423 / 431 and
comparator gaps remain 52.

## Boundaries

- This is compile-time XSLT 1.x compatibility behavior, not a runtime version
  branch.
- The declaration is ignored; it cannot introduce resource authority,
  executable instructions, static declarations, or retained state.
- Unknown declarations in supported `1.0`, `2.0`, and `3.0` stylesheets remain
  explicit failures.
- Instruction-level fallback and other forward-compatible attributes retain
  their existing independent boundaries.
- The result is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Verification

- The focused ignore-and-strictness compiler regressions pass.
- The four unchanged positive cases compare exactly.
- The unchanged XSLT 1.0 negative case retains `FXST1002`.
- The complete 3,173-case catalog sweep produced the counters above.

## Normative reference

- [XSLT 1.0 section 2.5, Forwards-Compatible Processing](https://www.w3.org/TR/xslt-10/#forwards)
