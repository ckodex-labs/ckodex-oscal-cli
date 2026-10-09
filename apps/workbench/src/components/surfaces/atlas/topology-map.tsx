"use client";

import * as React from "react";
import type { AtlasControl, AtlasFamily } from "@/lib/atlas-types";
import type { SspImplementedRequirement } from "@/lib/atlas-types";
import type { LensMode } from "@/lib/oscal-types";
import {
  displayId,
  implState,
  statusTone,
  type AtlasIndex,
} from "./model";

export interface TopologyMapProps {
  index: AtlasIndex;
  byControl: Map<string, SspImplementedRequirement>;
  selectedId: string;
  onSelectControl: (id: string) => void;
  hoverId: string | null;
  onHoverControl: (id: string | null) => void;
  query: string;
  pulseActive: boolean;
  onTriggerPulse: () => void;
  lens?: LensMode;
}

interface FamilyCoord {
  id: string;
  cx: number;
  cy: number;
  r: number;
  short: string;
}

// 20 NIST SP 800-53 Rev 5 families arranged in domain clusters on 1180x740 canvas
const FAMILY_COORDS: FamilyCoord[] = [
  // Access & Personnel cluster (top-left)
  { id: "pe", cx: 80, cy: 80, r: 58, short: "Physical" },
  { id: "ps", cx: 240, cy: 60, r: 58, short: "Personnel" },
  { id: "ac", cx: 190, cy: 220, r: 80, short: "Access Ctrl" },
  { id: "ia", cx: 70, cy: 350, r: 68, short: "Ident & Auth" },
  { id: "ma", cx: 210, cy: 400, r: 58, short: "Maintenance" },
  { id: "mp", cx: 120, cy: 530, r: 56, short: "Media Prot" },

  // Audit, Assessment & Governance (top-center / right)
  { id: "au", cx: 420, cy: 110, r: 68, short: "Audit" },
  { id: "ca", cx: 600, cy: 90, r: 64, short: "Assessment" },
  { id: "pl", cx: 770, cy: 75, r: 58, short: "Planning" },
  { id: "pm", cx: 930, cy: 75, r: 54, short: "Program Mgmt" },
  { id: "pt", cx: 1070, cy: 95, r: 54, short: "Privacy/PII" },

  // Configuration & Operations (center)
  { id: "cm", cx: 400, cy: 280, r: 76, short: "Config Mgmt" },
  { id: "cp", cx: 580, cy: 250, r: 72, short: "Contingency" },
  { id: "sa", cx: 770, cy: 230, r: 70, short: "Acquisition" },
  { id: "sr", cx: 960, cy: 250, r: 66, short: "Supply Chain" },

  // System Protection & Integrity (center-bottom)
  { id: "sc", cx: 390, cy: 470, r: 80, short: "System & Comms" },
  { id: "si", cx: 610, cy: 430, r: 72, short: "Integrity" },
  { id: "ra", cx: 810, cy: 390, r: 66, short: "Risk Assess" },

  // Incident & Awareness (bottom-center)
  { id: "ir", cx: 540, cy: 620, r: 70, short: "Incident Resp" },
  { id: "at", cx: 720, cy: 590, r: 56, short: "Training" },
];

const FAMILY_COORD_MAP = new Map(FAMILY_COORDS.map((f) => [f.id, f]));

// 18 Authoritative cross-family architectural corridors
const CORRIDORS: [string, string, string][] = [
  ["ac", "ia", "Identity Boundary"],
  ["ac", "au", "Audit Logging"],
  ["ac", "sc", "Boundary Protection"],
  ["ac", "cm", "Privilege Baseline"],
  ["ac", "pe", "Physical Boundary"],
  ["ac", "ps", "Personnel Screening"],
  ["ac", "ma", "Maintenance Access"],
  ["sc", "si", "Channel Integrity"],
  ["sc", "ir", "Egress Filtering"],
  ["si", "ra", "Vulnerability Assessment"],
  ["au", "si", "Anomaly Detection"],
  ["cm", "cp", "Failover Baseline"],
  ["cm", "ca", "Continuous Monitoring"],
  ["cm", "sa", "Secure Architecture"],
  ["ca", "pl", "Security Plan"],
  ["sa", "sr", "Supply Chain Provenance"],
  ["sr", "ra", "Vendor Threat Model"],
  ["ir", "at", "Incident Drills"],
];

