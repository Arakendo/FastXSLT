# XSLT30 `castable-001` Built-In Atomic Execution

Date: 2026-08-26

## Native case

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/expr/castable/_castable-test-set.xml`
- Case: `castable-001`
- Dependency: `XSLT20+`
- Environment/source: `castbl01` / `castbl01.xml`
- Stylesheet: `castable-001.xsl`
- Assertion: file-backed `castable-001.out`

The unmodified stylesheet performs 24 `castable as` evaluations: 12 expected
true results over their corresponding source values and 12 expected false
results across incompatible type families. The serialized result matches the
upstream file exactly.

## Owned semantic seams

The XPath layer now parses a path operand and one admitted `xs` built-in target,
atomizes exactly one selected node through its controlled XDM string value, and
evaluates lexical castability. The admitted targets are:

```text
string, boolean, integer, decimal, float, double,
duration, dayTimeDuration, yearMonthDuration,
dateTime, date, time
```

Lexical recognition is implemented in the XPath owner rather than delegated to
Rust floating-point or date/time parsing. Integer recognition is not limited to
a host fixed-width value. Castability charges one XPath operation; navigation
and atomization retain their XPath-node-visit and XDM-string-value work domains.

The case also exposed a separate result-construction requirement. XML parsing
now retains local namespace declarations separately from attributes, the owned
tree preserves those declarations, compilation derives the prefixed in-scope
bindings needed by literal result elements, and XML serialization emits a
binding only when it differs from the parent scope. The XSLT namespace remains
excluded from literal results. This produces the native `xmlns:xs` declaration
on `out` without redundantly repeating it on every child.

## Conservation

The complete `expr/castable` denominator remains nine discovered: seven
selected and two schema-aware profile exclusions. Selected execution advances
to one pass, three engine-unsupported cases, and three harness-unsupported
cases, with no failures or hidden cases.

## Claim boundary

This evidence does not establish general XDM atomic values, casting that
produces typed values, typed variables, type promotion, QName/static-prefix
resolution, occurrence indicators, schema-defined types, or every lexical edge
case of the named XML Schema datatypes. Empty or multi-item operands return
false only within the admitted castability shape.

Namespace evidence is likewise narrow: it retains local declarations and
serializes inherited prefixed bindings needed by an unnamespaced literal result
element. It does not establish namespace nodes as a navigable XPath axis,
namespace fixup for arbitrary constructed names, default-namespace handling,
or general namespaced result serialization.
