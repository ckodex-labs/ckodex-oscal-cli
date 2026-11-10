/**
 * Local Development HTTP-to-CLI Bridge (/api/cli)
 *
 * Exposes a READ-ONLY subset of the local `mizan` CLI to the Workbench when it
 * runs under `next dev` / `next start`. It does not exist in the static export
 * (GitHub Pages), where the UI uses build-time SNAPSHOT data instead.
 *
 * Security contract (this endpoint executes a local binary):
 * - Only allowlisted, non-mutating subcommands are accepted. No output-path
 *   flags, no write/fix/daemon/tx/cas-put style verbs.
 * - Requests must be `application/json` (forces a CORS preflight for
 *   cross-origin callers, which this route never approves) and must come from
 *   the same origin as the Host header. This blocks drive-by CSRF from other
 *   sites while the dev server is running.
 * - File arguments must resolve inside the repository root.
 */

import { NextRequest, NextResponse } from "next/server";
import fs from "fs";
import path from "path";

import { callOscalCli } from "@/lib/mcp-client";
import {
  MAX_BODY_BYTES,
  bridgeError as fail,
  guardBridgeRequest,
  readBoundedJsonBody,
} from "@/lib/bridge-guard";

const PROJECT_ROOT = path.resolve(process.cwd(), "../..");

/** Allowed command paths. Each entry is the leading argv tokens. */
const ALLOWED: string[][] = [
  ["--version"],
  ["inspect"],
  ["validate"],
  ["catalog", "list"],
  ["blast-radius"],
  ["fedramp", "validate"],
  ["policy", "rulepack", "list"],
  ["fsm", "status"],
  ["waive", "list"],
  ["utcp", "manual"],
  ["utcp", "config"],
  ["utcp", "search"],
  ["doctor"],
];

const FORBIDDEN_FLAGS = new Set([
  "-o",
  "--output",
  "--write",
  "--fix",
  "--apply",
  "--capture",
  "--capture-dir",
]);

function isAllowed(argv: string[]): boolean {
  return ALLOWED.some((prefix) => prefix.every((tok, i) => argv[i] === tok));
}

function looksLikeFilePath(val: string): boolean {
  const lower = val.toLowerCase();
  if (
    val.includes("/") ||
    val.includes("\\") ||
    val.includes("..") ||
    lower.endsWith(".json") ||
    lower.endsWith(".yaml") ||
    lower.endsWith(".yml") ||
    lower.endsWith(".xml") ||
    lower.endsWith(".rego") ||
    lower.endsWith(".md")
  ) {
    return true;
  }
  try {
    const candidate = path.resolve(PROJECT_ROOT, val);
    return fs.lstatSync(candidate).isSymbolicLink();
  } catch {
    return false;
  }
}

function resolveSafePath(rawPath: string): string | null {
  if (!rawPath || rawPath.includes("\0")) {
    return null;
  }
  const resolved = path.resolve(PROJECT_ROOT, rawPath);
  if (
    resolved !== PROJECT_ROOT &&
    !resolved.startsWith(PROJECT_ROOT + path.sep)
  ) {
    return null;
  }
  const rel = path.relative(PROJECT_ROOT, resolved);
  if (rel.startsWith("..") || path.isAbsolute(rel)) {
    return null;
  }
  let repoReal = PROJECT_ROOT;
  try {
    if (fs.existsSync(PROJECT_ROOT)) {
      repoReal = fs.realpathSync(PROJECT_ROOT);
    }
  } catch {
    return null;
  }
  let current = PROJECT_ROOT;
  const parts = rel.split(path.sep).filter(Boolean);
  for (const part of parts) {
    current = path.join(/*turbopackIgnore: true*/ current, part);
    let stat: fs.Stats | undefined;
    try {
      stat = fs.lstatSync(/*turbopackIgnore: true*/ current);
    } catch {
      stat = undefined;
    }
    if (stat) {
      if (stat.isSymbolicLink()) {
        return null;
      }
      try {
        const realCurrent = fs.realpathSync(/*turbopackIgnore: true*/ current);
        if (
          realCurrent !== repoReal &&
          !realCurrent.startsWith(repoReal + path.sep)
        ) {
          return null;
        }
      } catch {
        return null;
      }
    }
  }
  return resolved;
}

/** Health probe used by the UI to decide whether LIVE mode is available. */
export async function GET() {
  try {
    const out = await callOscalCli(["--version"]);
    return NextResponse.json({
      success: true,
      transport: "http-cli-bridge",
      version: out.trim(),
    });
  } catch (err: unknown) {
    return fail(503, err instanceof Error ? err.message : String(err));
  }
}

export async function POST(req: NextRequest) {
  const rejected = guardBridgeRequest(req);
  if (rejected) return rejected;

  const parsedBody = await readBoundedJsonBody(req, MAX_BODY_BYTES);
  if (!parsedBody.ok) return parsedBody.response;
  const body = parsedBody.value;
  const { argv } = (body ?? {}) as { argv?: unknown };
  if (
    !Array.isArray(argv) ||
    argv.length === 0 ||
    !argv.every((a) => typeof a === "string")
  ) {
    return fail(400, "Expected { argv: string[] }");
  }
  const args = argv as string[];

  if (!isAllowed(args)) {
    return fail(403, `Command '${args.slice(0, 3).join(" ")}' is not on the read-only allowlist`);
  }
  if (
    args.some((a) => {
      const flagName = a.includes("=") ? a.slice(0, a.indexOf("=")) : a;
      if (FORBIDDEN_FLAGS.has(flagName) || a.startsWith("-o")) {
        return true;
      }
      if (a.startsWith("-") && !a.startsWith("--")) {
        let rest = flagName.slice(1);
        while (rest.length > 0 && "qvh".includes(rest[0])) {
          rest = rest.slice(1);
        }
        if (rest.startsWith("o")) {
          return true;
        }
      }
      return false;
    })
  ) {
    return fail(403, "Output/write flags are not permitted through the bridge");
  }
  for (const a of args) {
    if (a.startsWith("-")) {
      if (a.includes("..")) {
        return fail(403, `Path '${a}' is outside the repository`);
      }
      const eqIdx = a.indexOf("=");
      if (eqIdx !== -1) {
        const val = a.slice(eqIdx + 1);
        if (looksLikeFilePath(val) && !resolveSafePath(val)) {
          return fail(403, `Path '${val}' is outside the repository`);
        }
      } else if (!a.startsWith("--") && a.length > 2) {
        let rest = a.slice(1);
        while (rest.length > 1 && "qvh".includes(rest[0])) {
          rest = rest.slice(1);
        }
        const candidates = new Set([a.slice(2), rest.slice(1)]);
        for (const val of candidates) {
          if (val && looksLikeFilePath(val) && !resolveSafePath(val)) {
            return fail(403, `Path '${val}' is outside the repository`);
          }
        }
      }
      continue;
    }
    if (looksLikeFilePath(a) && !resolveSafePath(a)) {
      return fail(403, `Path '${a}' is outside the repository`);
    }
  }

  const started = Date.now();
  try {
    const output = await callOscalCli([...args, "--format", "json"]);
    let parsed: unknown;
    try {
      parsed = JSON.parse(output);
    } catch {
      parsed = { raw: output };
    }
    return NextResponse.json({
      success: true,
      transport: "http-cli-bridge",
      argv: args,
      durationMs: Date.now() - started,
      data: parsed,
    });
  } catch (err: unknown) {
    return fail(500, err instanceof Error ? err.message : String(err));
  }
}