function getHexPolygonPoints(cx: number, cy: number, r: number): string {
  const pts: string[] = [];
  for (let i = 0; i < 6; i++) {
    const angle = (Math.PI / 3) * i - Math.PI / 6;
    const x = cx + r * Math.cos(angle);
    const y = cy + r * Math.sin(angle);
    pts.push(`${x.toFixed(1)},${y.toFixed(1)}`);
  }
  return pts.join(" ");
}

interface LaidOutNode {
  control: AtlasControl;
  x: number;
  y: number;
  size: number;
}

function layoutFamilyControls(
  controls: AtlasControl[],
  cx: number,
  cy: number,
  r: number,
): LaidOutNode[] {
  const n = controls.length;
  if (n === 0) return [];
  const cols = n > 30 ? 7 : n > 18 ? 6 : n > 12 ? 5 : n > 6 ? 4 : 3;
  const rows = Math.ceil(n / cols);
  const nodeSize = n > 30 ? 9.5 : n > 20 ? 11 : 12;
  const gap = 3.5;
  const step = nodeSize + gap;
  const totalW = cols * step - gap;
  const totalH = rows * step - gap;
  const startX = cx - totalW / 2 + nodeSize / 2;
  const startY = cy - totalH / 2 + 13 + nodeSize / 2;

  return controls.map((control, i) => {
    const row = Math.floor(i / cols);
    const col = i % cols;
    const itemsInThisRow =
      row === rows - 1 && n % cols !== 0 ? n % cols : cols;
    const rowOffsetX = ((cols - itemsInThisRow) * step) / 2;
    return {
      control,
      x: startX + col * step + rowOffsetX,
      y: startY + row * step,
      size: nodeSize,
    };
  });
}

