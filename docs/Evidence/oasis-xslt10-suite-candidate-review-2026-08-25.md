# OASIS XSLT/XPath 1.0 Suite Candidate Review

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Candidate | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Archive | `XSLT-testsuite-04.ZIP` |
| Archive SHA-256 | `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5` |
| Download size | 5,453,011 bytes |
| Disposition | Local-only legacy reference; do not admit to the public repository |
| Decision pressure | AR-0001 XSLT 1.0-first alternative |

## Authoritative sources

The [OASIS XSLT Conformance TC page](https://www.oasis-open.org/committees/xslt/)
states that the committee completed its work in April 2005 with Committee Draft
04, could not continue, and published the collection as-is with unresolved
items captured in a doubts file. The archive remains available from the
[OASIS download endpoint](https://www.oasis-open.org/committees/download.php/12171/XSLT-testsuite-04.ZIP).

The committee's [review policy](https://www.oasis-open.org/committees/xslt/reviews.htm)
describes specification-cited tests, multiple-processor review, infoset-oriented
comparison, discretionary behavior, gray areas, and escalation to the W3C when
the specification could not settle an expected result.

## Acquisition and inspection method

The archive was downloaded into ignored `.workbench` storage and hashed before
extraction. No included executable, script, stylesheet, or test was run.
Inspection used repository tools and PowerShell XML parsing over `catalog.xml`
and `doubts.xml`.

The extracted package contains 8,897 files under `DOCS`, `TESTS`, and `TOOLS`.
Its own README describes a prototype suite covering XSLT 1.0 and XPath 1.0 and
states that it does not include a complete test harness.

## Catalog observations

| Observation | Count |
| --- | ---: |
| Catalog `test-case` entries | 3,173 |
| Distinct catalog IDs | 3,166 |
| Duplicate catalog IDs | 7 |
| Standard-operation cases | 2,742 |
| Expected execution-error cases | 431 |
| XML comparisons | 2,736 |
| Manual comparisons | 5 |
| HTML comparisons | 1 |

The catalog provides contributor, path, purpose, XSLT/XPath specification
citation, scenario, input, output, and comparison metadata. This is useful for a
legacy compatibility harness, but duplicate IDs require an external stable case
identity rather than trusting `id` alone.

## Doubts observations

| Observation | Count |
| --- | ---: |
| Doubts-file case entries | 3,174 |
| Distinct doubts-file IDs | 3,161 |
| Explicit `doubt` annotations | 86 |
| Gray-area annotations | 34 |
| Serialization annotations | 18 |
| Extension annotations | 4 |
| Processor-specific annotations | 3 |

The doubts overlay is valuable evidence and must be applied before interpreting
a result. It also confirms that raw pass/fail counts would be misleading.

## Platform and harness constraints

The suite assumes working-directory-sensitive relative paths and includes old
network-oriented inputs. Android's historical adapter for this exact archive
also documents case-insensitive-filesystem assumptions and references to
unavailable hosts. A FastXSLT harness would need explicit offline resource
mapping, case-preserving path handling, scenario interpretation, infoset/result
comparison, doubts filtering, and deterministic classification of infrastructure
failures.

## Licensing and redistribution boundary

The package documentation carries an OASIS copyright notice allowing copies and
derivative explanatory works when notices are preserved, while restricting
modification of the documents themselves. Contributed test collections also
carry their own terms and notices.

In particular, the included Microsoft contribution notice describes a
non-transferable, non-sublicensable license granted to OASIS and retains
Microsoft copyright. Other files include IBM, W3C, publisher, and embedded-data
notices. Those facts do not establish that FastXSLT can redistribute the whole
archive under, or alongside, its MIT project without a focused legal review.

Therefore:

- do not add the archive, extracted files, or an unofficial mirror to FastXSLT;
- do not copy selected tests into the MIT corpus;
- retain only the public acquisition URL, checksum, measurements, and review
  conclusions;
- require an explicit license review before any future automated acquisition or
  redistribution; and
- use locally acquired cases only as non-published compatibility evidence until
  that review exists.

## Architectural conclusion

The archive prevents the XSLT 1.0 alternative from being dismissed for lack of
tests, but it is weaker than QT3/XSLT30 as a reproducible public denominator. Its
age, closed maintenance state, unresolved annotations, duplicate identities,
environment assumptions, and redistribution complexity increase harness cost.

Current evidence therefore favors a modern XDM-oriented internal model with a
named staged compatibility slice over a 1.0-only internal model. This does not
select an advertised standards profile: representative consumer transforms and
an accepted ADR are still required.
