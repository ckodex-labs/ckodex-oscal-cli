/**
 * Provenance: where a piece of rendered data came from.
 *
 * Every panel in the Workbench renders a ProvenanceTag. This is the UI's
 * equivalent of the evidence language used in the engine:
 *
 *   LIVE      returned just now by the local `mizan` binary (dev / desktop only)
 *   SNAPSHOT  real `mizan` output captured at build time, with command + digest
 *   FIXTURE   illustrative data that the engine did not produce
 *   LOCAL     produced in this browser session; unsigned and not persisted
 *
 * Nothing in the UI may describe LOCAL or FIXTURE data as signed, verified,
 * or produced by the engine.
 */

export type ProvenanceKind = "live" | "snapshot" | "fixture" | "local";

export interface SnapshotCommandRecord {
  /** Stable key, e.g. "nist-moderate-resolve". */
  id: string;
  /** Human description of what was run. */
  title: string;
  /** argv passed to mizan (paths are repo-relative). */
  argv: string[];
  exitCode: number;
  durationMs: number;
  /** sha256 of the raw stdout bytes, hex. */
  stdoutSha256: string;
  /** Output file under /snapshot/, if stdout was captured. */
  output?: string;
  /** First 2 KB of stderr when the command failed. */
  stderr?: string;
}

export interface SnapshotManifest {
  schema: "mizan.workbench.snapshot/v1";
  generatedAt: string;
  mizanVersion: string;
  gitCommit: string;
  gitDirty: boolean;
  inputs: { path: string; sha256: string; source?: string }[];
  commands: SnapshotCommandRecord[];
  /** Commands deliberately not run (network, server, interactive), with why. */
  skipped?: { id: string; reason: string }[];
}

export interface Provenance {
  kind: ProvenanceKind;
  /** Short label shown in the tag, e.g. "mizan resolve". */
  label: string;
  /** Longer explanation shown on hover / in the details popover. */
  detail?: string;
  command?: string;
  digest?: string;
  generatedAt?: string;
  gitCommit?: string;
}

export const PROVENANCE_COPY: Record<
  ProvenanceKind,
  { name: string; meaning: string }
> = {
  live: {
    name: "LIVE",
    meaning: "Returned just now by the local mizan binary.",
  },
  snapshot: {
    name: "SNAPSHOT",
    meaning:
      "Real mizan output captured when this site was built. Command and output digest are recorded in the snapshot manifest.",
  },
  fixture: {
    name: "FIXTURE",
    meaning:
      "Illustrative data. The engine did not produce it and it describes no real system.",
  },
  local: {
    name: "LOCAL",
    meaning:
      "Produced in this browser session. The digest is a SHA-256 of the event text; it is not a signature and is not persisted.",
  },
};

export function fixture(label: string, detail?: string): Provenance {
  return { kind: "fixture", label, detail };
}

export function local(label: string, detail?: string): Provenance {
  return { kind: "local", label, detail };
}

export function fromSnapshot(
  manifest: SnapshotManifest,
  commandId: string,
): Provenance {
  const rec = manifest.commands.find((c) => c.id === commandId);
  if (!rec) {
    return {
      kind: "fixture",
      label: "missing snapshot",
      detail: `Snapshot command '${commandId}' is not in the manifest.`,
    };
  }
  return {
    kind: "snapshot",
    label: `mizan ${rec.argv[0] ?? ""}`.trim(),
    detail: rec.title,
    command: `mizan ${rec.argv.join(" ")}`,
    digest: `sha256:${rec.stdoutSha256}`,
    generatedAt: manifest.generatedAt,
    gitCommit: manifest.gitCommit,
  };
}

export async function sha256Hex(text: string): Promise<string> {
  const buf = await crypto.subtle.digest(
    "SHA-256",
    new TextEncoder().encode(text),
  );
  return Array.from(new Uint8Array(buf))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}
