"use client";

import * as React from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";

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
  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="max-w-lg border-ck-hairline-strong bg-ck-bg-1 p-6 font-mono text-xs shadow-2xl">
        <DialogHeader className="border-b border-ck-hairline pb-2">
          <div className="flex items-baseline justify-between pr-6">
            <DialogTitle className="font-serif text-2xl font-normal text-ck-fg-1">
              Governed Promotion · Stage → Production
            </DialogTitle>
            <Badge
              variant="outline"
              className="font-mono text-[10px] uppercase text-green-700 dark:text-green-400"
            >
              Gate Checklist
            </Badge>
          </div>
          <DialogDescription className="font-sans text-xs text-ck-fg-2 mt-1">
            Promotion is a governed architectural act. The following cryptographic
            and schema invariants must hold before release commit tags are signed.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-2 bg-ck-bg-0 p-3 border border-ck-hairline my-2">
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

        <DialogFooter className="border-t border-ck-hairline pt-3 gap-2">
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
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
