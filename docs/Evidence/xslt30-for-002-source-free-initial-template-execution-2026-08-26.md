# XSLT30 `for-002` Source-Free Initial-Template Execution

Date: 2026-08-26

## Native case

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/expr/for/_for-test-set.xml`
- Case: `for-002`
- Dependency: `XSLT20+`
- Principal source/environment: none
- Invocation entry: initial template `main`
- Stylesheet: `for-002.xsl`
- Assertion: inline `<out>11, 12, 21, 22</out>`

## Invocation boundary

The private transform request now owns an explicit invocation-entry variant:
either a qualified principal-source resource or a named initial template. A
principal-source entry retains the existing authority, snapshot-admission, XML
parse, and XDM construction path. An initial-template entry is validated
against the compiled stylesheet and executes without inventing a source
resource, document, or context item.

The entry choice is invocation state. The compiled stylesheet owns the static
named-template definition but does not acquire a mutable or default invocation
entry. Unknown initial-template names fail request admission with structured
identity `FXRT0004`; initial-template parameters remain explicitly unsupported
in this private seam.

## Expression and result behavior

The compiler recognizes the native two-binding integer expression:

```xpath
for $i in (10, 20), $j in (1, 2)
return ($i + $j)
```

The evaluator preserves clause order and produces `11`, `12`, `21`, `22`.
`xsl:value-of` retains its native `separator=", "` and joins the atomic result
items into one result text node. The complete serialized output matches the
inline XML assertion after the serializer's XML declaration is accounted for.

Atomic expression work has a separate `xpath-operation` charge domain rather
than being mislabeled as node navigation. The focused test charges four
operations for the four additions and proves that a three-operation limit stops
before the fourth result. Result text and serialized bytes retain their
existing independent limits.

## Conservation and claim boundary

At this checkpoint, the complete `expr/for` denominator was two passed and two
engine-unsupported, with no remaining harness-unsupported case. Native
`for-003` subsequently advanced in
[XSLT30 `for-003` Focus-Preserving Empty-Sum Execution](xslt30-for-003-focus-preserving-empty-sum-execution-2026-08-26.md).
This evidence does not establish general initial-template parameters, dynamic
context, arbitrary `for` clauses, integer types, overflow behavior, operators,
atomic sequences, or general `xsl:value-of` sequence conversion. The admitted
parser accepts only two literal integer bindings and their ordered addition
return.
