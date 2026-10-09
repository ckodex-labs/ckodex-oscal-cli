#!/usr/bin/env node
// Build-time snapshot generator for the Mizan Workbench.
//
// Runs the real `mizan` CLI against vendored inputs and writes its raw
// stdout, plus two small projections, under apps/workbench/public/snapshot/.
// The GitHub Pages site is a static export and cannot call the engine at
// runtime; this directory is its only engine-derived data source.
//
// Contract: apps/workbench/src/lib/provenance.ts (SnapshotManifest).
//
// Rules this script keeps:
//   - Every command attempted is recorded, including failures. A failing
//     command never aborts the run and is never dropped.
//   - Stdout is written byte-for-byte; the recorded digest is of those bytes.
//   - No network access. Commands that need a server or network are listed
//     in `skipped` with the reason.
//   - Projections are derived only from files on disk; nothing is invented.
//
// Dependency-free: Node 22 standard library only.

import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { homedir } from "node:os";
import { dirname, isAbsolute, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const REPO = resolve(SCRIPT_DIR, "..", "..", "..");
const OUT_DIR = join(REPO, "apps", "workbench", "public", "snapshot");
// Scratch files for commands that write artifacts (-o). Repo-relative so that
// paths echoed on stdout are stable across machines and runs.
const WORK_REL = "apps/workbench/.snapshot-work";
const WORK_DIR = join(REPO, WORK_REL);

const NIST_COMMIT = "78650f02ad9321bb7b817846f8fbd4f2bcd620de";
const NIST_SOURCE = `usnistgov/oscal-content@${NIST_COMMIT}`;
const NIST_CATALOG = "examples/nist-800-53-r5/NIST_SP-800-53_rev5_catalog.json";
const NIST_MODERATE =
  "examples/nist-800-53-r5/NIST_SP-800-53_rev5_MODERATE-baseline_profile.json";
const SAMPLE_SSP = "examples/sample-ssp.json";
const INVENTORY_SBOM = "examples/inventory-sbom.json";

const STDERR_LIMIT = 2048;
const COMMAND_TIMEOUT_MS = 180_000;
const MAX_BUFFER = 256 * 1024 * 1024;

// ---------------------------------------------------------------------------
// helpers

function die(msg) {
  process.stderr.write(`gen-snapshot: ERROR: ${msg}\n`);
  process.exit(1);
}

function sha256(buf) {
  return createHash("sha256").update(buf).digest("hex");
}

function fileSha256(rel) {
  return sha256(readFileSync(join(REPO, rel)));
}

function readJson(rel) {
  return JSON.parse(readFileSync(join(REPO, rel), "utf8"));
}

function git(args) {
  const r = spawnSync("git", args, { cwd: REPO, encoding: "utf8" });
  if (r.status !== 0) {
    die(`git ${args.join(" ")} failed: ${(r.stderr || "").trim()}`);
  }
  return r.stdout;
}

function isExecutableFile(p) {
  try {
    return statSync(p).isFile();
  } catch {
    return false;
  }
}

function resolveBinary() {
  const fromEnv = process.env.MIZAN_BIN;
  if (fromEnv) {
    // Explicit request: never silently fall back to another binary.
    const candidates = isAbsolute(fromEnv)
      ? [fromEnv]
      : [resolve(process.cwd(), fromEnv), resolve(REPO, fromEnv)];
    const hit = candidates.find(isExecutableFile);
    if (!hit) {
      die(`MIZAN_BIN=${fromEnv} does not exist (tried: ${candidates.join(", ")})`);
    }
    return hit;
  }
  const targetRoot = process.env.CARGO_TARGET_DIR
    ? resolve(REPO, process.env.CARGO_TARGET_DIR)
    : join(homedir(), ".cache", "cargo-target");
  const candidates = [
    join(REPO, "target", "release", "mizan"),
    join(targetRoot, "release", "mizan"),
    join(targetRoot, "debug", "mizan"),
  ];
  const hit = candidates.find(isExecutableFile);
  if (!hit) {
    die(
      `no mizan binary found. Set MIZAN_BIN or build with 'cargo build --release --bin mizan'. Tried:\n  ${candidates.join("\n  ")}`,
    );
  }
  return hit;
}

// ---------------------------------------------------------------------------
// command execution

const BIN = resolveBinary();
const commands = [];
const skipped = [];

function run(id, title, argv) {
  const started = process.hrtime.bigint();
  const r = spawnSync(BIN, argv, {
    cwd: REPO,
    maxBuffer: MAX_BUFFER,
    timeout: COMMAND_TIMEOUT_MS,
    // Keep credentials and endpoints out of offline commands.
    env: { ...process.env, MIZAN_TOKEN: "", OSCALIFY_TOKEN: "", NO_COLOR: "1" },
  });
  const durationMs = Number((process.hrtime.bigint() - started) / 1_000_000n);
  const stdout = r.stdout ?? Buffer.alloc(0);
  const stderr = r.stderr ?? Buffer.alloc(0);

  let exitCode;
  let failureNote = "";
  if (r.error) {
    exitCode = -1;
    failureNote = `spawn error: ${r.error.message}\n`;
  } else if (r.status === null) {
    exitCode = -1;
    failureNote = `terminated by signal ${r.signal}\n`;
  } else {
    exitCode = r.status;
  }

  const rec = {
    id,
    title,
    argv,
    exitCode,
    durationMs,
    stdoutSha256: sha256(stdout),
  };

  if (stdout.length > 0) {
    let isJson = false;
    try {
      JSON.parse(stdout.toString("utf8"));
      isJson = true;
    } catch {
      isJson = false;
    }
    const name = `${id}.${isJson ? "json" : "txt"}`;
    writeFileSync(join(OUT_DIR, name), stdout);
    rec.output = name;
  }

  if (exitCode !== 0) {
    rec.stderr = (failureNote + stderr.toString("utf8")).slice(0, STDERR_LIMIT);
  }

  commands.push(rec);
  const status = exitCode === 0 ? "ok  " : "FAIL";
  process.stdout.write(
    `  ${status} exit=${String(exitCode).padStart(2)} ${String(stdout.length).padStart(8)}B ${String(durationMs).padStart(6)}ms  ${id}\n`,
  );
  return rec;
}

function skip(id, reason) {
  skipped.push({ id, reason });
  process.stdout.write(`  skip                                    ${id}: ${reason}\n`);
}

// ---------------------------------------------------------------------------
// projections

function propValue(obj, name) {
  const p = (obj.props ?? []).find((x) => x.name === name);
  return p ? p.value : undefined;
}

/** Walk every control (including enhancements) under catalog groups. */
function walkCatalog(catalog, visit) {
  const walk = (controls, family, parent) => {
    for (const c of controls ?? []) {
      visit(c, family, parent);
      walk(c.controls, family, c.id);
    }
  };
  for (const g of catalog.groups ?? []) walk(g.controls, g.id, null);
  // Ungrouped top-level controls, if any, have no family.
  walk(catalog.controls, null, null);
}

function paramLabel(p) {
  if (typeof p.label === "string") return p.label;
  if (p.select && Array.isArray(p.select.choice)) {
    const how = p.select["how-many"] === "one-or-more" ? "one or more" : "one";
    return `Selection (${how}): ${p.select.choice.join("; ")}`;
  }
  return null;
}

/**
 * Flatten the 'statement' part: labels and prose in document order.
 * Parameter insertions are rendered as [<param-id>] so the UI can map them to
 * `params` without the projection altering wording.
 */
function flattenStatement(control) {
  const statement = (control.parts ?? []).find((p) => p.name === "statement");
  if (!statement) return "";
  const pieces = [];
  const walk = (part) => {
    const label = propValue(part, "label");
    if (label) pieces.push(label);
    if (typeof part.prose === "string") pieces.push(part.prose);
    for (const sub of part.parts ?? []) walk(sub);
  };
  walk(statement);
  let text = pieces
    .join(" ")
    .replace(/\{\{\s*insert:\s*param,\s*([^}\s]+)\s*\}\}/g, "[$1]")
    .replace(/\s+/g, " ")
    .trim();
  if (text.length > 600) text = `${text.slice(0, 597)}...`;
  return text;
}

