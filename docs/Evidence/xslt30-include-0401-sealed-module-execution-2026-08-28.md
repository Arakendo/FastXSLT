# XSLT30 include-0401 Sealed Module Execution

| Field | Value |
| --- | --- |
| Date | 2026-08-28 |
| Suite revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Test set | `tests/decl/include/_include-test-set.xml` |
| Case | `include-0401` |
| Disposition | Selected / passed |
| Denominator after experiment | 1 passed; 15 harness-unsupported / not-run |

## Executed resource graph

The harness reads the immutable upstream catalog, its inline source, and the
two upstream stylesheet files. It then closes those file handles and admits
three owned byte resources to one bounded `ResourceSnapshot`:

```text
include-0401.xsl
  -- xsl:include href="include-0401a.xsl" --> include-0401a.xsl

principal source (inline catalog content)
  -- transform with compiled program --> serialized result
```

The principal logical identity is an absolute HTTPS-shaped IRI used only as
identity and base. The snapshot resolver applies RFC 3986/3987 mechanics to the
relative `href`, performs the second of two permitted resolution attempts, and
finds the secondary bytes in the sealed snapshot. No URL dereference,
filesystem lookup, callback, retained file handle, or live snapshot mutation is
available during compilation or execution.

## Semantic result

The compiler treats the secondary literal result root as a simplified
stylesheet and constructs its implicit `/` template without concatenating XML
text. That template retains the secondary resource identity in its source
location. Module assembly also retains the principal global `$greeting` value,
`Hi there!`, for use by the included template.

The ordinary in-memory transform-set runtime produces:

```xml
<out><in>Hi there!</in></out>
```

The harness compares this exactly with the upstream `assert-xml` string after
removing an optional XML declaration from each side.

## Negative authority evidence

A companion test supplies a principal stylesheet whose relative include
resolves to an unadmitted sibling. Compilation returns structured missing-
resource code `FXRS0002` containing the resolved logical identity. It neither
opens a local file nor attempts the network.

## Claim boundary

This admits one executable case, not general `xsl:include` conformance. The
private slice accepts exactly one include and a simplified secondary
stylesheet. Fragment selection, recursive includes, import precedence,
duplicate-match priority, included output declarations, catalogs, and live
resolution remain unsupported or unselected. The later
[sealed dependency accounting experiment](sealed-stylesheet-dependency-accounting-2026-08-28.md)
puts this case through atomic graph preparation without widening its module
semantics.
