"use client";

import * as React from "react";
import { EvidenceItem } from "@/lib/atlas-data";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

interface LedgerSurfaceProps {
  evidence: EvidenceItem[];
}

export function LedgerSurface({ evidence }: LedgerSurfaceProps) {
  const [filterType, setFilterType] = React.useState<
    "all" | "signed" | "automation" | "human"
  >("all");
  const [sampledIndex, setSampledIndex] = React.useState<number | null>(null);

  const filtered = evidence.filter((e) => {
    if (filterType === "signed") return e.signed;
    if (filterType === "automation") return e.m === "automation";
    if (filterType === "human") return e.m === "human";
    return true;
  });

  const handleSample = () => {
    if (filtered.length > 0) {
      const arr = new Uint32Array(1);
      crypto.getRandomValues(arr);
      const rand = arr[0] % filtered.length;
      setSampledIndex(rand);
    }
  };

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex flex-wrap items-baseline justify-between border-b border-ck-hairline pb-2 gap-2">
        <div className="flex items-baseline gap-3 min-w-0">
          <h1 className="font-serif text-2xl font-normal tracking-tight text-ck-fg-1 whitespace-nowrap shrink-0">
            The Ledger
          </h1>
          <span className="text-xs text-ck-fg-mute font-mono hidden md:inline">
            Evidence ordered by freshness and strength. Evidence decays; the
            ledger preserves truth.
          </span>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          <Badge
            variant="outline"
            className="font-mono text-[10px] uppercase text-green-700 dark:text-green-400 whitespace-nowrap shrink-0"
          >
            {evidence.filter((e) => e.signed).length} Cryptographically Signed
            Receipts
          </Badge>
        </div>
      </div>

      {/* Toolbar */}
      <div className="flex items-center justify-between border border-ck-hairline bg-ck-bg-1 p-2 shadow-sm font-mono text-xs">
        <div className="flex items-center gap-1.5">
          <Button
            size="sm"
            variant={filterType === "all" ? "default" : "outline"}
            onClick={() => setFilterType("all")}
            className="h-7 text-xs font-mono"
          >
            All Evidence ({evidence.length})
          </Button>
          <Button
            size="sm"
            variant={filterType === "signed" ? "default" : "outline"}
            onClick={() => setFilterType("signed")}
            className="h-7 text-xs font-mono"
          >
            Signed Only ({evidence.filter((e) => e.signed).length})
          </Button>
          <Button
            size="sm"
            variant={filterType === "automation" ? "default" : "outline"}
            onClick={() => setFilterType("automation")}
            className="h-7 text-xs font-mono"
          >
            CI/CD Telemetry (
            {evidence.filter((e) => e.m === "automation").length})
          </Button>
          <Button
            size="sm"
            variant={filterType === "human" ? "default" : "outline"}
            onClick={() => setFilterType("human")}
            className="h-7 text-xs font-mono"
          >
            Human Audits ({evidence.filter((e) => e.m === "human").length})
          </Button>
        </div>

        <Button
          size="sm"
          variant="outline"
          onClick={handleSample}
          className="h-7 text-xs font-mono"
        >
          Spot Audit Sample
        </Button>
      </div>

      {/* Evidence Stream */}
      <div className="border border-ck-hairline-strong bg-ck-bg-0 divide-y divide-ck-hairline shadow-sm font-mono text-xs">
        {filtered.map((item, idx) => {
          const isSampled = sampledIndex === idx;
          const weight = Math.max(0.05, 1 - item.days / 365);

          return (
            <div
              key={idx}
              className={`p-3 flex items-center justify-between gap-4 transition-colors ${
                isSampled
                  ? "bg-ck-bg-2 border-l-4 border-ck-accent"
                  : "hover:bg-ck-bg-1"
              }`}
            >
              {/* Trust Mark & Description */}
              <div className="flex items-center gap-3 min-w-0">
                <span className="font-bold text-sm text-ck-fg-1">
                  {item.signed ? "◆" : item.m === "automation" ? "⊢" : "○"}
                </span>
                <div>
                  <span className="font-sans font-medium text-sm text-ck-fg-1 block truncate">
                    {item.t}
                  </span>
                  <span className="text-[11px] text-ck-fg-mute">
                    by {item.by} · attests {item.ids.join(", ")} · {item.m}
                  </span>
                </div>
              </div>

              {/* Age & Decay Bar */}
              <div className="flex items-center gap-4 flex-shrink-0">
                <span className="text-xs text-ck-fg-2 w-16 text-right">
                  {item.age}
                </span>

                <div
                  className="w-24 h-2 bg-ck-bg-2 border border-ck-hairline-strong overflow-hidden"
                  title="Evidentiary Weight"
                >
                  <div
                    className="h-full bg-ck-fg-1"
                    style={{ width: `${Math.round(weight * 100)}%` }}
                  />
                </div>

                {/* Hash */}
                <span className="w-36 shrink-0 text-right font-mono text-[11px]">
                  {item.signed ? (
                    <Badge
                      variant="outline"
                      className="font-mono text-[10px] text-green-700 dark:text-green-400 whitespace-nowrap tabular-nums"
                    >
                      {item.hash}
                    </Badge>
                  ) : (
                    <span className="text-ck-fg-mute text-[10px] whitespace-nowrap">
                      ○ unsigned
                    </span>
                  )}
                </span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
