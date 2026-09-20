# OASIS XSLT 1.0 Module Decimal-Format Composition

Date: 2026-09-19  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Finding

Each stylesheet module previously resolved `format-number()` against only the
`xsl:decimal-format` declarations in that physical module. The module compiler
then composed already-specialized programs. A default decimal format declared
by an included or imported module therefore did not bind calls in the
principal module, even though the declaration belongs to the composed
stylesheet static context.

## Repair

- Retain compiled decimal-format definitions as immutable stylesheet-derived
  state in the private `StylesheetProgram`.
- Merge distinct definitions while composing admitted include/import graphs,
  then specialize every retained `format-number()` expression against the
  effective definitions after composition.
- Let a principal declaration shadow the same-name imported declaration and
  rebind expressions in imported templates to that effective higher-precedence
  definition.
- Keep same-name decimal formats across included modules explicit as
  `FXST1093`; the current slice does not approximate the full same-precedence
  property-merging/error-recovery rules.
- Preserve direct typed formatting plans at execution. No runtime declaration
  registry, resolver access, or version branch was added.

Focused tests cover dependency-to-principal binding, principal-over-import
precedence across the whole program, and the explicit same-precedence include
boundary.

## Corpus result

The unchanged Lotus `numberformat45` import case and `numberformat46` include
case now both compare exactly:

- catalog cases: unchanged at 3,173;
- initialized: unchanged at 2,033;
- successfully executed: 1,932 -> 1,934;
- exact XML-semantic matches: 1,803 -> 1,805;
- visible XML mismatches: unchanged at 54;
- execution failures: 101 -> 99.

## Boundary conclusion

This tranche establishes effective default decimal-format binding across the
currently admitted module graphs and higher-precedence principal shadowing.
It does not claim complete same-name composition at one import precedence,
dynamic third-argument format names, arbitrary module graphs, or a public
decimal-format representation.
