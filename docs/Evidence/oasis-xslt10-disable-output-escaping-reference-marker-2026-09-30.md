# OASIS XSLT 1.0 Disable-Output-Escaping Reference Marker

## Purpose

Record the first AR-0024 implementation tranche for XSLT 1.0
`disable-output-escaping="yes"` without turning authored text into semantic
result markup or exposing a new public XDM/API contract.

## Implemented boundary

- Exact-version-1.0 `xsl:text` and `xsl:value-of` may retain a private compiled
  plan fact.
- Execution creates a distinct private result-text marker. Ordinary text keeps
  the existing escaped path and adjacent marked/unmarked values do not
  coalesce across the boundary.
- XML and HTML serialization omit ordinary markup escaping only for marked
  text. Text serialization remains character-for-character equivalent.
- Attribute, comment, and processing-instruction string construction consumes
  the lexical value and discards the marker.
- Invocation-owned temporary trees retain marker membership by temporary node
  identity so `xsl:copy-of` can preserve the request without changing the
  ordinary temporary text-node kind.
- XSLT versions outside the exact 1.0 compatibility path retain explicit
  `FXST1060` unsupported behavior.

The marker remains private to the engine. It is not parsed as markup and does
not create element, namespace, attribute, or node identity semantics.

## Comparator boundary

Some archival expected results intentionally contain ill-formed lexical output
created by disabled escaping while still requesting XML comparison. The local
OASIS-only verifier first attempts its normal document and fragment comparison.
Only when both actual and expected values are unparseable does it use a bounded
lexical fallback that removes inter-tag layout whitespace and normalizes an
immediately empty start/end pair to the equivalent empty-element spelling.
Content differences remain mismatches. This is verifier behavior, not engine
serialization behavior or a claim that the result is well-formed XML.

## Conserved measurement

The hash-verified OASIS XSLT/XPath 1.0 Committee Draft 04 denominator remains
3,173 catalog cases.

| Counter | Result |
| --- | ---: |
| Exact expected-result matches | 2,316 / 3,173 (73.00%) |
| Initialized | 2,499 |
| Successfully executed | 2,453 |
| Initialization failures | 671 |
| Execution failures | 46 |
| Visible comparison mismatches | 2 |
| Comparator gaps | 3 |
| Expected-error credit | 423 / 431 (98.14%) |

Relative to the preceding 2,287 / 3,173 snapshot, the strict exact lower bound
increases by 29 cases. The former `FXST1060` initialization frontier is gone;
cases that expose later semantic, reference-encoding, or comparison differences
remain visible rather than receiving credit.

## Verification

- Focused compile tests prove plan-fact retention.
- Focused runtime tests prove ordinary escaping, marked XML serialization,
  temporary-tree copy preservation, and marker loss in attribute/comment/PI
  string construction.
- A focused verifier test proves the malformed-result fallback normalizes only
  the admitted layout/empty-element differences and still rejects differing
  raw content.
- The complete local OASIS sweep produced the conserved counters above.

## Non-claims

- This does not accept AR-0024 or stabilize a public result-node type.
- This does not promise that marked XML output is well formed.
- This does not establish all character-map, encoding, adapter, or
  serialized-byte-budget interactions requested by AR-0024.
- This remains compatibility evidence, not an XSLT 1.0 conformance claim.
