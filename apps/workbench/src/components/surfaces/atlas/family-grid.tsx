"use client";

/**
 * Family grid: one labeled block per control family, one cell per control.
 * Enhancements are grouped with their parent control.
 *
 * Color encodes only what the snapshot supports:
 *   - SSP implementation state (ssp-status.json), never negative by absence
 *   - catalog membership (in baseline or not) and withdrawn status (atlas.json)
 */

import * as React from "react";
import { cn } from "@/lib/utils";
import type {
  AtlasControl,
  SspImplementedRequirement,
} from "@/lib/atlas-types";
import {
  displayId,
  implLabel,
  implState,
  shortId,
  statusTone,
  type AtlasIndex,
  type ImplState,
} from "./model";

export type GridMode = "baseline" | "catalog";

const WITHDRAWN_BG: React.CSSProperties = {
  backgroundImage:
    "repeating-linear-gradient(135deg, var(--ck-hairline-strong) 0 2px, transparent 2px 6px)",
};

const TONE_CELL: Record<"pos" | "warn" | "info" | "neutral", string> = {
  pos: "border-ck-pos bg-ck-pos-bg text-ck-pos",
  warn: "border-ck-warn bg-ck-warn-bg text-ck-warn",
  info: "border-ck-info bg-ck-info-bg text-ck-info",
  neutral: "border-ck-hairline-strong bg-ck-bg-2 text-ck-fg-1",
};

export const CELL_CLASS = {
  withdrawn: "border-ck-hairline-strong text-ck-fg-mute line-through",
  notInBaseline:
    "border-dotted border-ck-hairline-strong text-ck-fg-mute opacity-60",
  undeclared: "border-dashed border-ck-hairline-strong bg-ck-bg-0 text-ck-fg-2",
  declaredEmpty: "border-ck-unk bg-ck-unk-bg text-ck-fg-1",
  selected:
    "border-solid border-ck-fg-1 bg-ck-fg-1 text-ck-bg-0 opacity-100 shadow-[0_0_0_2px_var(--ck-accent)]",
} as const;

export function statusCellClass(status: string): string {
  return TONE_CELL[statusTone(status)];
}

export function cellClass(c: AtlasControl, s: ImplState): string {
  if (c.withdrawn) return CELL_CLASS.withdrawn;
  if (!c.in_baseline) return CELL_CLASS.notInBaseline;
  if (s.kind === "undeclared") return CELL_CLASS.undeclared;
  if (s.kind === "declared-empty") return CELL_CLASS.declaredEmpty;
  return statusCellClass(s.status);
}

/** Swatch for the legend, using the same classes as the cells. */
export function Swatch({
  className,
  style,
}: {
  className: string;
  style?: React.CSSProperties;
}) {
  return (
    <span
      aria-hidden
      style={style}
      className={cn(
        "inline-block h-3.5 w-5 shrink-0 rounded-sm border",
        className,
      )}
    />
  );
}

export const WITHDRAWN_STYLE = WITHDRAWN_BG;

interface Group {
  rootId: string;
  root: AtlasControl | null; // null when the root itself is filtered out
  members: AtlasControl[]; // enhancements that are visible
}

interface FamilyBlock {
  id: string;
  title: string;
  totalInCatalog: number;
  inBaseline: number;
  visibleCount: number;
  groups: Group[];
}

export function buildBlocks(
  index: AtlasIndex,
  visible: AtlasControl[],
): FamilyBlock[] {
  const byFamily = new Map<string, Map<string, Group>>();
  const counts = new Map<string, number>();
  for (const c of visible) {
    const rootId = c.parent ?? c.id;
    let groups = byFamily.get(c.family);
    if (!groups) {
      groups = new Map();
      byFamily.set(c.family, groups);
    }
    let g = groups.get(rootId);
    if (!g) {
      g = { rootId, root: null, members: [] };
      groups.set(rootId, g);
    }
    if (c.parent) g.members.push(c);
    else g.root = c;
    counts.set(c.family, (counts.get(c.family) ?? 0) + 1);
  }
  return index.atlas.families
    .filter((f) => byFamily.has(f.id))
    .map((f) => ({
      id: f.id,
      title: f.title,
      totalInCatalog: f.total_in_catalog,
      inBaseline: f.in_baseline,
      visibleCount: counts.get(f.id) ?? 0,
      groups: Array.from(byFamily.get(f.id)!.values()),
    }));
}

