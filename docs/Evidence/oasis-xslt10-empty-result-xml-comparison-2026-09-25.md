# OASIS XSLT 1.0 empty-result XML comparison

Date: 2026-09-25

## Result

The local OASIS `compare="XML"` adapter now treats an XML declaration as
serialization metadata when comparing an empty result tree. A declaration-only
actual result and an empty expected result therefore compare equal, while a
declaration-only result still differs from any result containing a node.

This is a harness correction, not an engine serialization change. XML
declarations, their encoding spelling, and their presence are outside the
semantic node comparison already used for non-empty XML results.

## Unchanged case

`Microsoft/Text__78242#1` executes an empty `xsl:text` instruction. FastXSLT
serializes the empty XML result with an XML declaration; the archival expected
file is empty. The corrected XML comparator ignores the declaration and admits
the case without weakening comparisons against non-empty results.

The complete archival measurement moves from 2,111 to **2,112 exact
XML-semantic matches out of 3,173 catalog cases (66.56%)**. Initialized cases
remain 2,328, successful executions remain 2,281, visible XML mismatches fall
from 85 to 84, and comparator-unsupported outcomes remain 77.

## Validation

- `cargo test -p fastxslt oasis_xml_comparator_ignores_a_declaration_for_an_empty_result_tree_only --all-features`
- `scripts/measure-oasis-xslt10.ps1`

