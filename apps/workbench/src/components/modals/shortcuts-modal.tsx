"use client";

import * as React from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

interface ShortcutsModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export function ShortcutsModal({ isOpen, onClose }: ShortcutsModalProps) {
  if (!isOpen) return null;

  const shortcuts = [
    { key: "1", label: "The Atlas", desc: "OSCAL spatial graph of record" },
    {
      key: "2",
      label: "The Bridge",
      desc: "Cross-framework mapping & edge validation",
    },
    {
      key: "3",
      label: "The Composer",
      desc: "OSCAL parameter tailoring & profile diff",
    },
    {
      key: "4",
      label: "The Ledger",
      desc: "Cryptographic evidence decay & Merkle logs",
    },
    {
      key: "5",
      label: "The Docket",
      desc: "POA&M pressure-sorted priority queue",
    },
    {
      key: "6",
      label: "The Pipeline",
      desc: "Pre-commit persona & deterministic gates",
    },
    {
      key: "7",
      label: "Jurisdictions & SLSA",
      desc: "NIST, CCCS, EUCS & SLSA v1.2 attestations",
    },
    {
      key: "8",
      label: "Policy Gates & SBOM",
      desc: "In-process Rego evaluation & CycloneDX",
    },
    {
      key: "9",
      label: "Root Fabric & Identity",
      desc: "SPIFFE/SPIRE SVIDs & multi-tenant isolation",
    },
    {
      key: "T",
      label: "Cycle Theme",
      desc: "Switch between Ledger, Vault, and High-Contrast",
    },
    {
      key: "L",
      label: "Cycle Lens",
      desc: "Switch between Author, Architect, Engineer, Assessor, Risk-Owner, CISO",
    },
    {
      key: "⌘K",
      label: "Toggle Copilot",
      desc: "Open/close embedded Atlas compliance copilot",
    },
    {
      key: "Esc",
      label: "Dismiss",
      desc: "Close open inspector, drawers, or dialogs",
    },
  ];

  return (
    <div
      className="fixed inset-0 z-50 bg-black/60 backdrop-blur-xs flex items-center justify-center p-4"
      onClick={onClose}
    >
      <div
        className="border border-ck-hairline-strong bg-ck-bg-1 max-w-xl w-full p-6 shadow-2xl space-y-4 font-mono text-xs animate-in fade-in zoom-in-95"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-baseline justify-between border-b border-ck-hairline pb-2">
          <div className="flex items-center gap-2">
            <h2 className="font-serif text-2xl font-normal text-ck-fg-1">
              Keyboard Navigation
            </h2>
            <span className="font-mono text-[11px] text-ck-fg-mute">
              · Rapid Operator Ergonomics
            </span>
          </div>
          <Badge
            variant="outline"
            className="font-mono text-[10px] text-ck-accent"
          >
            HOTKEYS
          </Badge>
        </div>

        <p className="font-sans text-xs text-ck-fg-2">
          Mizan is designed for zero-latency keyboard-first governance. Every
          surface, lens, and ledger verification is directly reachable.
        </p>

        <div className="grid grid-cols-2 gap-2 bg-ck-bg-0 p-3 border border-ck-hairline">
          {shortcuts.map((s) => (
            <div
              key={s.key}
              className="flex items-center justify-between p-1.5 rounded-xs hover:bg-ck-bg-1 transition-colors"
            >
              <div className="flex items-center gap-2 min-w-0">
                <kbd className="px-1.5 py-0.5 border border-ck-hairline-strong bg-ck-bg-2 text-ck-fg-1 font-mono text-[11px] font-semibold rounded-xs shadow-xs">
                  {s.key}
                </kbd>
                <span className="font-medium text-ck-fg-1 truncate">
                  {s.label}
                </span>
              </div>
              <span className="text-[10px] text-ck-fg-mute text-right truncate ml-2">
                {s.desc}
              </span>
            </div>
          ))}
        </div>

        <div className="flex items-center justify-between border-t border-ck-hairline pt-3">
          <span className="text-[11px] text-ck-fg-mute">
            Press <kbd className="px-1 py-0.2 border border-ck-hairline">?</kbd>{" "}
            anywhere to toggle this sheet
          </span>
          <Button
            size="sm"
            variant="outline"
            onClick={onClose}
            className="h-8 text-xs font-mono"
          >
            Close
          </Button>
        </div>
      </div>
    </div>
  );
}
