# XSLT30 `conflict-resolution-0801` Current Mode

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-0801`
- Stylesheet: `conflict-resolution-0801.xsl`
- Environment: shared `conflict-resolution-08` principal source

## Method

The metadata-driven helper executes the pinned source and stylesheet through a
bounded sealed snapshot and identified batch of one. Mode-qualified `/`
patterns compile as document-node match rules in the ordinary typed template
selector, allowing distinct bodies for modes `a` and `b`.

The initial unmoded rule dispatches `foo` in mode `a` and `bar` in mode `b`.
Each matched template calls the same named template. The runtime preserves the
invoking template's current mode across that call, and the named template's
`xsl:apply-templates select="/" mode="#current"` redispatches the document node
in the preserved mode. The root path remains namespace-insensitive despite the
stylesheet-wide `xpath-default-namespace`.

## Result

| Case | Expected result string | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-0801` | `[a][b]` | equal | passed |

The two calls to the shared named template retain distinct current modes and
select the corresponding document-node templates.

## Conservation

The existing `initial-mode-005` case still passes after mode-qualified document
rules move through normal matched-template dispatch. Unmoded `/` remains the
direct-entry root-template path, and built-in rules continue propagating their
requested mode.

## Claim boundary

This evidence admits `#current` only on `xsl:apply-templates` and preserves it
through nested instructions and named-template calls. It does not admit
`#current` as a template-declaration mode, mode QNames, mode declarations, or
mode properties. Multi-mode `#default` behavior is evidenced separately by
`conflict-resolution-0802`.
