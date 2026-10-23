# OASIS XSLT 1.0 Result Default-Namespace Fixup

Date: 2026-09-28  
Status: Verified implementation and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

Three successfully executed cases remained outside the exact denominator
because FastXSLT serialized two declarations for the default namespace on one
element. A representative result was:

```xml
<yyy xmlns="http://testguys.com" xmlns=""></yyy>
```

The semantic element itself was unnamespaced. Its retained namespace slice
also contained a conflicting non-empty default binding. The serializer emitted
that binding and then emitted an empty undeclaration to preserve the element's
expanded name, producing invalid XML.

## Change

Both the scoped namespace stack and its complete-clone reference now apply the
same per-element namespace fixup:

- only the final binding for one prefix can be effective on an element;
- an unnamespaced element cannot activate a non-empty default binding;
- an empty default binding is omitted when no non-empty inherited default must
  be undeclared; and
- a required undeclaration is still emitted when an unnamespaced child appears
  beneath an active non-empty default namespace.

The immutable compiled namespace slices remain unchanged and shared under
ADR-0018. Fixup remains private to serialization, and the complete-clone path
continues to be the differential oracle for the scoped implementation.

## Conserved result

The unchanged cases now compare exactly:

- `Lotus/namespace_namespace102#1`;
- `Lotus/namespace_namespace104#1`; and
- `Microsoft/Namespace-alias__91782#1`.

The strict exact lower bound rises from 2,248 to **2,251 / 3,173 (70.94%)**.
Comparator gaps fall from 20 to 17. Initialization remains 2,440, successful
execution remains 2,392, visible mismatches remain 15, and expected-error
credit remains 423 / 431.

## Boundaries

- This does not alter namespace ownership, compiled-slice sharing, or source
  namespace storage.
- It does not admit namespace-axis nodes or a public namespace API.
- It does not treat duplicate namespace attributes as valid output; it prevents
  their creation.
- It does not infer broader namespace-alias coverage from the one unchanged
  case that now serializes correctly.

## Verification

```powershell
cargo test -p fastxslt --all-features namespace_fixup_
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