export function FamilyGrid({
  index,
  visible,
  byControl,
  mode,
  selectedId,
  onSelect,
  onHover,
}: {
  index: AtlasIndex;
  visible: AtlasControl[];
  byControl: Map<string, SspImplementedRequirement>;
  mode: GridMode;
  selectedId: string;
  onSelect: (id: string) => void;
  onHover: (id: string | null) => void;
}) {
  const blocks = React.useMemo(
    () => buildBlocks(index, visible),
    [index, visible],
  );
  const containerRef = React.useRef<HTMLDivElement>(null);
  const activeId = visible.some((c) => c.id === selectedId)
    ? selectedId
    : visible[0]?.id;

  const onKeyDown = (e: React.KeyboardEvent<HTMLDivElement>) => {
    const keys = [
      "ArrowRight",
      "ArrowDown",
      "ArrowLeft",
      "ArrowUp",
      "Home",
      "End",
    ];
    if (!keys.includes(e.key)) return;
    const cells = Array.from(
      containerRef.current?.querySelectorAll<HTMLButtonElement>(
        "[data-cell]",
      ) ?? [],
    );
    const i = cells.findIndex((el) => el === document.activeElement);
    if (i < 0) return;
    e.preventDefault();
    let next = i;
    if (e.key === "ArrowRight" || e.key === "ArrowDown")
      next = Math.min(cells.length - 1, i + 1);
    if (e.key === "ArrowLeft" || e.key === "ArrowUp") next = Math.max(0, i - 1);
    if (e.key === "Home") next = 0;
    if (e.key === "End") next = cells.length - 1;
    cells[next]?.focus();
  };

  const renderCell = (c: AtlasControl, isEnhancement: boolean) => {
    const s = implState(c.id, byControl);
    const selected = c.id === selectedId;
    const status = c.withdrawn
      ? "withdrawn"
      : !c.in_baseline
        ? "not in Moderate baseline"
        : implLabel(s);
    return (
      <button
        key={c.id}
        type="button"
        data-cell
        data-control-id={c.id}
        tabIndex={c.id === activeId ? 0 : -1}
        aria-pressed={selected}
        aria-label={`${displayId(c.id)} ${c.title}, ${status}`}
        title={`${displayId(c.id)}  ${c.title}\n${status}`}
        onClick={() => onSelect(c.id)}
        onFocus={() => onHover(c.id)}
        onMouseEnter={() => onHover(c.id)}
        style={c.withdrawn ? WITHDRAWN_BG : undefined}
        className={cn(
          "h-7 shrink-0 rounded-sm border px-1.5 font-mono text-2xs leading-none transition-colors",
          "hover:border-ck-fg-3 focus-visible:outline-none focus-visible:shadow-[0_0_0_2px_var(--ck-accent)]",
          isEnhancement ? "min-w-[2.25rem]" : "min-w-[3.25rem] font-semibold",
          cellClass(c, s),
          selected && CELL_CLASS.selected,
        )}
      >
        {isEnhancement ? shortId(c.id) : displayId(c.id)}
      </button>
    );
  };

  return (
    <div
      ref={containerRef}
      onKeyDown={onKeyDown}
      onMouseLeave={() => onHover(null)}
      className="space-y-4"
      role="group"
      aria-label={
        mode === "baseline"
          ? "Moderate baseline controls by family"
          : "NIST SP 800-53 catalog controls by family"
      }
    >
      {blocks.map((b) => (
        <section key={b.id} aria-labelledby={`fam-${b.id}`} className="min-w-0">
          <header className="mb-1.5 flex flex-wrap items-baseline gap-x-2 gap-y-0.5">
            <h3
              id={`fam-${b.id}`}
              className="flex items-baseline gap-2 text-sm text-ck-fg-1"
            >
              <span className="font-mono text-xs font-semibold">
                {b.id.toUpperCase()}
              </span>
              <span className="font-medium">{b.title}</span>
            </h3>
            <span className="font-mono text-2xs text-ck-fg-mute ck-num">
              {mode === "baseline"
                ? `${b.inBaseline} in baseline`
                : `${b.totalInCatalog} in catalog, ${b.inBaseline} in baseline`}
              {b.visibleCount !==
                (mode === "baseline" ? b.inBaseline : b.totalInCatalog) &&
                `, ${b.visibleCount} shown`}
            </span>
          </header>
          <div className="flex flex-wrap gap-1.5">
            {b.groups.map((g) => (
              <div
                key={g.rootId}
                className={cn(
                  "flex max-w-full flex-wrap items-center gap-0.5",
                  g.members.length > 0 &&
                    "rounded border border-ck-hairline bg-ck-bg-0 p-0.5",
                )}
              >
                {g.root ? (
                  renderCell(g.root, false)
                ) : (
                  <span className="px-1 font-mono text-2xs text-ck-fg-mute">
                    {displayId(g.rootId)}
                  </span>
                )}
                {g.members.map((m) => renderCell(m, true))}
              </div>
            ))}
          </div>
        </section>
      ))}
    </div>
  );
}