export function TopologyMap({
  index,
  byControl,
  selectedId,
  onSelectControl,
  hoverId,
  onHoverControl,
  query,
  pulseActive,
  onTriggerPulse,
  lens = "architect",
}: TopologyMapProps) {
  const [zoom, setZoom] = React.useState(1);
  const [pan, setPan] = React.useState({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = React.useState(false);
  const [dragStart, setDragStart] = React.useState({ x: 0, y: 0 });
  const [focusedFam, setFocusedFam] = React.useState<string | null>(null);
  const [semanticZoom, setSemanticZoom] = React.useState<"posture" | "controls">(
    lens === "ciso" ? "posture" : "controls",
  );
  const [overlay, setOverlay] = React.useState<"state" | "params" | "corridors">(
    "state",
  );

  const containerRef = React.useRef<HTMLDivElement>(null);

  const selectedControl = index.byId.get(selectedId);
  const selectedFamilyId = selectedControl ? selectedControl.family : null;

  // Handle drag pan
  const handleMouseDown = (e: React.MouseEvent) => {
    // Only drag on canvas background, not controls
    if ((e.target as HTMLElement).tagName === "svg" || (e.target as HTMLElement).tagName === "rect") {
      setIsDragging(true);
      setDragStart({ x: e.clientX - pan.x, y: e.clientY - pan.y });
    }
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

  // Pre-calculate family controls and layout
  const familyNodes = React.useMemo(() => {
    const map = new Map<string, LaidOutNode[]>();
    for (const fc of FAMILY_COORDS) {
      const controlsInFam = index.baseline.filter((c) => c.family === fc.id);
      map.set(fc.id, layoutFamilyControls(controlsInFam, fc.cx, fc.cy, fc.r));
    }
    return map;
  }, [index.baseline]);

  // Normalized search query for dimming non-matching nodes
  const nq = query.trim().toLowerCase();

  return (
    <div className="space-y-3 font-mono">
      {/* Map Sub-Toolbar */}
      <div className="flex flex-wrap items-center justify-between gap-2 rounded-md border border-ck-hairline bg-ck-bg-1 p-2 text-xs">
        <div className="flex flex-wrap items-center gap-1.5">
          <span className="ck-eyebrow mr-1 hidden sm:inline">Semantic Zoom:</span>
          <div className="inline-flex rounded-md border border-ck-hairline-strong bg-ck-bg-0 p-0.5">
            <button
              type="button"
              onClick={() => setSemanticZoom("posture")}
              className={`rounded px-2.5 py-1 text-2xs transition-colors ${
                semanticZoom === "posture"
                  ? "bg-ck-fg-1 text-ck-bg-0 font-semibold"
                  : "text-ck-fg-mute hover:text-ck-fg-1"
              }`}
            >
              Posture (Families)
            </button>
            <button
              type="button"
              onClick={() => setSemanticZoom("controls")}
              className={`rounded px-2.5 py-1 text-2xs transition-colors ${
                semanticZoom === "controls"
                  ? "bg-ck-fg-1 text-ck-bg-0 font-semibold"
                  : "text-ck-fg-mute hover:text-ck-fg-1"
              }`}
            >
              Controls (Nodes)
            </button>
          </div>

          <span className="ck-eyebrow mx-1 hidden md:inline">Overlay:</span>
          <div className="inline-flex rounded-md border border-ck-hairline-strong bg-ck-bg-0 p-0.5">
            <button
              type="button"
              onClick={() => setOverlay("state")}
              className={`rounded px-2 py-1 text-2xs transition-colors ${
                overlay === "state"
                  ? "bg-ck-fg-1 text-ck-bg-0 font-semibold"
                  : "text-ck-fg-mute hover:text-ck-fg-1"
              }`}
            >
              Implementation
            </button>
            <button
              type="button"
              onClick={() => setOverlay("corridors")}
              className={`rounded px-2 py-1 text-2xs transition-colors ${
                overlay === "corridors"
                  ? "bg-ck-fg-1 text-ck-bg-0 font-semibold"
                  : "text-ck-fg-mute hover:text-ck-fg-1"
              }`}
            >
              Corridors
            </button>
            <button
              type="button"
              onClick={() => setOverlay("params")}
              className={`rounded px-2 py-1 text-2xs transition-colors ${
                overlay === "params"
                  ? "bg-ck-fg-1 text-ck-bg-0 font-semibold"
                  : "text-ck-fg-mute hover:text-ck-fg-1"
              }`}
            >
              Parameters
            </button>
          </div>
        </div>

        <div className="flex items-center gap-1.5">
          <button
            type="button"
            onClick={onTriggerPulse}
            disabled={pulseActive}
            className={`inline-flex items-center gap-1.5 rounded-md border px-2.5 py-1 text-2xs font-semibold transition-all ${
              pulseActive
                ? "border-ck-accent bg-ck-accent text-white shadow-xs"
                : "border-ck-hairline-strong bg-ck-bg-0 text-ck-fg-1 hover:bg-ck-bg-2"
            }`}
          >
            <span
              className={`h-1.5 w-1.5 rounded-full ${
                pulseActive ? "bg-white animate-ping" : "bg-ck-accent"
              }`}
            />
            {pulseActive ? "Radiating Pulse..." : "Replay Change Pulse"}
          </button>

          <button
            type="button"
            onClick={handleFit}
            className="rounded border border-ck-hairline bg-ck-bg-0 px-2 py-1 text-2xs text-ck-fg-2 hover:bg-ck-bg-2"
            title="Reset Pan and Zoom to Fit Territory"
          >
            Fit
          </button>
          <div className="inline-flex rounded border border-ck-hairline bg-ck-bg-0">
            <button
              type="button"
              onClick={() => setZoom((z) => Math.min(2.5, z + 0.2))}
              className="px-2 py-1 text-xs text-ck-fg-1 hover:bg-ck-bg-2"
              title="Zoom In"
            >
              +
            </button>
            <button
              type="button"
              onClick={() => setZoom((z) => Math.max(0.6, z - 0.2))}
              className="border-l border-ck-hairline px-2 py-1 text-xs text-ck-fg-1 hover:bg-ck-bg-2"
              title="Zoom Out"
            >
              -
            </button>
          </div>
        </div>
      </div>

      {/* SVG Canvas Map */}
      <div
        ref={containerRef}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        className="relative h-[480px] md:h-[520px] lg:h-[560px] w-full overflow-hidden rounded-md border border-ck-hairline-strong bg-ck-bg-0 shadow-sm select-none"
      >
        <svg
          viewBox="0 0 1180 740"
          preserveAspectRatio="xMidYMid meet"
          className="h-full w-full cursor-grab active:cursor-grabbing"
        >
          {/* Subtle territory grid pattern background */}
          <defs>
            <pattern
              id="atlas-grid"
              width="40"
              height="40"
              patternUnits="userSpaceOnUse"
            >
              <path
                d="M 40 0 L 0 0 0 40"
                fill="none"
                stroke="var(--ck-hairline)"
                strokeWidth="0.5"
              />
            </pattern>
          </defs>
          <rect width="1180" height="740" fill="url(#atlas-grid)" />

          <g
            transform={`translate(${pan.x}, ${pan.y}) scale(${zoom})`}
            className="transition-transform duration-100 ease-out"
          >
            {/* Dependency Corridors */}
            {CORRIDORS.map(([fam1, fam2, label], idx) => {
              const f1 = FAMILY_COORD_MAP.get(fam1);
              const f2 = FAMILY_COORD_MAP.get(fam2);
              if (!f1 || !f2) return null;

              const dx = f2.cx - f1.cx;
              const dy = f2.cy - f1.cy;
              const dist = Math.hypot(dx, dy) || 1;
              const ux = dx / dist;
              const uy = dy / dist;

              // Clip from hexagon perimeter to hexagon perimeter
              const x1 = f1.cx + f1.r * ux;
              const y1 = f1.cy + f1.r * uy;
              const x2 = f2.cx - f2.r * ux;
              const y2 = f2.cy - f2.r * uy;

              const isConnectedToSelected =
                selectedFamilyId === fam1 || selectedFamilyId === fam2;
              const isConnectedToFocused =
                focusedFam === fam1 || focusedFam === fam2;
              const isHighlighted =
                isConnectedToSelected || isConnectedToFocused || pulseActive;

              return (
                <g key={`corridor-${idx}`}>
                  <title>{`${fam1.toUpperCase()} <-> ${fam2.toUpperCase()}: ${label}`}</title>
                  <line
                    x1={x1}
                    y1={y1}
                    x2={x2}
                    y2={y2}
                    stroke={
                      isHighlighted
                        ? "var(--ck-accent)"
                        : "var(--ck-hairline-strong)"
                    }
                    strokeWidth={isHighlighted ? 2.2 : 1.2}
                    strokeOpacity={isHighlighted ? 0.95 : 0.4}
                    className={pulseActive ? "animate-corridor-flow" : ""}
                  />
                  {(overlay === "corridors" || isHighlighted) && (
                    <text
                      x={(x1 + x2) / 2}
                      y={(y1 + y2) / 2 - 4}
                      textAnchor="middle"
                      className="fill-ck-fg-mute font-mono text-[9px] pointer-events-none"
                    >
                      {label}
                    </text>
                  )}
                </g>
              );
            })}

            {/* Radiating Pulse Waves when active */}
            {pulseActive && selectedFamilyId && FAMILY_COORD_MAP.has(selectedFamilyId) && (
              <g className="pointer-events-none">
                {(() => {
                  const fc = FAMILY_COORD_MAP.get(selectedFamilyId)!;
                  return (
                    <>
                      <circle
                        cx={fc.cx}
                        cy={fc.cy}
                        r={fc.r + 30}
                        fill="none"
                        stroke="var(--ck-accent)"
                        strokeWidth="2"
                        className="animate-ping"
                      />
                      <circle
                        cx={fc.cx}
                        cy={fc.cy}
                        r={fc.r + 70}
                        fill="none"
                        stroke="var(--ck-accent)"
                        strokeWidth="1.5"
                        strokeDasharray="4 4"
                        className="animate-pulse"
                      />
                    </>
                  );
                })()}
              </g>
            )}

            {/* 20 Family Hexagons */}
            {FAMILY_COORDS.map((fc) => {
              const fam = index.familyById.get(fc.id);
              const baselineCount = fam?.in_baseline ?? 0;
              const isSelectedFam = selectedFamilyId === fc.id;
              const isFocused = focusedFam === fc.id;
              const nodes = familyNodes.get(fc.id) ?? [];

              // Calculate family completion percentage
              let implementedInFam = 0;
              for (const n of nodes) {
                const s = implState(n.control.id, byControl);
                if (s.kind === "declared" && s.status === "implemented") {
                  implementedInFam++;
                }
              }
              const covPct =
                nodes.length > 0
                  ? Math.round((implementedInFam / nodes.length) * 100)
                  : 0;

              return (
                <g
                  key={fc.id}
                  onClick={(e) => {
                    e.stopPropagation();
                    setFocusedFam((prev) => (prev === fc.id ? null : fc.id));
                  }}
                  className="cursor-pointer"
                >
                  <title>{`${fc.id.toUpperCase()}: ${fam?.title ?? fc.short} (${baselineCount} baseline controls)`}</title>

                  {/* Hexagon Shape */}
                  <polygon
                    points={getHexPolygonPoints(fc.cx, fc.cy, fc.r)}
                    fill="var(--ck-fg-1)"
                    fillOpacity={isFocused ? 0.09 : isSelectedFam ? 0.06 : 0.025}
                    stroke={
                      isSelectedFam
                        ? "var(--ck-accent)"
                        : isFocused
                          ? "var(--ck-fg-1)"
                          : "var(--ck-hairline-strong)"
                    }
                    strokeWidth={isSelectedFam ? 2 : 1}
                    className="transition-colors duration-150"
                  />

                  {/* Family Header */}
                  <text
                    x={fc.cx}
                    y={fc.cy - fc.r + 18}
                    textAnchor="middle"
                    className="font-mono text-[11px] font-bold uppercase fill-ck-fg-1 pointer-events-none tracking-wider"
                  >
                    {fc.id.toUpperCase()}
                  </text>
                  <text
                    x={fc.cx}
                    y={fc.cy - fc.r + 29}
                    textAnchor="middle"
                    className="font-mono text-[8.5px] fill-ck-fg-mute pointer-events-none"
                  >
                    {fc.short}
                  </text>

                  {/* Posture Mode Display */}
                  {semanticZoom === "posture" && (
                    <g className="pointer-events-none">
                      <text
                        x={fc.cx}
                        y={fc.cy + 10}
                        textAnchor="middle"
                        className="font-serif text-2xl font-normal fill-ck-fg-1"
                      >
                        {covPct}%
                      </text>
                      <text
                        x={fc.cx}
                        y={fc.cy + 25}
                        textAnchor="middle"
                        className="font-mono text-[9px] fill-ck-fg-mute"
                      >
                        {baselineCount > 0
                          ? `${baselineCount} controls`
                          : "Governance"}
                      </text>
                    </g>
                  )}

                  {/* Controls Mode: Granular Control Honeycomb Nodes */}
                  {semanticZoom === "controls" &&
                    nodes.map((node) => {
                      const c = node.control;
                      const isSelected = selectedId === c.id;
                      const isHovered = hoverId === c.id;
                      const s = implState(c.id, byControl);
                      const half = node.size / 2;

                      // Check query match
                      const matches =
                        !nq ||
                        c.id.includes(nq) ||
                        c.title.toLowerCase().includes(nq);

                      let fill = "var(--ck-bg-2)";
                      let fillOpacity = 0.85;
                      let stroke = "var(--ck-hairline-strong)";

                      if (overlay === "state") {
                        if (s.kind === "declared") {
                          if (s.status === "implemented") {
                            fill = "var(--ck-pos)";
                            stroke = "var(--ck-pos)";
                          } else if (s.status === "partial") {
                            fill = "var(--ck-warn)";
                            stroke = "var(--ck-warn)";
                          } else {
                            fill = "var(--ck-info)";
                            stroke = "var(--ck-info)";
                          }
                        } else if (s.kind === "declared-empty") {
                          fill = "var(--ck-unk)";
                          stroke = "var(--ck-unk)";
                        } else {
                          // undeclared
                          fill = "var(--ck-bg-0)";
                          fillOpacity = 0.4;
                          stroke = "var(--ck-hairline-strong)";
                        }
                      } else if (overlay === "params") {
                        const paramCount = c.params?.length ?? 0;
                        if (paramCount > 5) {
                          fill = "var(--ck-accent)";
                        } else if (paramCount > 0) {
                          fill = "var(--ck-info)";
                        } else {
                          fill = "var(--ck-bg-2)";
                        }
                      }

                      if (!matches) {
                        fillOpacity = 0.12;
                      }

                      return (
                        <g
                          key={c.id}
                          onClick={(e) => {
                            e.stopPropagation();
                            onSelectControl(c.id);
                          }}
                          onMouseEnter={() => onHoverControl(c.id)}
                          onMouseLeave={() => onHoverControl(null)}
                          className="cursor-pointer"
                        >
                          <title>
                            {`${displayId(c.id)} · ${c.title} · ${
                              s.kind === "declared"
                                ? s.status
                                : s.kind === "declared-empty"
                                  ? "declared EMPTY"
                                  : "undeclared in SSP"
                            }`}
                          </title>

                          <rect
                            x={node.x - half}
                            y={node.y - half}
                            width={node.size}
                            height={node.size}
                            rx={1.5}
                            fill={fill}
                            fillOpacity={fillOpacity}
                            stroke={
                              isSelected
                                ? "var(--ck-accent)"
                                : isHovered
                                  ? "var(--ck-fg-1)"
                                  : stroke
                            }
                            strokeWidth={isSelected ? 2 : 1}
                          />

                          {/* Active Selection Glow Ring */}
                          {isSelected && (
                            <circle
                              cx={node.x}
                              cy={node.y}
                              r={half + 5}
                              fill="none"
                              stroke="var(--ck-accent)"
                              strokeWidth="1.8"
                            />
                          )}

                          {/* Hover Focus Ring */}
                          {isHovered && !isSelected && (
                            <circle
                              cx={node.x}
                              cy={node.y}
                              r={half + 3.5}
                              fill="none"
                              stroke="var(--ck-fg-1)"
                              strokeWidth="1.2"
                              strokeDasharray="2 2"
                            />
                          )}
                        </g>
                      );
                    })}
                </g>
              );
            })}
          </g>
        </svg>

        {/* Change Pulse Notification Toast */}
        {pulseActive && (
          <div className="absolute top-3 left-3 flex items-center gap-2 rounded-md border border-ck-accent bg-ck-bg-1 px-3 py-1.5 shadow-md">
            <span className="h-2 w-2 rounded-full bg-ck-accent animate-ping" />
            <span className="font-mono text-xs font-bold text-ck-accent">
              RADIATING CHANGE PULSE
            </span>
            <span className="font-mono text-xs text-ck-fg-2">
              Parameter update propagating outward along cross-family corridors…
            </span>
          </div>
        )}

        {/* Canvas Legend Overlay */}
        <div className="absolute bottom-3 left-3 flex flex-wrap items-center gap-3 rounded-md border border-ck-hairline bg-ck-bg-1/90 px-3 py-1.5 text-2xs backdrop-blur-xs">
          <span className="font-semibold text-ck-fg-1">Status:</span>
          <span className="flex items-center gap-1">
            <span className="h-2 w-2 rounded-xs bg-[var(--ck-pos)]" />
            <span className="text-ck-fg-2">Implemented</span>
          </span>
          <span className="flex items-center gap-1">
            <span className="h-2 w-2 rounded-xs bg-[var(--ck-warn)]" />
            <span className="text-ck-fg-2">Partial</span>
          </span>
          <span className="flex items-center gap-1">
            <span className="h-2 w-2 rounded-xs bg-[var(--ck-info)]" />
            <span className="text-ck-fg-2">Planned</span>
          </span>
          <span className="flex items-center gap-1">
            <span className="h-2 w-2 rounded-xs border border-ck-hairline-strong bg-ck-bg-0" />
            <span className="text-ck-fg-mute">Undeclared</span>
          </span>
        </div>
      </div>
    </div>
  );
}
