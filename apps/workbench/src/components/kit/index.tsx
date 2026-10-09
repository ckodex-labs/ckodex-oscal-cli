"use client";

/**
 * Workbench UI kit. All surfaces compose these primitives.
 *
 * Invariants:
 * - Panel REQUIRES a provenance. There is no unlabeled panel.
 * - Toolbar wraps; it never overflows its container.
 * - EmptyState distinguishes EMPTY (nothing exists) from UNKNOWN (not
 *   established). Neither is rendered as negative.
 * - StateBadge pairs color with a glyph and text.
 */

import * as React from "react";
import { cn } from "@/lib/utils";
import type { Provenance } from "@/lib/provenance";
import { ProvenanceTag } from "./provenance-tag";

export { ProvenanceTag };

/* ------------------------------------------------------------------ */
/* Page header                                                         */
/* ------------------------------------------------------------------ */

export function PageHeader({
  title,
  description,
  eyebrow,
  actions,
  meta,
}: {
  title: string;
  description?: React.ReactNode;
  eyebrow?: string;
  actions?: React.ReactNode;
  meta?: React.ReactNode;
}) {
  return (
    <header className="flex flex-col gap-3 border-b border-ck-hairline pb-5 md:flex-row md:items-end md:justify-between">
      <div className="min-w-0 space-y-1.5">
        {eyebrow && <p className="ck-eyebrow">{eyebrow}</p>}
        <h1 className="font-serif text-xl md:text-2xl text-ck-fg-1 leading-none">
          {title}
        </h1>
        {description && (
          <p className="max-w-3xl text-base text-ck-fg-3">{description}</p>
        )}
        {meta && <div className="flex flex-wrap items-center gap-2 pt-1">{meta}</div>}
      </div>
      {actions && <div className="flex flex-wrap items-center gap-2">{actions}</div>}
    </header>
  );
}

/* ------------------------------------------------------------------ */
/* Panel                                                               */
/* ------------------------------------------------------------------ */

export function Panel({
  title,
  provenance,
  subtitle,
  actions,
  children,
  className,
  bodyClassName,
  id,
}: {
  title: React.ReactNode;
  provenance: Provenance;
  subtitle?: React.ReactNode;
  actions?: React.ReactNode;
  children: React.ReactNode;
  className?: string;
  bodyClassName?: string;
  id?: string;
}) {
  return (
    <section
      id={id}
      data-panel
      className={cn(
        "min-w-0 rounded-lg border border-ck-hairline-strong bg-ck-bg-1",
        provenance.kind === "fixture" && "border-dashed",
        className,
      )}
    >
      <div className="flex flex-wrap items-start justify-between gap-2 border-b border-ck-hairline px-4 py-3">
        <div className="min-w-0">
          <h2 className="text-sm font-semibold text-ck-fg-1">{title}</h2>
          {subtitle && <p className="text-xs text-ck-fg-mute mt-0.5">{subtitle}</p>}
        </div>
        <div className="flex flex-wrap items-center gap-2">
          {actions}
          <ProvenanceTag provenance={provenance} />
        </div>
      </div>
      <div className={cn("p-4", bodyClassName)}>{children}</div>
    </section>
  );
}

/* ------------------------------------------------------------------ */
/* Toolbar                                                             */
/* ------------------------------------------------------------------ */

export function Toolbar({
  children,
  className,
}: {
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <div
      role="toolbar"
      className={cn("flex flex-wrap items-center gap-2 min-w-0", className)}
    >
      {children}
    </div>
  );
}

export function Segmented<T extends string>({
  value,
  options,
  onChange,
  label,
}: {
  value: T;
  options: { value: T; label: React.ReactNode }[];
  onChange: (v: T) => void;
  label: string;
}) {
  return (
    <div className="ck-segment" role="radiogroup" aria-label={label}>
      {options.map((o) => (
        <button
          key={o.value}
          type="button"
          role="radio"
          aria-checked={value === o.value}
          data-active={value === o.value}
          className="ck-segment-btn"
          onClick={() => onChange(o.value)}
        >
          {o.label}
        </button>
      ))}
    </div>
  );
}

