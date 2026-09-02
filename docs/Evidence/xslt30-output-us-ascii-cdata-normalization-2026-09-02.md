# XSLT30 US-ASCII CDATA normalization -- 2026-09-02

| Field | Value |
| --- | --- |
| Suite | W3C XSLT 3.0 test suite |
| Revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Test set | `tests/decl/output/_output-test-set.xml` |
| Cases | `output-0115b`, `output-0115c`, `output-0115d`, `output-0115e` |
| Result | Four unchanged cases selected and passed |

## Result

The private byte-result lane now admits `encoding="US-ASCII"` for one exact
XHTML CDATA shape. It first executes ordinary character expansion and requested
Unicode normalization, then converts each non-ASCII character inside selected
CDATA text into an uppercase hexadecimal character reference by closing the
CDATA section, emitting the reference as markup, and reopening the section.
The final byte stream is ASCII and retains the requested encoding name in the
XML declaration.

The four native cases establish three distinct rules:

- `output-0115b` emits U+00AA outside the surrounding CDATA runs as `&#xAA;`;
- `output-0115c` produces the same result despite a character map for U+00AA,
  proving CDATA-selected text bypasses character mapping;
- `output-0115d` applies NFD first, retaining ASCII `c` inside CDATA and
  emitting decomposed U+0327 as `&#x327;`; and
- `output-0115e` applies NFC first and emits composed U+00E7 as `&#xE7;`.

The existing serialized-byte path charges the intermediate character stream;
the US-ASCII expansion then charges the exact additional bytes introduced by
CDATA boundaries and character references. The final expanded length is
checked against the host byte ceiling before publication.

## Boundary

This is not general US-ASCII serialization. A non-ASCII character outside
selected CDATA text remains explicitly unsupported (`FXSR1009`), as do
arbitrary legacy encodings, byte-order marks for this lane, and replacement or
lossy encoding. The existing ISO-8859-1 experiment remains restricted to ASCII
result characters. NFD joins NFC as an admitted requested normalization form;
NFKC, NFKD, and fully-normalized output remain unsupported.

With these four promotions, `decl/output` records 191 passes, one profile
exclusion, and 40 visible default not-run cases. The 531-case conserved XSLT30
subtotal records 389 passes, three engine-unsupported cases, 51 profile
exclusions, and 88 visible default not-run cases.

## Validation

The focused corpus test executes the unchanged stylesheets from the pinned
submodule, verifies the exact requested encoding, requires an all-ASCII byte
result and XML declaration, and checks the native permitted hexadecimal
serialization alternatives. Workspace-wide validation is recorded in the
implementing commit.

