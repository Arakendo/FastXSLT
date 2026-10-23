# OASIS XSLT 1.0 Current Source-Node Argument

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an exact `current()` template argument retain the caller's source-node
identity and compose with a callee's `count($variable)`,
`normalize-space($variable)`, and source-attribute computed construction?

## Changes

- An exact `current()` selected template argument now retains the current
  source node in the existing invocation-local typed source-node frame.
- Under XSLT 1.0 compatibility, `normalize-space($variable)` applies first-node
  conversion to a source-node variable; an empty node-set contributes the empty
  string rather than a fabricated error.
- A computed `xsl:attribute` can read an unqualified current-source attribute
  and can count a typed source-node variable.
- Computed-attribute count materialization retains explicit result-node and
  XPath-operation charges. Missing or non-node variables fail rather than being
  coerced into a plausible count.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,473 | 1,475 | +2 |
| Executed successfully | 1,311 | 1,312 | +1 |
| Expected-result XML matches | 1,197 | 1,198 | +1 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 162 | 163 | +1 |

The unchanged `Lotus/select_select79#1` case now passes. It sends the caller's
`doc` node to two selected `foo` templates, proves `count($node)` remains one,
reads each `foo/@name`, and obtains the normalized string value of the original
`doc` node. The additional initialized execution failure belongs to another
case exposed by the widened initialization frontier; it remains visible rather
than being reclassified or excluded.

The strict standard-operation lower bound becomes
`1,198 / 2,742 = 43.69%`; the conservative all-catalog ratio becomes
`1,198 / 3,173 = 37.76%`. These remain compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche admits exact `current()` as a template argument and the named
XSLT 1.0 compositions above. It does not add general `current()` placement,
arbitrary node-sequence functions in computed attributes, temporary-tree or
cross-document node arguments, or a general expression evaluator.

## Verification

- A focused runtime test passes the current source node through
  `xsl:with-param`, constructs attributes from the callee source focus and node
  count, and normalizes the caller node's string value.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