function buildAtlas(resolvedRel, resolveCmdId) {
  const full = readJson(NIST_CATALOG).catalog;
  const resolved = readJson(resolvedRel).catalog;

  const baseline = new Map();
  walkCatalog(resolved, (c) => baseline.set(c.id, c));

  const families = (full.groups ?? []).map((g) => ({
    id: g.id,
    title: g.title,
    total_in_catalog: 0,
    in_baseline: 0,
  }));
  const famIndex = new Map(families.map((f) => [f.id, f]));

  const controls = [];
  walkCatalog(full, (c, family, parent) => {
    const fam = famIndex.get(family);
    const inBaseline = baseline.has(c.id);
    if (fam) {
      fam.total_in_catalog += 1;
      if (inBaseline) fam.in_baseline += 1;
    }
    const withdrawn = propValue(c, "status") === "withdrawn";
    if (inBaseline) {
      // Content comes from the resolved catalog: that is what the engine
      // produced for this baseline.
      const rc = baseline.get(c.id);
      controls.push({
        id: c.id,
        family,
        title: rc.title ?? c.title,
        parent,
        in_baseline: true,
        withdrawn: propValue(rc, "status") === "withdrawn",
        params: (rc.params ?? []).map((p) => ({ id: p.id, label: paramLabel(p) })),
        statement: flattenStatement(rc),
      });
    } else {
      controls.push({ id: c.id, family, title: c.title, parent, in_baseline: false, withdrawn });
    }
  });

  // Controls the engine placed in the baseline that do not exist in the full
  // catalog would be a decoherence; surface them instead of dropping them.
  const known = new Set(controls.map((c) => c.id));
  const unmatched = [...baseline.keys()].filter((id) => !known.has(id));

  return {
    schema: "mizan.workbench.atlas/v1",
    derivedFrom: {
      catalog: { path: NIST_CATALOG, sha256: fileSha256(NIST_CATALOG), source: NIST_SOURCE },
      profile: { path: NIST_MODERATE, sha256: fileSha256(NIST_MODERATE), source: NIST_SOURCE },
      resolvedCatalog: {
        command: resolveCmdId,
        sha256: fileSha256(resolvedRel),
        note: "Engine output; embeds a generation timestamp and uuid, so this digest changes per run.",
      },
    },
    statementFormat:
      "Labels and prose of the 'statement' part in document order, whitespace-collapsed, parameter insertions rendered as [param-id], truncated to 600 characters with a trailing '...'.",
    counts: {
      catalogControls: controls.length,
      baselineControls: baseline.size,
      baselineFamilies: families.filter((f) => f.in_baseline > 0).length,
      unmatchedBaselineControls: unmatched.length,
    },
    unmatchedBaselineControls: unmatched,
    families,
    controls,
  };
}

