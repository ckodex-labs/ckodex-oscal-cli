"use client";

import * as React from "react";
import { Badge } from "@/components/ui/badge";

export interface ReceiptEntry {
  ev: string;
  d: string;
  hash: string;
  type: "observed" | "signed" | "quarantined" | "derived";
}

interface AtlasEvidenceMarginProps {
  isOpen: boolean;
  onToggle: () => void;
  receipts: ReceiptEntry[];
}

export function AtlasEvidenceMargin({
  isOpen,
  onToggle,
  receipts,
}: AtlasEvidenceMarginProps) {
  const [timeMachineIndex, setTimeMachineIndex] = React.useState(
    receipts.length - 1,
  );

  if (!isOpen) {
    return (
      <aside className="w-10 border-l border-ck-hairline-strong bg-ck-bg-1 flex flex-col items-center py-3 select-none">
        <button
          type="button"
          onClick={onToggle}
          className="text-ck-fg-mute hover:text-ck-fg-1 p-1"
          title="Expand Evidence Margin"
        >
          <span className="font-mono text-xs font-bold [writing-mode:vertical-lr] tracking-widest uppercase">
            Evidence ({receipts.length}) ◀
          </span>
        </button>
      </aside>
    );
  }

  const activeReceipts = receipts.slice(0, timeMachineIndex + 1);

  return (
    <aside className="w-80 border-l border-ck-hairline-strong bg-ck-bg-1 flex flex-col justify-between p-3 font-mono text-xs shadow-sm">
      <div className="space-y-3">
        <div className="flex items-center justify-between border-b border-ck-hairline pb-2">
          <span className="text-[10px] uppercase font-bold text-ck-fg-mute">
            Live Evidence Stream
          </span>
          <button
            type="button"
            onClick={onToggle}
            className="text-ck-fg-mute hover:text-ck-fg-1 text-xs"
            title="Collapse Margin"
          >
            ▶
          </button>
        </div>

        {/* Receipt Stream */}
        <div className="space-y-2 max-h-[500px] overflow-y-auto pr-1">
          {activeReceipts.map((rc, idx) => {
            let symbol = "⊢";
            let color = "text-ck-fg-1";
            if (rc.type === "signed") {
              symbol = "◆";
              color = "text-green-700 dark:text-green-400";
            } else if (rc.type === "quarantined") {
              symbol = "⊘";
              color = "text-red-700 dark:text-red-400";
            } else if (rc.type === "derived") {
              symbol = "⇝";
              color = "text-ck-accent";
            }

            return (
              <div
                key={idx}
                className="border border-ck-hairline bg-ck-bg-0 p-2 space-y-1 shadow-sm text-[11px]"
              >
                <div className="flex items-center justify-between">
                  <span className={`font-bold ${color}`}>
                    {symbol} {rc.type}
                  </span>
                  <span className="text-ck-fg-mute text-[10px]">{rc.d}</span>
                </div>
                <div className="font-sans text-xs text-ck-fg-1 truncate">
                  {rc.ev}
                </div>
                <div className="text-[10px] text-ck-fg-mute font-mono truncate">
                  {rc.hash}
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Time Machine Scrubber */}
      <div className="border-t border-ck-hairline pt-3 space-y-2">
        <div className="flex items-center justify-between">
          <span className="text-[10px] uppercase font-bold text-ck-fg-mute">
            Time Machine
          </span>
          <Badge variant="outline" className="text-[9px] font-mono">
            {timeMachineIndex + 1} / {receipts.length} Receipts
          </Badge>
        </div>
        <input
          type="range"
          min="0"
          max={receipts.length - 1}
          value={timeMachineIndex}
          onChange={(e) => setTimeMachineIndex(parseInt(e.target.value, 10))}
          className="w-full accent-ck-fg-1 cursor-pointer"
        />
        <span className="text-[10px] text-ck-fg-mute block">
          Replay timeline cursor back to any historical ledger state
        </span>
      </div>
    </aside>
  );
}
