"use client";

import * as React from "react";
import { LensMode } from "@/lib/oscal-types";

interface LensSelectorProps {
  lens: LensMode;
  onLensChange: (l: LensMode) => void;
}

export function LensSelector({ lens, onLensChange }: LensSelectorProps) {
  const [open, setOpen] = React.useState(false);

  const lenses: { id: LensMode; name: string; exposure: string }[] = [
    { id: "author", name: "Author", exposure: "Prose & Params" },
    {
      id: "architect",
      name: "Architect",
      exposure: "Component Graphs & Topology",
    },
    { id: "engineer", name: "Engineer", exposure: "GitOps AST & Policy Rules" },
    { id: "assessor", name: "Assessor", exposure: "Observations & Evidence" },
    { id: "risk-owner", name: "Risk Owner", exposure: "Blast Radius & POA&M" },
    { id: "ciso", name: "CISO", exposure: "Executive Bento & Baselines" },
  ];

  return (
    <div className="relative inline-flex">
      <button
        type="button"
        onClick={() => setOpen(!open)}
        className="inline-flex items-center gap-1.5 border border-ck-hairline-strong bg-ck-bg-1 px-2.5 py-1 font-mono text-[11px] font-medium text-ck-fg-2 hover:text-ck-fg-1"
        title="Adapts density, vocabulary and schema exposure — one app, not six"
      >
        <span className="text-ck-fg-mute">lens:</span>
        <span className="font-semibold text-ck-fg-1">{lens}</span>
        <span className="text-ck-fg-mute text-[10px]">▾</span>
      </button>

      {open && (
        <>
          <div className="fixed inset-0 z-40" onClick={() => setOpen(false)} />
          <div
            role="menu"
            className="absolute top-[calc(100%+4px)] right-0 z-50 min-w-[240px] border border-ck-fg-1 bg-ck-bg-1 p-1 shadow-[4px_4px_0_var(--ck-fg-1)]"
          >
            {lenses.map((l) => (
              <button
                key={l.id}
                type="button"
                onClick={() => {
                  onLensChange(l.id);
                  setOpen(false);
                }}
                className={`flex w-full items-center justify-between px-3 py-1.5 text-left font-mono text-[11px] transition-colors ${
                  lens === l.id
                    ? "bg-ck-bg-2 font-semibold text-ck-fg-1"
                    : "text-ck-fg-2 hover:bg-ck-bg-2"
                }`}
              >
                <span>{l.name}</span>
                <span className="text-[10px] text-ck-fg-mute">
                  {l.exposure}
                </span>
              </button>
            ))}
            <div className="mt-1.5 border-t border-ck-hairline px-3 py-1.5 font-mono text-[10px] text-ck-fg-mute">
              one app, not six — the lens adapts, the ledger doesn&apos;t
            </div>
          </div>
        </>
      )}
    </div>
  );
}
