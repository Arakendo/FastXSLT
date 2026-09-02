# XSLT30 comment carriage return -- 2026-09-02

## Result

FastXSLT executes the unchanged source-free W3C XSLT30 `output-0723` case. Its
static `xsl:comment` select expression constructs `[`, XML codepoint 13, and
`]`; XML serialization preserves the carriage return directly and produces the
exact native assertion result `<a><!--[\r]--></a>`.

## Boundary

The admitted expression is bounded compile-time string concatenation composed
of single-quoted literals and one-integer `codepoints-to-string()` calls.
Generated codepoints must be XML 1.0 characters, and the existing comment
lexical guard still rejects `--` or a trailing hyphen. Dynamic operands,
general sequence arguments, and runtime string-function dispatch remain
unsupported.

The result remains subject to the ordinary result-node and serialized-byte
limits; this evidence does not select comment recovery outside the XML lexical
constraints.

## Denominator movement

The complete `decl/output` denominator moves from 196 to 197 passes and from 35
to 34 visible default not-run cases; its one profile exclusion is unchanged.
Across the eleven conserved XSLT30 denominators, the total moves from 394 to
395 passes and from 83 to 82 visible default not-run cases, with three
engine-unsupported cases and 51 profile exclusions unchanged.
