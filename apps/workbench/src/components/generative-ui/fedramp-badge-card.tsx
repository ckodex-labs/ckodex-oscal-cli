"use client";

import * as React from "react";
import { FedrampReport } from "@/lib/oscal-types";
import { Badge } from "@/components/ui/badge";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { CheckCircle2, AlertTriangle, XCircle } from "lucide-react";

interface FedrampBadgeCardProps {
  report: FedrampReport;
}

export function FedrampBadgeCard({ report }: FedrampBadgeCardProps) {
  const isPassed = report.passed ?? report.is_compliant ?? false;
  const docKind = report.document_kind || report.kind || "OSCAL Document";
  const violationCount = report.violation_count ?? report.failed_rules ?? report.findings.length;
  const ruleCount = report.rule_count_evaluated ?? report.total_rules_checked ?? ((report.passed_rules ?? 0) + (report.failed_rules ?? report.findings.length));

  return (
    <Card className="my-2 border-ck-hairline-strong bg-ck-bg-1 shadow-[3px_3px_0_var(--ck-fg-1)]">
      <CardHeader className="flex flex-row items-center justify-between pb-2 bg-ck-bg-2/50">
        <div className="flex items-center gap-2">
          {isPassed ? (
            <CheckCircle2 className="h-5 w-5 text-green-600 dark:text-green-400" />
          ) : (
            <XCircle className="h-5 w-5 text-red-600 dark:text-red-400" />
          )}
          <div>
            <CardTitle className="text-base font-serif text-ck-fg-1">
              FedRAMP PMO Baseline Validation
            </CardTitle>
            <p className="font-mono text-[11px] text-ck-fg-mute">
              Baseline:{" "}
              <span className="uppercase font-semibold text-ck-fg-1">
                {report.baseline}
              </span>{" "}
              · {docKind}
            </p>
          </div>
        </div>

        <Badge
          variant={isPassed ? "success" : "destructive"}
          className="text-xs uppercase"
        >
          {isPassed ? "COMPLIANT" : `${violationCount} VIOLATIONS`}
        </Badge>
      </CardHeader>

      <CardContent className="space-y-3 pt-3 font-mono text-xs text-ck-fg-2">
        <div className="flex items-center justify-between border border-ck-hairline p-2 bg-ck-bg-0">
          <span className="text-ck-fg-mute">PMO Rules Evaluated:</span>
          <span className="font-semibold text-ck-fg-1">
            {ruleCount}
          </span>
        </div>

        {report.findings.length > 0 ? (
          <div className="space-y-2">
            <div className="text-[11px] font-semibold text-ck-fg-1">
              Violations Requiring Remediation:
            </div>
            {report.findings.map((f, idx) => (
              <div
                key={idx}
                className="border-l-2 border-red-500 bg-ck-bg-0 p-2 text-[11px] space-y-1"
              >
                <div className="flex items-center justify-between font-semibold text-ck-fg-1">
                  <div className="flex items-center gap-1">
                    <AlertTriangle className="h-3 w-3 text-red-500" />
                    <span>
                      [{f.rule_id}] {f.title}
                    </span>
                  </div>
                  <span className="ck-hash uppercase text-[10px]">
                    {f.severity}
                  </span>
                </div>
                <div className="text-ck-fg-mute">{f.detail}</div>
                <div className="text-[10px] text-ck-fg-mute">
                  Target: <span className="text-ck-fg-1">{f.target}</span>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="text-center py-2 text-green-700 dark:text-green-400 font-semibold text-[11px]">
            All FedRAMP PMO baseline rules and parameters satisfied.
          </div>
        )}
      </CardContent>
    </Card>
  );
}
