# XSLT30 UTF-16 byte lane -- 2026-09-02

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0140` case through the
private byte-result lane. The result begins with the UTF-16 big-endian byte
order mark, carries the exact `encoding="UTF-16"` XML declaration, and decodes
to the expected XHTML result including `HelloÁ`.

The final encoded byte count is checked against the host ceiling before the
byte vector is returned. The ordinary string-result lane remains UTF-8-only and
continues to report non-UTF-8 requests explicitly.

## Decomposition

Before adding the encoder, `serialization.rs` had reached ADR-0004's 2,000-line
calibration trigger. Physical US-ASCII CDATA conversion moved to a private
`byte_encoding` module together with the new UTF-16BE conversion. The semantic
serializer retains validation, method selection, result traversal, and byte
budget coordination; the extracted module owns only physical code-unit
conversion. The parent source unit returns to 1,973 lines after this slice.

## Boundary

This selects deterministic UTF-16BE with a required BOM for the private
test/evidence byte lane. It does not widen the public string result, select a
public byte-result API, admit UTF-16LE as a requested label, or claim arbitrary
encoding-provider support.

## Denominator movement

The complete `decl/output` denominator moves from 197 to 198 passes and from 34
to 33 visible default not-run cases; its one profile exclusion is unchanged.
Across the eleven conserved XSLT30 denominators, the total moves from 395 to
396 passes and from 82 to 81 visible default not-run cases, with three
engine-unsupported cases and 51 profile exclusions unchanged.