function buildSspStatus() {
  const doc = readJson(SAMPLE_SSP);
  const ssp = doc["system-security-plan"] ?? {};
  const reqs = ssp["control-implementation"]?.["implemented-requirements"] ?? [];

  const statusOf = (obj) => {
    // OSCAL 1.x by-component: { "implementation-status": { state } }.
    const s = obj["implementation-status"];
    if (s && typeof s.state === "string") return s.state;
    // FedRAMP-style extension prop on implemented-requirement.
    const prop = propValue(obj, "implementation-status");
    return typeof prop === "string" ? prop : null;
  };

  return {
    schema: "mizan.workbench.ssp-status/v1",
    derivedFrom: { path: SAMPLE_SSP, sha256: fileSha256(SAMPLE_SSP) },
    systemTitle: ssp.metadata?.title ?? null,
    semantics:
      "Lists only controls this SSP declares. A control absent from this list is undeclared, not failed. implementation_status is null (presence EMPTY) when the SSP states no status.",
    implemented_requirements: reqs.map((r) => {
      const status = statusOf(r);
      return {
        control_id: r["control-id"],
        uuid: r.uuid ?? null,
        implementation_status: status,
        status_presence: status === null ? "EMPTY" : "PRESENT",
        by_components: (r["by-components"] ?? []).map((bc) => {
          const bs = statusOf(bc);
          return {
            component_uuid: bc["component-uuid"] ?? null,
            implementation_status: bs,
            status_presence: bs === null ? "EMPTY" : "PRESENT",
          };
        }),
      };
    }),
  };
}

