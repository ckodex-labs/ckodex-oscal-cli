"use client";

import * as React from "react";
import { BlastRadiusReport } from "@/lib/oscal-types";
import { Badge } from "@/components/ui/badge";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { ShieldAlert, GitFork, ArrowDownRight } from "lucide-react";

interface BlastRadiusCardProps {
  report: BlastRadiusReport;
}

export function BlastRadiusCard({ report }: BlastRadiusCardProps) {
  return (
    <Card className="my-2 border-ck-hairline-strong bg-ck-bg-1 shadow-[3px_3px_0_var(--ck-fg-1)]">
      <CardHeader className="flex flex-row items-center justify-between pb-2 bg-ck-bg-2/50">
        <div>
          <div className="flex items-center gap-2">
            <ShieldAlert className="h-4 w-4 text-accent" />
            <CardTitle className="text-base font-serif text-ck-fg-1">
              Blast Radius Analysis
            </CardTitle>
          </div>
          <p className="mt-0.5 font-mono text-[11px] text-ck-fg-mute">
            Target:{" "}
            <span className="font-semibold text-ck-fg-1">
              {report.target_id}
            </span>{" "}
            ({report.target_kind})
          </p>
        </div>
        <div className="text-right">
          <Badge
            variant={
              report.risk_exposure_score > 6.0 ? "destructive" : "accent"
            }
            className="text-[11px]"
          >
            Risk {report.risk_exposure_score.toFixed(1)} / 10.0
          </Badge>
          {report.is_critical_path && (
            <div className="mt-1 font-mono text-[10px] font-semibold text-red-600 dark:text-red-400">
              CRITICAL PATH
            </div>
          )}
        </div>
      </CardHeader>

      <CardContent className="space-y-3 pt-3 font-mono text-xs text-ck-fg-2">
        {/* Metric Summary */}
        <div className="grid grid-cols-2 gap-2 border border-ck-hairline p-2 bg-ck-bg-0">
          <div>
            <span className="text-[10px] text-ck-fg-mute uppercase">
              Direct Dependents
            </span>
            <div className="text-sm font-semibold text-ck-fg-1">
              {report.direct_dependents.length}
            </div>
          </div>
          <div>
            <span className="text-[10px] text-ck-fg-mute uppercase">
              Transitive Dependents
            </span>
            <div className="text-sm font-semibold text-ck-fg-1">
              {report.transitive_dependents.length}
            </div>
          </div>
        </div>

        {/* Affected Controls List */}
        {report.direct_dependents.length > 0 && (
          <div>
            <div className="mb-1 flex items-center gap-1 text-[11px] font-semibold text-ck-fg-1">
              <GitFork className="h-3 w-3" />
              Direct Dependent Controls:
            </div>
            <div className="flex flex-wrap gap-1">
              {report.direct_dependents.map((dep, idx) => {
                const id = typeof dep === "string" ? dep : dep.id;
                const title = typeof dep === "string" ? dep : (dep.title || dep.id);
                return (
                  <span key={id || idx} className="ck-hash text-[11px]" title={title}>
                    {id}
                  </span>
                );
              })}
            </div>
          </div>
        )}

        {/* Downstream Impact Path */}
        {report.downstream_impact_paths && report.downstream_impact_paths.length > 0 && (
          <div className="border-t border-ck-hairline pt-2">
            <div className="mb-1 text-[11px] font-semibold text-ck-fg-1">
              Propagation Trajectory:
            </div>
            <div className="space-y-1">
              {report.downstream_impact_paths.slice(0, 3).map((p, idx) => (
                <div
                  key={idx}
                  className="flex items-center gap-1 text-[11px] text-ck-fg-mute"
                >
                  <ArrowDownRight className="h-3 w-3 flex-shrink-0 text-accent" />
                  <span>{p.join(" → ")}</span>
                </div>
              ))}
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
