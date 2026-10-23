# OASIS XSLT 1.0 Name Path Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 653 definite unchanged XML passes; 920 initialized cases |
| Result | 657 definite unchanged XML passes; 931 initialized cases |
| Disposition | Shared typed node-name composition; not a conformance claim |

## Change

The value compiler now composes `name()` with an admitted typed location path.
The runtime evaluates the path through the existing charged navigation model,
returns the empty string for an empty sequence, and requires zero or one node.
It returns the retained lexical name for unnamespaced elements and attributes.

The implementation does not fabricate a prefix for a namespaced expanded name.
The prepared XDM does not yet preserve the source lexical prefix needed by
`fn:name`, so such a selected node remains an explicit `FXRT1008` unsupported
boundary. Multi-node arguments remain `XPTY0004`; this shared modern primitive
does not silently adopt XSLT 1.0 first-node conversion while AR-0019 is open.

## Unchanged cases

Four cases move to definite XML comparison passes:

- both `Lotus/axes_axes116` identities exercise
  `name(/descendant-or-self::north)`;
- `Lotus/namespace_namespace08#1` exercises an unnamespaced attribute path;
- `Lotus/position_position108#1` exercises a following-axis path with a
  positional predicate.

Seven more cases move beyond initialization without being credited as passes:

- three Microsoft output cases execute successfully but reach existing XML
  comparator limitations around their document declarations;
- `Lotus/string_string32#1` reports `XPTY0004` for a multi-node argument;
- `Lotus/string_string33#1` and `string34#1` report `FXRT1008` because lexical
  namespace prefixes are not retained;
- `Microsoft/Output__84012#1` reaches the existing `SEPM0004` serialization
  boundary.

Qualified attribute paths requiring more general QName path parsing remain at
initialization. No failure above is counted as a pass.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 920 | 931 | +11 |
| Execution succeeded | 731 | 738 | +7 |
| XML comparison passes | 653 | 657 | +4 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 16 | 19 | +3 |
| Execution failures | 189 | 193 | +4 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **657 / 2,742 = 23.96%** of standard-operation
cases and **657 / 3,173 = 20.71%** of the complete catalog.

## Verification

A first-party production-path test covers element, attribute, and empty paths.
A separate test proves multi-node `XPTY0004`. The complete local OASIS sweep
confirms the four intended passes, leaves all later boundaries visible, and
introduces no mismatch, expected-error leak, or panic.
