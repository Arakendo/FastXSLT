# Stylesheet Output Compilation Decomposition Review

Date: 2026-08-29

## Trigger

ADR-0004 requires cohesion inspection during substantive modification of a
1,001–2,000-line source unit. The single-document stylesheet compiler reached
1,450 physical lines after independent output, template-priority, static
namespace, and mode-dispatch campaigns. Its prior review retained a named
1,500-line reopening trigger; the next compiler-bearing slice would approach or
cross that threshold.

This checkpoint performs a behavior-preserving extraction before further
semantic work.

## Responsibility inventory

Before extraction, `golden_stylesheet_experiment.rs` owned both stylesheet
composition and all `xsl:output` details. Output compilation is independently
coherent:

- validate the admitted output-declaration attributes and empty content;
- classify supported output methods and UTF-8 encoding;
- parse version-sensitive XSLT boolean lexicals;
- retain serialization settings in the compiled program; and
- provide the absence/default settings used by single-document and module
  assembly.

It does not compile templates, patterns, instructions, globals, modules, or
named references. It owns no serialization execution, result transfer,
resource authority, filesystem/network behavior, host policy, or public API.

## Extraction

| Owner | Physical lines after extraction | Responsibility |
| --- | ---: | --- |
| `compile/golden_stylesheet_experiment.rs` | 1,325 | single-document composition, top-level templates/globals, shared compiler structure and diagnostics, integration tests |
| `compile/output_compiler.rs` | 138 | `xsl:output` declaration and default-settings compilation |

The output owner consumes stylesheet XDM plus the parent's structural and
diagnostic helpers and returns the existing private `OutputSettings`. The
parent calls it while assembling one program. Module assembly receives the same
default settings through a visibility restricted to `crate::compile`.

Dependency direction is one way. The child does not import template,
instruction, module, runtime, snapshot, host, or serialization-execution
owners. No new semantic representation, callback, broad compiler context,
crate, or public boundary is introduced.

## Conservation

The extraction must preserve output defaults, XSLT 2.0 and 3.0 boolean lexical
distinctions, supported/unsupported method and encoding diagnostics, retained
media type and serialization options, module-assembly default comparison,
source locations, corpus dispositions, public/ABI behavior, and the workspace
unsafe surface.

Focused output tests and the pinned include path run through the same compiled
program. The closing gate is the complete `scripts/verify.ps1` suite.

## Disposition

Accept the private output-compilation owner. The extraction reduces production
responsibility coupling rather than moving arbitrary lines: future output
semantics no longer require editing template assembly, and template campaigns
no longer navigate output lexical policy.

Retain the 1,325-line parent at this checkpoint. Reopen it at 1,500 lines, when
globals or template assembly demonstrate another independent owner, when its
integration tests require a named invariant harness, or when ordinary changes
again span unrelated top-level responsibilities.
