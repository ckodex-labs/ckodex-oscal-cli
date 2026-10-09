"use client";

import * as React from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { ALL_SURFACES } from "@/components/shell/app-shell";

interface ShortcutsModalProps {
  isOpen: boolean;
  onClose: () => void;
}

/**
 * Lists exactly the shortcuts implemented in app/page.tsx. Surface keys are
 * read from the shell's surface table so the two cannot drift.
 */
const GLOBAL: { keys: string[]; label: string }[] = [
  { keys: ["?"], label: "Show or hide this list" },
  { keys: ["T"], label: "Cycle theme: Ledger, Vault, HC" },
  { keys: ["L"], label: "Cycle lens" },
  { keys: ["Cmd", "K"], label: "Open or close the Copilot drawer (Ctrl+K on Windows and Linux)" },
  { keys: ["Esc"], label: "Close this list and the Copilot drawer" },
];

function Kbd({ children }: { children: React.ReactNode }) {
  return (
    <kbd className="inline-flex h-6 min-w-[24px] items-center justify-center rounded-sm border border-ck-hairline-strong bg-ck-bg-0 px-1.5 font-mono text-xs text-ck-fg-1">
      {children}
    </kbd>
  );
}

export function ShortcutsModal({ isOpen, onClose }: ShortcutsModalProps) {
  const surfaces = [...ALL_SURFACES].sort((a, b) => {
    const ka = a.key === "0" ? 10 : Number(a.key);
    const kb = b.key === "0" ? 10 : Number(b.key);
    return ka - kb;
  });
  return (
    <Dialog.Root open={isOpen} onOpenChange={(o) => !o && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-50 bg-black/50" />
        <Dialog.Content className="fixed left-1/2 top-1/2 z-50 flex max-h-[min(640px,90vh)] w-[min(560px,calc(100vw-24px))] -translate-x-1/2 -translate-y-1/2 flex-col rounded-lg border border-ck-hairline-strong bg-ck-bg-1 text-ck-fg-1 shadow-xl">
          <div className="flex items-start justify-between gap-3 border-b border-ck-hairline px-4 py-3">
            <div className="min-w-0">
              <Dialog.Title className="text-base font-semibold">Keyboard shortcuts</Dialog.Title>
              <Dialog.Description className="text-xs text-ck-fg-3">
                Single-key shortcuts are ignored while typing in a text field.
              </Dialog.Description>
            </div>
            <Dialog.Close className="shrink-0 rounded-md border border-ck-hairline-strong bg-ck-bg-0 px-2 py-0.5 text-xs text-ck-fg-3 hover:text-ck-fg-1">
              Close
            </Dialog.Close>
          </div>
          <div className="min-h-0 overflow-y-auto px-4 py-3">
            <h3 className="ck-eyebrow mb-1.5">Surfaces</h3>
            <ul className="mb-4 grid grid-cols-1 gap-x-6 gap-y-1 sm:grid-cols-2">
              {surfaces.map((s) => (
                <li key={s.id} className="flex min-w-0 items-center gap-2 text-sm text-ck-fg-2">
                  <Kbd>{s.key}</Kbd>
                  <span className="min-w-0 truncate">{s.label}</span>
                </li>
              ))}
            </ul>
            <h3 className="ck-eyebrow mb-1.5">Global</h3>
            <ul className="space-y-1.5">
              {GLOBAL.map((g) => (
                <li key={g.label} className="flex min-w-0 items-start gap-2 text-sm text-ck-fg-2">
                  <span className="flex shrink-0 items-center gap-1">
                    {g.keys.map((k) => (
                      <Kbd key={k}>{k}</Kbd>
                    ))}
                  </span>
                  <span className="min-w-0 pt-0.5">{g.label}</span>
                </li>
              ))}
            </ul>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