/* ------------------------------------------------------------------ */
/* Stats                                                               */
/* ------------------------------------------------------------------ */

export function StatTile({
  label,
  value,
  hint,
  tone = "neutral",
}: {
  label: string;
  value: React.ReactNode;
  hint?: React.ReactNode;
  tone?: Tone;
}) {
  return (
    <div className="min-w-0 rounded-md border border-ck-hairline bg-ck-bg-0 px-3 py-2.5">
      <p className="ck-eyebrow truncate">{label}</p>
      <p className={cn("mt-1 text-lg font-semibold ck-num", TONE_TEXT[tone])}>{value}</p>
      {hint && <p className="mt-0.5 text-xs text-ck-fg-mute">{hint}</p>}
    </div>
  );
}

export function StatGrid({ children }: { children: React.ReactNode }) {
  return (
    <div className="grid grid-cols-2 gap-2 sm:grid-cols-3 xl:grid-cols-4">{children}</div>
  );
}

/* ------------------------------------------------------------------ */
/* State badges (vector-state dimensions)                              */
/* ------------------------------------------------------------------ */

export type Tone = "pos" | "neg" | "warn" | "info" | "unk" | "neutral";

const TONE_TEXT: Record<Tone, string> = {
  pos: "text-ck-pos",
  neg: "text-ck-neg",
  warn: "text-ck-warn",
  info: "text-ck-info",
  unk: "text-ck-unk",
  neutral: "text-ck-fg-1",
};

const TONE_BADGE: Record<Tone, string> = {
  pos: "text-ck-pos bg-ck-pos-bg border-ck-pos",
  neg: "text-ck-neg bg-ck-neg-bg border-ck-neg",
  warn: "text-ck-warn bg-ck-warn-bg border-ck-warn",
  info: "text-ck-info bg-ck-info-bg border-ck-info",
  unk: "text-ck-unk bg-ck-unk-bg border-ck-unk border-dashed",
  neutral: "text-ck-fg-2 bg-ck-bg-2 border-ck-hairline-strong",
};

const TONE_GLYPH: Record<Tone, string> = {
  pos: "\u2713", // check
  neg: "\u2715", // cross
  warn: "!",
  info: "i",
  unk: "?",
  neutral: "\u00B7",
};

export function StateBadge({
  tone,
  children,
  className,
  glyph = true,
}: {
  tone: Tone;
  children: React.ReactNode;
  className?: string;
  glyph?: boolean;
}) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 whitespace-nowrap rounded-sm border px-1.5 py-px font-mono text-2xs font-medium uppercase tracking-wide",
        TONE_BADGE[tone],
        className,
      )}
    >
      {glyph && <span aria-hidden>{TONE_GLYPH[tone]}</span>}
      {children}
    </span>
  );
}

/* ------------------------------------------------------------------ */
/* Legend                                                              */
/* ------------------------------------------------------------------ */

export function Legend({
  items,
  className,
}: {
  items: { swatch: React.ReactNode; label: string; count?: number }[];
  className?: string;
}) {
  return (
    <ul className={cn("flex flex-wrap items-center gap-x-4 gap-y-1.5 text-xs text-ck-fg-3", className)}>
      {items.map((it) => (
        <li key={it.label} className="inline-flex items-center gap-1.5">
          {it.swatch}
          <span>{it.label}</span>
          {it.count !== undefined && (
            <span className="font-mono text-ck-fg-mute ck-num">{it.count}</span>
          )}
        </li>
      ))}
    </ul>
  );
}

/* ------------------------------------------------------------------ */
/* Empty / unknown / error states                                      */
/* ------------------------------------------------------------------ */

