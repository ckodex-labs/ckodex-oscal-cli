import { spawn } from "child_process";
import path from "path";
import { BlastRadiusReport, FedrampReport, MergeReport } from "./oscal-types";

// Path to project root containing the Rust oscal-cli executable
const PROJECT_ROOT = path.resolve(process.cwd(), "../..");

export async function callOscalCli(args: string[]): Promise<string> {
  return new Promise((resolve, reject) => {
    const child = spawn("cargo", ["run", "--quiet", "--", ...args], {
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
        reject(new Error(`oscal-cli failed (${code}): ${stderr || stdout}`));
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
