#!/usr/bin/env node
// Crash-safe static HTML export builder for GitHub Pages and Tauri Desktop.
//
// Next.js `output: "export"` cannot bundle dynamic local HTTP-to-CLI bridge
// Route Handlers (`src/app/api/*`), which only run under `next dev` / `next start`
// (static deployments read build-time `public/snapshot/*` or Tauri IPC instead).
// This script atomically stashes `src/app/api` on the same filesystem during
// `next build`, restores it on exit/signals, and preserves `out/.gitkeep`.

import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const WORKBENCH_DIR = resolve(SCRIPT_DIR, "..");
const API_DIR = join(WORKBENCH_DIR, "src", "app", "api");
const STASH_DIR = join(WORKBENCH_DIR, "src", ".api-export-stash");
const NEXT_CACHE_DIR = join(WORKBENCH_DIR, ".next");
const OUT_DIR = join(WORKBENCH_DIR, "out");
const GITKEEP = join(OUT_DIR, ".gitkeep");

// Recover from any previously interrupted SIGKILL run before proceeding.
if (!existsSync(API_DIR) && existsSync(STASH_DIR)) {
  renameSync(STASH_DIR, API_DIR);
}

let stashed = false;
function restoreApiRoutes() {
  if (stashed && existsSync(STASH_DIR) && !existsSync(API_DIR)) {
    renameSync(STASH_DIR, API_DIR);
    stashed = false;
  }
}

for (const sig of ["SIGINT", "SIGTERM", "SIGHUP"]) {
  process.on(sig, () => {
    restoreApiRoutes();
    process.exit(1);
  });
}
process.on("uncaughtException", (err) => {
  restoreApiRoutes();
  console.error(err);
  process.exit(1);
});

let exitCode = 1;
try {
  if (existsSync(API_DIR)) {
    renameSync(API_DIR, STASH_DIR);
    stashed = true;
  }

  // Clear cached `.next/types` and `.next/dev/types` from prior `next dev`/`next build`
  // runs so TypeScript validation does not reference stashed `/api` routes.
  rmSync(NEXT_CACHE_DIR, { recursive: true, force: true });

  const nextBin = join(
    WORKBENCH_DIR,
    "node_modules",
    ".bin",
    process.platform === "win32" ? "next.cmd" : "next",
  );

  const result = spawnSync(nextBin, ["build"], {
    cwd: WORKBENCH_DIR,
    stdio: "inherit",
    shell: process.platform === "win32",
    env: {
      ...process.env,
      NODE_OPTIONS: "",
      NEXT_OUTPUT_EXPORT: "true",
    },
  });

  if (existsSync(OUT_DIR) && !existsSync(GITKEEP)) {
    mkdirSync(OUT_DIR, { recursive: true });
    writeFileSync(GITKEEP, "");
  }

  exitCode = result.status ?? 1;
} finally {
  restoreApiRoutes();
}

process.exit(exitCode);
