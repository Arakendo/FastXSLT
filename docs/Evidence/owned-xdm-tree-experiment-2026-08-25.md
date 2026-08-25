# Owned XDM Tree Experiment

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Scope | First private XML-event to engine-owned document model |
| Input | `corpus/golden/hello/input.xml` and two focused in-memory fixtures |
| Implementation | `crates/fastxslt/src/xdm/owned_tree_experiment.rs` under `cfg(test)` |
| Informs | AR-0007, AR-0008, and M1 Slice 2 |

## Question

Can the leading parser experiment hand owned semantic data to an engine-owned
tree without retaining input bytes, parser events, dependency node handles, or
parser-specific lifetimes?

## Method

The private XML adapter was extended to emit parser-independent, owned events:
expanded names, normalized attribute values, text, comments, processing
instructions, and byte spans. A separate XDM-owned builder consumes those events
into private nodes. The golden input was copied into a vector, parsed, and the
input vector was explicitly dropped before the tree was built and queried.

The experiment remains version-neutral where AR-0001 has not decided semantics.
It is not a public XDM API or a conformance result.

## Observations

Three focused XDM cases pass in addition to the ten resource/XML cases:

- the golden document retains expanded element names, parent/child relations,
  string value, logical resource identity, and non-empty byte spans after the
  input allocation is dropped;
- structurally equal sibling elements and attributes receive distinct opaque
  node identities;
- attributes belong to an element but are not children;
- semantic document order is assigned by a separate tree traversal rather than
  exposed as arena identity;
- adjacent text, predefined references, and CDATA become one text node with the
  value `one&twothree`; and
- comments and processing instructions have owned event and node paths rather
  than being discarded by the parser adapter.

No parser or `quick-xml` type is stored in the XDM tree. No filesystem path,
open handle, resolver, or ambient authority is stored either.

## Capability inventory

The first golden source currently pressures these semantic operations:

- document root and child traversal;
- element expanded name;
- parent relation;
- node identity distinct from value equality;
- semantic document order;
- descendant text aggregation for element string value; and
- source resource identity plus byte span.

The stylesheet slice will additionally pressure attributes, namespace bindings,
matching from the document node, literal result construction, and XPath child
steps. Ancestors, reverse axes, keys, repeated arbitrary navigation, typed
values, namespace nodes, base URI, and schema annotations have not been
justified by this experiment.

## Limitations and next pressure

The node kinds and arena are deliberately private. Namespace declaration nodes,
base URI, typed values, exact document-order rules for namespace nodes, line and
column indexing, whitespace stripping, and the selected XDM edition remain open.
The current builder assumes it receives structurally balanced events from the
XML adapter; production construction will need a typed invariant or explicit
failure path at that seam.

The next useful step is to parse the golden stylesheet into the same owned model
and recognize only its root stylesheet, output declaration, root template,
literal result element, text, and `xsl:value-of` syntax. That work must remain
private until AR-0001 resolves the initial standards profile.
