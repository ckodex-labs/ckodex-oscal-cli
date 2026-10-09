"use client";

import * as React from "react";
import type { Provenance } from "@/lib/provenance";
import type { FedrampReport } from "@/lib/oscal-types";
import { StateBadge, type Tone } from "@/components/kit";
import { AnswerCard, Fact } from "./answer-card";

function severityTone(s: string): Tone {
  const l = s.toLowerCase();
  if (l === "high" || l === "critical") return "neg";
  if (l === "medium") return "warn";
  return "neutral";
}

/** Output of `mizan fedramp validate <doc> --baseline <b>`. */
export function FedrampBadgeCard({
  report,
  provenance,
}: {
  report: FedrampReport;
  provenance: Provenance;
}) {
  const findings = report.findings ?? [];
  const compliant = report.is_compliant ?? report.passed;
  const total = report.total_rules_checked ?? report.rule_count_evaluated;
  const failed = report.failed_rules ?? report.violation_count;
  const findingsCount =
    (report as FedrampReport & { findings_count?: number }).findings_count ?? findings.length;

  return (
    <AnswerCard
      title={`FedRAMP ${report.baseline} check`}
      provenance={provenance}
      actions={
        compliant === undefined ? (
          <StateBadge tone="unk">Unknown</StateBadge>
        ) : compliant ? (
          <StateBadge tone="pos">Compliant</StateBadge>
        ) : (
          <StateBadge tone="neg">Non-compliant</StateBadge>
        )
      }
    >
      <dl className="grid grid-cols-1 gap-1 sm:grid-cols-2">
        <Fact label="Document">
          <span className="font-mono text-xs">{report.file ?? "unknown"}</span>
        </Fact>
        <Fact label="Result">
          <span className="font-mono ck-num">{failed ?? "unknown"}</span> of{" "}
          <span className="font-mono ck-num">{total ?? "unknown"}</span> rules failed (
          <span className="font-mono ck-num">{findingsCount}</span> finding{findingsCount === 1 ? "" : "s"})
        </Fact>
      </dl>
      <ul className="space-y-1">
        {findings.map((f, i) => (
          <li key={`${f.rule_id}-${i}`} className="flex min-w-0 flex-wrap items-baseline gap-x-2 text-xs">
            <StateBadge tone={severityTone(f.severity)} glyph={false}>
              {f.severity}
            </StateBadge>
            <span className="font-mono text-ck-fg-mute">{f.rule_id}</span>
            <span className="text-ck-fg-1">{f.title}</span>
          </li>
        ))}
      </ul>
    </AnswerCard>
  );
}
