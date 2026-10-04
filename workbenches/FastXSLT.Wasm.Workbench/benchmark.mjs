import { createRequire } from "node:module";
import { readFileSync, statSync } from "node:fs";
import { performance } from "node:perf_hooks";
import { fileURLToPath } from "node:url";
import { dirname, join, resolve } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const repositoryRoot = resolve(here, "..", "..");
const packageDirectory = join(repositoryRoot, ".workbench", "wasm-bindgen");
const require = createRequire(import.meta.url);
const moduleStart = performance.now();
const wasm = require(join(packageDirectory, "fastxslt_wasm_workbench.js"));
const moduleLoadMs = performance.now() - moduleStart;
const stylesheet = readFileSync(
  join(repositoryRoot, "vendor", "xslt30-test", "tests", "expr", "for", "for-004.xsl"),
);
const requestedIterations = Number.parseInt(process.argv[2] ?? "2000", 10);
const tiers = [
  { name: "items-5", items: 5, multiplier: 4 },
  { name: "items-50", items: 50, multiplier: 2 },
  { name: "items-500", items: 500, multiplier: 1 },
];
const decoder = new TextDecoder();
const encoder = new TextEncoder();
const measurements = [];
const smoke = runSmokeAndControlParity();

for (const tier of tiers) {
  const source = encoder.encode(buildSource(tier.items));
  const expected = `<?xml version="1.0" encoding="UTF-8"?><out>${tier.items}.00</out>`;
  const copyIterations = 1000;
  for (let index = 0; index < 100; index += 1) {
    wasm.copyProbe(source);
  }
  const copyStart = performance.now();
  for (let index = 0; index < copyIterations; index += 1) {
    if (wasm.copyProbe(source) !== source.byteLength) {
      throw new Error("copy probe length mismatch");
    }
  }
  const copyProbeNs = ((performance.now() - copyStart) * 1_000_000) / copyIterations;

  const setupStart = performance.now();
  const creation = wasm.WasmEngine.create(
    `urn:fastxslt:wasm:${tier.name}:source`,
    source,
    `urn:fastxslt:wasm:${tier.name}:stylesheet`,
    stylesheet,
  );
  const setupMs = performance.now() - setupStart;
  if (!creation.succeeded) {
    throw new Error(`${creation.code}/${creation.category}: ${creation.detail}`);
  }
  const engine = creation.takeEngine();
  if (engine === undefined) {
    throw new Error("successful creation did not return an engine");
  }

  for (let index = 0; index < 100; index += 1) {
    const warm = engine.transformBytes(`warm-${tier.name}-${index}`);
    assertOutcome(warm, expected);
  }

  const iterations = Math.max(1, requestedIterations * tier.multiplier);
  let executionMs = 0;
  let transferMs = 0;
  let resultBytes = 0;
  const overallStart = performance.now();
  for (let index = 0; index < iterations; index += 1) {
    const executionStart = performance.now();
    const outcome = engine.transformBytes(`measure-${tier.name}-${index}`);
    executionMs += performance.now() - executionStart;
    if (!outcome.succeeded) {
      throw new Error(`${outcome.code}/${outcome.category}: ${outcome.detail}`);
    }
    const transferStart = performance.now();
    const bytes = outcome.takeResultBytes();
    transferMs += performance.now() - transferStart;
    resultBytes = bytes.byteLength;
    if (index === 0 && decoder.decode(bytes) !== expected) {
      throw new Error(`semantic mismatch for ${tier.name}`);
    }
  }
  const overallMs = performance.now() - overallStart;
  measurements.push({
    tier: tier.name,
    items_per_transform: tier.items,
    source_bytes: source.byteLength,
    result_bytes: resultBytes,
    iterations,
    copy_probe_ns_per_call: round(copyProbeNs),
    compile_prepare_and_input_copy_ms: round(setupMs),
    transforms_per_second: round((iterations * 1000) / overallMs),
    mean_execution_and_serialization_us: round((executionMs * 1000) / iterations),
    mean_result_transfer_us: round((transferMs * 1000) / iterations),
    known_retained_capacity_bytes: engine.knownRetainedCapacityBytes,
    prepared_xdm_capacity_bytes: engine.preparedXdmCapacityBytes,
    prepared_xdm_node_count: engine.preparedXdmNodeCount,
    linear_memory_pages_after_tier: wasm.linearMemoryPages(),
  });
}

