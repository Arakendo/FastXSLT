# XSLT30 Template Preview Denominator

| Field | Value |
| --- | --- |
| Date | 2026-08-26 |
| Suite revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Test set | `tests/decl/template/_template-test-set.xml` |
| Cases | `template-001` through `template-006` |
| Local overlay | `corpus/overlays/xslt30/private-slice-v0.toml` |
| Claim | First conserved preview denominator; no XSLT conformance claim |

## Question

Can FastXSLT define a first coherent upstream preview denominator that retains
unsupported cases rather than selecting only an already-green stylesheet?

## Selection

The preview now inventories the complete six-case XSLT30 `template` test set.
Every case has one referenced environment, one stylesheet, an explicit
standards dependency, and a top-level `assert-xml` result:

| Case | Dependency | Required behavior | Disposition |
| --- | --- | --- | --- |
| `template-001` | `XSLT10+` | comment node tests and modes | Engine unsupported (`FXST1009`) |
| `template-002` | `XSLT10+` | processing-instruction node tests and modes | Engine unsupported (`FXST1009`) |
| `template-003` | `XSLT10+` | `node()` tests and modes | Engine unsupported (`FXST1009`) |
| `template-004` | `XSLT10+` | attribute selection/patterns and modes | Engine unsupported (`FXST1009`) |
| `template-005` | `XSLT10+` | named templates, parameters, conditionals, calls, and recursion | Engine unsupported (`FXST1010`) |
| `template-006` | `XSLT20+` | root template and empty literal result element | Passed |

The five unsupported cases are recorded with selection disposition
`engine-unsupported` and execution disposition `not-run`. The executable case
is selected and passed through its upstream environment and XML assertion.

## Executable checks

The focused corpus test:

- proves the overlay contains exactly six entries for the upstream test set;
- resolves every case by its suite-native identity;
- retains each case's `spec`, environment, stylesheet, and `assert-xml` shape;
- imports each stylesheet into a bounded memory-resident snapshot;
- compiles `template-006` successfully;
- requires the other five stylesheets to fail specifically as unsupported; and
- keeps valid named-template syntax distinct from malformed stylesheet input.

The compiler now reports `FXST1010` for a named `xsl:template` outside the
private slice. It no longer reaches the misleading generic “missing match”
invalid-input outcome for that valid XSLT construct.

## Conservation

```text
discovered template cases (6)
    = selected and passed (1)
    + engine unsupported / not run (5)
```

No case is omitted because its stylesheet is difficult. This denominator is
small enough to review and complete enough to exercise ADR-0006's distinction
between discovery, selection, execution, and comparison.

## Limitations and next pressure

This test set is a first preview denominator, not the complete FastXSLT product
profile. It deliberately reveals large missing feature families. The next
standards-profile ADR must state that implementation proceeds through named,
conserved upstream selections and that passing this denominator does not imply
XSLT 1.0, 2.0, or 3.0 conformance.

The next executable widening should add a complete upstream family whose
dependencies pressure one intentionally selected semantic feature rather than
merely adding another isolated green case.
