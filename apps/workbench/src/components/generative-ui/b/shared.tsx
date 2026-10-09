"use client";

/**
 * Shared helpers for the group B surfaces (Pipeline, Jurisdictions & SLSA,
 * Policy Gates & SBOM). Presentation only: nothing here invents data.
 */

import * as React from "react";
import { cn } from "@/lib/utils";
import { useEngine, useSnapshot, type EngineData } from "@/lib/engine";
import type { Provenance, SnapshotCommandRecord } from "@/lib/provenance";
import { EmptyState } from "@/components/kit";

/** Strip the global `--format json` prefix so labels name the subcommand. */
export function subcommand(argv: string[]): string[] {
  const out = [...argv];
  if (out[0] === "--format" && out.length >= 2) out.splice(0, 2);
  return out;
}

export interface SnapResult<T> extends EngineData<T> {
  record: SnapshotCommandRecord | null;
  /** `mizan <argv>` exactly as captured. */
  command: string;
}

/**
 * useSnapshot plus the manifest record, and a provenance label that names the
 * subcommand (e.g. "mizan pipeline run") instead of the leading global flag.
 */
export function useSnap<T>(commandId: string, labelWords = 2): SnapResult<T> {
  const state = useSnapshot<T>(commandId);
  const { manifest } = useEngine();
  const record = manifest?.commands.find((c) => c.id === commandId) ?? null;
  const command = record ? `mizan ${record.argv.join(" ")}` : `mizan (${commandId})`;
  let provenance: Provenance | null = state.provenance;
  if (provenance && record && provenance.kind === "snapshot") {
    const words = subcommand(record.argv).slice(0, labelWords).join(" ");
    provenance = { ...provenance, label: `mizan ${words}`.trim() };
  }
  return { ...state, provenance, record, command };
}

/** Number or an explicit UNKNOWN marker. Never coerces missing to 0. */
export function num(v: number | undefined | null): string {
  return typeof v === "number" && Number.isFinite(v) ? String(v) : "UNKNOWN";
}

export function bool(v: boolean | undefined | null): string {
  return typeof v === "boolean" ? String(v) : "UNKNOWN";
}

export function pretty(v: unknown): string {
  return JSON.stringify(v, null, 2);
}

/** Loading / failure rendering for a snapshot that has no data. */
export function SnapFallback({
  title,
  state,
}: {
  title: string;
  state: { loading: boolean; error: string | null };
}) {
  if (state.loading) {
    return (
      <div
        className="rounded-lg border border-ck-hairline bg-ck-bg-1 px-4 py-6 text-sm text-ck-fg-mute"
        aria-busy="true"
      >
        Loading {title}...
      </div>
    );
  }
  return (
    <EmptyState kind="unknown" title={`${title}: not available`}>
      {state.error ?? "The snapshot did not contain this output."}
    </EmptyState>
  );
}

/** Short factual note, used for known engine defects and data caveats. */
export function Note({
  tone = "info",
  title,
  children,
  className,
}: {
  tone?: "info" | "warn";
  title: string;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <div
      role="note"
      className={cn(
        "rounded-md border px-3 py-2 text-sm",
        tone === "warn"
          ? "border-ck-warn bg-ck-warn-bg text-ck-fg-2"
          : "border-ck-info bg-ck-info-bg text-ck-fg-2",
        className,
      )}
    >
      <p
        className={cn(
          "text-xs font-semibold",
          tone === "warn" ? "text-ck-warn" : "text-ck-info",
        )}
      >
        {title}
      </p>
      <div className="mt-0.5 text-sm text-ck-fg-2">{children}</div>
    </div>
  );
}

/** Field list. Values are rendered as-is; long identifiers wrap. */
export function Fields({
  items,
  className,
}: {
  items: { label: string; value: React.ReactNode; mono?: boolean }[];
  className?: string;
}) {
  return (
    <dl className={cn("grid grid-cols-1 gap-x-6 gap-y-2 sm:grid-cols-[minmax(0,9rem)_minmax(0,1fr)]", className)}>
      {items.map((it) => (
        <React.Fragment key={it.label}>
          <dt className="text-xs text-ck-fg-mute sm:pt-0.5">{it.label}</dt>
          <dd
            className={cn(
              "min-w-0 break-words text-sm text-ck-fg-1",
              it.mono && "break-all font-mono text-xs sm:pt-0.5",
            )}
          >
            {it.value}
          </dd>
        </React.Fragment>
      ))}
    </dl>
  );
}

export const btnClass =
  "inline-flex items-center gap-1.5 rounded-md border border-ck-hairline-strong bg-ck-bg-0 px-3 py-1.5 text-sm font-medium text-ck-fg-1 hover:bg-ck-bg-2 disabled:cursor-not-allowed disabled:text-ck-fg-mute focus-visible:outline focus-visible:outline-2 focus-visible:outline-ck-accent";

export const btnPrimaryClass =
  "inline-flex items-center gap-1.5 rounded-md border border-ck-accent bg-ck-accent px-3 py-1.5 text-sm font-medium text-white hover:brightness-110 disabled:cursor-not-allowed disabled:brightness-75 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ck-accent";

export function Code({ children, nowrap }: { children: React.ReactNode; nowrap?: boolean }) {
  return (
    <code className={cn("font-mono text-xs text-ck-fg-1", nowrap ? "whitespace-nowrap" : "break-all")}>
      {children}
    </code>
  );
}
