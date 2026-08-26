# Private Built-In Rules and XSLT30 Fit Review

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| XSLT30 revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Golden | `corpus/golden/built-in-template-rules` |
| Claim | Private semantic and suite-fit evidence; no conformance claim |

## Suite-fit screen

The review searched upstream stylesheets containing `xsl:apply-templates` and
screened for the private compiler's admitted XSLT elements, one root template,
exact unprefixed element-name patterns, and absent or relative child-name
selection. It then inspected the closest syntax-light candidates and their
test-set metadata.

No complete case fit the admitted semantics. The closest default-selection and
simple-pattern candidate, `id-036`, also requires the XPath `id()` function, an
attribute path, DTD-backed ID behavior, and the suite's `feature=dtd`
dependency. Other superficially small candidates require modes, predicates,
unions, axes, literal-result attributes, complex match patterns, or output
parameters outside the slice.

This was a targeted mechanical screen followed by manual review, not a complete
dependency classification of all 14,600 cases. No overlay case was added and no
case was reclassified merely to produce a pass.

## Implemented evidence

The existing private reference path now supports the common built-in behavior
needed by the peer-derived invoice family:

- absent `select` applies templates to the context node's children;
- an exact compiled element rule is selected by expanded name;
- unmatched document and element nodes recursively apply templates to their
  children;
- unmatched text nodes contribute their string content;
- unmatched attribute, comment, and processing-instruction nodes contribute
  no result; and
- the private XPath path `.` selects the current dynamic context item.

Each selected-node dispatch charges local XSLT work. Context-item selection
charges one XPath node visit. Compiled template rules remain stylesheet state;
the context node, result construction, and controls remain invocation state.

## Golden result

The source contains an `invoice` with two `item` children. The root template
uses default apply-template selection, the built-in element rule traverses the
unmatched `invoice`, and the exact `item` rule emits `xsl:value-of select="."`:

```xml
<items><entry>apple</entry><entry>pear</entry></items>
```

## Limitations

- The implementation does not yet cover built-in rules for modes or typed
  modes, namespace-sensitive patterns, priority, or conflict resolution.
- Attribute selection is outside the private XPath slice even though the
  built-in attribute rule's no-output behavior is represented internally.
- Recursion is bounded indirectly by admitted XML depth and invocation work;
  no public recursion-limit contract is selected.
- The workspace now has 50 tests: 49 passing and one ignored manual probe.
