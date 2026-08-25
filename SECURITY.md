# Security Policy

## Pre-stability status

FastXSLT is an early-stage XSLT engine. The repository currently contains a
buildable scaffold and design records, not a usable transformation engine. It
has not been independently reviewed or audited.

Do not treat FastXSLT as a production security boundary. No release currently
claims safe processing of hostile XML or XSLT.

## Intended threat scope

FastXSLT is designed to process inputs that may be untrusted. The intended
engine boundary therefore includes:

- explicit authority for stylesheet imports, document lookup, collections,
  unparsed text, external entities, extension functions, and host callbacks;
- no ambient filesystem or network access when the host supplies no resolver;
- bounded parsing, recursion, node and sequence growth, retained diagnostics,
  output, and other resource use where the selected profile permits bounds;
- distinct diagnostics for invalid input, unsupported behavior, denied
  authority, exhausted budgets, cancellation, host failure, and internal
  failure; and
- memory-resident compilation and execution after host-controlled resource
  admission, without hidden temporary files, spill files, or disk caches.

These are design requirements, not current implementation guarantees. The
accepted contracts are maintained in the software design document and ADRs.

## Explicit limits

FastXSLT does not aim to protect against:

- an attacker that can read or modify the embedding process's memory;
- malicious code already running inside the host process;
- operating-system, hypervisor, or hardware compromise;
- side-channel resistance;
- denial of service beyond limits the host explicitly configures and the
  engine documents as enforceable; or
- disclosure caused by a host-provided resolver, extension, diagnostic sink,
  logger, serializer, or output sink.

Keeping admitted inputs in memory avoids engine-owned file handles and repeated
path access. It does not hide bytes from the host process, prevent swap or crash
dump exposure, or bypass security scanning of the host's initial file read or
explicit output publication.

## Reporting a vulnerability

Prefer GitHub's [private vulnerability reporting](https://github.com/Arakendo/FastXSLT/security/advisories/new)
when it is enabled. If that route is unavailable, open a public issue requesting
a private contact channel, but do not include exploit details or sensitive
input in the issue.

This pre-stability project is maintained on a best-effort basis:

- there is no response or remediation SLA;
- there is no promised CVE or coordinated-disclosure process;
- fixes normally land on `main`; and
- no supported backport branches currently exist.

## Supported versions

There is no supported production release yet. This section will name supported
release lines when FastXSLT begins making stability or security-support claims.