// ---------------------------------------------------------------------------
// main

process.stdout.write(`gen-snapshot: repo=${REPO}\ngen-snapshot: mizan=${BIN}\n`);

for (const rel of [NIST_CATALOG, NIST_MODERATE, SAMPLE_SSP, INVENTORY_SBOM]) {
  if (!existsSync(join(REPO, rel))) die(`missing vendored input ${rel}`);
}

rmSync(OUT_DIR, { recursive: true, force: true });
mkdirSync(OUT_DIR, { recursive: true });
rmSync(WORK_DIR, { recursive: true, force: true });
mkdirSync(WORK_DIR, { recursive: true });

const J = ["--format", "json"];
const w = (name) => `${WORK_REL}/${name}`;

const version = run("version", "Engine version", ["--version"]);
run("catalog-list", "Built-in jurisdiction catalogs", [...J, "catalog", "list"]);

run("nist-catalog-inspect", "Inspect NIST SP 800-53 Rev 5 catalog", [...J, "inspect", NIST_CATALOG]);
run("nist-catalog-validate", "Validate NIST SP 800-53 Rev 5 catalog", [...J, "validate", NIST_CATALOG]);

const resolvedRel = w("nist-moderate-resolved.json");
const resolveRec = run("nist-moderate-resolve", "Resolve NIST Moderate baseline profile to a catalog", [
  ...J,
  "resolve",
  NIST_MODERATE,
  "-o",
  resolvedRel,
]);
run("nist-moderate-inspect", "Inspect the resolved Moderate catalog", [...J, "inspect", resolvedRel]);
run("nist-moderate-validate", "Validate the resolved Moderate catalog", [...J, "validate", resolvedRel]);

run("ssp-inspect", "Inspect sample SSP", [...J, "inspect", SAMPLE_SSP]);
run("ssp-validate", "Validate sample SSP", [...J, "validate", SAMPLE_SSP]);
run("ssp-fedramp-validate", "FedRAMP Moderate rules against sample SSP", [
  ...J,
  "fedramp",
  "validate",
  SAMPLE_SSP,
  "--baseline",
  "moderate",
]);

const firstControl =
  readJson(SAMPLE_SSP)["system-security-plan"]?.["control-implementation"]?.[
    "implemented-requirements"
  ]?.[0]?.["control-id"];
if (firstControl) {
  run("ssp-blast-radius", `Blast radius of ${firstControl} in sample SSP`, [
    ...J,
    "blast-radius",
    "--target",
    firstControl,
    SAMPLE_SSP,
  ]);
} else {
  skip("ssp-blast-radius", "sample SSP declares no implemented-requirements to target");
}

run("policy-rulepack-list", "Built-in policy rulepacks", [...J, "policy", "rulepack", "list"]);

const sbom = readJson(INVENTORY_SBOM);
if (sbom.bomFormat === "CycloneDX") {
  run("sbom-import", "Import CycloneDX inventory SBOM as an OSCAL component-definition", [
    ...J,
    "sbom",
    "import",
    "-i",
    INVENTORY_SBOM,
    "-o",
    w("sbom-component-definition.json"),
  ]);
} else {
  skip("sbom-import", `${INVENTORY_SBOM} is not CycloneDX (bomFormat=${sbom.bomFormat})`);
}

