# OASIS XSLT 1.0 Negative and NaN `format-number()` Semantics

- Date: 2026-09-24
- Status: Verified semantic and compatibility evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

The decimal formatter selected the complete negative subpicture whenever the
input had a negative IEEE-754 sign. That produced two related defects in the
XSLT 1.0 compatibility path:

- negative zero retained a minus sign instead of using the positive
  subpicture; and
- the negative subpicture incorrectly replaced the positive subpicture's
  digit, grouping, decimal, and multiplier shape rather than contributing only
  its prefix and suffix; and
- NaN incorrectly retained picture affixes, producing values such as `NaNc`
  instead of the decimal format's NaN token alone.

The unchanged Microsoft `testOn-0.00` case therefore serialized `-0.00` where
the expected XSLT 1.0 result is `0.00`. The broader pattern-separator fixture
also exposed `#.00;(#)` as `(3)` instead of `(3.12)`.

## Implemented slice

XSLT 1.0 compatibility now treats both IEEE-754 zero signs as positive for
subpicture selection. Modern XPath formatting deliberately retains its
negative-zero behavior. For a nonzero negative value, the selected negative
subpicture supplies only prefix and suffix; finite digit formatting continues
to use the positive subpicture's numeric shape.

NaN now returns the selected decimal format's NaN token after the picture has
been parsed and validated, without applying picture affixes, scaling, or an
IEEE sign bit. This rule is shared by the compatibility and modern paths.

This is confined to the existing pure decimal formatter. It adds no runtime
state, resource authority, locale selection, host callbacks, or alternate
numeric representation.

## Verification

Focused unit oracles prove version-sensitive zero handling and the shared
negative-affix and NaN rules:

- XSLT 1.0 `format-number(-0.00, '0.00;(0.00)')` produces `0.00`;
- the modern path for the same expression produces `(0.00)`; and
- XSLT 1.0 `format-number(-3.12, '#.00;(#)')` produces `(3.12)`; and
- both profiles format `format-number('bad', '#%')` as `NaN`, not `NaN%`.

The unchanged `Microsoft/XSLTFunctions__testOn-0.00#1` case becomes exact.
The full sweep retains **2,311 initialized** and **2,259 successfully
executed** cases, reduces visible XML mismatches from 78 to **77**, and raises
the exact lower bound from 2,096 to **2,097 / 3,173 (66.09%)**. The larger
Microsoft pattern fixtures remain visibly uncredited because their archival
expected-byte decoding and other independent formatting differences remain;
the corrected subpicture and NaN results are not mislabeled as complete
passes.

## Reproduction

```powershell
cargo test -p fastxslt --all-features format_number_experiment::tests
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/XSLTFunctions__testOn-0.00#1'
./scripts/measure-oasis-xslt10.ps1
```
