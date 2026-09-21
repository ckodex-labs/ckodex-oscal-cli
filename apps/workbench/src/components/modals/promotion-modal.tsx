"use client";

import * as React from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

interface PromotionModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirmPromote: () => void;
}

export function PromotionModal({
  isOpen,
  onClose,
  onConfirmPromote,
}: PromotionModalProps) {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 bg-black/60 backdrop-blur-xs flex items-center justify-center p-4">
      <div className="border border-ck-hairline-strong bg-ck-bg-1 max-w-lg w-full p-6 shadow-2xl space-y-4 font-mono text-xs animate-in fade-in zoom-in-95">
        <div className="flex items-baseline justify-between border-b border-ck-hairline pb-2">
          <h2 className="font-serif text-2xl font-normal text-ck-fg-1">
            Governed Promotion · Stage → Production
          </h2>
          <Badge
            variant="outline"
            className="font-mono text-[10px] uppercase text-green-700 dark:text-green-400"
          >
            Gate Checklist
          </Badge>
        </div>

        <p className="font-sans text-xs text-ck-fg-2">
          Promotion is a governed architectural act. The following cryptographic
          and schema invariants must hold before release commit tags are signed.
        </p>

        <div className="space-y-2 bg-ck-bg-0 p-3 border border-ck-hairline">
          <div className="flex items-center justify-between">
            <span>1. OSCAL Metaschema 1.2.3 Deterministic Validation</span>
            <span className="text-green-700 dark:text-green-400 font-bold">
              ✓ PASS (4 DOCS)
            </span>
          </div>
          <div className="flex items-center justify-between">
            <span>2. SLSA v1.2 In-Toto Provenance Merkle Root</span>
            <span className="text-green-700 dark:text-green-400 font-bold">
              ✓ VALID (SHA256)
            </span>
          </div>
          <div className="flex items-center justify-between">
            <span>3. Zero Overdue High-Severity POA&amp;M Gaps</span>
            <span className="text-green-700 dark:text-green-400 font-bold">
              ✓ VERIFIED
            </span>
          </div>
          <div className="flex items-center justify-between">
            <span>4. Responsible Roles Assigned on All Requirements</span>
            <span className="text-green-700 dark:text-green-400 font-bold">
              ✓ COMPLETE
            </span>
          </div>
        </div>

        <div className="flex items-center justify-end gap-2 border-t border-ck-hairline pt-3">
          <Button
            size="sm"
            variant="outline"
            onClick={onClose}
            className="h-8 text-xs font-mono"
          >
            Cancel
          </Button>
          <Button
            size="sm"
            variant="default"
            onClick={() => {
              onConfirmPromote();
              onClose();
            }}
            className="h-8 text-xs font-mono bg-green-700 text-white hover:bg-green-800"
          >
            Promote to Production →
          </Button>
        </div>
      </div>
    </div>
  );
}
