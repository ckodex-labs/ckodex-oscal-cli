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
import { StringDecoder } from "string_decoder";
import { BlastRadiusReport, FedrampReport, MergeReport } from "./oscal-types";

// Path to project root containing the Rust oscal-cli executable
const PROJECT_ROOT = path.resolve(process.cwd(), "../..");

export const DEFAULT_CLI_TIMEOUT_MS = 30_000;
export const MAX_OUTPUT_BYTES = 16 * 1024 * 1024;
export const MAX_CONCURRENT_PROCS = 8;

let activeProcs = 0;
const waitQueue: Array<() => void> = [];

function acquireProcSlot(): Promise<() => void> {
  return new Promise((resolve) => {
    const grant = () => {
      activeProcs += 1;
      let released = false;
      resolve(() => {
        if (released) return;
        released = true;
        activeProcs = Math.max(0, activeProcs - 1);
        const next = waitQueue.shift();
        if (next) next();
      });
    };
    if (activeProcs < MAX_CONCURRENT_PROCS) {
      grant();
    } else {
      waitQueue.push(grant);
    }
  });
}

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

export interface CliExecError extends Error {
  exitCode?: number | null;
  stdout?: string;
  stderr?: string;
}

export async function callOscalCli(
  args: string[],
  timeoutMs = DEFAULT_CLI_TIMEOUT_MS,
): Promise<string> {
  const releaseSlot = await acquireProcSlot();
  const { bin, argsPrefix } = getMizanBinary();
  const fullArgs = [...argsPrefix, ...args];

  return new Promise<string>((resolve, reject) => {
    let settled = false;
    let killEscalationTimer: ReturnType<typeof setTimeout> | null = null;

    const child = spawn(/*turbopackIgnore: true*/ bin, fullArgs, {
      cwd: PROJECT_ROOT,
      env: { ...process.env, RUST_LOG: "error" },
    });

    const terminateChild = () => {
      try {
        child.kill("SIGTERM");
      } catch {
        // ignore if already exited
      }
      killEscalationTimer = setTimeout(() => {
        try {
          child.kill("SIGKILL");
        } catch {
          // ignore if already exited
        }
      }, 2_000);
      killEscalationTimer.unref?.();
    };

    const finish = (fn: () => void) => {
      if (settled) return;
      settled = true;
      clearTimeout(timeoutTimer);
      if (killEscalationTimer) {
        clearTimeout(killEscalationTimer);
      }
      releaseSlot();
      fn();
    };

    const timeoutTimer = setTimeout(() => {
      terminateChild();
      finish(() => {
        reject(new Error(`mizan CLI timed out after ${timeoutMs} ms`));
      });
    }, timeoutMs);
    timeoutTimer.unref?.();

    const stdoutDecoder = new StringDecoder("utf8");
    const stderrDecoder = new StringDecoder("utf8");
    let stdout = "";
    let stderr = "";
    let stdoutBytes = 0;
    let stderrBytes = 0;

    child.stdout.on("data", (chunk: Buffer) => {
      stdoutBytes += chunk.length;
      if (stdoutBytes > MAX_OUTPUT_BYTES) {
        terminateChild();
        finish(() => {
          reject(
            new Error(
              `mizan CLI stdout exceeded ${MAX_OUTPUT_BYTES} byte limit`,
            ),
          );
        });
        return;
      }
      stdout += stdoutDecoder.write(chunk);
    });

    child.stderr.on("data", (chunk: Buffer) => {
      stderrBytes += chunk.length;
      if (stderrBytes > MAX_OUTPUT_BYTES) {
        terminateChild();
        finish(() => {
          reject(
            new Error(
              `mizan CLI stderr exceeded ${MAX_OUTPUT_BYTES} byte limit`,
            ),
          );
        });
        return;
      }
      stderr += stderrDecoder.write(chunk);
    });

    child.on("close", (code) => {
      stdout += stdoutDecoder.end();
      stderr += stderrDecoder.end();
      const trimmedStdout = stdout.trim();
      const trimmedStderr = stderr.trim();

      finish(() => {
        if (code === 0) {
          resolve(trimmedStdout);
          return;
        }
        const detail =
          trimmedStdout.startsWith("{") || trimmedStdout.startsWith("[")
            ? trimmedStdout
            : trimmedStderr || trimmedStdout;
        const err: CliExecError = new Error(
          `mizan CLI failed (${code ?? 1}): ${detail}`,
        );
        err.exitCode = code;
        err.stdout = trimmedStdout;
        err.stderr = trimmedStderr;
        reject(err);
      });
    });

    child.on("error", (err) => {
      finish(() => {
        reject(err);
      });
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
