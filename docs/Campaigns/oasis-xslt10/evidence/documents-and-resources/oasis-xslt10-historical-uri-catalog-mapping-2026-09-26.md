# OASIS XSLT 1.0 Historical URI Catalog Mapping

- Date: 2026-09-26
- Status: Verified corpus-adapter evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Several standard-operation fixtures reference a stylesheet module by an
archival `file:` or absolute HTTP URI even though the referenced bytes are
present beside the case in the local archive. Treating those URI strings as
ambient filesystem or network authority would violate FastXSLT's resource
model; leaving them unmapped hid valid include/import semantics behind a
workbench `FXRS0002` failure.

## Bounded adapter mapping

The OASIS adapter may now map those historical URI spellings to a same-case
physical fixture only for catalog standard-operation cases. The logical
identity remains the URI resolved by the stylesheet. The adapter reads the
candidate only while building the sealed snapshot, canonicalizes it beneath
the case directory, and admits the immutable bytes under that logical
identity. Engine compilation still performs no filesystem or network access.

Expected-error cases do not receive this compatibility mapping. The adapter
also carries one explicit correction for `Include__77745`, whose doubts entry
says the remote resource is unavailable and whose local same-named file is
demonstrably not the content used to create the reference result. This prevents
an archival external-reference error or a non-equivalent local fixture from
becoming a success merely because a similarly named file exists.

## Result

Seven former standard-operation missing-resource cases reach a more precise
disposition:

- six compare exactly: Lotus `impincl27`, Microsoft imports `91150`, `91156`,
  and `91165`, and Microsoft includes `77504` and `84464`;
- Microsoft import `91158` reaches its locally stored ASP module and reports a
  structured stylesheet XML failure.

Doubt-annotated Microsoft include `77745` remains a missing-resource case. Its
doubts entry says the remote resource is not supplied, and the archive's local
same-named file does not match the expected remote content.

The conserved sweep moves from 2,342 to **2,348 initialized** cases, from
2,291 to **2,297 successful executions**, and from 2,134 to **2,140 exact
expected-result matches (67.44%)**. Initialization failures fall from 828 to
822. The expected-error denominator remains 423 / 431, comparator gaps remain
67, and visible XML mismatches remain zero.

This is host/corpus mapping evidence, not general URI catalog selection and not
permission for ambient acquisition.

## Verification

```powershell
cargo test -p fastxslt --all-features maps_historical_oasis_dependency_uris_to_case_local_physical_files
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
