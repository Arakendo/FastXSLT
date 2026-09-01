# XSLT30 output invalid-boolean tranche -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 cases `output-0280` through
`output-0283a`. Each case matches its native error alternative through static
error `XTSE0020` with an invalid category and the stylesheet resource location.

The eight cases pair XSLT 2.0 and XSLT 3.0 invalid lexicals for:

- `indent`;
- `omit-xml-declaration`;
- `standalone`;
- `undeclare-prefixes`.

They extend the existing six-case `output-0197` through `output-0199a` tranche
for byte-order marks, URI-attribute escaping, and content-type insertion. The
same compiler-owned boolean lexical policy now covers all seven admitted
properties without deferring malformed stylesheet values to serialization.

The adjacent unchanged `output-0284` separately validates `doctype-public`
against the XML public-identifier character set before the unsupported XML 1.1
serialization request can obscure the native `XTSE0020` alternative. This is a
lexical validation companion, not a ninth boolean case.

## Boundary conservation

This admission does not select `SEPM0016` as FastXSLT's behavior; the upstream
`any-of` explicitly permits the native static error. XSLT 2.0 remains bounded
to `yes` and `no`. XSLT 3.0 additionally accepts whitespace-normalized
`true`/`false` and `1`/`0`, while case variants such as `TRUE`, `YES`, and
`True` remain invalid.

## Denominator movement

The complete output denominator moves from 84 to 93 passes and from 148 to 139
visible default not-run cases. Across the eleven conserved XSLT30 denominators,
the total moves from 282 to 291 passes, with 3 engine unsupported cases, 50
profile exclusions, and 187 visible default not-run cases.
