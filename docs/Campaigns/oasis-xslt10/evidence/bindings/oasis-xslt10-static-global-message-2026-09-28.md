# OASIS XSLT 1.0 Static Global Message -- 2026-09-28

Date: 2026-09-28  
Status: Verified semantic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

Unchanged Lotus `message_message16` constructs a global parameter value from a
non-terminating, static `xsl:message` followed by a source-path
`xsl:value-of`. FastXSLT already owned both the bounded global temporary-text
plan and invocation-owned message observations, but the global constructor
did not compose them.

The neighboring `variable_variable40` case requires general `xsl:apply-templates`
execution while constructing a global temporary tree. That broader behavior is
not inferred by this tranche and remains visibly unsupported.

## Change

The XSLT 1.0 global temporary-text plan may retain a static, non-terminating
message part. Compilation accepts only a message body that lowers completely
to static text. Runtime charges the message instruction, text result node, and
text bytes before recording the observation on invocation-local control state.
The message contributes no characters or nodes to the temporary tree.

Dynamic message constructors, terminating global messages, general instruction
execution, and template dispatch during global construction remain outside
this bounded representation.

## Corpus result

Unchanged `Lotus/message_message16#1` initializes, executes, and compares
exactly. The remaining full-template-dispatch case moves to its later explicit
global-constructor frontier and earns no inferred credit.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,430 | 2,431 | +1 |
| Executed successfully | 2,380 | 2,381 | +1 |
| Initialization failures | 740 | 739 | -1 |
| Execution failures | 50 | 50 | 0 |
| Exact XML expected-result matches | 2,219 | 2,220 | +1 |
| XSLT-Result-Tree exact matches | 1,594 | 1,595 | +1 |

The conservative exact-match ratio is `2,220 / 3,173 = 69.97%`.
Expected-error credit remains 423 / 431, comparator gaps remain 53, and XML
mismatches remain nine.

## Verification

A focused runtime test proves message/tree separation, invocation-local
observation, and failure before message publication when the result-text budget
is exhausted. The unchanged archival case then proves the composed behavior.

```powershell
cargo test -p fastxslt --all-features xslt10_static_global_message_is_invocation_owned_and_not_tree_content
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/message_message16#1'
./scripts/verify.ps1
```

## Normative references

- [XSLT 1.0 section 11.4, Top-level Variables and Parameters](https://www.w3.org/TR/1999/REC-xslt-19991116#top-level-variables)
- [XSLT 1.0 section 13, Messages](https://www.w3.org/TR/1999/REC-xslt-19991116#message)
