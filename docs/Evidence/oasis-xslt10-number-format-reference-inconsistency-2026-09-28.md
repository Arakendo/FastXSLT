# OASIS XSLT 1.0 Number-Format Reference Inconsistency

Date: 2026-09-28  
Status: Verified corpus disposition  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

The unchanged `Microsoft/Number__91027#1` stylesheet exercises multi-level
`xsl:number` formatting with leading/trailing punctuation, punctuation-only
formats, and empty number lists. FastXSLT executes it, but the archival result
expects punctuation to be reused in ways not defined by XSLT 1.0.

## Standards check

XSLT 1.0 section 7.7.1 defines these relevant rules:

- a leading non-alphanumeric token prefixes the constructed string;
- a trailing non-alphanumeric token suffixes it;
- only non-alphanumeric tokens between format tokens are separator tokens;
- when there are more numbers than format tokens, the last format token is
  reused; and
- in the absence of a separator token, a period separates successive numbers.

Therefore `format=";1"` formats the case's list as `;1.1.1`, not
`;1;1;1`, and `format="*"` uses the default numeric token with period
separators rather than treating the same punctuation as every separator.
When the selected count pattern produces an empty number list, `xsl:number`
does not emit the format's punctuation alone.

FastXSLT follows these rules. The archival expected output does not provide a
usable semantic oracle for this case.

## Disposition

`Microsoft/Number__91027#1` now carries an exact
`unusable-reference-result` disposition and receives no pass credit.

After this classification:

- exact expected-result matches remain `2,221 / 3,173 = 70.00%`;
- visible XML mismatches fall from seven to six; and
- unusable archival reference results rise from 55 to 56.

## Boundaries

- No result bytes, corpus metadata, or engine semantics were changed.
- The complete-catalog denominator remains 3,173.
- This does not claim general locale-specific numbering support.
- The suite's bare doubt inventory entry is not treated as substantive
  gray-area metadata and does not manufacture credit.

## Verification

```powershell
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Number__91027#1'
./scripts/verify.ps1
```

## Standards source

- [W3C XSLT 1.0 section 7.7.1](https://www.w3.org/TR/1999/REC-xslt-19991116#convert)
