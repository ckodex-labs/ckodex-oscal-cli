"use client";

/**
 * Derived, read-only indexes over the atlas.json and ssp-status.json
 * snapshots. Pure functions plus thin hooks; no data is invented here.
 */

import * as React from "react";
import { useSnapshot, type EngineData } from "@/lib/engine";
import type {
  Atlas,
  AtlasControl,
  AtlasFamily,
  AtlasParam,
  SspImplementedRequirement,
  SspStatus,
} from "@/lib/atlas-types";

export interface AtlasIndex {
  atlas: Atlas;
  byId: Map<string, AtlasControl>;
  familyById: Map<string, AtlasFamily>;
  /** Direct enhancements of each control, in catalog order. */
  children: Map<string, AtlasControl[]>;
  baseline: AtlasControl[];
  withdrawnCount: number;
}

export function buildIndex(atlas: Atlas): AtlasIndex {
  const byId = new Map<string, AtlasControl>();
  const children = new Map<string, AtlasControl[]>();
  let withdrawnCount = 0;
  for (const c of atlas.controls) {
    byId.set(c.id, c);
    if (c.withdrawn) withdrawnCount += 1;
    if (c.parent) {
      const list = children.get(c.parent) ?? [];
      list.push(c);
      children.set(c.parent, list);
    }
  }
  return {
    atlas,
    byId,
    familyById: new Map(atlas.families.map((f) => [f.id, f])),
    children,
    baseline: atlas.controls.filter((c) => c.in_baseline),
    withdrawnCount,
  };
}

export function useAtlas(): EngineData<Atlas> & { index: AtlasIndex | null } {
  const snap = useSnapshot<Atlas>("nist-moderate-resolve", {
    file: "atlas.json",
  });
  const index = React.useMemo(
    () =>
      snap.data && Array.isArray(snap.data.controls)
        ? buildIndex(snap.data)
        : null,
    [snap.data],
  );
  return { ...snap, index };
}

export function useSspStatus(): EngineData<SspStatus> & {
  byControl: Map<string, SspImplementedRequirement>;
} {
  const snap = useSnapshot<SspStatus>("ssp-inspect", {
    file: "ssp-status.json",
  });
  const byControl = React.useMemo(
    () =>
      new Map(
        (snap.data?.implemented_requirements ?? []).map((r) => [
          r.control_id.toLowerCase(),
          r,
        ]),
      ),
    [snap.data],
  );
  return { ...snap, byControl };
}

/* ------------------------------------------------------------------ */
/* Display helpers                                                     */
/* ------------------------------------------------------------------ */

/** "ac-2" -> "AC-2", "ac-2.1" -> "AC-2(1)" (NIST display form). */
export function displayId(id: string): string {
  const [base, enh] = id.split(".");
  const up = base.toUpperCase();
  return enh ? `${up}(${enh})` : up;
}

/** Short label used inside an enhancement cell: "ac-2.13" -> "(13)". */
export function shortId(id: string): string {
  const enh = id.split(".")[1];
  return enh ? `(${enh})` : id.toUpperCase();
}

/** Normalize user search text so "AC-2(1)" and "ac-2.1" both match. */
export function normalizeQuery(q: string): string {
  return q
    .trim()
    .toLowerCase()
    .replace(/\((\d+)\)/g, ".$1")
    .replace(/\s+/g, " ");
}

export function matchesQuery(c: AtlasControl, nq: string): boolean {
  if (!nq) return true;
  return c.id.includes(nq) || c.title.toLowerCase().includes(nq);
}

/* ------------------------------------------------------------------ */
/* SSP implementation state                                            */
/* ------------------------------------------------------------------ */

export type ImplState =
  | { kind: "undeclared" }
  | { kind: "declared-empty"; req: SspImplementedRequirement }
  | { kind: "declared"; status: string; req: SspImplementedRequirement };

export function implState(
  id: string,
  byControl: Map<string, SspImplementedRequirement>,
): ImplState {
  const req = byControl.get(id);
  if (!req) return { kind: "undeclared" };
  if (req.status_presence !== "PRESENT" || !req.implementation_status) {
    return { kind: "declared-empty", req };
  }
  return { kind: "declared", status: req.implementation_status, req };
}

/** OSCAL implementation-status tokens -> tone. Unrecognized tokens stay neutral. */
export function statusTone(
  status: string,
): "pos" | "warn" | "info" | "neutral" {
  switch (status) {
    case "implemented":
      return "pos";
    case "partial":
      return "warn";
    case "planned":
    case "alternative":
      return "info";
    default:
      return "neutral";
  }
}

export function implLabel(s: ImplState): string {
  if (s.kind === "undeclared") return "No implementation declared";
  if (s.kind === "declared-empty") return "Declared in SSP, status EMPTY";
  return `Declared: ${s.status}`;
}

/* ------------------------------------------------------------------ */
/* Statement parsing                                                   */
/* ------------------------------------------------------------------ */

export type StatementSegment =
  { kind: "text"; text: string } | { kind: "param"; param: AtlasParam };

/**
 * Split a projected statement into text and parameter segments. Only bracket
 * tokens that exactly match a declared param id become params; other bracketed
 * text (e.g. "[AU-2a]" cross-references in au-12) stays literal.
 */
export function parseStatement(
  statement: string,
  params: AtlasParam[],
): StatementSegment[] {
  const byId = new Map(params.map((p) => [p.id, p]));
  const out: StatementSegment[] = [];
  const re = /\[([^\]\s]+)\]/g;
  let last = 0;
  let m: RegExpExecArray | null;
  while ((m = re.exec(statement))) {
    const p = byId.get(m[1]);
    if (!p) continue;
    if (m.index > last)
      out.push({ kind: "text", text: statement.slice(last, m.index) });
    out.push({ kind: "param", param: p });
    last = m.index + m[0].length;
  }
  if (last < statement.length)
    out.push({ kind: "text", text: statement.slice(last) });
  return out;
}

export function isTruncated(statement: string): boolean {
  return statement.endsWith("...");
}
