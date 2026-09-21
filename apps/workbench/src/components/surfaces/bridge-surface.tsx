"use client";

import * as React from "react";
import { BridgeEdge, MappingRow, getRelationshipGlyph } from "@/lib/atlas-data";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

interface BridgeSurfaceProps {
  mer: MappingRow[];
  iso: MappingRow[];
  csf: MappingRow[];
  edgesIso: BridgeEdge[];
  edgesCsf: BridgeEdge[];
  onEdgeConfirmHuman?: (edgeId: string) => void;
}

export function BridgeSurface({
  mer,
  iso,
  csf,
  edgesIso,
  edgesCsf,
  onEdgeConfirmHuman,
}: BridgeSurfaceProps) {
  const [targetFw, setTargetFw] = React.useState<"iso" | "csf">("iso");
  const [viewMode, setViewMode] = React.useState<"diagram" | "ledger">(
    "diagram",
  );
  const [selectedEdgeId, setSelectedEdgeId] = React.useState<string | null>(
    null,
  );

  const rightRows = targetFw === "iso" ? iso : csf;
  const activeEdges = targetFw === "iso" ? edgesIso : edgesCsf;

  const bridgeHeight = Math.max(mer.length, rightRows.length) * 84;

  const selectedEdge = activeEdges.find((e) => e.id === selectedEdgeId) || null;

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex flex-wrap items-baseline justify-between border-b border-ck-hairline pb-2 gap-2">
        <div className="flex items-baseline gap-3 min-w-0">
          <h1 className="font-serif text-2xl font-normal tracking-tight text-ck-fg-1 whitespace-nowrap shrink-0">
            The Bridge
          </h1>
          <span className="text-xs text-ck-fg-mute font-mono hidden md:inline">
            Internal practice on left, external framework on right. Every edge
            is a provable claim.
          </span>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          <Badge
            variant="outline"
            className="font-mono text-[10px] uppercase text-green-700 dark:text-green-400 whitespace-nowrap shrink-0"
          >
            Federation Assurance 94.2% · Merkle Provable
          </Badge>
        </div>
      </div>

      {/* Toolbar */}
      <div className="flex items-center justify-between border border-ck-hairline bg-ck-bg-1 p-2 shadow-sm font-mono text-xs">
        {/* Framework Selector */}
        <div className="flex items-center gap-2">
          <span className="text-[10px] uppercase text-ck-fg-mute">
            Target Framework:
          </span>
          <Button
            size="sm"
            variant={targetFw === "iso" ? "default" : "outline"}
            onClick={() => {
              setTargetFw("iso");
              setSelectedEdgeId(null);
            }}
            className="h-7 text-xs font-mono"
          >
            ISO/IEC 27001:2022 ({iso.length})
          </Button>
          <Button
            size="sm"
            variant={targetFw === "csf" ? "default" : "outline"}
            onClick={() => {
              setTargetFw("csf");
              setSelectedEdgeId(null);
            }}
            className="h-7 text-xs font-mono"
          >
            NIST CSF 2.0 ({csf.length})
          </Button>
        </div>

        {/* View Mode */}
        <div className="flex items-center border border-ck-hairline bg-ck-bg-0 p-0.5">
          <button
            onClick={() => setViewMode("diagram")}
            className={`px-2 py-1 text-[11px] font-mono transition-colors ${
              viewMode === "diagram"
                ? "bg-ck-fg-1 text-ck-bg-0"
                : "text-ck-fg-mute"
            }`}
          >
            diagram
          </button>
          <button
            onClick={() => setViewMode("ledger")}
            className={`px-2 py-1 text-[11px] font-mono transition-colors ${
              viewMode === "ledger"
                ? "bg-ck-fg-1 text-ck-bg-0"
                : "text-ck-fg-mute"
            }`}
          >
            ledger
          </button>
        </div>
      </div>

      {/* Relationship Glyphs Legend */}
      <div className="flex items-center gap-4 border border-ck-hairline bg-ck-bg-1 px-3 py-1.5 font-mono text-[11px] text-ck-fg-mute overflow-x-auto">
        <span className="text-[10px] uppercase font-bold text-ck-fg-1">
          Forms:
        </span>
        <span className="inline-flex items-center gap-1">
          <svg width="18" height="14" viewBox="-13 -10 26 20">
            <circle
              cx="0"
              cy="0"
              r="6.5"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.2"
            />
            <circle
              cx="0"
              cy="0"
              r="4"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.2"
            />
          </svg>
          equal-to
        </span>
        <span className="inline-flex items-center gap-1">
          <svg width="18" height="14" viewBox="-13 -10 26 20">
            <circle
              cx="-1.5"
              cy="0"
              r="6"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.2"
            />
            <circle
              cx="1.5"
              cy="0"
              r="6"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.2"
            />
          </svg>
          equivalent-to
        </span>
        <span className="inline-flex items-center gap-1">
          <svg width="18" height="14" viewBox="-13 -10 26 20">
            <circle
              cx="1.5"
              cy="0"
              r="7.5"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.2"
            />
            <circle
              cx="-1.5"
              cy="0"
              r="3.2"
              fill="currentColor"
              stroke="currentColor"
              strokeWidth="1.2"
            />
          </svg>
          subset-of
        </span>
        <span className="inline-flex items-center gap-1">
          <svg width="18" height="14" viewBox="-13 -10 26 20">
            <circle
              cx="-1.5"
              cy="0"
              r="7.5"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.2"
            />
            <circle
              cx="1.5"
              cy="0"
              r="3.2"
              fill="currentColor"
              stroke="currentColor"
              strokeWidth="1.2"
            />
          </svg>
          superset-of
        </span>
        <span className="inline-flex items-center gap-1">
          <svg width="18" height="14" viewBox="-13 -10 26 20">
            <circle
              cx="-3.5"
              cy="0"
              r="5.5"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.2"
            />
            <circle
              cx="3.5"
              cy="0"
              r="5.5"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.2"
            />
          </svg>
          intersects-with
        </span>
      </div>

      {/* Main Bridge View */}
      {viewMode === "diagram" ? (
        <div className="border border-ck-hairline-strong bg-ck-bg-0 p-4 overflow-x-auto shadow-sm">
          <div className="grid grid-cols-[1fr_260px_1fr] items-start min-w-[760px] gap-2">
            {/* Left: Meridian Internal Standard */}
            <div className="space-y-4 font-mono text-xs">
              <span className="text-[10px] uppercase text-ck-fg-mute block border-b border-ck-hairline pb-1">
                Internal Practice · urn:meridian:catalog:mer
              </span>
              {mer.map((m, idx) => (
                <div
                  key={m.id}
                  className="h-14 border border-ck-hairline-strong bg-ck-bg-1 p-2 flex flex-col justify-center shadow-sm"
                >
                  <span className="font-bold text-ck-fg-1">{m.id}</span>
                  <span className="text-ck-fg-2 truncate font-sans text-[11px]">
                    {m.t}
                  </span>
                </div>
              ))}
            </div>

            {/* Middle: SVG Mapping Curves & Relation Plates */}
            <div className="relative pt-6">
              <svg
                width="260"
                height={bridgeHeight}
                viewBox={`0 0 260 ${bridgeHeight}`}
                className="block"
              >
                {activeEdges.map((e) => {
                  const y1 = e.l * 72 + 36;
                  const y2 = e.r * 72 + 36;
                  const isSel = selectedEdgeId === e.id;
                  const d = `M 0 ${y1} C 130 ${y1}, 130 ${y2}, 260 ${y2}`;
                  const glyph = getRelationshipGlyph(e.rel);

                  return (
                    <g
                      key={e.id}
                      onClick={() => setSelectedEdgeId(e.id)}
                      className="cursor-pointer"
                    >
                      <path
                        d={d}
                        fill="none"
                        stroke={isSel ? "var(--ck-accent)" : "var(--ck-fg-1)"}
                        strokeWidth={Math.max(1.2, e.cov * 3)}
                        strokeDasharray={
                          e.rat === "syntactic"
                            ? "3 3"
                            : e.rat === "semantic"
                              ? "5 4"
                              : "0"
                        }
                        opacity={e.conf}
                      />
                      {/* Relationship Plate */}
                      <g transform={`translate(130, ${(y1 + y2) / 2})`}>
                        <rect
                          x="-58"
                          y="-16"
                          width="116"
                          height="32"
                          fill="var(--ck-bg-1)"
                          stroke={
                            isSel
                              ? "var(--ck-accent)"
                              : "var(--ck-hairline-strong)"
                          }
                          strokeWidth={isSel ? 2 : 1}
                        />
                        <circle
                          cx={glyph.c1x - 34}
                          cy="0"
                          r={glyph.c1r}
                          fill={glyph.c1f}
                          stroke="var(--ck-fg-1)"
                          strokeWidth="1.2"
                        />
                        <circle
                          cx={glyph.c2x - 34}
                          cy="0"
                          r={glyph.c2r}
                          fill={glyph.c2f}
                          stroke="var(--ck-fg-1)"
                          strokeWidth="1.2"
                        />
                        <text
                          x="-14"
                          y="4"
                          fill="var(--ck-fg-1)"
                          fontFamily="monospace"
                          fontSize="9.5"
                          fontWeight="bold"
                        >
                          {e.rel}
                        </text>
                      </g>
                    </g>
                  );
                })}
              </svg>
            </div>

            {/* Right: Target Framework Controls */}
            <div className="space-y-4 font-mono text-xs">
              <span className="text-[10px] uppercase text-ck-fg-mute block border-b border-ck-hairline pb-1">
                {targetFw === "iso"
                  ? "ISO/IEC 27001:2022 Controls"
                  : "NIST CSF 2.0 Subcategories"}
              </span>
              {rightRows.map((r, idx) => (
                <div
                  key={r.id}
                  className="h-14 border border-ck-hairline-strong bg-ck-bg-1 p-2 flex flex-col justify-center shadow-sm"
                >
                  <span className="font-bold text-ck-fg-1">{r.id}</span>
                  <span className="text-ck-fg-2 truncate font-sans text-[11px]">
                    {r.t}
                  </span>
                </div>
              ))}
            </div>
          </div>
        </div>
      ) : (
        /* Mapping Ledger Table */
        <div className="border border-ck-hairline-strong bg-ck-bg-1 overflow-x-auto shadow-sm font-mono text-xs">
          <table className="w-full text-left border-collapse">
            <thead className="bg-ck-bg-2 border-b border-ck-hairline text-ck-fg-mute uppercase text-[10px]">
              <tr>
                <th className="p-2">Internal Practice</th>
                <th className="p-2">Relationship</th>
                <th className="p-2">Framework Target</th>
                <th className="p-2">Rationale</th>
                <th className="p-2 text-right">Confidence</th>
                <th className="p-2 text-right">Coverage</th>
                <th className="p-2">Method</th>
                <th className="p-2">Status</th>
              </tr>
            </thead>
            <tbody>
              {activeEdges.map((e) => {
                const fromRow = mer[e.l] || { id: "MER-?" };
                const toRow = rightRows[e.r] || { id: "TARGET-?" };
                const isSel = selectedEdgeId === e.id;
                return (
                  <tr
                    key={e.id}
                    onClick={() => setSelectedEdgeId(e.id)}
                    className={`border-b border-ck-hairline cursor-pointer hover:bg-ck-bg-2 ${
                      isSel ? "bg-ck-bg-2 font-semibold" : ""
                    }`}
                  >
                    <td className="p-2 font-bold">{fromRow.id}</td>
                    <td className="p-2">{e.rel}</td>
                    <td className="p-2 font-bold">{toRow.id}</td>
                    <td className="p-2 text-ck-fg-mute">{e.rat}</td>
                    <td className="p-2 text-right">
                      {Math.round(e.conf * 100)}%
                    </td>
                    <td className="p-2 text-right">
                      {Math.round(e.cov * 100)}%
                    </td>
                    <td className="p-2">{e.m}</td>
                    <td className="p-2">
                      <Badge
                        variant="outline"
                        className="text-[10px] uppercase font-mono"
                      >
                        {e.st}
                      </Badge>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}

      {/* Selected Edge Provenance / Dispute Inspector */}
      {selectedEdge && (
        <div className="border border-ck-hairline-strong bg-ck-bg-1 p-4 shadow-sm font-mono text-xs space-y-3">
          <div className="flex items-center justify-between border-b border-ck-hairline pb-2">
            <div className="flex items-center gap-2">
              <span className="text-sm font-bold text-ck-fg-1">
                {mer[selectedEdge.l]?.id} → {rightRows[selectedEdge.r]?.id}
              </span>
              <Badge
                variant="outline"
                className="text-[10px] font-mono uppercase"
              >
                {selectedEdge.rel}
              </Badge>
            </div>
            <Button
              size="sm"
              variant="outline"
              onClick={() => setSelectedEdgeId(null)}
              className="h-6 text-[10px] font-mono"
            >
              Close
            </Button>
          </div>

          <div className="grid grid-cols-4 gap-3 text-[11px]">
            <div className="border border-ck-hairline bg-ck-bg-0 p-2">
              <span className="text-ck-fg-mute block text-[10px]">
                ASSERTION ORIGIN
              </span>
              <span className="font-semibold text-ck-fg-1">
                {selectedEdge.by}
              </span>
            </div>
            <div className="border border-ck-hairline bg-ck-bg-0 p-2">
              <span className="text-ck-fg-mute block text-[10px]">
                CONFIDENCE / COVERAGE
              </span>
              <span className="font-semibold text-ck-fg-1">
                {Math.round(selectedEdge.conf * 100)}% conf ·{" "}
                {Math.round(selectedEdge.cov * 100)}% cov
              </span>
            </div>
            <div className="border border-ck-hairline bg-ck-bg-0 p-2">
              <span className="text-ck-fg-mute block text-[10px]">
                CRYPTOGRAPHIC EVIDENCE
              </span>
              <span className="font-semibold text-ck-fg-1">
                {selectedEdge.sealed ? selectedEdge.hash : "○ NO PROOF OBJECT"}
              </span>
            </div>
            <div className="border border-ck-hairline bg-ck-bg-0 p-2">
              <span className="text-ck-fg-mute block text-[10px]">STATUS</span>
              <span className="font-semibold text-ck-fg-1">
                {selectedEdge.st.toUpperCase()}
              </span>
            </div>
          </div>

          {selectedEdge.m === "automation" && onEdgeConfirmHuman && (
            <div className="flex items-center justify-between border-t border-ck-hairline pt-2 bg-ck-bg-2 p-2">
              <span className="text-ck-fg-2 font-sans text-xs">
                ⇝ Suggested by AI automation — requires human verification
                before leaving draft.
              </span>
              <Button
                size="sm"
                variant="default"
                onClick={() => onEdgeConfirmHuman(selectedEdge.id)}
                className="h-7 text-xs font-mono"
              >
                Confirm as Human Assertion
              </Button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
