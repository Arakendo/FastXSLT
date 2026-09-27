# OASIS XSLT 1.0 Outer-Current Name and Attribute Predicate

- Date: 2026-09-24
- Status: Verified semantic and compatibility evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Three unchanged cases share the XSLT 1.0 value expression
`//*[name()=name(current()) and @cell='2']`. FastXSLT already retained the
outer source focus for bounded `name(current())` predicates, but this exact
composition with an unqualified literal attribute filter stopped at
`FXXP1019` before typed path execution.

## Implemented slice

XSLT 1.0 compilation now recognizes that exact descendant predicate family as
an owned typed value plan containing only the unqualified attribute name and
literal value. Execution performs charged document-order descendant and
attribute traversal, compares each candidate's lexical name with the outer
current node, and applies XSLT 1.0 first-node string conversion to the first
matching candidate.

A focused runtime test proves:

- the outer current node survives descendant predicate evaluation;
- the attribute filter excludes same-name siblings with other values;
- a differently named element with the requested attribute is excluded;
- the first matching descendant supplies the string value; and
- the same expression remains outside modern XPath compilation.

No general boolean-predicate grammar, dynamic attribute value, qualified
attribute name, alternate evaluator, or modern-XPath widening was introduced.

## Corpus result

The unchanged cases below now initialize, execute, and compare exactly:

- `Microsoft/Miscellaneous__78302#1`;
- `Microsoft/XSLTFunctions__84422#1`; and
- `Microsoft/XSLTFunctions__84484#1`.

The complete sweep moves from:

- 2,288 to 2,291 initialized cases;
- 2,236 to 2,239 successfully executed cases; and
- 2,077 to 2,080 exact expected-result matches.

Initialization failures fall from 847 to 844. Execution failures and XML
mismatches remain 52 and 76. The exact compatibility lower bound is therefore
**2,080 / 3,173 (65.55%)**.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_current_name_and_attribute_predicate
cargo test -p fastxslt --all-features xslt_current_predicate_does_not_enter
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Miscellaneous__78302#1'
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