run("fsm-status", "Compliance FSM status at its initial state", [...J, "fsm", "status"]);

const pipeDir = w("pipeline");
run("pipeline-run", "End-to-end pipeline (US jurisdiction, inventory SBOM, all built-in rules)", [
  ...J,
  "pipeline",
  "run",
  "-j",
  "us",
  "--sbom",
  INVENTORY_SBOM,
  "-o",
  pipeDir,
]);
run("pipeline-export-sarif", "Export the pipeline assessment results to SARIF", [
  ...J,
  "export",
  "sarif",
  "-i",
  `${pipeDir}/oscal-assessment-results.json`,
  "-o",
  w("pipeline-results.sarif"),
]);

const NEEDS_SERVER = "requires a running OSCALify gRPC server; the Pages build has none and the script makes no network calls";
skip("search/model/entity/framework/snapshot/release/claim/evidence/graph/import", NEEDS_SERVER);
skip("health", NEEDS_SERVER);
skip("audit", "audits a live Kubernetes cluster; no cluster in the build environment");
skip("mcp", "long-running stdio server, not a one-shot command");
skip("daemon/watch", "long-running background processes, not one-shot commands");
skip("tui/gui/workbench", "interactive front-ends, no machine output");

// Projections.
let atlasBytes = 0;
if (resolveRec.exitCode === 0 && existsSync(join(REPO, resolvedRel))) {
  const atlas = JSON.stringify(buildAtlas(resolvedRel, resolveRec.id));
  writeFileSync(join(OUT_DIR, "atlas.json"), atlas);
  atlasBytes = Buffer.byteLength(atlas);
  process.stdout.write(`  proj atlas.json ${atlasBytes}B\n`);
} else {
  skip("atlas.json", "nist-moderate-resolve failed; no resolved catalog to project");
}

const sspStatus = JSON.stringify(buildSspStatus(), null, 2);
writeFileSync(join(OUT_DIR, "ssp-status.json"), sspStatus);
process.stdout.write(`  proj ssp-status.json ${Buffer.byteLength(sspStatus)}B\n`);

// Manifest.
const manifest = {
  schema: "mizan.workbench.snapshot/v1",
  generatedAt: new Date().toISOString(),
  mizanVersion:
    version.exitCode === 0
      ? readFileSync(join(OUT_DIR, version.output), "utf8").trim()
      : "UNKNOWN",
  gitCommit: git(["rev-parse", "HEAD"]).trim(),
  gitDirty: git(["status", "--porcelain"]).trim().length > 0,
  inputs: [
    { path: NIST_CATALOG, sha256: fileSha256(NIST_CATALOG), source: NIST_SOURCE },
    { path: NIST_MODERATE, sha256: fileSha256(NIST_MODERATE), source: NIST_SOURCE },
    { path: SAMPLE_SSP, sha256: fileSha256(SAMPLE_SSP), source: "repository example" },
    { path: INVENTORY_SBOM, sha256: fileSha256(INVENTORY_SBOM), source: "repository example" },
  ],
  commands,
  skipped,
};
writeFileSync(join(OUT_DIR, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);

rmSync(WORK_DIR, { recursive: true, force: true });

const failed = commands.filter((c) => c.exitCode !== 0);
process.stdout.write(
  `gen-snapshot: ${commands.length} commands, ${failed.length} non-zero exit, ${skipped.length} skipped -> ${relative(REPO, OUT_DIR)}\n`,
);
for (const f of failed) {
  process.stdout.write(`  non-zero: ${f.id} exit=${f.exitCode}: ${(f.stderr ?? "").split("\n")[0]}\n`);
}
// A failing engine command is recorded data, not a script failure.
