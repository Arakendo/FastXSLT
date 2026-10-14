# Boolean variable relations

First-party MIT-licensed regression fixture authored on 2026-10-13. No upstream
fixture content is copied and this fixture earns no conformance credit.

The invocation supplies the top-level `flag` parameter as the typed boolean
`true`, replacing the numeric default. The ordinary production execution path
must compare source node-sets to that boolean by whole-set truth, including an
empty selection, and convert the scalar boolean to one for a numeric relation.
Negative and positive node text distinguish whole-set truth from existential
numeric comparison. The test uses no demand-resolution hooks.
