# AR-0016 Source Access Inventory and Safe Reference

Date: 2026-08-30

## Scope

This record inventories the current production source-semantic access paths
and records the first safe reference implementation for exact
`xsl:strip-space elements="*"`. It does not select the eventual optimized
visibility representation or general whitespace-declaration semantics.

## Current source access inventory

All current source-tree consumers ultimately receive one private
`xdm::owned_tree_experiment::Document`. Its owned semantic accessors are:

- identity and provenance: `document_node`, `kind`, `name`, `prefix`, and
  `location`;
- relationships: `children`, `attributes`, `parent`, and
  `namespace_declarations`;
- direct values: `value`;
- containing values: `string_value`, `string_value_controlled`, and
  `visit_string_value_controlled`.

The semantic consumers of those accessors are:

| Owner | Entry points and observations |
| --- | --- |
| XPath navigation | `xpath::path_experiment::evaluate_location_path_controlled` owns axes, descendant traversal, predicates, document ordering, focus position, and focus size. The focused numeric, castable, count, distinct-values, and decimal evaluators call this same path owner rather than traversing a second tree. |
| XPath/string values | Castable, decimal, and distinct-values evaluators use the controlled document string-value methods. They therefore follow effective child relationships recursively. |
| Invocation globals | `runtime_context::materialize_global_defaults` evaluates source-dependent global paths against the document supplied to the invocation. |
| Template selection | `runtime::template_selector` uses `kind`, `name`, `parent`, `children`, `attributes`, string values, and the shared XPath evaluator for pattern tests, sibling positions, and path patterns. |
| Built-in rules and apply selection | `golden_runtime_experiment::apply_builtin_template`, `select_apply_nodes`, and named-child helpers traverse `children` and `attributes` from the invocation document and calculate focus size from those effective sequences. |
| Value construction | `runtime::value_evaluator` uses controlled string-value visitation; context-name evaluation uses the same document. |
| Source copying | `copy_source_node`, `execute_source_element_copy`, and `xsl:sequence` source copying traverse the invocation document's effective children while retaining source names, namespaces, attributes, and values. |
| Prepared/direct execution | `execute_program_with_parameters` and the initial-mode entry compose the stylesheet policy before globals, XPath, template selection, built-ins, values, or copying receive the source. The workbench/direct lifecycle reaches the same `execute_program` seam. |
| Transform-set entry | Source parsing and XDM construction remain source-only. Initial element lookup observes an element sequence whose membership is unaffected by this exact text-only policy; the stable `NodeId` is then interpreted through the derived reference. |
| Diagnostics and result construction | Source locations remain stored on the unchanged node slots. Result construction consumes semantic values from the effective invocation document and does not reopen or mutate source bytes. |

Compiler uses of `Document` inspect stylesheet XDM, not an invocation source,
and are outside the effective-source composition seam. Corpus catalog readers
and verification-ledger readers are likewise harness metadata consumers rather
than transformation semantics.

## Compiled policy

`StylesheetProgram` now retains a private `SourceWhitespacePolicy`. Absence of a
declaration is `Preserve`. The only admitted non-default form is the exact
empty declaration `xsl:strip-space elements="*"`. Other name tests remain
structured unsupported behavior; `xsl:preserve-space`, declaration precedence,
`xml:space`, and schema-aware whitespace are not inferred.

## Safe complete-derived-document reference

For a stripping invocation, FastXSLT clones the complete prepared `Document`
and removes strip-eligible whitespace-only text nodes from cloned element-child
relationships. It deliberately retains every physical node slot in the same
order. Consequently every visible node keeps the original `NodeId`, parent,
name, value, namespace data, source location, and document-order identity.
Hidden text nodes cannot be discovered through effective child/descendant
navigation or containing string values.

The reference uses XML whitespace characters only: space, tab, carriage return,
and line feed. Construction charges one `XdmNode` unit per inspected source
node and observes the invocation's cancellation and budget control. The clone
is invocation-owned and dropped after execution; it is not written into the
prepared input, stylesheet, generation, snapshot, or a global cache.

This representation is intentionally expensive. Cloning all payload and node
storage makes it a clear safe semantic oracle for a later immutable visibility
view; it is not evidence that cloning should become the optimized path. The
subsequent visibility prototype and its first differential measurement are
recorded in
[AR-0016 Visibility-View Prototype](ar-0016-visibility-view-prototype-2026-08-30.md).

## Executable controls

- A focused XDM control proves filtered relationships and enclosing string
  values while preserving visible `NodeId` and `SourceLocation` equality.
- Budget and deterministic cancellation controls stop derivation at real
  `XdmNode` charge points.
- One prepared source executes under preserving and stripping stylesheets in
  succession. The preserving result retains indentation in the document string
  value, the stripping result produces `AB`, and the original prepared string
  value remains unchanged afterward.
- The pinned, unmodified XSLT30 `mode-1301` source, stylesheet, initial entry,
  and expected XML now pass through the same runtime path.

The complete mode denominator therefore records 41 passes, 44 profile
exclusions, and 84 visible default not-run cases out of 169. Across the 11
conserved XSLT30 denominators, the total is 235 passes, 3 engine-unsupported
cases, 49 profile exclusions, and 244 visible default not-run cases out of 531.

## Remaining AR-0016 work

- add broader differential controls for explicit XPath positions, copying, and
  every currently supported node kind;
- execute concurrent strip/preserve invocations and overlapping generation
  replacement against one prepared source;
- prototype an invocation-owned visibility representation only against this
  safe reference oracle;
- measure clone and view preparation latency, peak/retained memory, warm
  throughput, tail latency, and reuse break-even;
- retain general declaration matching, import precedence, `xml:space`, and
  typed whitespace as separate future widening work.
