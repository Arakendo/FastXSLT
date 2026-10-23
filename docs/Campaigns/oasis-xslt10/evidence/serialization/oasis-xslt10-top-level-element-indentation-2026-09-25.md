# OASIS XSLT 1.0 top-level element indentation

Date: 2026-09-25

## Result

The XML serializer now inserts one indentation line break between adjacent
top-level result elements when the effective output settings request
`indent="yes"`.

The rule is deliberately narrower than whitespace normalization. Existing
top-level text remains semantic result content, text still separates
neighboring elements when present, mixed element content is unchanged, and
`indent="no"` does not add whitespace. Output budget and cancellation
enforcement continue through the existing bounded writer.

## Corpus effect

Before host-environment classification, eight unchanged Microsoft whitespace
cases move from visible mismatch to exact XML-semantic comparison pass:

- `Whitespaces__91221` and `Whitespaces__91225`;
- `Whitespaces__91421`, `Whitespaces__91424`, `Whitespaces__91429`,
  `Whitespaces__91431`, `Whitespaces__91437`, and `Whitespaces__91439`.

The `91221` through `91228` family explicitly varies the historical MSXML
`preserveWhiteSpace` source-parser setting. FastXSLT's engine-owned source model
does not expose that ambient parser switch. The measurement therefore gives
all eight cases one explicit `host-parser-policy-excluded` disposition instead
of claiming four coincidental passes and four engine mismatches.

After that classification, the complete archival measurement advances from
2,112 to **2,116 exact XML-semantic matches out of 3,173 catalog cases
(66.69%)**. Initialized cases remain 2,328, successful executions remain 2,281,
visible XML mismatches fall from 84 to 72, eight cases carry the explicit host-
parser-policy exclusion, and comparator-unsupported outcomes remain 77. The
net four-pass gain reflects six newly exact `914xx` cases minus two previously
counted `912xx` coincidences that are now honestly excluded.

## Validation

- `cargo test -p fastxslt requested_indentation_separates_adjacent_top_level_elements_only`
- `cargo test -p fastxslt oasis_host_parser_whitespace_policy_exclusion_is_exact_and_bounded --all-features`
- `scripts/measure-oasis-xslt10.ps1 -TraceCase Microsoft/Whitespaces__91221#1`
- `scripts/measure-oasis-xslt10.ps1`
