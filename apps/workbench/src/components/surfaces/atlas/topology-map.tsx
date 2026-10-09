"use client";

import * as React from "react";
import type { AtlasControl, AtlasFamily } from "@/lib/atlas-types";
import type { SspImplementedRequirement } from "@/lib/atlas-types";
import type { LensMode } from "@/lib/oscal-types";
import { StatementText } from "./statement";
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
  onOpenInComposer?: (id: string) => void;
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

const POAM_CONTROLS = new Set(["ac-3", "sc-7", "ia-2", "ac-17", "si-4"]);

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
  onOpenInComposer,
}: TopologyMapProps) {
  const [zoom, setZoom] = React.useState(1);
  const [pan, setPan] = React.useState({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = React.useState(false);
  const [dragStart, setDragStart] = React.useState({ x: 0, y: 0 });
  const [focusedFam, setFocusedFam] = React.useState<string | null>(null);
  const [dossierOpen, setDossierOpen] = React.useState(false);
  const [semanticZoom, setSemanticZoom] = React.useState<"posture" | "controls">(
    lens === "ciso" ? "posture" : "controls",
  );
  const [overlay, setOverlay] = React.useState<"state" | "params" | "corridors">(
    "state",
  );
  const [hoveredCorridor, setHoveredCorridor] = React.useState<{
    fam1: string;
    fam2: string;
    label: string;
  } | null>(null);

  const containerRef = React.useRef<HTMLDivElement>(null);

  const selectedControl = index.byId.get(selectedId);
  const selectedFamilyId = selectedControl ? selectedControl.family : null;
  const hoveredControl = hoverId ? index.byId.get(hoverId) : null;

  // Non-passive wheel event listener for focal zooming
  React.useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const onWheel = (e: WheelEvent) => {
      e.preventDefault();
      const rect = el.getBoundingClientRect();
      const clientX = e.clientX - rect.left;
      const clientY = e.clientY - rect.top;

      // SVG viewBox coordinates (1180 x 740)
      const svgX = (clientX / rect.width) * 1180;
      const svgY = (clientY / rect.height) * 740;

      const factor = e.deltaY < 0 ? 1.15 : 0.87;
      setZoom((prevZoom) => {
        const nextZoom = Math.min(4.5, Math.max(0.4, Number((prevZoom * factor).toFixed(3))));
        setPan((prevPan) => ({
          x: svgX - ((svgX - prevPan.x) / prevZoom) * nextZoom,
          y: svgY - ((svgY - prevPan.y) / prevZoom) * nextZoom,
        }));
        return nextZoom;
      });
    };

    el.addEventListener("wheel", onWheel, { passive: false });
    return () => el.removeEventListener("wheel", onWheel);
  }, []);

  // Handle pointer-based drag pan with pointer capture
  const handlePointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    // If clicking an interactive UI element or control node, do not initiate canvas drag
    const el = e.target as Element | null;
    if (el?.closest("button, input, select, textarea, [data-control-node]")) {
      return;
    }
    setIsDragging(true);
    setDragStart({ x: e.clientX - pan.x, y: e.clientY - pan.y });
    try {
      (e.currentTarget as Element).setPointerCapture(e.pointerId);
    } catch {}
  };

  const handlePointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!isDragging) return;
    setPan({ x: e.clientX - dragStart.x, y: e.clientY - dragStart.y });
  };

  const handlePointerUp = (e: React.PointerEvent<HTMLDivElement>) => {
    if (isDragging) {
      setIsDragging(false);
      try {
        (e.currentTarget as Element).releasePointerCapture(e.pointerId);
      } catch {}
    }
  };

  const nudgePan = (dx: number, dy: number) => {
    setPan((p) => ({ x: p.x + dx, y: p.y + dy }));
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowLeft") {
      e.preventDefault();
      nudgePan(80, 0);
    } else if (e.key === "ArrowRight") {
      e.preventDefault();
      nudgePan(-80, 0);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      nudgePan(0, 80);
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      nudgePan(0, -80);
    } else if (e.key === "+" || e.key === "=") {
      e.preventDefault();
      setZoom((z) => Math.min(4.5, Number((z + 0.25).toFixed(2))));
    } else if (e.key === "-") {
      e.preventDefault();
      setZoom((z) => Math.max(0.4, Number((z - 0.25).toFixed(2))));
    } else if (e.key === "0") {
      e.preventDefault();
      handleFit();
    }
  };

  const handleFit = () => {
    setZoom(1);
    setPan({ x: 0, y: 0 });
    setFocusedFam(null);
  };

  const handleFamilyDoubleClick = (famId: string) => {
    const fc = FAMILY_COORD_MAP.get(famId);
    if (!fc) return;
    const targetZoom = 2.4;
    const targetPanX = 1180 / 2 - fc.cx * targetZoom;
    const targetPanY = 740 / 2 - fc.cy * targetZoom;
    setZoom(targetZoom);
    setPan({ x: targetPanX, y: targetPanY });
    setFocusedFam(famId);
    setSemanticZoom("controls");
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

        <div className="flex flex-wrap items-center gap-1.5">
          {/* Pan / Move Controls */}
          <span className="ck-eyebrow hidden lg:inline">Move:</span>
          <div className="inline-flex rounded-md border border-ck-hairline-strong bg-ck-bg-0 p-0.5" title="Pan Map Canvas (Arrow Keys or Drag)">
            <button
              type="button"
              onClick={() => nudgePan(80, 0)}
              className="px-1.5 py-0.5 text-2xs text-ck-fg-2 hover:text-ck-fg-1 hover:bg-ck-bg-2 rounded transition-colors"
              title="Pan Left (ArrowLeft)"
            >
              &#9664;
            </button>
            <button
              type="button"
              onClick={() => nudgePan(0, 80)}
              className="px-1.5 py-0.5 text-2xs text-ck-fg-2 hover:text-ck-fg-1 hover:bg-ck-bg-2 rounded transition-colors"
              title="Pan Up (ArrowUp)"
            >
              &#9650;
            </button>
            <button
              type="button"
              onClick={() => nudgePan(0, -80)}
              className="px-1.5 py-0.5 text-2xs text-ck-fg-2 hover:text-ck-fg-1 hover:bg-ck-bg-2 rounded transition-colors"
              title="Pan Down (ArrowDown)"
            >
              &#9660;
            </button>
            <button
              type="button"
              onClick={() => nudgePan(-80, 0)}
              className="px-1.5 py-0.5 text-2xs text-ck-fg-2 hover:text-ck-fg-1 hover:bg-ck-bg-2 rounded transition-colors"
              title="Pan Right (ArrowRight)"
            >
              &#9654;
            </button>
          </div>

          {/* Zoom Level & Steppers */}
          <span className="ck-eyebrow hidden lg:inline">Zoom:</span>
          <div className="inline-flex items-center rounded-md border border-ck-hairline-strong bg-ck-bg-0 p-0.5">
            <button
              type="button"
              onClick={() => setZoom((z) => Math.max(0.4, Number((z - 0.25).toFixed(2))))}
              className="px-2 py-0.5 text-xs text-ck-fg-1 hover:bg-ck-bg-2 rounded transition-colors"
              title="Zoom Out (-)"
            >
              -
            </button>
            <button
              type="button"
              onClick={handleFit}
              className="px-1.5 py-0.5 text-2xs font-mono font-bold text-ck-accent hover:bg-ck-bg-2 rounded transition-colors"
              title="Click to Reset Zoom & Pan (0)"
            >
              {Math.round(zoom * 100)}%
            </button>
            <button
              type="button"
              onClick={() => setZoom((z) => Math.min(4.5, Number((z + 0.25).toFixed(2))))}
              className="px-2 py-0.5 text-xs text-ck-fg-1 hover:bg-ck-bg-2 rounded transition-colors"
              title="Zoom In (+)"
            >
              +
            </button>
          </div>

          <button
            type="button"
            onClick={handleFit}
            className="rounded border border-ck-hairline-strong bg-ck-bg-0 px-2 py-1 text-2xs text-ck-fg-2 hover:bg-ck-bg-2 transition-colors"
            title="Reset Pan and Zoom to Fit Territory"
          >
            Fit
          </button>

          {/* Dossier Detail Toggle */}
          <button
            type="button"
            onClick={() => setDossierOpen((v) => !v)}
            className={`rounded border px-2.5 py-1 text-2xs font-semibold transition-colors ${
              dossierOpen
                ? "border-ck-accent bg-ck-accent text-white"
                : "border-ck-hairline-strong bg-ck-bg-0 text-ck-fg-1 hover:bg-ck-bg-2"
            }`}
            title="Toggle Control Detail Dossier"
          >
            {dossierOpen ? "Close Detail" : "Control Detail"}
          </button>

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
            {pulseActive ? "Pulse..." : "Pulse"}
          </button>
        </div>
      </div>

      {/* SVG Canvas Map */}
      <div
        ref={containerRef}
        tabIndex={0}
        onKeyDown={handleKeyDown}
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onPointerCancel={handlePointerUp}
        className="relative h-[480px] md:h-[540px] lg:h-[600px] w-full overflow-hidden rounded-md border border-ck-hairline-strong bg-ck-bg-0 shadow-sm select-none focus:outline-none focus:ring-1 focus:ring-ck-accent"
      >
        <svg
          viewBox="0 0 1180 740"
          preserveAspectRatio="xMidYMid meet"
          className="h-full w-full cursor-grab active:cursor-grabbing"
        >
          {/* Subtle territory grid pattern background and animations */}
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
            <linearGradient id="radar-beam-gradient" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stopColor="var(--ck-accent)" stopOpacity="0.4" />
              <stop offset="50%" stopColor="var(--ck-accent)" stopOpacity="0.12" />
              <stop offset="100%" stopColor="var(--ck-accent)" stopOpacity="0" />
            </linearGradient>
            <style>{`
              @keyframes corridorDash {
                to { stroke-dashoffset: -20; }
              }
              @keyframes radarSweep {
                from { transform: rotate(0deg); }
                to { transform: rotate(360deg); }
              }
              .corridor-active-flow {
                stroke-dasharray: 5 4;
                animation: corridorDash 0.85s linear infinite;
              }
            `}</style>
          </defs>
          <rect width="1180" height="740" fill="url(#atlas-grid)" />

          {/* GovX Tactical Precision Canvas Framing & Coordinates */}
          <g className="pointer-events-none select-none">
            {/* Outer Border with Inset Reticles */}
            <rect
              x={12}
              y={12}
              width={1156}
              height={716}
              fill="none"
              stroke="var(--ck-hairline-strong)"
              strokeWidth="0.8"
              strokeDasharray="4 8"
              opacity={0.5}
            />

            {/* Corner Crosshairs & Coordinates */}
            {/* Top-Left */}
            <path d="M 16 26 L 28 26 M 22 20 L 22 32" stroke="var(--ck-accent)" strokeWidth="1" opacity={0.8} />
            <text x={34} y={29} className="font-mono text-[8px] fill-ck-fg-mute tracking-wider">[0000, 0000] NW-GRID</text>

            {/* Top-Right */}
            <path d="M 1152 26 L 1164 26 M 1158 20 L 1158 32" stroke="var(--ck-accent)" strokeWidth="1" opacity={0.8} />
            <text x={1146} y={29} textAnchor="end" className="font-mono text-[8px] fill-ck-fg-mute tracking-wider">[1180, 0000] NE-GRID</text>

            {/* Bottom-Left */}
            <path d="M 16 714 L 28 714 M 22 708 L 22 720" stroke="var(--ck-accent)" strokeWidth="1" opacity={0.8} />
            <text x={34} y={717} className="font-mono text-[8px] fill-ck-fg-mute tracking-wider">[0000, 0740] SW-GRID</text>

            {/* Bottom-Right */}
            <path d="M 1152 714 L 1164 714 M 1158 708 L 1158 720" stroke="var(--ck-accent)" strokeWidth="1" opacity={0.8} />
            <text x={1146} y={717} textAnchor="end" className="font-mono text-[8px] fill-ck-fg-mute tracking-wider">[1180, 0740] SE-GRID</text>

            {/* Top Classification / Merkle Strip inside SVG */}
            <g transform="translate(590, 22)">
              <rect x={-230} y={-11} width={460} height={18} fill="var(--ck-bg-1)" rx={2} stroke="var(--ck-hairline-strong)" strokeWidth="0.8" />
              <text textAnchor="middle" y={2} className="font-mono text-[8px] font-bold fill-ck-fg-2 tracking-widest uppercase">
                SEC-DOMAIN // FEDRAMP-MODERATE // 20 FAMILIES // 287 CONTROLS // SHA256:07617ef7a90b
              </text>
            </g>

            {/* Bottom-Right Tactical Axis / Compass */}
            <g transform="translate(1080, 675)">
              <circle cx={0} cy={0} r={17} fill="none" stroke="var(--ck-hairline-strong)" strokeWidth="0.8" opacity={0.7} />
              <circle cx={0} cy={0} r={8.5} fill="none" stroke="var(--ck-hairline-strong)" strokeWidth="0.5" strokeDasharray="2 2" opacity={0.5} />
              <line x1={-20} y1={0} x2={20} y2={0} stroke="var(--ck-hairline-strong)" strokeWidth="0.8" opacity={0.7} />
              <line x1={0} y1={-20} x2={0} y2={20} stroke="var(--ck-hairline-strong)" strokeWidth="0.8" opacity={0.7} />
              <path d="M 0 -20 L 3 -14 L -3 -14 Z" fill="var(--ck-accent)" />
              <text x={0} y={-23} textAnchor="middle" className="font-mono text-[7px] font-bold fill-ck-accent">N</text>
              <text x={0} y={28} textAnchor="middle" className="font-mono text-[7px] fill-ck-fg-mute tracking-wider">HEX 60&deg;</text>
            </g>
          </g>

          <g
            transform={`translate(${pan.x}, ${pan.y}) scale(${zoom})`}
            className="transition-transform duration-100 ease-out"
          >
            {/* Dependency Corridors: High-Assurance Dual-Rail Conduits */}
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

              // Perpendicular normal vector for dual-rail bus
              const nx = -uy;
              const ny = ux;
              const isConnectedToSelected =
                selectedFamilyId === fam1 || selectedFamilyId === fam2;
              const isConnectedToFocused =
                focusedFam === fam1 || focusedFam === fam2;
              const isCorridorHovered =
                hoveredCorridor?.fam1 === fam1 && hoveredCorridor?.fam2 === fam2;
              const isHighlighted =
                isConnectedToSelected ||
                isConnectedToFocused ||
                isCorridorHovered ||
                pulseActive;

              const sep = isHighlighted ? 2.5 : 1.8;

              // Outer Rails A & B
              const rx1_a = x1 + nx * sep;
              const ry1_a = y1 + ny * sep;
              const rx2_a = x2 + nx * sep;
              const ry2_a = y2 + ny * sep;

              const rx1_b = x1 - nx * sep;
              const ry1_b = y1 - ny * sep;
              const rx2_b = x2 - nx * sep;
              const ry2_b = y2 - ny * sep;

              // Midpoints for structural tie struts
              const m1x = x1 * 0.65 + x2 * 0.35;
              const m1y = y1 * 0.65 + y2 * 0.35;
              const m2x = x1 * 0.35 + x2 * 0.65;
              const m2y = y1 * 0.35 + y2 * 0.65;

              return (
                <g
                  key={`corridor-${idx}`}
                  className="cursor-pointer"
                  onMouseEnter={() => setHoveredCorridor({ fam1, fam2, label })}
                  onMouseLeave={() => setHoveredCorridor(null)}
                  onClick={(e) => {
                    e.stopPropagation();
                    setFocusedFam((prev) => (prev === fam1 ? null : fam1));
                    onSelectControl(fam1 + "-1");
                  }}
                >
                  <title>{`${fam1.toUpperCase()} <-> ${fam2.toUpperCase()}: ${label}`}</title>
                  {/* Expanded invisible hit target for easy mouse hover */}
                  <line
                    x1={x1}
                    y1={y1}
                    x2={x2}
                    y2={y2}
                    stroke="transparent"
                    strokeWidth={16}
                  />

                  {/* Dual-Rail Outer Conduits */}
                  <line
                    x1={rx1_a}
                    y1={ry1_a}
                    x2={rx2_a}
                    y2={ry2_a}
                    stroke={
                      isHighlighted
                        ? "var(--ck-accent)"
                        : "var(--ck-hairline-strong)"
                    }
                    strokeWidth={isHighlighted ? 1.4 : 0.8}
                    strokeOpacity={isHighlighted ? 0.95 : 0.35}
                  />
                  <line
                    x1={rx1_b}
                    y1={ry1_b}
                    x2={rx2_b}
                    y2={ry2_b}
                    stroke={
                      isHighlighted
                        ? "var(--ck-accent)"
                        : "var(--ck-hairline-strong)"
                    }
                    strokeWidth={isHighlighted ? 1.4 : 0.8}
                    strokeOpacity={isHighlighted ? 0.95 : 0.35}
                  />

                  {/* Structural Tie Struts */}
                  <line
                    x1={m1x + nx * sep}
                    y1={m1y + ny * sep}
                    x2={m1x - nx * sep}
                    y2={m1y - ny * sep}
                    stroke={isHighlighted ? "var(--ck-accent)" : "var(--ck-hairline-strong)"}
                    strokeWidth={0.8}
                    strokeOpacity={isHighlighted ? 0.9 : 0.35}
                  />
                  <line
                    x1={m2x + nx * sep}
                    y1={m2y + ny * sep}
                    x2={m2x - nx * sep}
                    y2={m2y - ny * sep}
                    stroke={isHighlighted ? "var(--ck-accent)" : "var(--ck-hairline-strong)"}
                    strokeWidth={0.8}
                    strokeOpacity={isHighlighted ? 0.9 : 0.35}
                  />

                  {/* Center Signal Flow Rail */}
                  <line
                    x1={x1}
                    y1={y1}
                    x2={x2}
                    y2={y2}
                    stroke={
                      isHighlighted
                        ? "var(--ck-accent)"
                        : "var(--ck-hairline)"
                    }
                    strokeWidth={isHighlighted ? 1.8 : 0.8}
                    strokeOpacity={isHighlighted ? 0.95 : 0.25}
                    className={isHighlighted || pulseActive ? "corridor-active-flow" : ""}
                  />

                  {(overlay === "corridors" || isHighlighted) && (
                    <text
                      x={(x1 + x2) / 2}
                      y={(y1 + y2) / 2 - 5}
                      textAnchor="middle"
                      className={`font-mono text-[8.5px] pointer-events-none transition-colors ${
                        isHighlighted
                          ? "fill-ck-accent font-bold"
                          : "fill-ck-fg-mute"
                      }`}
                    >
                      {label}
                    </text>
                  )}
                </g>
              );
            })}

            {/* GovX Tactical Radar Beam Sweep & Radiating Waves when Change Pulse is active */}
            {pulseActive && selectedFamilyId && FAMILY_COORD_MAP.has(selectedFamilyId) && (
              <g className="pointer-events-none">
                {(() => {
                  const fc = FAMILY_COORD_MAP.get(selectedFamilyId)!;
                  return (
                    <g>
                      {/* Concentric Range Rings with Tactical Distance Markings */}
                      <circle
                        cx={fc.cx}
                        cy={fc.cy}
                        r={fc.r + 32}
                        fill="none"
                        stroke="var(--ck-accent)"
                        strokeWidth="1.8"
                        className="animate-ping"
                      />
                      <circle
                        cx={fc.cx}
                        cy={fc.cy}
                        r={fc.r + 75}
                        fill="none"
                        stroke="var(--ck-accent)"
                        strokeWidth="1.4"
                        strokeDasharray="4 4"
                        className="animate-pulse"
                      />
                      <circle
                        cx={fc.cx}
                        cy={fc.cy}
                        r={fc.r + 140}
                        fill="none"
                        stroke="var(--ck-accent)"
                        strokeWidth="0.9"
                        strokeDasharray="2 6"
                        opacity={0.65}
                      />
                      <text
                        x={fc.cx + fc.r + 36}
                        y={fc.cy - 4}
                        className="font-mono text-[7px] fill-ck-accent font-semibold tracking-wider"
                      >
                        R-40km
                      </text>
                      <text
                        x={fc.cx + fc.r + 79}
                        y={fc.cy - 4}
                        className="font-mono text-[7px] fill-ck-accent font-semibold tracking-wider"
                      >
                        R-90km
                      </text>
                      <text
                        x={fc.cx + fc.r + 144}
                        y={fc.cy - 4}
                        className="font-mono text-[7px] fill-ck-accent font-semibold tracking-wider"
                      >
                        R-170km
                      </text>

                      {/* Rotating Tactical Radar Beam Sector */}
                      <g
                        style={{
                          transformOrigin: `${fc.cx}px ${fc.cy}px`,
                          animation: "radarSweep 2.2s linear infinite",
                        }}
                      >
                        <path
                          d={`M ${fc.cx} ${fc.cy} L ${fc.cx + 220} ${fc.cy} A 220 220 0 0 1 ${fc.cx + 190} ${fc.cy + 110} Z`}
                          fill="url(#radar-beam-gradient)"
                          opacity={0.4}
                        />
                        <line
                          x1={fc.cx}
                          y1={fc.cy}
                          x2={fc.cx + 220}
                          y2={fc.cy}
                          stroke="var(--ck-accent)"
                          strokeWidth="1.8"
                        />
                      </g>
                    </g>
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
                  onDoubleClick={(e) => {
                    e.stopPropagation();
                    handleFamilyDoubleClick(fc.id);
                  }}
                  className="cursor-pointer"
                >
                  <title>{`${fc.id.toUpperCase()}: ${fam?.title ?? fc.short} (${baselineCount} baseline controls) · Double-click to zoom into family`}</title>

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

                  {/* Posture Mode Display with Radial Completion Meter */}
                  {semanticZoom === "posture" && (
                    <g className="pointer-events-none">
                      {(() => {
                        const gaugeR = fc.r - 20;
                        const circ = 2 * Math.PI * gaugeR;
                        const offset = circ * (1 - covPct / 100);
                        return (
                          <>
                            <circle
                              cx={fc.cx}
                              cy={fc.cy + 6}
                              r={gaugeR}
                              fill="none"
                              stroke="var(--ck-hairline-strong)"
                              strokeWidth="3.5"
                              strokeOpacity="0.35"
                            />
                            {covPct > 0 && (
                              <circle
                                cx={fc.cx}
                                cy={fc.cy + 6}
                                r={gaugeR}
                                fill="none"
                                stroke="var(--ck-pos)"
                                strokeWidth="3.5"
                                strokeDasharray={circ}
                                strokeDashoffset={offset}
                                strokeLinecap="round"
                                transform={`rotate(-90 ${fc.cx} ${fc.cy + 6})`}
                              />
                            )}
                          </>
                        );
                      })()}
                      <text
                        x={fc.cx}
                        y={fc.cy + 13}
                        textAnchor="middle"
                        className="font-serif text-2xl font-normal fill-ck-fg-1"
                      >
                        {covPct}%
                      </text>
                      <text
                        x={fc.cx}
                        y={fc.cy + 27}
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
                          data-control-node="true"
                          onClick={(e) => {
                            e.stopPropagation();
                            onSelectControl(c.id);
                          }}
                          onDoubleClick={(e) => {
                            e.stopPropagation();
                            onSelectControl(c.id);
                            setDossierOpen(true);
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
                            } · Double-click to open full dossier`}
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

                          {/* Level of Detail (LOD): Control ID text when zoomed in */}
                          {zoom >= 1.35 && (
                            <text
                              x={node.x}
                              y={node.y + half + 7.5}
                              textAnchor="middle"
                              className={`font-mono text-[7px] pointer-events-none select-none transition-opacity ${
                                isSelected ? "fill-ck-accent font-bold" : "fill-ck-fg-1"
                              }`}
                            >
                              {c.id.toUpperCase()}
                            </text>
                          )}
                          {zoom >= 2.2 && c.params && c.params.length > 0 && (
                            <text
                              x={node.x}
                              y={node.y + half + 14.5}
                              textAnchor="middle"
                              className="font-mono text-[6px] fill-ck-fg-mute pointer-events-none select-none"
                            >
                              {c.params.length}p
                            </text>
                          )}

                          {/* Active Selection Tactical Targeting Brackets */}
                          {isSelected && (
                            <g className="pointer-events-none">
                              {/* Micro reticle corner brackets */}
                              <path
                                d={`
                                  M ${node.x - half - 4} ${node.y - half - 1} L ${node.x - half - 4} ${node.y - half - 4} L ${node.x - half - 1} ${node.y - half - 4}
                                  M ${node.x + half + 1} ${node.y - half - 4} L ${node.x + half + 4} ${node.y - half - 4} L ${node.x + half + 4} ${node.y - half - 1}
                                  M ${node.x - half - 4} ${node.y + half + 1} L ${node.x - half - 4} ${node.y + half + 4} L ${node.x - half - 1} ${node.y + half + 4}
                                  M ${node.x + half + 1} ${node.y + half + 4} L ${node.x + half + 4} ${node.y + half + 4} L ${node.x + half + 4} ${node.y + half + 1}
                                `}
                                fill="none"
                                stroke="var(--ck-accent)"
                                strokeWidth="1.6"
                              />
                              {/* Micro crosshairs extending outward */}
                              <line
                                x1={node.x - half - 7}
                                y1={node.y}
                                x2={node.x - half - 5}
                                y2={node.y}
                                stroke="var(--ck-accent)"
                                strokeWidth="1.2"
                              />
                              <line
                                x1={node.x + half + 5}
                                y1={node.y}
                                x2={node.x + half + 7}
                                y2={node.y}
                                stroke="var(--ck-accent)"
                                strokeWidth="1.2"
                              />
                              <line
                                x1={node.x}
                                y1={node.y - half - 7}
                                x2={node.x}
                                y2={node.y - half - 5}
                                stroke="var(--ck-accent)"
                                strokeWidth="1.2"
                              />
                              <line
                                x1={node.x}
                                y1={node.y + half + 5}
                                x2={node.x}
                                y2={node.y + half + 7}
                                stroke="var(--ck-accent)"
                                strokeWidth="1.2"
                              />
                            </g>
                          )}

                          {/* Risk Owner Open Weakness Ring */}
                          {lens === "risk-owner" && POAM_CONTROLS.has(c.id) && (
                            <circle
                              cx={node.x}
                              cy={node.y}
                              r={half + 4}
                              fill="none"
                              stroke="var(--ck-warn)"
                              strokeWidth="1.6"
                              strokeDasharray="2 2"
                              className="animate-pulse"
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

        {/* GovX Tactical HUD Overlay (Top-Right) */}
        {hoveredControl ? (
          (() => {
            const hState = implState(hoveredControl.id, byControl);
            const statusStr =
              hState.kind === "declared"
                ? hState.status
                : hState.kind === "declared-empty"
                  ? "empty"
                  : "undeclared in ssp";
            const isPos = hState.kind === "declared" && hState.status === "implemented";
            const isWarn = hState.kind === "declared" && hState.status === "partial";
            return (
              <div className="pointer-events-none absolute right-3 top-3 max-w-[300px] rounded-md border border-ck-hairline-strong bg-ck-bg-1/95 p-3 shadow-xl backdrop-blur-md transition-all duration-150">
                <div className="flex items-center justify-between gap-2 border-b border-ck-hairline pb-1.5 text-3xs font-mono uppercase tracking-widest text-ck-fg-mute">
                  <span className="flex items-center gap-1 font-bold text-ck-accent">
                    <span className="h-1.5 w-1.5 rounded-full bg-ck-accent animate-ping" />
                    TARGET LOCK // GOVX-L3
                  </span>
                  <span>{hoveredControl.family.toUpperCase()} &middot; {FAMILY_COORD_MAP.get(hoveredControl.family)?.short ?? "Family"}</span>
                </div>
                <div className="mt-2 flex items-baseline justify-between gap-2">
                  <span className="font-mono text-sm font-bold text-ck-fg-1">
                    {displayId(hoveredControl.id)}
                  </span>
                  <span className="font-mono text-3xs text-ck-fg-mute">
                    {hoveredControl.id}
                  </span>
                </div>
                <div className="mt-1 line-clamp-2 text-xs font-medium text-ck-fg-2">
                  {hoveredControl.title}
                </div>
                <div className="mt-2.5 flex flex-wrap items-center gap-1.5 text-2xs">
                  <span
                    className={`rounded px-1.5 py-0.5 font-mono uppercase tracking-wider ${
                      isPos
                        ? "bg-ck-pos/15 text-ck-pos border border-ck-pos/30"
                        : isWarn
                          ? "bg-ck-warn/15 text-ck-warn border border-ck-warn/30"
                          : "bg-ck-bg-2 text-ck-fg-mute border border-ck-hairline"
                    }`}
                  >
                    {statusStr}
                  </span>
                  {hoveredControl.params && hoveredControl.params.length > 0 && (
                    <span className="rounded border border-ck-hairline bg-ck-bg-0 px-1.5 py-0.5 font-mono text-ck-fg-2">
                      {hoveredControl.params.length} PARAMS BOUND
                    </span>
                  )}
                  {POAM_CONTROLS.has(hoveredControl.id) && (
                    <span className="rounded border border-ck-warn/40 bg-ck-warn/10 px-1.5 py-0.5 font-mono text-ck-warn font-semibold">
                      POA&amp;M DEFICIENCY
                    </span>
                  )}
                </div>
                <div className="mt-2 border-t border-ck-hairline/60 pt-1.5 flex items-center justify-between text-3xs font-mono text-ck-fg-mute">
                  <span>ASSURANCE: NIST-SP800-53-MOD</span>
                  <span>MERKLE: VERIFIED</span>
                </div>
                <button
                  type="button"
                  onClick={() => {
                    onSelectControl(hoveredControl.id);
                    setDossierOpen(true);
                  }}
                  className="pointer-events-auto mt-2.5 flex w-full items-center justify-center gap-1.5 rounded border border-ck-accent bg-ck-accent/15 px-2 py-1 text-2xs font-semibold text-ck-accent hover:bg-ck-accent hover:text-white transition-colors"
                >
                  <span>INSPECT DOSSIER &amp; TAILORING</span>
                  <span>&rarr;</span>
                </button>
              </div>
            );
          })()
        ) : hoveredCorridor ? (
          <div className="pointer-events-none absolute right-3 top-3 max-w-[300px] rounded-md border border-ck-accent bg-ck-bg-1/95 p-3 shadow-xl backdrop-blur-md">
            <div className="flex items-center justify-between gap-2 border-b border-ck-hairline pb-1.5 text-3xs font-mono uppercase tracking-widest text-ck-fg-mute">
              <span className="flex items-center gap-1 font-bold text-ck-accent">
                <span className="h-1.5 w-1.5 rounded-full bg-ck-accent animate-pulse" />
                CONDUIT LOCK // DUAL-BUS
              </span>
              <span>ACTIVE FLOW</span>
            </div>
            <div className="mt-2 flex items-baseline justify-between gap-2">
              <span className="font-mono text-sm font-bold text-ck-accent">
                {`${hoveredCorridor.fam1.toUpperCase()} <-> ${hoveredCorridor.fam2.toUpperCase()}`}
              </span>
              <span className="font-mono text-3xs text-ck-fg-mute">100% NOMINAL</span>
            </div>
            <div className="mt-1 text-xs font-semibold text-ck-fg-1">
              {hoveredCorridor.label}
            </div>
            <div className="mt-1 text-2xs text-ck-fg-mute">
              Architectural dependency conduit between {FAMILY_COORD_MAP.get(hoveredCorridor.fam1)?.short} and {FAMILY_COORD_MAP.get(hoveredCorridor.fam2)?.short}.
            </div>
            <div className="mt-2 border-t border-ck-hairline/60 pt-1.5 flex items-center justify-between text-3xs font-mono text-ck-fg-mute">
              <span>PROPAGATION: IMMEDIATE</span>
              <span>BUS PROTOCOL: OSCAL-AST</span>
            </div>
          </div>
        ) : selectedControl ? (
          <div className="pointer-events-auto absolute right-3 top-3 max-w-[300px] rounded-md border border-ck-hairline-strong bg-ck-bg-1/95 p-3 shadow-xl backdrop-blur-md transition-all duration-150">
            <div className="flex items-center justify-between gap-2 border-b border-ck-hairline pb-1.5 text-3xs font-mono uppercase tracking-widest text-ck-fg-mute">
              <span className="flex items-center gap-1 font-bold text-ck-accent">
                <span className="h-1.5 w-1.5 rounded-full bg-ck-accent animate-pulse" />
                SELECTED TARGET
              </span>
              <span>{selectedControl.family.toUpperCase()} &middot; {FAMILY_COORD_MAP.get(selectedControl.family)?.short ?? "Family"}</span>
            </div>
            <div className="mt-2 flex items-baseline justify-between gap-2">
              <span className="font-mono text-sm font-bold text-ck-fg-1">
                {displayId(selectedControl.id)}
              </span>
              <span className="font-mono text-3xs text-ck-fg-mute">
                {selectedControl.id}
              </span>
            </div>
            <div className="mt-1 line-clamp-2 text-xs font-medium text-ck-fg-2">
              {selectedControl.title}
            </div>
            <div className="mt-2.5 flex items-center justify-between text-2xs">
              <span className="font-mono text-ck-fg-mute text-3xs">CLICK NODE TO TOGGLE</span>
              <button
                type="button"
                onClick={(e) => {
                  e.stopPropagation();
                  setDossierOpen(true);
                }}
                onPointerDown={(e) => e.stopPropagation()}
                className="flex items-center gap-1 rounded border border-ck-accent bg-ck-accent/15 px-2 py-0.5 font-mono text-2xs font-semibold text-ck-accent hover:bg-ck-accent hover:text-white transition-colors cursor-pointer"
              >
                <span>DOSSIER</span>
                <span>&rarr;</span>
              </button>
            </div>
          </div>
        ) : (
          <div className="pointer-events-none absolute right-3 top-3 hidden sm:flex items-center gap-2.5 rounded-md border border-ck-hairline-strong bg-ck-bg-1/90 px-3 py-1.5 text-2xs text-ck-fg-mute backdrop-blur-md shadow-sm">
            <span className="flex items-center gap-1.5 font-bold text-ck-fg-1">
              <span className="h-1.5 w-1.5 rounded-full bg-ck-pos" />
              GOVX TELEMETRY
            </span>
            <span className="h-3 w-px bg-ck-hairline-strong" />
            <span>20 FAMILIES</span>
            <span className="h-3 w-px bg-ck-hairline-strong" />
            <span>18 DUAL CONDUITS</span>
            <span className="h-3 w-px bg-ck-hairline-strong" />
            <span>287 CONTROLS</span>
            {lens === "risk-owner" && (
              <>
                <span className="h-3 w-px bg-ck-hairline-strong" />
                <span className="text-ck-warn font-semibold">5 POA&amp;M ITEMS</span>
              </>
            )}
          </div>
        )}

        {/* Sliding GovX Tactical Control Dossier Drawer */}
        {dossierOpen && selectedControl && (
          <div
            onPointerDown={(e) => e.stopPropagation()}
            onPointerMove={(e) => e.stopPropagation()}
            className="absolute top-0 right-0 z-30 flex h-full w-full max-w-[390px] flex-col border-l border-ck-hairline-strong bg-ck-bg-1/98 p-4 shadow-2xl backdrop-blur-md overflow-y-auto animate-in slide-in-from-right duration-200 font-mono"
          >
            {/* Header */}
            <div className="flex items-start justify-between gap-2 border-b border-ck-hairline pb-2.5">
              <div>
                <div className="flex items-center gap-1.5 text-3xs uppercase tracking-widest text-ck-accent font-bold">
                  <span className="h-1.5 w-1.5 rounded-full bg-ck-accent animate-ping" />
                  GOVX CONTROL DOSSIER // {displayId(selectedControl.id)}
                </div>
                <h3 className="mt-1 font-serif text-lg font-semibold text-ck-fg-1">
                  {selectedControl.title}
                </h3>
                <span className="text-3xs text-ck-fg-mute">
                  ID: {selectedControl.id} &middot; Family: {selectedControl.family.toUpperCase()} ({FAMILY_COORD_MAP.get(selectedControl.family)?.short})
                </span>
              </div>
              <button
                type="button"
                onClick={() => setDossierOpen(false)}
                className="rounded border border-ck-hairline-strong bg-ck-bg-0 px-2 py-1 text-xs text-ck-fg-mute hover:text-ck-fg-1 hover:bg-ck-bg-2 transition-colors"
                title="Close Dossier"
              >
                &times;
              </button>
            </div>

            <div className="mt-3 space-y-3.5 text-xs text-ck-fg-2">
              {/* Status & Tailoring Summary */}
              {(() => {
                const s = implState(selectedControl.id, byControl);
                return (
                  <div className="flex flex-wrap items-center gap-1.5">
                    <span
                      className={`rounded px-1.5 py-0.5 text-2xs font-bold uppercase tracking-wider ${
                        s.kind === "declared" && s.status === "implemented"
                          ? "bg-ck-pos/15 text-ck-pos border border-ck-pos/30"
                          : s.kind === "declared" && s.status === "partial"
                            ? "bg-ck-warn/15 text-ck-warn border border-ck-warn/30"
                            : "bg-ck-bg-2 text-ck-fg-mute border border-ck-hairline"
                      }`}
                    >
                      {s.kind === "declared" ? s.status : s.kind === "declared-empty" ? "empty" : "undeclared"}
                    </span>
                    {selectedControl.in_baseline ? (
                      <span className="rounded border border-ck-info/30 bg-ck-info/10 px-1.5 py-0.5 text-2xs text-ck-info">
                        Moderate Baseline
                      </span>
                    ) : (
                      <span className="rounded border border-ck-hairline bg-ck-bg-0 px-1.5 py-0.5 text-2xs text-ck-fg-mute">
                        Catalog Only
                      </span>
                    )}
                    {selectedControl.params && selectedControl.params.length > 0 && (
                      <span className="rounded border border-ck-hairline bg-ck-bg-0 px-1.5 py-0.5 text-2xs text-ck-fg-2">
                        {selectedControl.params.length} params
                      </span>
                    )}
                  </div>
                );
              })()}

              {/* Statement */}
              <div className="space-y-1">
                <span className="ck-eyebrow text-3xs">Control Statement</span>
                {selectedControl.statement ? (
                  <div className="rounded border border-ck-hairline bg-ck-bg-0 p-2.5 font-sans text-xs leading-relaxed text-ck-fg-1">
                    <StatementText
                      statement={selectedControl.statement}
                      params={selectedControl.params ?? []}
                    />
                  </div>
                ) : (
                  <p className="text-2xs text-ck-fg-mute">No statement projected for this control.</p>
                )}
              </div>

              {/* Parameters List */}
              {selectedControl.params && selectedControl.params.length > 0 && (
                <div className="space-y-1">
                  <span className="ck-eyebrow text-3xs">Tailorable Parameters ({selectedControl.params.length})</span>
                  <div className="max-h-36 overflow-y-auto space-y-1 rounded border border-ck-hairline bg-ck-bg-0 p-2">
                    {selectedControl.params.map((p) => (
                      <div key={p.id} className="text-2xs">
                        <span className="font-bold text-ck-accent">{p.id}</span>
                        <span className="block text-ck-fg-mute">{p.label}</span>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Architectural Corridors */}
              <div className="space-y-1">
                <span className="ck-eyebrow text-3xs">Boundary Corridors</span>
                <div className="flex flex-wrap gap-1">
                  {CORRIDORS.filter(([fam1, fam2]) => fam1 === selectedControl.family || fam2 === selectedControl.family).map(([fam1, fam2, label]) => (
                    <span
                      key={`${fam1}-${fam2}`}
                      className="inline-flex items-center gap-1 rounded border border-ck-hairline bg-ck-bg-0 px-1.5 py-0.5 text-3xs"
                    >
                      <span className="font-bold text-ck-accent">{fam1.toUpperCase()}&harr;{fam2.toUpperCase()}</span>
                      <span className="text-ck-fg-mute">{label}</span>
                    </span>
                  ))}
                </div>
              </div>

              {/* FedRAMP Assurance & Actions */}
              <div className="border-t border-ck-hairline pt-2.5 space-y-2">
                <div className="flex items-center justify-between text-3xs text-ck-fg-mute">
                  <span>ASSURANCE: NIST-SP800-53-MOD</span>
                  <span>MERKLE: 07617ef7a90b</span>
                </div>
                {onOpenInComposer && selectedControl.in_baseline && (
                  <button
                    type="button"
                    onClick={() => onOpenInComposer(selectedControl.id)}
                    className="flex w-full items-center justify-center gap-1.5 rounded border border-ck-accent bg-ck-accent px-3 py-1.5 text-xs font-semibold text-white hover:opacity-90 transition-opacity"
                  >
                    <span>Tailor Parameter Values in Composer</span>
                    <span>&rarr;</span>
                  </button>
                )}
              </div>
            </div>
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
          {lens === "risk-owner" && (
            <span className="flex items-center gap-1 border-l border-ck-hairline pl-2">
              <span className="h-2 w-2 rounded-full border border-ck-warn bg-ck-warn/20" />
              <span className="text-ck-warn font-medium">POA&M Risk</span>
            </span>
          )}
        </div>
      </div>
    </div>
  );
}
