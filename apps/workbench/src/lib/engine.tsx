"use client";

/**
 * Engine access for the Workbench.
 *
 * Resolution order for any data request:
 *   1. LIVE      local `mizan` via /api/cli (only exists under next dev/start)
 *   2. SNAPSHOT  build-time engine output under /snapshot/ (always on Pages)
 *   3. nothing   the caller decides whether to show a labeled FIXTURE or an
 *                explicit UNKNOWN state. This module never fabricates data.
 */

import * as React from "react";
import {
  fromSnapshot,
  type Provenance,
  type SnapshotManifest,
} from "@/lib/provenance";

export const BASE_PATH = process.env.NEXT_PUBLIC_BASE_PATH ?? "";

export type EngineStatus =
  | { kind: "probing" }
  | { kind: "live"; version: string; manifest: SnapshotManifest | null }
  | { kind: "snapshot"; manifest: SnapshotManifest }
  | { kind: "offline"; reason: string };

interface EngineContextValue {
  status: EngineStatus;
  manifest: SnapshotManifest | null;
  refresh: () => void;
}

const EngineContext = React.createContext<EngineContextValue | null>(null);

async function fetchJson<T>(url: string, init?: RequestInit): Promise<T> {
  const res = await fetch(url, { cache: "no-store", ...init });
  if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
  return (await res.json()) as T;
}

export async function loadManifest(): Promise<SnapshotManifest | null> {
  try {
    const m = await fetchJson<SnapshotManifest>(
      `${BASE_PATH}/snapshot/manifest.json`,
    );
    return m.schema === "mizan.workbench.snapshot/v1" ? m : null;
  } catch {
    return null;
  }
}

async function probeLive(): Promise<string | null> {
  try {
    const r = await fetchJson<{ success: boolean; version?: string }>(
      `${BASE_PATH}/api/cli`,
    );
    return r.success && r.version ? r.version : null;
  } catch {
    return null;
  }
}

export function EngineProvider({ children }: { children: React.ReactNode }) {
  const [status, setStatus] = React.useState<EngineStatus>({ kind: "probing" });
  const [manifest, setManifest] = React.useState<SnapshotManifest | null>(null);
  const [nonce, setNonce] = React.useState(0);

  React.useEffect(() => {
    let cancelled = false;
    (async () => {
      const [m, liveVersion] = await Promise.all([loadManifest(), probeLive()]);
      if (cancelled) return;
      setManifest(m);
      if (liveVersion) setStatus({ kind: "live", version: liveVersion, manifest: m });
      else if (m) setStatus({ kind: "snapshot", manifest: m });
      else
        setStatus({
          kind: "offline",
          reason:
            "No local engine and no snapshot. Run `npm run snapshot` in apps/workbench.",
        });
    })();
    return () => {
      cancelled = true;
    };
  }, [nonce]);

  const value = React.useMemo(
    () => ({ status, manifest, refresh: () => setNonce((n) => n + 1) }),
    [status, manifest],
  );
  return <EngineContext.Provider value={value}>{children}</EngineContext.Provider>;
}

export function useEngine(): EngineContextValue {
  const ctx = React.useContext(EngineContext);
  if (!ctx) throw new Error("useEngine must be used inside <EngineProvider>");
  return ctx;
}

export interface EngineData<T> {
  data: T | null;
  provenance: Provenance | null;
  loading: boolean;
  error: string | null;
  /** Exit code the engine returned when the snapshot was captured. */
  exitCode: number | null;
}

/**
 * Load the output of a snapshot command by id (see manifest.commands[].id),
 * or a derived projection file (e.g. "atlas") via `file`.
 */
export function useSnapshot<T>(
  commandId: string,
  opts: { file?: string } = {},
): EngineData<T> {
  const { manifest, status } = useEngine();
  const [state, setState] = React.useState<EngineData<T>>({
    data: null,
    provenance: null,
    loading: true,
    error: null,
    exitCode: null,
  });

  React.useEffect(() => {
    if (status.kind === "probing") return;
    if (!manifest) {
      setState({
        data: null,
        provenance: null,
        loading: false,
        error: "No snapshot manifest available",
        exitCode: null,
      });
      return;
    }
    const rec = manifest.commands.find((c) => c.id === commandId);
    const file = opts.file ?? rec?.output;
    if (!file) {
      setState({
        data: null,
        provenance: fromSnapshot(manifest, commandId),
        loading: false,
        error: rec
          ? `Command exited ${rec.exitCode}; no output captured`
          : `No snapshot entry '${commandId}'`,
        exitCode: rec?.exitCode ?? null,
      });
      return;
    }
    let cancelled = false;
    fetchJson<T>(`${BASE_PATH}/snapshot/${file}`)
      .then((data) => {
        if (cancelled) return;
        setState({
          data,
          provenance: fromSnapshot(manifest, commandId),
          loading: false,
          error: null,
          exitCode: rec?.exitCode ?? null,
        });
      })
      .catch((e: unknown) => {
        if (cancelled) return;
        setState({
          data: null,
          provenance: fromSnapshot(manifest, commandId),
          loading: false,
          error: e instanceof Error ? e.message : String(e),
          exitCode: rec?.exitCode ?? null,
        });
      });
    return () => {
      cancelled = true;
    };
  }, [manifest, status.kind, commandId, opts.file]);

  return state;
}

/** Run a read-only command against the local engine. Throws when not LIVE. */
export async function runLive<T = unknown>(
  argv: string[],
): Promise<{ data: T; provenance: Provenance; durationMs: number }> {
  const res = await fetch(`${BASE_PATH}/api/cli`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ argv }),
  });
  const body = (await res.json()) as {
    success: boolean;
    data?: T;
    error?: string;
    durationMs?: number;
  };
  if (!res.ok || !body.success) {
    throw new Error(body.error ?? `${res.status} ${res.statusText}`);
  }
  return {
    data: body.data as T,
    durationMs: body.durationMs ?? 0,
    provenance: {
      kind: "live",
      label: `mizan ${argv[0] ?? ""}`.trim(),
      command: `mizan ${argv.join(" ")} --format json`,
      generatedAt: new Date().toISOString(),
    },
  };
}
