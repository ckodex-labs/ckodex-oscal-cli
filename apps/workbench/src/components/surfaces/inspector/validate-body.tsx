"use client";

import * as React from "react";
import { DataTable, EmptyState, StateBadge } from "@/components/kit";
import { countByLevel, type ValidateReport } from "@/components/generative-ui/engine-types";

function validity(v: boolean | undefined) {
  if (v === undefined) return <StateBadge tone="unk">Unknown</StateBadge>;
  return v ? <StateBadge tone="pos">Valid</StateBadge> : <StateBadge tone="neg">Invalid</StateBadge>;
}

/**
 * Validate result: verdict row plus a scrollable diagnostics table.
 * Render inside a Panel with bodyClassName="p-0".
 */
export function ValidateBody({ report, exitCode }: { report: ValidateReport; exitCode: number | null }) {
  const diags = report.diagnostics ?? [];
  const c = countByLevel(diags);
  return (
    <div>
      <div className="flex flex-wrap items-center gap-x-4 gap-y-2 px-4 py-3 text-sm">
        <span className="inline-flex items-center gap-1.5">
          <span className="text-ck-fg-mute">Overall</span>
          {validity(report.is_valid)}
        </span>
        <span className="inline-flex items-center gap-1.5">
          <span className="text-ck-fg-mute">Schema</span>
          {validity(report.schema_valid)}
        </span>
        <span className="inline-flex items-center gap-1.5">
          <span className="text-ck-fg-mute">Constraints</span>
          {validity(report.constraints_valid)}
        </span>
        <span className="text-ck-fg-3">
          <span className="font-mono text-ck-fg-1">{c.error}</span> errors,{" "}
          <span className="font-mono text-ck-fg-1">{c.warning}</span> warnings
          {c.other > 0 && (
            <>
              , <span className="font-mono text-ck-fg-1">{c.other}</span> other
            </>
          )}
        </span>
        {exitCode !== null && (
          <span className="text-ck-fg-3">
            exit <span className="font-mono text-ck-fg-1">{exitCode}</span>
          </span>
        )}
      </div>
      <div className="max-h-[360px] overflow-y-auto border-t border-ck-hairline px-4 pb-2">
        <DataTable
          caption="Validation diagnostics"
          columns={[
            { key: "level", label: "Level", className: "w-20" },
            { key: "code", label: "Code" },
            { key: "message", label: "Message" },
          ]}
          rows={diags.map((d) => ({
            level:
              d.level.toLowerCase() === "error" ? (
                <StateBadge tone="neg">{d.level}</StateBadge>
              ) : d.level.toLowerCase() === "warning" ? (
                <StateBadge tone="warn">{d.level}</StateBadge>
              ) : (
                <StateBadge tone="info">{d.level}</StateBadge>
              ),
            code: <span className="font-mono text-xs text-ck-fg-2">{d.code ?? "none"}</span>,
            message: (
              <div className="min-w-0">
                <p className="text-ck-fg-1">{d.message}</p>
                {d.path && <p className="break-all font-mono text-2xs text-ck-fg-mute">{d.path}</p>}
              </div>
            ),
          }))}
          empty={
            <div className="py-3">
              <EmptyState kind="empty" title="No diagnostics">
                The engine reported no errors or warnings for this document.
              </EmptyState>
            </div>
          }
        />
      </div>
    </div>
  );
}
