"use client";

import * as React from "react";

export type SurfaceId =
  | "atlas"
  | "bridge"
  | "composer"
  | "ledger"
  | "docket"
  | "pipeline"
  | "jurisdiction"
  | "cicd"
  | "fabric";

interface AtlasNavProps {
  activeSurface: SurfaceId;
  onSelectSurface: (id: SurfaceId) => void;
  counts: Record<string, string | number>;
}

// Crisp 14x14 architectural SVG icons for each surface
const SurfaceIcons: Record<SurfaceId, React.ReactNode> = {
  atlas: (
    <svg
      width="14"
      height="14"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.3"
    >
      <rect x="2" y="2" width="5" height="5" />
      <rect x="9" y="2" width="5" height="5" />
      <rect x="2" y="9" width="5" height="5" />
      <rect x="9" y="9" width="5" height="5" />
    </svg>
  ),
  bridge: (
    <svg
      width="14"
      height="14"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.3"
    >
      <circle cx="4" cy="8" r="2.5" />
      <circle cx="12" cy="8" r="2.5" />
      <line x1="6.5" y1="8" x2="9.5" y2="8" strokeDasharray="1.5 1.5" />
    </svg>
  ),
  composer: (
    <svg
      width="14"
      height="14"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.3"
    >
      <path d="M3 2.5h7l3 3V13.5H3z" />
      <line x1="5.5" y1="7" x2="10.5" y2="7" />
      <line x1="5.5" y1="10" x2="8.5" y2="10" />
    </svg>
  ),
  ledger: (
    <svg
      width="14"
      height="14"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.3"
    >
      <rect x="3" y="2" width="10" height="12" />
      <line x1="5.5" y1="5.5" x2="10.5" y2="5.5" />
      <line x1="5.5" y1="8.5" x2="10.5" y2="8.5" />
      <line x1="5.5" y1="11.5" x2="8.5" y2="11.5" />
    </svg>
  ),
  docket: (
    <svg
      width="14"
      height="14"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.3"
    >
      <circle cx="8" cy="8" r="6" />
      <polyline points="8 4.5 8 8 10.5 9.5" />
    </svg>
  ),
  pipeline: (
    <svg
      width="14"
      height="14"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.3"
    >
      <circle cx="4" cy="4" r="2" />
      <circle cx="12" cy="12" r="2" />
      <path d="M4 6v4a2 2 0 0 0 2 2h4" />
    </svg>
  ),
  jurisdiction: (
    <svg
      width="14"
      height="14"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.3"
    >
      <circle cx="8" cy="8" r="6" />
      <ellipse cx="8" cy="8" rx="2.5" ry="6" />
      <line x1="2" y1="8" x2="14" y2="8" />
    </svg>
  ),
  cicd: (
    <svg
      width="14"
      height="14"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.3"
    >
      <path d="M8 2.5l5 2.5v4c0 3-2.5 5-5 5.5-2.5-.5-5-2.5-5-5.5V5l5-2.5z" />
      <polyline points="6 8 7.5 9.5 10.5 6.5" />
    </svg>
  ),
  fabric: (
    <svg
      width="14"
      height="14"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.3"
    >
      <rect x="2" y="2" width="4" height="4" />
      <rect x="10" y="2" width="4" height="4" />
      <rect x="6" y="10" width="4" height="4" />
      <path d="M4 6v2h8V6" />
      <line x1="8" y1="8" x2="8" y2="10" />
    </svg>
  ),
};

