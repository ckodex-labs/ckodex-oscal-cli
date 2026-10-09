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
import path from "path";

export const dynamic = "force-static";
import { callOscalCli } from "@/lib/mcp-client";
import { bridgeError as fail, guardBridgeRequest } from "@/lib/bridge-guard";

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
  ["doctor"],
];

const FORBIDDEN_FLAGS = new Set(["-o", "--output", "--write", "--fix", "--apply"]);

function isAllowed(argv: string[]): boolean {
  return ALLOWED.some((prefix) => prefix.every((tok, i) => argv[i] === tok));
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

  let body: unknown;
  try {
    body = await req.json();
  } catch {
    return fail(400, "Body is not valid JSON");
  }
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
  if (args.some((a) => FORBIDDEN_FLAGS.has(a) || a.startsWith("--output="))) {
    return fail(403, "Output/write flags are not permitted through the bridge");
  }
  for (const a of args) {
    if (a.startsWith("-")) continue;
    if (a.includes("/") || a.endsWith(".json") || a.endsWith(".yaml")) {
      const resolved = path.resolve(PROJECT_ROOT, a);
      if (!resolved.startsWith(PROJECT_ROOT + path.sep)) {
        return fail(403, `Path '${a}' is outside the repository`);
      }
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
