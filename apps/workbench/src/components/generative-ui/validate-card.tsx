"use client";

import * as React from "react";
import type { Provenance } from "@/lib/provenance";
import { StateBadge } from "@/components/kit";
import { AnswerCard } from "./answer-card";
import { countByLevel, type ValidateReport } from "./engine-types";

/** Summary of `mizan validate <doc>`; shows the first few diagnostics. */
export function ValidateCard({
  report,
  provenance,
  exitCode,
  limit = 4,
}: {
  report: ValidateReport;
  provenance: Provenance;
  exitCode: number | null;
  limit?: number;
}) {
  const diags = report.diagnostics ?? [];
  const c = countByLevel(diags);
  return (
    <AnswerCard
      title={<span className="font-mono text-xs">{report.file}</span>}
      provenance={provenance}
      actions={
        report.is_valid ? <StateBadge tone="pos">Valid</StateBadge> : <StateBadge tone="neg">Invalid</StateBadge>
      }
    >
      <p className="text-xs text-ck-fg-3">
        schema {report.schema_valid === undefined ? "unknown" : report.schema_valid ? "valid" : "invalid"}
        {" / "}constraints{" "}
        {report.constraints_valid === undefined ? "unknown" : report.constraints_valid ? "valid" : "invalid"}
        {" / "}
        <span className="font-mono">{c.error}</span> errors, <span className="font-mono">{c.warning}</span> warnings
        {exitCode !== null && (
          <>
            {" / "}exit <span className="font-mono">{exitCode}</span>
          </>
        )}
      </p>
      {diags.length > 0 && (
        <ul className="space-y-1">
          {diags.slice(0, limit).map((d, i) => (
            <li key={i} className="min-w-0 text-xs">
              <span className={d.level.toLowerCase() === "error" ? "text-ck-neg" : "text-ck-warn"}>{d.level}</span>{" "}
              <span className="text-ck-fg-1">{d.message}</span>
              {d.path && <span className="block break-all font-mono text-2xs text-ck-fg-mute">{d.path}</span>}
            </li>
          ))}
          {diags.length > limit && (
            <li className="text-xs text-ck-fg-mute">
              {diags.length - limit} more in the Inspector (key 0).
            </li>
          )}
        </ul>
      )}
    </AnswerCard>
  );
}
