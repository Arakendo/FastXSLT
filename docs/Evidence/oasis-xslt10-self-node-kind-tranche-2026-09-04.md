# OASIS XSLT 1.0 Self Node-Kind Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 718 definite unchanged XML passes; 996 initialized cases |
| Result | 721 definite unchanged XML passes; 999 initialized cases |
| Disposition | Shared typed self-axis node-kind mechanics; not a conformance claim |

## Change

The typed `self` axis now admits explicit `text()`, `comment()`, and
`processing-instruction()` tests. Each test retains the supplied node only when
its XDM kind matches, uses the same typed path execution as existing `self::*`
and `self::node()`, and charges the candidate visit through the invocation-owned
XPath work budget.

This does not add named processing-instruction tests on `self`, generalize these
kind tests to other axes, or widen predicate composition.

## Unchanged cases

Three Lotus cases move directly from initialization boundaries to definite XML
comparison passes:

- `axes65`, covering an empty `self::text()` selection from an element;
- `axes66`, covering an empty `self::comment()` selection from an element;
- `axes67`, covering an empty `self::processing-instruction()` selection from
  an element.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 996 | 999 | +3 |
| Execution succeeded | 799 | 802 | +3 |
| XML comparison passes | 718 | 721 | +3 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 19 | 19 | 0 |
| Execution failures | 197 | 197 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **721 / 2,742 = 26.29%** of standard-operation
cases and **721 / 3,173 = 22.72%** of the complete catalog.

## Verification

A first-party typed-path test covers successful self selection for text,
comment, and processing-instruction nodes as well as a kind mismatch. The
complete local OASIS sweep confirms all three intended cases pass without a new
mismatch, comparator gap, runtime failure, unexpected success, or panic.
