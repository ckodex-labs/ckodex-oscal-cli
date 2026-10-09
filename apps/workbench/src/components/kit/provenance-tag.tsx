"use client";

import * as React from "react";
import { cn } from "@/lib/utils";
import { PROVENANCE_COPY, type Provenance } from "@/lib/provenance";

const STYLE: Record<Provenance["kind"], string> = {
  live: "text-ck-pos bg-ck-pos-bg border-ck-pos",
  snapshot: "text-ck-info bg-ck-info-bg border-ck-info",
  fixture: "text-ck-warn border-ck-warn ck-hatch",
  local: "text-ck-unk bg-ck-unk-bg border-ck-unk",
};

/** Glyphs so the kind is distinguishable without color. */
const GLYPH: Record<Provenance["kind"], string> = {
  live: "\u25CF", // filled circle
  snapshot: "\u25A0", // filled square
  fixture: "\u25B3", // open triangle
  local: "\u25CB", // open circle
};

function shortTime(iso?: string) {
  if (!iso) return null;
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toISOString().replace("T", " ").slice(0, 16) + "Z";
}

/**
 * Compact provenance badge. Click to reveal command, digest, commit and time.
 * Rendered in every Panel header.
 */
export function ProvenanceTag({
  provenance,
  className,
}: {
  provenance: Provenance;
  className?: string;
}) {
  const copy = PROVENANCE_COPY[provenance.kind];
  return (
    <details
      className={cn("relative inline-block group", className)}
      data-provenance={provenance.kind}
    >
      <summary
        className={cn(
          "list-none cursor-pointer select-none inline-flex items-center gap-1.5 rounded-sm border px-1.5 py-0.5",
          "font-mono text-2xs font-medium tracking-wide",
          "[&::-webkit-details-marker]:hidden",
          STYLE[provenance.kind],
        )}
        title={copy.meaning}
      >
        <span aria-hidden>{GLYPH[provenance.kind]}</span>
        <span>{copy.name}</span>
        <span className="font-normal opacity-80 max-w-[16ch] truncate">
          {provenance.label}
        </span>
      </summary>
      <div
        role="note"
        className="absolute right-0 z-40 mt-1 w-[min(380px,85vw)] rounded-md border border-ck-hairline-strong bg-ck-bg-0 p-3 shadow-lg text-xs text-ck-fg-2 space-y-2"
      >
        <p className="text-ck-fg-1 font-medium">
          {copy.name}: {copy.meaning}
        </p>
        {provenance.detail && <p>{provenance.detail}</p>}
        <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 font-mono text-2xs">
          {provenance.command && (
            <>
              <dt className="text-ck-fg-mute">command</dt>
              <dd className="break-all">{provenance.command}</dd>
            </>
          )}
          {provenance.digest && (
            <>
              <dt className="text-ck-fg-mute">stdout</dt>
              <dd className="break-all">{provenance.digest}</dd>
            </>
          )}
          {provenance.gitCommit && (
            <>
              <dt className="text-ck-fg-mute">commit</dt>
              <dd>{provenance.gitCommit.slice(0, 12)}</dd>
            </>
          )}
          {provenance.generatedAt && (
            <>
              <dt className="text-ck-fg-mute">captured</dt>
              <dd>{shortTime(provenance.generatedAt)}</dd>
            </>
          )}
        </dl>
      </div>
    </details>
  );
}
