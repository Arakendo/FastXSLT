# Temporary path for each golden fixture

This first-party, MIT-licensed fixture was authored for FastXSLT on 2026-10-03.
It is not copied from an upstream suite and does not establish conformance.

The stylesheet exercises global and local temporary paths, nested position/size
focus and empty selections. The runtime's private for-each tests compare the
serialized result with `expected.xml`. The principal input has no DTD or external
dependency. Sorting and general temporary-path scalar evaluation are outside
this fixture's scope.
