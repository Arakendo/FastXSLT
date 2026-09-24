# OASIS XSLT 1.0 Bounded Exact Template Priorities

Date: 2026-09-23  
Status: Local compatibility and shared-core evidence

## Question

Can the shared template-selection model compare much larger and finer explicit
priorities without heap allocation or binary floating-point ordering on the
runtime dispatch path?

## Implemented boundary

`TemplatePriority` now stores a sign, a bounded `u128` whole part, and an
18-place fractional part. The value remains immutable and `Copy`; comparison
uses the two numeric components directly and does not allocate during template
selection. This replaces the previous signed-millionths representation while
preserving the standard default priorities exactly.

The parser admits decimal whole parts representable by `u128` and at most 18
fractional digits. Values outside that private exact domain remain explicit
unsupported work. Invalid decimal lexicals remain static errors. This is a
shared representation improvement, not an XSLT 1.0-only runtime path.

## Corpus effect

Against the unchanged 3,173-case OASIS archive:

- initialized cases rise from 2,248 to 2,254;
- successful executions rise from 2,184 to 2,190;
- exact XML comparisons rise from 2,035 to 2,040 / 3,173 (64.29%);
- five unchanged Microsoft priority/conflict cases become exact;
- `Microsoft/ConflictResolution__77622#1` becomes a visible comparison
  mismatch because its expected result relies on XSLT 1.0 binary-number
  rounding making two distinct decimal priorities tie, whereas the shared
  modern exact-decimal model orders them distinctly;
- initialization failures fall from 887 to 881, execution failures remain 64,
  and comparison mismatches rise from 66 to 67; and
- expected-error unexpected successes remain five and no execution panic
  occurs.

## Verification

Focused compiler tests compare 36-digit whole priorities and adjacent
18-place fractional priorities, retain the ordinary integer/default ordering,
and keep a 19-place value outside the bounded domain. The full corpus then
exercises those priorities through actual template dispatch.

The archive remains local and unmodified. The visible legacy rounding mismatch
is not counted as a pass and is not used to alter modern semantics.
