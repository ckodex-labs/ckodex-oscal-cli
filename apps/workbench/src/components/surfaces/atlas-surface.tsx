"use client";

import * as React from "react";
import { AtlasControl, RegionFamily } from "@/lib/atlas-data";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

interface AtlasSurfaceProps {
  fams: RegionFamily[];
  ctrls: AtlasControl[];
  selectedId: string | null;
  onSelectControl: (id: string) => void;
  overlay: "state" | "freshness" | "drift" | "poam";
  onOverlayChange: (ov: "state" | "freshness" | "drift" | "poam") => void;
  semanticZoom: "posture" | "controls";
  onSemanticZoomChange: (z: "posture" | "controls") => void;
  isOutline: boolean;
  onToggleOutline: () => void;
  onReplayPulse: () => void;
  pulseActive: boolean;
  onOpenInComposer?: (id: string) => void;
}

export function AtlasSurface({
  fams,
  ctrls,
  selectedId,
  onSelectControl,
  overlay,
  onOverlayChange,
  semanticZoom,
  onSemanticZoomChange,
  isOutline,
  onToggleOutline,
  onReplayPulse,
  pulseActive,
  onOpenInComposer,
}: AtlasSurfaceProps) {
  const [zoom, setZoom] = React.useState(1);
  const [pan, setPan] = React.useState({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = React.useState(false);
  const [dragStart, setDragStart] = React.useState({ x: 0, y: 0 });
  const [focusedFam, setFocusedFam] = React.useState<string | null>(null);

  const staleCount = ctrls.filter((c) => c.days > 180).length;
  const driftCount = ctrls.filter((c) => c.drift).length;
  const poamCount = ctrls.filter((c) => c.poam).length;

  const handleMouseDown = (e: React.MouseEvent) => {
    setIsDragging(true);
    setDragStart({ x: e.clientX - pan.x, y: e.clientY - pan.y });
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (!isDragging) return;
    setPan({ x: e.clientX - dragStart.x, y: e.clientY - dragStart.y });
  };

  const handleMouseUp = () => {
    setIsDragging(false);
  };

  const handleFit = () => {
    setZoom(1);
    setPan({ x: 0, y: 0 });
    setFocusedFam(null);
  };

  // Corridors math
  const fc: Record<string, RegionFamily> = {};
  fams.forEach((f) => (fc[f.id] = f));

  const edgePt = (f: RegionFamily, tx2: number, ty2: number) => {
    const cx = f.cx || f.x + (f.w || 100) / 2;
    const cy = f.cy || f.y + (f.h || 100) / 2;
    const dx = tx2 - cx;
    const dy = ty2 - cy;
    const len = Math.hypot(dx, dy) || 1;
    const w = f.w || 120;
    const h = f.h || 120;
    const t = Math.min(
      dx !== 0 ? w / 2 / Math.abs(dx) : Infinity,
      dy !== 0 ? h / 2 / Math.abs(dy) : Infinity,
    );
    const kVal = Math.min(1, t + 8 / len);
    return [cx + dx * kVal, cy + dy * kVal];
  };

  const corridorsList: [string, string][] = [
    ["AC", "AU"],
    ["AC", "IA"],
    ["AC", "SC"],
    ["SC", "SI"],
    ["SC", "IR"],
    ["AC", "CM"],
    ["CM", "CP"],
    ["CM", "CA"],
    ["SI", "RA"],
    ["AU", "SI"],
  ];

  const renderedCorridors = corridorsList.map(([c1, c2]) => {
    const A = fc[c1];
    const B = fc[c2];
    if (!A || !B) return "";
    const a = edgePt(A, B.cx || B.x + 50, B.cy || B.y + 50);
    const b = edgePt(B, A.cx || A.x + 50, A.cy || A.y + 50);
    return `M ${a[0].toFixed(1)} ${a[1].toFixed(1)} L ${b[0].toFixed(1)} ${b[1].toFixed(1)}`;
  });

  const selectedControlObj = ctrls.find((c) => c.id === selectedId) || null;

  return (
    <div className="space-y-4">
      {/* Header Banner */}
      <div className="flex flex-wrap items-baseline justify-between border-b border-ck-hairline pb-2 gap-2">
        <div className="flex items-baseline gap-3 min-w-0">
          <h1 className="font-serif text-2xl font-normal tracking-tight text-ck-fg-1 whitespace-nowrap shrink-0">
            The Atlas
          </h1>
          <span className="text-xs text-ck-fg-mute font-mono truncate hidden md:inline">
            One territory, every claim traceable. Positions are stable.
          </span>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          <Badge
            variant="outline"
            className="font-mono text-[10px] uppercase text-green-700 dark:text-green-400 whitespace-nowrap shrink-0"
          >
            NIST SP 800-53 r5 Tailored · MER-MOD
          </Badge>
        </div>
      </div>

      {/* Facet Toolbar */}
      <div className="flex items-center justify-between border border-ck-hairline bg-ck-bg-1 p-2 shadow-sm font-mono text-xs">
        {/* Overlay Tabs */}
        <div className="flex items-center gap-1.5">
          <Button
            size="sm"
            variant={overlay === "state" ? "default" : "outline"}
            onClick={() => onOverlayChange("state")}
            className="h-7 text-xs font-mono"
          >
            State ({ctrls.length})
          </Button>
          <Button
            size="sm"
            variant={overlay === "freshness" ? "default" : "outline"}
            onClick={() => onOverlayChange("freshness")}
            className="h-7 text-xs font-mono"
          >
            Freshness ({staleCount})
          </Button>
          <Button
            size="sm"
            variant={overlay === "drift" ? "default" : "outline"}
            onClick={() => onOverlayChange("drift")}
            className="h-7 text-xs font-mono"
          >
            Drift ({driftCount})
          </Button>
          <Button
            size="sm"
            variant={overlay === "poam" ? "default" : "outline"}
            onClick={() => onOverlayChange("poam")}
            className="h-7 text-xs font-mono"
          >
            POA&amp;M ({poamCount})
          </Button>
        </div>

        {/* Zoom & Action Controls */}
        <div className="flex items-center gap-2">
          <div className="flex items-center border border-ck-hairline bg-ck-bg-0 p-0.5">
            <button
              onClick={() => onSemanticZoomChange("posture")}
              className={`px-2 py-1 text-[11px] font-mono transition-colors ${
                semanticZoom === "posture"
                  ? "bg-ck-fg-1 text-ck-bg-0"
                  : "text-ck-fg-mute"
              }`}
            >
              posture
            </button>
            <button
              onClick={() => onSemanticZoomChange("controls")}
              className={`px-2 py-1 text-[11px] font-mono transition-colors ${
                semanticZoom === "controls"
                  ? "bg-ck-fg-1 text-ck-bg-0"
                  : "text-ck-fg-mute"
              }`}
            >
              controls
            </button>
          </div>
          <Button
            size="sm"
            variant="outline"
            onClick={onToggleOutline}
            className="h-7 text-xs font-mono"
          >
            {isOutline ? "Map View" : "Outline View"}
          </Button>
          <Button
            size="sm"
            variant={pulseActive ? "default" : "outline"}
            onClick={onReplayPulse}
            className={`h-7 text-xs font-mono transition-all ${
              pulseActive
                ? "bg-ck-accent text-white border-ck-accent shadow-xs"
                : "hover:bg-ck-bg-2"
            }`}
          >
            {pulseActive ? "Radiating Pulse..." : "Replay Change Pulse"}
          </Button>
        </div>
      </div>

      {/* Posture Bento Metric Bar */}
      {semanticZoom === "posture" && (
        <div className="grid grid-cols-5 gap-2 font-mono">
          <div className="border border-ck-hairline bg-ck-bg-1 p-2.5 shadow-sm">
            <span className="text-[10px] text-ck-fg-mute uppercase block">
              Coverage · Computed
            </span>
            <span className="font-serif text-xl font-normal text-ck-fg-1">
              71.4%
            </span>
            <span className="text-[10px] text-ck-fg-mute block mt-0.5">
              68 of 85 controls
            </span>
          </div>

          <div className="border border-ck-hairline bg-ck-bg-1 p-2.5 shadow-sm">
            <span className="text-[10px] text-ck-fg-mute uppercase block">
              Coverage · 12 Wk Trend
            </span>
            <svg
              width="100%"
              height="24"
              viewBox="0 0 120 24"
              preserveAspectRatio="none"
              className="my-1"
            >
              <polyline
                points="0,20 11,19 22,17 33,18 44,14 55,12 66,13 77,9 88,7 99,8 110,4 120,6"
                fill="none"
                stroke="var(--ck-fg-1)"
                strokeWidth="1.5"
              />
            </svg>
            <span className="text-[10px] text-green-700 dark:text-green-400 block font-semibold">
              Δ +4.1 since June
            </span>
          </div>

          <div className="border border-ck-hairline bg-ck-bg-1 p-2.5 shadow-sm">
            <span className="text-[10px] text-ck-fg-mute uppercase block">
              Stale · &gt;180 Days
            </span>
            <span className="font-serif text-xl font-normal text-ck-fg-1">
              {staleCount}
            </span>
            <span className="text-[10px] text-ck-fg-mute block mt-0.5">
              Oldest 470 d
            </span>
          </div>

          <div className="border border-ck-hairline bg-ck-bg-1 p-2.5 shadow-sm">
            <span className="text-[10px] text-ck-fg-mute uppercase block">
              Open POA&amp;M
            </span>
            <span className="font-serif text-xl font-normal text-ck-fg-1">
              {poamCount}
            </span>
            <span className="text-[10px] text-accent block mt-0.5 font-semibold">
              1 Past Deadline
            </span>
          </div>

          <div className="border border-ck-hairline bg-ck-bg-1 p-2.5 shadow-sm">
            <span className="text-[10px] text-ck-fg-mute uppercase block">
              ⊭ Blocked Mappings
            </span>
            <span className="font-serif text-xl font-normal text-ck-fg-1">
              1
            </span>
            <span className="text-[10px] text-ck-fg-mute block mt-0.5">
              has-incompatibility
            </span>
          </div>
        </div>
      )}

      {/* Main Canvas / Map Fabric */}
      {!isOutline ? (
        <div className="relative border border-ck-hairline-strong bg-ck-bg-0 h-[560px] overflow-hidden shadow-[2px_2px_0_var(--ck-fg-1)]">
          <svg
            viewBox="0 0 1180 720"
            preserveAspectRatio="xMidYMid meet"
            onMouseDown={handleMouseDown}
            onMouseMove={handleMouseMove}
            onMouseUp={handleMouseUp}
            className="w-full h-full cursor-grab active:cursor-grabbing block"
          >
            <rect width="1180" height="720" fill="transparent" />

            <g
              transform={`translate(${pan.x}, ${pan.y}) scale(${zoom})`}
              className="transition-transform duration-300 ease-out"
            >
              {/* Corridors */}
              {renderedCorridors.map((dStr, idx) => (
                <path
                  key={idx}
                  d={dStr}
                  stroke={
                    pulseActive
                      ? "var(--ck-accent)"
                      : "var(--ck-hairline-strong)"
                  }
                  strokeWidth={pulseActive ? 2 : 1}
                  fill="none"
                  opacity={pulseActive ? 0.95 : 0.6}
                  className={pulseActive ? "animate-corridor-flow" : ""}
                />
              ))}

              {/* Radiating Pulse Waves when pulseActive is true */}
              {pulseActive && fc["AC"] && (
                <g className="pointer-events-none">
                  <circle
                    cx={fc["AC"].cx || fc["AC"].x + 60}
                    cy={fc["AC"].cy || fc["AC"].y + 60}
                    r="80"
                    fill="none"
                    stroke="var(--ck-accent)"
                    strokeWidth="2"
                    className="animate-ping"
                  />
                  <circle
                    cx={fc["AC"].cx || fc["AC"].x + 60}
                    cy={fc["AC"].cy || fc["AC"].y + 60}
                    r="150"
                    fill="none"
                    stroke="var(--ck-accent)"
                    strokeWidth="1.5"
                    strokeDasharray="4 4"
                    className="animate-pulse"
                  />
                </g>
              )}

              {/* Family Regions */}
              {fams.map((f) => {
                const covPct = Math.round((f.cov || 0) * 100);
                const isFocus = focusedFam === f.id;
                return (
                  <g
                    key={f.id}
                    onClick={() => setFocusedFam(isFocus ? null : f.id)}
                    className="cursor-pointer"
                  >
                    <rect
                      x={f.x}
                      y={f.y}
                      width={f.w || 120}
                      height={f.h || 120}
                      fill="var(--ck-fg-1)"
                      fillOpacity={isFocus ? 0.12 : 0.04}
                      stroke={
                        isFocus ? "var(--ck-fg-1)" : "var(--ck-hairline-strong)"
                      }
                      strokeWidth="1"
                    />
                    <foreignObject
                      x={f.x}
                      y={f.y - 20}
                      width="240"
                      height="16"
                      className="overflow-visible pointer-events-none"
                    >
                      <div className="font-mono text-[11px] font-semibold tracking-wider text-ck-fg-2 uppercase">
                        {f.id} · {f.short}
                      </div>
                    </foreignObject>

                    {semanticZoom === "posture" && (
                      <foreignObject
                        x={(f.cx || f.x + 50) - 50}
                        y={(f.cy || f.y + 50) - 20}
                        width="100"
                        height="40"
                        className="overflow-visible pointer-events-none"
                      >
                        <div className="font-serif text-2xl text-ck-fg-1 text-center">
                          {covPct}%
                        </div>
                        <div className="font-mono text-[10px] text-ck-fg-mute text-center">
                          {f.n} controls
                        </div>
                      </foreignObject>
                    )}
                  </g>
                );
              })}

              {/* Control Nodes */}
              {semanticZoom === "controls" &&
                ctrls.map((c) => {
                  const isSelected = selectedId === c.id;
                  const isStale = c.days > 180;
                  const half = (9 + Math.min(c.deps, 12) * 0.65) / 2;

                  let fillOp = 0.9;
                  let stroke = "var(--ck-fg-1)";
                  if (c.st === "partial") fillOp = 0.4;
                  if (c.st === "planned") fillOp = 0.1;
                  if (c.st === "unassessed") fillOp = 0.0;
                  if (c.st === "na") fillOp = 0.2;

                  if (overlay === "freshness") {
                    const weight = Math.max(0, 1 - c.days / 470);
                    fillOp = weight;
                  }

                  if (overlay === "drift" && !c.drift) fillOp = 0.1;
                  if (overlay === "poam" && !c.poam) fillOp = 0.1;

                  return (
                    <g
                      key={c.id}
                      onClick={(e) => {
                        e.stopPropagation();
                        onSelectControl(c.id);
                      }}
                      className="cursor-pointer"
                    >
                      <title>{`${c.id} · ${c.title} · ${c.st} (${c.days}d ago)`}</title>
                      <rect
                        x={c.x - half}
                        y={c.y - half}
                        width={half * 2}
                        height={half * 2}
                        fill="var(--ck-fg-1)"
                        fillOpacity={fillOp}
                        stroke={isSelected ? "var(--ck-accent)" : stroke}
                        strokeWidth={isSelected ? 2 : 1}
                      />

                      {/* Selection Ring */}
                      {isSelected && (
                        <circle
                          cx={c.x}
                          cy={c.y}
                          r={half + 6}
                          fill="none"
                          stroke="var(--ck-accent)"
                          strokeWidth="1.8"
                        />
                      )}

                      {/* Stale Ring */}
                      {isStale && overlay === "freshness" && (
                        <circle
                          cx={c.x}
                          cy={c.y}
                          r={half + 4}
                          fill="none"
                          stroke="var(--ck-fg-mute)"
                          strokeWidth="1"
                          strokeDasharray="2 2"
                        />
                      )}

                      {/* Pulse Animation */}
                      {pulseActive && (
                        <circle
                          cx={c.x}
                          cy={c.y}
                          r={half + 10}
                          fill="none"
                          stroke="var(--ck-accent)"
                          strokeWidth="1.4"
                          className="animate-ping"
                        />
                      )}
                    </g>
                  );
                })}
            </g>
          </svg>

          {/* Active Radiating Pulse Notification Banner */}
          {pulseActive && (
            <div className="absolute top-3 left-3 bg-ck-bg-1 border border-ck-accent px-3 py-1.5 shadow-md flex items-center gap-2.5 font-mono text-xs animate-in fade-in duration-150 z-10">
              <span className="w-2 h-2 rounded-full bg-ck-accent animate-ping" />
              <span className="font-bold text-ck-accent text-[11px]">
                RADIATING CHANGE PULSE
              </span>
              <span className="text-ck-fg-2 text-[11px]">
                Profile MER-MOD parameter update radiating across 10 control
                corridors…
              </span>
            </div>
          )}

          {/* Map Controls */}
          <div className="absolute top-3 right-3 flex flex-col gap-1 font-mono">
            <button
              onClick={() => setZoom((z) => Math.min(3, z + 0.3))}
              className="w-7 h-7 bg-ck-bg-1 border border-ck-hairline-strong text-ck-fg-1 flex items-center justify-center text-sm font-semibold hover:bg-ck-bg-2 shadow-sm"
            >
              +
            </button>
            <button
              onClick={() => setZoom((z) => Math.max(0.6, z - 0.3))}
              className="w-7 h-7 bg-ck-bg-1 border border-ck-hairline-strong text-ck-fg-1 flex items-center justify-center text-sm font-semibold hover:bg-ck-bg-2 shadow-sm"
            >
              −
            </button>
            <button
              onClick={handleFit}
              className="w-7 h-7 bg-ck-bg-1 border border-ck-hairline-strong text-ck-fg-1 flex items-center justify-center text-[10px] font-medium hover:bg-ck-bg-2 shadow-sm uppercase"
            >
              fit
            </button>
            <span className="text-[10px] text-ck-fg-mute text-center block mt-1">
              {Math.round(zoom * 100)}%
            </span>
          </div>
        </div>
      ) : (
        /* Structured Outline Table */
        <div className="border border-ck-hairline-strong bg-ck-bg-1 max-h-[560px] overflow-y-auto font-mono text-xs shadow-sm">
          <table className="w-full text-left border-collapse">
            <thead className="bg-ck-bg-2 border-b border-ck-hairline text-ck-fg-mute uppercase text-[10px]">
              <tr>
                <th className="p-2">Control ID</th>
                <th className="p-2">Family</th>
                <th className="p-2">Title</th>
                <th className="p-2">Status</th>
                <th className="p-2">Assessed</th>
                <th className="p-2">Dependencies</th>
              </tr>
            </thead>
            <tbody>
              {ctrls.map((c) => (
                <tr
                  key={c.id}
                  onClick={() => {
                    onSelectControl(c.id);
                    onToggleOutline();
                  }}
                  className={`border-b border-ck-hairline cursor-pointer hover:bg-ck-bg-2 ${
                    selectedId === c.id ? "bg-ck-bg-2 font-semibold" : ""
                  }`}
                >
                  <td className="p-2 font-semibold text-ck-fg-1">{c.id}</td>
                  <td className="p-2 text-ck-fg-mute">{c.fam}</td>
                  <td className="p-2 text-ck-fg-1 font-sans">
                    {c.title || "—"}
                  </td>
                  <td className="p-2">
                    <Badge
                      variant="outline"
                      className="text-[10px] uppercase font-mono"
                    >
                      {c.st}
                    </Badge>
                  </td>
                  <td className="p-2 text-ck-fg-mute">{c.days} d ago</td>
                  <td className="p-2 text-ck-fg-1">{c.deps}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Control Inspector Drawer / Summary */}
      {selectedControlObj && (
        <div className="border border-ck-hairline-strong bg-ck-bg-1 p-4 shadow-sm font-mono text-xs space-y-3">
          <div className="flex items-center justify-between border-b border-ck-hairline pb-2">
            <div className="flex items-center gap-2">
              <span className="text-base font-bold text-ck-fg-1">
                {selectedControlObj.id}
              </span>
              <span className="font-serif text-lg text-ck-fg-1">
                {selectedControlObj.title}
              </span>
              <Badge
                variant="outline"
                className="text-[10px] font-mono uppercase"
              >
                {selectedControlObj.st}
              </Badge>
            </div>
            <div className="flex items-center gap-2">
              {onOpenInComposer && (
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => onOpenInComposer(selectedControlObj.id)}
                  className="h-7 text-xs font-mono"
                >
                  Open in Composer →
                </Button>
              )}
            </div>
          </div>

          <div className="grid grid-cols-4 gap-3 text-[11px]">
            <div className="border border-ck-hairline bg-ck-bg-0 p-2">
              <span className="text-ck-fg-mute block text-[10px]">
                EVIDENTIARY FRESHNESS
              </span>
              <span className="font-semibold text-ck-fg-1">
                {selectedControlObj.days} days ago
              </span>
            </div>
            <div className="border border-ck-hairline bg-ck-bg-0 p-2">
              <span className="text-ck-fg-mute block text-[10px]">
                DOWNSTREAM DEPENDENTS
              </span>
              <span className="font-semibold text-ck-fg-1">
                {selectedControlObj.deps} implementations
              </span>
            </div>
            <div className="border border-ck-hairline bg-ck-bg-0 p-2">
              <span className="text-ck-fg-mute block text-[10px]">
                POA&amp;M STATUS
              </span>
              <span className="font-semibold text-ck-fg-1">
                {selectedControlObj.poam ? "OPEN REMEDIATION" : "NO POA&M"}
              </span>
            </div>
            <div className="border border-ck-hairline bg-ck-bg-0 p-2">
              <span className="text-ck-fg-mute block text-[10px]">
                CRYPTOGRAPHIC ATTESTATION
              </span>
              <span className="font-semibold text-green-700 dark:text-green-400">
                {selectedControlObj.att ? "◆ ATTESTED" : "○ CLAIMED"}
              </span>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
