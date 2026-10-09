"use client";

import * as React from "react";
import { cn } from "@/lib/utils";
import { StateBadge } from "@/components/kit";
import {
  getRelationshipGlyph,
  type BridgeEdge,
  type MappingRelationship,
  type MappingRow,
} from "@/lib/atlas-data";
import { pct } from "./shared";

/* ------------------------------------------------------------------ */
/* Visual encodings (shared with the legend)                           */
/* ------------------------------------------------------------------ */

export const RELATIONSHIPS: { rel: MappingRelationship; label: string }[] = [
  { rel: "equal-to", label: "equal to" },
  { rel: "equivalent-to", label: "equivalent to" },
  { rel: "subset-of", label: "subset of" },
  { rel: "superset-of", label: "superset of" },
  { rel: "intersects-with", label: "intersects with" },
];

export function relLabel(rel: MappingRelationship) {
  return rel.replace(/-/g, " ");
}

export type ConfidenceBand = "high" | "medium" | "low";

export function confidenceBand(conf: number): ConfidenceBand {
  if (conf >= 0.8) return "high";
  if (conf >= 0.6) return "medium";
  return "low";
}

export function edgeStroke(e: Pick<BridgeEdge, "conf" | "st" | "der">, selected: boolean) {
  const band = confidenceBand(e.conf);
  return {
    stroke: selected
      ? "var(--ck-accent)"
      : e.st === "draft"
        ? "var(--ck-info)"
        : "var(--ck-fg-2)",
    strokeWidth: selected ? 3 : band === "high" ? 2.5 : 1.5,
    strokeDasharray: e.der ? "1 4" : band === "low" ? "5 4" : undefined,
  };
}

export function RelationshipGlyph({
  rel,
  size = 18,
  className,
}: {
  rel: MappingRelationship;
  size?: number;
  className?: string;
}) {
  const g = getRelationshipGlyph(rel);
  return (
    <svg
      width={size}
      height={size}
      viewBox="-11 -11 22 22"
      aria-hidden
      className={cn("shrink-0", className)}
    >
      <circle cx={g.c1x} cy={0} r={g.c1r} fill={g.c1f} stroke="currentColor" strokeWidth={1.4} />
      <circle cx={g.c2x} cy={0} r={g.c2r} fill={g.c2f} stroke="currentColor" strokeWidth={1.4} />
    </svg>
  );
}

export function LineSample({
  conf,
  st = "complete",
  der,
}: {
  conf: number;
  st?: BridgeEdge["st"];
  der?: boolean;
}) {
  const s = edgeStroke({ conf, st, der }, false);
  return (
    <svg width={28} height={10} aria-hidden className="shrink-0">
      <line x1={1} y1={5} x2={27} y2={5} {...s} />
    </svg>
  );
}

/* ------------------------------------------------------------------ */
/* Geometry                                                            */
/* ------------------------------------------------------------------ */

const ROW_H = 56;
const GAP = 8;
const rowY = (i: number) => i * (ROW_H + GAP) + ROW_H / 2;

/** Point on the cubic M0,yl C50,yl 50,yr 100,yr at parameter t. */
function bezier(t: number, yl: number, yr: number) {
  const u = 1 - t;
  const x = 150 * t * u + 100 * t * t * t;
  const y = yl * (u * u * u + 3 * u * u * t) + yr * (3 * u * t * t + t * t * t);
  return { x, y };
}

/* ------------------------------------------------------------------ */
/* Diagram                                                             */
/* ------------------------------------------------------------------ */

