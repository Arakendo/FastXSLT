# OASIS XSLT 1.0 Cross-Module Attribute-Set Linking

Date: 2026-09-23  
Status: Local compatibility and shared compiler evidence

## Question

Can attribute-set declarations and uses compose across an admitted stylesheet
package while preserving import precedence, include order, static error
classification, and a lookup-free execution path?

## Implemented boundary

Each stylesheet module now retains private compiled attribute-set declarations:
the expanded set name, direct inheritance references, complete static dependency
references, typed computed attributes, source location, and import precedence.
Result constructors retain only unresolved expanded set names until the complete
stylesheet package has been assembled.

A private package-link step then:

- validates undefined and circular references across the complete module graph;
- composes same-name declarations in stable import-precedence order;
- preserves the declaration position of included modules;
- applies referenced sets before a declaration's own attributes;
- preserves explicit constructor attributes as the final overrides;
- retains required namespace bindings; and
- writes the resolved attributes into the immutable instruction plan.

Execution performs no attribute-set name lookup and owns no module-composition
state. The linker is also used for single-module stylesheets, so there is no
second attribute-set semantic path. Dynamic attribute-set names remain outside
this static slice.

## Corpus effect

Against the unchanged 3,173-case OASIS archive:

- initialized cases rise from 2,254 to 2,260;
- successful executions rise from 2,190 to 2,196;
- exact XML comparisons rise from 2,040 to 2,046 / 3,173 (64.48%);
- all six unchanged standard-result cases previously stopped by `FXST1065`
  become exact: Lotus `attribset37`, `attribset38`, `attribset45`, and
  `attribset46`, plus Microsoft `ImportAttributeSetOfSameName` and
  `IncludeAttributeSetOfSamePrecedence`;
- Microsoft `AttributeSets__91091`, the seventh former `FXST1065` case, now
  reaches and preserves its expected static error for an empty attribute-set
  name;
- initialization failures fall from 881 to 875; and
- execution failures remain 64, comparison mismatches remain 67,
  expected-error unexpected successes remain five, and no execution panic
  occurs.

## Verification

Focused tests cover same-name include composition, a template in one imported
module referring to declarations in a sibling import and the principal module,
same-name import-precedence override with non-conflicting attribute retention,
undefined references, direct and indirect cycles, nested-constructor cycles,
explicit constructor overrides, and source-copy use.

The local OASIS sweep then executes the unchanged package graphs. The archive
remains local and unmodified; this is compatibility evidence, not a conformance
claim.
