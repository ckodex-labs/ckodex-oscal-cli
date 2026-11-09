/**
 * Multi-Tier OSCAL Transport Adapter
 *
 * ARCHITECTURAL CONTRACT:
 * - Tier 1 (Desktop Native): Tauri IPC (`window.__TAURI__.core.invoke`) when running inside the Tauri shell.
 * - Tier 2 (Local Web Server): Node.js `child_process.spawn` invoking the local `mizan` CLI binary via `callOscalCli`.
 * - Tier 3 (Static / Air-Gapped Web Distribution): In-browser OSCAL AST kernel and WebCrypto Merkle engine.
 *
 * NOTE: `callOscalCli` is designed for server-side execution (Next.js Node.js runtime) and local dev bridges.
 */

import { spawn } from "child_process";
import path from "path";
import fs from "fs";
import { BlastRadiusReport, FedrampReport, MergeReport } from "./oscal-types";

// Path to project root containing the Rust oscal-cli executable
const PROJECT_ROOT = path.resolve(process.cwd(), "../..");

function getMizanBinary(): { bin: string; argsPrefix: string[] } {
  const customBin = process.env.MIZAN_BIN;
  if (customBin && fs.existsSync(/*turbopackIgnore: true*/ customBin)) {
    return { bin: customBin, argsPrefix: [] };
  }

  const homeDir = process.env.HOME || process.env.USERPROFILE || "";
  const candidates: string[] = [];
  if (homeDir) {
    candidates.push(
      path.join(/*turbopackIgnore: true*/ homeDir, ".cache/cargo-target/release/mizan"),
      path.join(/*turbopackIgnore: true*/ homeDir, ".cache/cargo-target/debug/mizan"),
    );
  }
  candidates.push(
    path.resolve(/*turbopackIgnore: true*/ PROJECT_ROOT, "target/release/mizan"),
    path.resolve(/*turbopackIgnore: true*/ PROJECT_ROOT, "target/debug/mizan"),
  );

  const existing = candidates.filter((p) => {
    try {
      return fs.statSync(/*turbopackIgnore: true*/ p).isFile();
    } catch {
      return false;
    }
  });
  if (existing.length > 0) {
    existing.sort(
      (a, b) =>
        fs.statSync(/*turbopackIgnore: true*/ b).mtimeMs -
        fs.statSync(/*turbopackIgnore: true*/ a).mtimeMs,
    );
    return { bin: existing[0], argsPrefix: [] };
  }

  return {
    bin: "cargo",
    argsPrefix: ["run", "--quiet", "--bin", "mizan", "--"],
  };
}

export async function callOscalCli(args: string[]): Promise<string> {
  const { bin, argsPrefix } = getMizanBinary();
  const fullArgs = [...argsPrefix, ...args];

  return new Promise((resolve, reject) => {
    const child = spawn(/*turbopackIgnore: true*/ bin, fullArgs, {
      cwd: PROJECT_ROOT,
      env: { ...process.env, RUST_LOG: "error" },
    });

    let stdout = "";
    let stderr = "";

    child.stdout.on("data", (data) => {
      stdout += data.toString();
    });

    child.stderr.on("data", (data) => {
      stderr += data.toString();
    });

    child.on("close", (code) => {
      if (code === 0) {
        resolve(stdout.trim());
      } else {
        reject(new Error(`mizan CLI failed (${code}): ${stderr || stdout}`));
      }
    });

    child.on("error", (err) => {
      reject(err);
    });
  });
}

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
    __TAURI__?: {
      core?: {
        invoke: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
      };
    };
  }
}

export async function getBlastRadius(
  target: string,
  file: string,
): Promise<BlastRadiusReport> {
  if (typeof window !== "undefined" && window.__TAURI__?.core) {
    return window.__TAURI__.core.invoke<BlastRadiusReport>(
      "compute_blast_radius",
      { file, target },
    );
  }
  const output = await callOscalCli([
    "blast-radius",
    file,
    "--target",
    target,
    "--format",
    "json",
  ]);
  return JSON.parse(output) as BlastRadiusReport;
}

export async function validateFedramp(
  file: string,
  baseline = "moderate",
): Promise<FedrampReport> {
  if (typeof window !== "undefined" && window.__TAURI__?.core) {
    return window.__TAURI__.core.invoke<FedrampReport>("validate_fedramp_pmo", {
      file,
      baseline,
    });
  }
  const output = await callOscalCli([
    "fedramp",
    "validate",
    file,
    "--baseline",
    baseline,
    "--format",
    "json",
  ]);
  return JSON.parse(output) as FedrampReport;
}

export async function sync3Way(
  base: string,
  upstream: string,
  local: string,
  strategy = "manual",
): Promise<MergeReport> {
  if (typeof window !== "undefined" && window.__TAURI__?.core) {
    return window.__TAURI__.core.invoke<MergeReport>("sync_3way", {
      base,
      upstream,
      local,
      strategy,
    });
  }
  const output = await callOscalCli([
    "sync",
    "--base",
    base,
    "--upstream",
    upstream,
    "--local",
    local,
    "--strategy",
    strategy,
    "--format",
    "json",
  ]);
  return JSON.parse(output) as MergeReport;
}
