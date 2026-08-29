# XSLT30 `conflict-resolution-1001` Variable-Filtered Paths

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-1001`
- Stylesheet: `conflict-resolution-1001.xsl`
- Source: `conflict-resolution-10.xml`

## Representation and execution

Compilation lowers each exact `planche/{section}/*[@type=$type]` expression to
a typed representation containing its unnamespaced parent steps, wildcard
element step, attribute expanded name, and global variable identity. The same
representation shape is owned independently by apply selection and template
matching; runtime dispatch does not reparse the lexical XPath.

The upstream invocation uses the global parameter's empty-string default. No
source element has an empty `type` attribute, so neither selection admits a
candidate and the root rule produces the asserted
`<planche><images/><dialogues/></planche>` result.

A supplemental test invokes the unchanged upstream stylesheet with
`type="enfant"`. This forces both filtered match rules and `xsl:copy-of
select="."` to execute. The result retains the selected `bart` and `lisa`
elements, their `type` attributes, and text while excluding the `parent`
entries.

## Results

| Invocation | Observation | Disposition |
| --- | --- | --- |
| Upstream default | Semantically equal empty `images` and `dialogues` wrappers | passed |
| Supplemental `type="enfant"` | Selected elements and attributes copied; parent entries absent | passed |

## Claim boundary

This evidence admits relative unnamespaced named-parent paths ending in
`*[@name=$variable]`, string-valued global predicates, and current element
copying with source attributes, element descendants, and text.

It does not admit arbitrary predicates, namespaced steps in this specialized
form, general comparisons or atomization, local-variable path predicates,
arbitrary `xsl:copy-of` selection, document/attribute/comment/processing-
instruction copying, or a general XPath optimizer.
