# OASIS XSLT 1.0 Text-Output XML-Parameter Isolation

- Date: 2026-09-26
- Status: Verified semantic and corpus evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Two unchanged standard-operation cases reached execution with the text output
method but failed as `SEPM0004`. FastXSLT applied XML-only `standalone` and
doctype document-element preconditions before dispatching to text
serialization:

- `Microsoft/Output__84008` composes an XML output declaration carrying
  `standalone` with a later text-method declaration; and
- `Microsoft/Output__84012` explicitly supplies XML serialization properties
  on a text output declaration.

For the text method these properties do not impose XML document-shape or XML
declaration constraints on the result.

## Bounded change

The shared serialization precondition owner now establishes the effective
output method before applying declaration, doctype, and single-document-element
requirements. The text method ignores those XML-only parameters while
preserving:

- pending result-attribute validation;
- literal descendant-text serialization;
- byte and work budgets;
- XML/XHTML precondition behavior; and
- the separate rule that unrepresentable characters cannot be replaced with
  XML references in literal text output.

A focused test combines text output, `standalone`, a doctype system identifier,
declaration omission, markup, and multiple descendant text nodes.

## Corpus result

Both unchanged cases become exact expected-result matches:

- `Output__84008` uses the existing XML-wrapped text-reference comparator; and
- `Output__84012` uses exact normalized non-XML comparison.

The conserved sweep changes as follows:

- exact matches rise from 2,153 to **2,155 / 3,173 (67.92%)**;
- successful executions rise from 2,313 to **2,315**;
- execution failures fall from 51 to **49**;
- XML-semantic pass credit rises from 2,153 to **2,155**;
- exact normalized non-XML comparisons rise from six to **seven**; and
- XML-wrapped text comparisons rise from two to **three**.

Initialization remains 2,364 with 806 classified initialization failures. The
two doubt-annotated numeric-rendering mismatches remain visible.

## Verification

```powershell
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
