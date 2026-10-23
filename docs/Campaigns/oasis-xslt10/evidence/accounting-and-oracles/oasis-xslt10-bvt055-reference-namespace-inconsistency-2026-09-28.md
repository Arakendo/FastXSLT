# OASIS XSLT 1.0 `BVTs_bvt055` Reference Namespace Inconsistency

Date: 2026-09-28  
Status: Exact one-case archival disposition

## Question

Is `Microsoft/BVTs_bvt055#1` still a comparator gap, or can the supplied
stylesheet and expected result establish a usable XML-semantic oracle?

## Finding

The supplied stylesheet constructs an attribute named `xml:foo`. Its expanded
name therefore uses the reserved XML namespace. FastXSLT serializes that
attribute with the required `xml` prefix.

The archival expected result instead declares:

```xml
xmlns:xp_1="http://www.w3.org/XML/1998/namespace"
```

and serializes the attribute as `xp_1:foo`. Namespaces in XML reserves that
namespace name for the `xml` prefix; binding it to `xp_1` makes the expected
document unusable as an XML-semantic reference. The same reference also
contains namespace names with spaces from this historical boundary test, but
the reserved-prefix violation alone is sufficient for the disposition.

## Result

The exact identity `BVTs_bvt055` is classified as an unusable archival
reference result. It receives no pass credit and causes no engine change.

After the preceding residual HTML comparator tranche, comparator gaps fall
from 8 to 7 and unusable-reference dispositions rise from 61 to 62. The strict
lower bound remains **2,256 / 3,173 (71.10%)**; lifecycle, mismatch,
expected-error, and policy counters are unchanged.

## Boundaries

- This does not treat prefix spelling as semantic where both documents are
  namespace-well-formed; the ordinary comparator still compares expanded
  names.
- This does not admit arbitrary namespace aliases for the XML namespace.
- The archive remains locally acquired and is not redistributed.

## Reproduction

```powershell
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/BVTs_bvt055'
./scripts/measure-oasis-xslt10.ps1
```