export function BridgeDiagram({
  source,
  target,
  edges,
  sourceLabel,
  targetLabel,
  selectedId,
  onSelect,
}: {
  source: MappingRow[];
  target: MappingRow[];
  edges: BridgeEdge[];
  sourceLabel: string;
  targetLabel: string;
  selectedId: string | null;
  onSelect: (id: string) => void;
}) {
  const rows = Math.max(source.length, target.length);
  const height = rows * (ROW_H + GAP) - GAP;
  const selected = edges.find((e) => e.id === selectedId) ?? null;

  const srcCount = React.useMemo(() => {
    const m = new Map<number, number>();
    edges.forEach((e) => m.set(e.l, (m.get(e.l) ?? 0) + 1));
    return m;
  }, [edges]);
  const tgtCount = React.useMemo(() => {
    const m = new Map<number, number>();
    edges.forEach((e) => m.set(e.r, (m.get(e.r) ?? 0) + 1));
    return m;
  }, [edges]);

  // Spread markers whose midpoints coincide along their own curves.
  const markers = React.useMemo(() => {
    const groups = new Map<number, BridgeEdge[]>();
    edges.forEach((e) => {
      const key = Math.round((rowY(e.l) + rowY(e.r)) / 2);
      groups.set(key, [...(groups.get(key) ?? []), e]);
    });
    const out: { e: BridgeEdge; x: number; y: number }[] = [];
    groups.forEach((g) => {
      g.forEach((e, i) => {
        const t = 0.5 + (i - (g.length - 1) / 2) * 0.24;
        const p = bezier(t, rowY(e.l), rowY(e.r));
        out.push({ e, x: p.x, y: p.y });
      });
    });
    return out;
  }, [edges]);

  // Draw the selected edge last so it sits on top.
  const ordered = React.useMemo(
    () => [...edges].sort((a, b) => Number(a.id === selectedId) - Number(b.id === selectedId)),
    [edges, selectedId],
  );

  return (
    <div className="min-w-0">
      <div className="mb-2 grid grid-cols-[minmax(0,1fr)_minmax(88px,180px)_minmax(0,1fr)] gap-2">
        <p className="ck-eyebrow truncate">{sourceLabel}</p>
        <p className="ck-eyebrow text-center">relationship</p>
        <p className="ck-eyebrow truncate text-right">{targetLabel}</p>
      </div>
      <div
        className="grid grid-cols-[minmax(0,1fr)_minmax(88px,180px)_minmax(0,1fr)] gap-2"
        style={{ height }}
      >
        <RowColumn
          rows={source}
          counts={srcCount}
          active={selected ? selected.l : null}
          align="left"
        />
        <div className="relative min-w-0">
          <svg
            className="absolute inset-0 h-full w-full overflow-visible"
            viewBox={`0 0 100 ${height}`}
            preserveAspectRatio="none"
            aria-hidden
          >
            {ordered.map((e) => {
              const yl = rowY(e.l);
              const yr = rowY(e.r);
              const s = edgeStroke(e, e.id === selectedId);
              return (
                <path
                  key={e.id}
                  d={`M0 ${yl} C50 ${yl}, 50 ${yr}, 100 ${yr}`}
                  fill="none"
                  vectorEffect="non-scaling-stroke"
                  {...s}
                  className="cursor-pointer"
                  onClick={() => onSelect(e.id)}
                />
              );
            })}
          </svg>
          {markers.map(({ e, x, y }) => {
            const isSel = e.id === selectedId;
            return (
              <button
                key={e.id}
                type="button"
                onClick={() => onSelect(e.id)}
                aria-pressed={isSel}
                aria-label={`Mapping ${source[e.l]?.id ?? ""} ${relLabel(e.rel)} ${target[e.r]?.id ?? ""}, confidence ${pct(e.conf)}, ${e.st}`}
                title={`${source[e.l]?.id} ${relLabel(e.rel)} ${target[e.r]?.id}`}
                className={cn(
                  "absolute flex h-7 w-7 -translate-x-1/2 -translate-y-1/2 items-center justify-center rounded-full border bg-ck-bg-1 transition-colors",
                  "focus:outline-none focus-visible:ring-2 focus-visible:ring-ck-accent",
                  isSel
                    ? "z-10 border-ck-accent text-ck-accent-text ring-2 ring-ck-accent"
                    : e.st === "draft"
                      ? "border-ck-info text-ck-info hover:bg-ck-bg-2"
                      : "border-ck-hairline-strong text-ck-fg-1 hover:bg-ck-bg-2",
                )}
                style={{ left: `${x}%`, top: y }}
              >
                <RelationshipGlyph rel={e.rel} size={18} />
              </button>
            );
          })}
        </div>
        <RowColumn
          rows={target}
          counts={tgtCount}
          active={selected ? selected.r : null}
          align="right"
        />
      </div>
    </div>
  );
}

function RowColumn({
  rows,
  counts,
  active,
  align,
}: {
  rows: MappingRow[];
  counts: Map<number, number>;
  active: number | null;
  align: "left" | "right";
}) {
  return (
    <ol className="flex min-w-0 flex-col" style={{ gap: GAP }}>
      {rows.map((r, i) => {
        const n = counts.get(i) ?? 0;
        const isActive = active === i;
        return (
          <li
            key={r.id}
            className={cn(
              "flex min-w-0 flex-col justify-center rounded-md border px-2.5",
              n === 0 && !isActive && "border-dashed",
              align === "right" && "items-end text-right",
              isActive
                ? "border-ck-accent bg-ck-bg-2"
                : n === 0
                  ? "border-ck-hairline-strong bg-ck-bg-1"
                  : "border-ck-hairline bg-ck-bg-0",
            )}
            style={{ height: ROW_H }}
          >
            <div
              className={cn(
                "flex min-w-0 max-w-full items-center gap-1.5",
                align === "right" && "flex-row-reverse",
              )}
            >
              <span className="truncate font-mono text-xs font-medium text-ck-fg-1">{r.id}</span>
              <span className="font-mono text-2xs text-ck-fg-mute ck-num" aria-label={`${n} mappings`}>
                {n}
              </span>
            </div>
            <p className="max-w-full line-clamp-2 break-words text-xs leading-4 text-ck-fg-3" title={r.t}>
              {r.t}
            </p>
          </li>
        );
      })}
    </ol>
  );
}
