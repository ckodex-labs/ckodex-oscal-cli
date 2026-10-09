"use client";

import * as React from "react";
import { cn } from "@/lib/utils";
import type { Provenance } from "@/lib/provenance";
import { ProvenanceTag } from "@/components/kit";

/**
 * Compact container for a chat answer. Like the kit Panel it cannot be
 * rendered without a provenance, but it is sized for the 288px drawer.
 */
export function AnswerCard({
  title,
  provenance,
  actions,
  children,
  className,
}: {
  title: React.ReactNode;
  provenance: Provenance;
  actions?: React.ReactNode;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <section
      data-panel
      className={cn(
        "min-w-0 rounded-md border border-ck-hairline-strong bg-ck-bg-0",
        provenance.kind === "fixture" && "border-dashed",
        className,
      )}
    >
      <div className="flex flex-wrap items-center justify-between gap-2 border-b border-ck-hairline px-3 py-1.5">
        <h3 className="min-w-0 truncate text-sm font-semibold text-ck-fg-1">{title}</h3>
        <div className="flex flex-wrap items-center gap-2">
          {actions}
          <ProvenanceTag provenance={provenance} />
        </div>
      </div>
      <div className="space-y-2 px-3 py-2 text-sm text-ck-fg-2">{children}</div>
    </section>
  );
}

/** Label/value row used inside answer cards. */
export function Fact({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex min-w-0 flex-wrap items-baseline gap-x-2">
      <dt className="text-xs text-ck-fg-mute">{label}</dt>
      <dd className="min-w-0 break-words text-ck-fg-1">{children}</dd>
    </div>
  );
}