export function EmptyState({
  kind,
  title,
  children,
}: {
  kind: "empty" | "unknown" | "error";
  title: string;
  children?: React.ReactNode;
}) {
  const label = { empty: "EMPTY", unknown: "UNKNOWN", error: "ENGINE ERROR" }[kind];
  const tone: Tone = kind === "error" ? "neg" : "unk";
  return (
    <div className="flex flex-col items-start gap-2 rounded-md border border-dashed border-ck-hairline-strong bg-ck-bg-0 px-4 py-6">
      <StateBadge tone={tone}>{label}</StateBadge>
      <p className="text-sm font-medium text-ck-fg-1">{title}</p>
      {children && <div className="text-sm text-ck-fg-3 max-w-prose">{children}</div>}
    </div>
  );
}

/* ------------------------------------------------------------------ */
/* Terminal                                                            */
/* ------------------------------------------------------------------ */

export function Terminal({
  command,
  output,
  exitCode,
  className,
  maxHeight = 420,
}: {
  command: string;
  output: string;
  exitCode?: number | null;
  className?: string;
  maxHeight?: number;
}) {
  return (
    <div className={cn("min-w-0 overflow-hidden rounded-md border border-ck-hairline-strong bg-[#111214] text-[#e6e4df]", className)}>
      <div className="flex items-center justify-between gap-2 border-b border-white/10 px-3 py-2">
        <code className="truncate font-mono text-xs text-[#c9c7c2]">$ {command}</code>
        {exitCode !== undefined && exitCode !== null && (
          <span
            className={cn(
              "shrink-0 font-mono text-2xs",
              exitCode === 0 ? "text-[#6ee7a0]" : "text-[#fca5a5]",
            )}
          >
            exit {exitCode}
          </span>
        )}
      </div>
      <pre
        className="overflow-auto whitespace-pre-wrap break-words p-3 font-mono text-xs leading-5"
        style={{ maxHeight }}
      >
        {output}
      </pre>
    </div>
  );
}

/* ------------------------------------------------------------------ */
/* Data table                                                          */
/* ------------------------------------------------------------------ */

export function DataTable({
  columns,
  rows,
  empty,
  caption,
}: {
  columns: { key: string; label: string; className?: string; numeric?: boolean }[];
  rows: Record<string, React.ReactNode>[];
  empty?: React.ReactNode;
  caption?: string;
}) {
  if (rows.length === 0 && empty) return <>{empty}</>;
  return (
    <div className="ck-scroll-x -mx-4 px-4">
      <table className="w-full min-w-[520px] border-collapse text-sm">
        {caption && <caption className="sr-only">{caption}</caption>}
        <thead>
          <tr className="border-b border-ck-hairline-strong">
            {columns.map((c) => (
              <th
                key={c.key}
                scope="col"
                className={cn(
                  "px-2 py-2 text-left ck-eyebrow font-normal",
                  c.numeric && "text-right",
                  c.className,
                )}
              >
                {c.label}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((r, i) => (
            <tr key={i} className="border-b border-ck-hairline last:border-0 hover:bg-ck-bg-0">
              {columns.map((c) => (
                <td
                  key={c.key}
                  className={cn("px-2 py-2 align-top text-ck-fg-2", c.numeric && "text-right ck-num font-mono", c.className)}
                >
                  {r[c.key]}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

/* ------------------------------------------------------------------ */
/* Read-only notice (phone widths)                                     */
/* ------------------------------------------------------------------ */

export function ReadOnlyNotice() {
  return (
    <p className="md:hidden rounded-md border border-ck-hairline bg-ck-bg-1 px-3 py-2 text-xs text-ck-fg-3">
      Read-only on small screens. Editing and engine actions are available on tablet and desktop.
    </p>
  );
}

/** Wrap mutating controls so they are hidden on phone widths. */
export function DesktopOnly({ children }: { children: React.ReactNode }) {
  return <div className="hidden md:contents">{children}</div>;
}