console.log(JSON.stringify({
  schema: "fastxslt-wasm-workbench-v1",
  node: process.version,
  module_load_ms: round(moduleLoadMs),
  wasm_binary_bytes: statSync(join(packageDirectory, "fastxslt_wasm_workbench_bg.wasm")).size,
  requested_base_iterations: requestedIterations,
  smoke,
  measurements,
}, null, 2));

function runSmokeAndControlParity() {
  const source = encoder.encode("<root/>");
  const principal = encoder.encode(
    '<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">' +
    '<xsl:include href="included.xsl"/><xsl:output omit-xml-declaration="yes"/>' +
    '</xsl:stylesheet>',
  );
  const included = encoder.encode(
    '<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">' +
    '<xsl:template match="/"><out>sealed</out></xsl:template></xsl:stylesheet>',
  );
  const coldStart = performance.now();
  const builder = new wasm.WasmEngineBuilder(
    "https://example.invalid/wasm/source.xml",
    source,
    "https://example.invalid/wasm/main.xsl",
    principal,
  );
  builder.addStylesheetResource("https://example.invalid/wasm/included.xsl", included);
  const creation = builder.build();
  const firstEngineMs = performance.now() - coldStart;
  if (!creation.succeeded) {
    throw new Error(`${creation.code}/${creation.category}: ${creation.detail}`);
  }
  const engine = creation.takeEngine();
  const exact = engine.transformBytes("sealed-include");
  assertOutcome(exact, "<out>sealed</out>");

  const limited = engine.transformWithXsltInstructionLimit("budget-zero", 0);
  if (limited.succeeded || limited.code !== "FXCT0002" || limited.category !== "limit") {
    throw new Error(`unexpected budget outcome: ${limited.code}/${limited.category}`);
  }
  assertOutcome(engine.transformBytes("after-budget"), "<out>sealed</out>");

  const malformed = wasm.WasmEngine.create(
    "https://example.invalid/wasm/malformed.xml",
    encoder.encode("<root>"),
    "https://example.invalid/wasm/malformed-main.xsl",
    encoder.encode(
      '<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">' +
      '<xsl:template match="/"><out/></xsl:template></xsl:stylesheet>',
    ),
  );
  if (malformed.succeeded || malformed.code !== "FXXD0002" || malformed.category !== "invalid") {
    throw new Error(`unexpected malformed XML outcome: ${malformed.code}/${malformed.category}`);
  }

  const depthIdentity = "https://example.invalid/wasm/depth.xml";
  const depth = wasm.WasmEngine.create(
    depthIdentity,
    encoder.encode("<n>".repeat(256) + "leaf" + "</n>".repeat(256)),
    "https://example.invalid/wasm/depth-main.xsl",
    included,
  );
  if (depth.succeeded || depth.code !== "FXRS0006" || depth.category !== "limit" ||
      depth.resourceIdentity !== depthIdentity || depth.locationStart !== 192 || depth.locationEnd !== 195) {
    throw new Error(`unexpected depth-limit outcome: ${depth.code}/${depth.category}`);
  }
  const depthDiagnostic = {
    code: depth.code, category: depth.category, resource: depth.resourceIdentity,
    span: [depth.locationStart, depth.locationEnd],
  };
  depth.free();
  assertOutcome(engine.transformBytes("after-failed-depth-creation"), "<out>sealed</out>");

  return {
    first_engine_include_compile_prepare_ms: round(firstEngineMs),
    sealed_include_exact: true,
    budget_failure_code: limited.code,
    budget_failure_category: limited.category,
    same_engine_reused_after_budget_failure: true,
    malformed_xml_code: malformed.code,
    malformed_xml_category: malformed.category,
    malformed_xml_resource: malformed.resourceIdentity,
    malformed_xml_span: [malformed.locationStart, malformed.locationEnd],
    depth_limit: depthDiagnostic,
    retained_engine_reused_after_failed_depth_creation: true,
  };
}

function buildSource(items) {
  let xml = '<?xml version="1.0"?><order>';
  for (let index = 0; index < items; index += 1) {
    xml += '<order-item price="1.00" qty="1"/>';
  }
  return `${xml}</order>`;
}

function assertOutcome(outcome, expected) {
  if (!outcome.succeeded) {
    throw new Error(`${outcome.code}/${outcome.category}: ${outcome.detail}`);
  }
  const actual = decoder.decode(outcome.takeResultBytes());
  if (actual !== expected) {
    throw new Error(`warm semantic mismatch: ${actual}`);
  }
}

function round(value) {
  return Math.round(value * 1000) / 1000;
}
