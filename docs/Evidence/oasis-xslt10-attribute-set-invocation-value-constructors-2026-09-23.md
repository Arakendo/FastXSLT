# OASIS XSLT 1.0 Attribute-Set Invocation Value Constructors

Date: 2026-09-23  
Status: Local compatibility evidence

## Question

Can a statically compiled `xsl:attribute-set` retain value constructors that
depend on the source context where the set is applied, without capturing
invocation state in the compiled stylesheet or admitting caller-local variable
scope?

## Implemented boundary

The private attribute-set compiler now admits two additional XSLT 1.0 value
forms:

- `xsl:value-of select="."`, evaluated against the application focus; and
- a bounded, variable-free sequence constructor, evaluated by the existing
  charged result-construction path.

Compiled attribute sets remain immutable stylesheet-derived state. Source
focus, runtime variables, cancellation, and work accounting remain invocation
owned. Constructors containing a lexical variable reference stay unsupported
unless they are the already admitted single global atomic-variable form; this
prevents a caller-local binding from being mistaken for the declaration's
static global binding.

`xsl:copy` still stores its static attributes in the older literal-attribute
representation. Its materializer now routes an admitted XSLT 1.0 sequence
constructor through the same execution function used by computed attributes,
rather than reaching an unreachable branch.

## Graph validation

Attribute-set dependency validation now inspects nested
`use-attribute-sets` applications inside an attribute value constructor as
well as the declaration's own attribute. This is validation only: nested set
references are not flattened onto the eventual result element. Direct
declaration references retain their existing expansion order.

Focused self-cycle and indirect-cycle tests prove that nested `xsl:copy`
applications report invalid `XTSE0720`. The unchanged Microsoft `err004`,
`err006`, and `91120` cases likewise expose circularity rather than falling
through to unsupported constructor handling or execution.

## Corpus effect

Against the unchanged 3,173-case OASIS archive:

- initialized cases rise from 2,233 to 2,236;
- successful executions rise from 2,173 to 2,176;
- exact XML comparisons rise from 2,025 to 2,027 / 3,173 (63.88%);
- `Microsoft/BVTs_bvt003#1` and `Microsoft/AttributeSets__91119#1`
  become exact;
- `Microsoft/AttributeSets_AttributeSets_WithPI#1` executes with the expected
  attribute value but remains a visible comparison mismatch because its
  reference output contains processor-added indentation inside an otherwise
  empty copied element;
- execution failures remain 60, expected-error unexpected successes remain
  five, and no execution panic remains; and
- the `FXST1065` initialization frontier falls from 13 to seven cases.

The seven remaining cases require attribute-set lookup or same-name
composition across imported/included stylesheet modules. This tranche does not
silently settle that import-precedence representation.

## Verification

- focused runtime tests cover application-focus string values and
  variable-free sequence construction;
- focused compiler tests cover nested self and indirect cycles;
- unchanged corpus traces cover the two new exact cases, the PI-shaped
  comparison boundary, and invalid nested cycles;
- the complete local sweep conserves all 3,173 identities.

The archive remains local and unmodified. This is compatibility evidence, not
a broad conformance claim.