export function AtlasNav({
  activeSurface,
  onSelectSurface,
  counts,
}: AtlasNavProps) {
  const groups = [
    {
      label: "territory",
      items: [
        {
          id: "atlas" as SurfaceId,
          label: "The Atlas",
          countKey: "atlas",
          keyNum: "1",
        },
        {
          id: "bridge" as SurfaceId,
          label: "The Bridge",
          countKey: "bridge",
          keyNum: "2",
        },
      ],
    },
    {
      label: "record",
      items: [
        {
          id: "composer" as SurfaceId,
          label: "The Composer",
          countKey: "composer",
          keyNum: "3",
        },
        {
          id: "ledger" as SurfaceId,
          label: "The Ledger",
          countKey: "ledger",
          keyNum: "4",
        },
      ],
    },
    {
      label: "operations",
      items: [
        {
          id: "docket" as SurfaceId,
          label: "The Docket",
          countKey: "docket",
          keyNum: "5",
        },
        {
          id: "pipeline" as SurfaceId,
          label: "The Pipeline",
          countKey: "pipeline",
          keyNum: "6",
        },
      ],
    },
    {
      label: "governance & fabric",
      items: [
        {
          id: "jurisdiction" as SurfaceId,
          label: "Jurisdictions & SLSA",
          countKey: "jurisdiction",
          keyNum: "7",
        },
        {
          id: "cicd" as SurfaceId,
          label: "Policy Gates & SBOM",
          countKey: "cicd",
          keyNum: "8",
        },
        {
          id: "fabric" as SurfaceId,
          label: "Root Fabric & Identity",
          countKey: "fabric",
          keyNum: "9",
        },
      ],
    },
  ];

  return (
    <nav
      aria-label="Surfaces"
      className="w-64 flex-shrink-0 border-r border-ck-hairline-strong bg-ck-bg-1 flex flex-col justify-between p-3 font-mono text-xs select-none"
    >
      <div className="space-y-4">
        {groups.map((g) => (
          <div key={g.label} className="space-y-1">
            <span className="text-[10px] uppercase font-bold text-ck-fg-mute tracking-wider block px-2 pb-0.5">
              {g.label}
            </span>
            {g.items.map((item) => {
              const isActive = activeSurface === item.id;
              const countVal =
                counts[item.countKey] !== undefined
                  ? counts[item.countKey]
                  : "";

              return (
                <button
                  key={item.id}
                  type="button"
                  onClick={() => onSelectSurface(item.id)}
                  className={`w-full flex items-center justify-between px-2.5 py-1.5 border-l-2 text-left transition-colors ${
                    isActive
                      ? "border-ck-fg-1 bg-ck-bg-0 text-ck-fg-1 font-semibold shadow-xs"
                      : "border-transparent text-ck-fg-2 hover:bg-ck-bg-2"
                  }`}
                  title={`${item.label} · press ${item.keyNum}`}
                >
                  <span className="flex items-center gap-2.5 min-w-0 flex-1">
                    <span
                      className={`shrink-0 ${isActive ? "text-ck-accent" : "text-ck-fg-mute"}`}
                    >
                      {SurfaceIcons[item.id]}
                    </span>
                    <span className="truncate text-[11.5px]">{item.label}</span>
                  </span>
                  <span className="text-[10px] text-ck-fg-mute font-mono shrink-0 ml-1.5 tabular-nums">
                    {countVal}
                  </span>
                </button>
              );
            })}
          </div>
        ))}
      </div>

      {/* Documentation & Live Evidence Links */}
      <div className="border-t border-ck-hairline pt-3 space-y-1 text-[11px]">
        <span className="text-[10px] uppercase font-bold text-ck-fg-mute tracking-wider block px-1 pb-0.5">
          Documentation & Evidence
        </span>
        <a
          href="./docs/api/mizan/index.html"
          target="_blank"
          rel="noopener noreferrer"
          className="w-full flex items-center justify-between px-2 py-1 text-ck-fg-2 hover:text-ck-fg-1 hover:bg-ck-bg-2 transition-colors rounded-xs"
        >
          <span className="flex items-center gap-2">
            <span className="text-ck-accent text-xs">[doc]</span>
            <span className="text-[11px]">Rust API Docs</span>
          </span>
          <span className="text-[9px] text-ck-fg-mute font-mono">{"->"}</span>
        </a>
        <a
          href="./capsule.html"
          target="_blank"
          rel="noopener noreferrer"
          className="w-full flex items-center justify-between px-2 py-1 text-ck-fg-2 hover:text-ck-fg-1 hover:bg-ck-bg-2 transition-colors rounded-xs"
        >
          <span className="flex items-center gap-2">
            <span className="text-green-500 text-xs">[sec]</span>
            <span className="text-[11px]">Evidence Capsule</span>
          </span>
          <span className="text-[9px] text-ck-fg-mute font-mono">{"->"}</span>
        </a>
        <a
          href="https://github.com/ckodex-labs/ckodex-oscal-cli/releases"
          target="_blank"
          rel="noopener noreferrer"
          className="w-full flex items-center justify-between px-2 py-1 text-ck-fg-2 hover:text-ck-fg-1 hover:bg-ck-bg-2 transition-colors rounded-xs"
        >
          <span className="flex items-center gap-2">
            <span className="text-ck-fg-mute text-xs">[pkg]</span>
            <span className="text-[11px]">Binary Releases</span>
          </span>
          <span className="text-[9px] text-ck-fg-mute font-mono">{"->"}</span>
        </a>
      </div>

      {/* Repo of Record Footer */}
      <div className="border-t border-ck-hairline pt-3 space-y-1 text-[11px]">
        <span className="text-[10px] uppercase font-bold text-ck-fg-mute block">
          Repo of Record
        </span>
        <div className="font-semibold text-ck-fg-1 font-mono text-xs">
          meridian/compliance
        </div>
        <div className="flex items-center gap-1.5 text-ck-fg-mute text-[10px] font-mono">
          <span>main</span>
          <span>·</span>
          <span className="underline">41c9e2b</span>
          <span className="ml-auto text-[9px] text-ck-fg-mute">1-9 keys</span>
        </div>
        <span className="text-[10px] text-ck-fg-mute block mt-0.5">
          The UI is a view onto the OSCAL GitOps repo
        </span>
      </div>
    </nav>
  );
}
