# Private Golden Transform Slice

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Revision under test | Working tree after `a380d53` |
| Fixture | `corpus/golden/hello` |
| Scope | Private version-neutral architecture experiment |
| Informs | AR-0001, AR-0003, AR-0004, AR-0007, AR-0008, and AR-0009 |

## Purpose

Execute one exact source/stylesheet/result case through the intended semantic
owners without creating a public API or claiming an XSLT/XPath version. The
fixture uses syntax shared by several candidate standards profiles, so passing
it tests architecture rather than choosing AR-0001.

## Exercised path

```text
bounded resource builder
    -> sealed in-memory snapshot
    -> stylesheet XML events
    -> owned stylesheet XDM
    -> private XSLT/XPath compilation
    -> identified unordered transform set
    -> source XML events and owned XDM per invocation
    -> semantic result tree
    -> separate XML serialization
    -> golden comparison
```

The stylesheet is compiled once and reused for two requests over one admitted
source. The experiment deliberately executes requests in reverse submission
order and stores results by logical request identity. A batch of one uses the
same execution and serialization functions.

## Admitted private syntax

The compiler recognizes only the fixture pressure:

- an XSLT-namespace `stylesheet` root with a retained declared version;
- zero or one XML `xsl:output` declaration, preserving absence for runtime
  method inference;
- one `xsl:template` with `match="/"`;
- literal result elements without attributes or result namespaces;
- literal text;
- `xsl:value-of`; and
- an unprefixed relative child-name path, specifically `greeting/name`.

The compiled form is XSLT-owned semantic data with retained source locations;
it does not retain XML parser events or XDM node handles. XPath parsing and
evaluation remain owned by the XPath layer.

## Results

The semantic result is inspected before serialization:

```text
document result
`-- element message
    `-- text "Hello, FastXSLT!"
```

The serializer then produces:

```xml
<message>Hello, FastXSLT!</message>
```

The corpus file contains a final LF as a repository text-file terminator. The
test compares the serializer output plus that harness terminator to the exact
fixture bytes; the semantic result and serializer string are asserted
separately.

Thirty-nine focused unit tests pass across resource, XML, XDM, XPath, compile,
runtime, batch, and serialization experiments. Negative cases establish private
machine identities and categories for:

- invalid stylesheet structure or missing required attributes;
- well-formed but unsupported XSLT instructions;
- well-formed but unsupported XPath syntax;
- missing source or stylesheet resources;
- explicit denial of an otherwise admitted source;
- duplicate request and result identities; and
- transform-set request budget exhaustion and bounded serialization output.

A request whose source identity equals a sibling's logical result identity is
rejected as missing because that result was never admitted to the snapshot. This
is direct evidence for ADR-0005's “produced does not mean admitted” rule.

## Capability and lifecycle observations

- The evaluated path requires document child navigation, element expanded-name
  matching, document order, and element string value.
- Source documents are parsed per invocation. This is AR-0009's safe reference
  behavior, not a performance recommendation.
- Compiled stylesheet state is reused while source XDM and result construction
  remain invocation-local.
- Logical result names resembling files grant no output authority; results stay
  in memory.
- Scheduling order does not affect result correlation or meaning in the two-case
  experiment.

## Limitations

This is not XSLT, XPath, XML, XDM, or serialization conformance evidence. It has
no public facade, host adapter, real parallel executor, deadline, parameters,
messages, namespaces in results, attributes in literal results, template
selection, general XPath, multi-node `xsl:value-of` conversion, non-output
runtime budgets, or failure collection. Private `FX*` identifiers are
experimental and are not stable standards codes or public compatibility
promises.

The next language work should use representative consumer transforms to close
AR-0001 before widening behavior. Cooperative cancellation now has private
charge-point and fault-injection evidence; deadlines, observation latency,
broader runtime budgets, and a real host boundary remain unresolved.
