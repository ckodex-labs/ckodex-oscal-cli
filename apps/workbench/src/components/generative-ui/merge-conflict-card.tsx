"use client";

import * as React from "react";
import { MergeReport } from "@/lib/oscal-types";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { GitMerge, Check, AlertCircle } from "lucide-react";

interface MergeConflictCardProps {
  report: MergeReport;
  onResolve?: (controlId: string, resolution: "ours" | "theirs") => void;
}

export function MergeConflictCard({
  report,
  onResolve,
}: MergeConflictCardProps) {
  const [resolvedMap, setResolvedMap] = React.useState<
    Record<string, "ours" | "theirs">
  >({});

  const handleChoice = (controlId: string, choice: "ours" | "theirs") => {
    setResolvedMap((prev) => ({ ...prev, [controlId]: choice }));
    if (onResolve) {
      onResolve(controlId, choice);
    }
  };

  return (
    <Card className="my-2 border-ck-hairline-strong bg-ck-bg-1 shadow-[3px_3px_0_var(--ck-fg-1)]">
      <CardHeader className="flex flex-row items-center justify-between pb-2 bg-ck-bg-2/50">
        <div className="flex items-center gap-2">
          <GitMerge className="h-5 w-5 text-accent" />
          <div>
            <CardTitle className="text-base font-serif text-ck-fg-1">
              3-Way GitOps AST Synchronization &amp; Merge
            </CardTitle>
            <p className="font-mono text-[11px] text-ck-fg-mute">
              Strategy:{" "}
              <span className="font-semibold text-ck-fg-1">
                {report.strategy}
              </span>{" "}
              · {report.controls_merged} controls processed
            </p>
          </div>
        </div>

        <Badge
          variant={report.is_clean ? "success" : "destructive"}
          className="text-xs"
        >
          {report.is_clean
            ? "CLEAN MERGE"
            : `${report.conflicts.length} CONFLICTS`}
        </Badge>
      </CardHeader>

      <CardContent className="space-y-3 pt-3 font-mono text-xs text-ck-fg-2">
        {/* Merged Statistics Summary */}
        <div className="grid grid-cols-4 gap-1.5 border border-ck-hairline p-2 bg-ck-bg-0 text-center text-[10px]">
          <div>
            <span className="text-ck-fg-mute block">Added (Upstream)</span>
            <span className="font-semibold text-ck-fg-1 text-xs">
              {report.added_from_upstream.length}
            </span>
          </div>
          <div>
            <span className="text-ck-fg-mute block">Local Additions</span>
            <span className="font-semibold text-ck-fg-1 text-xs">
              {report.preserved_local_additions.length}
            </span>
          </div>
          <div>
            <span className="text-ck-fg-mute block">Updated (Upstream)</span>
            <span className="font-semibold text-ck-fg-1 text-xs">
              {report.updated_from_upstream.length}
            </span>
          </div>
          <div>
            <span className="text-ck-fg-mute block">Conflicts</span>
            <span className="font-semibold text-red-600 dark:text-red-400 text-xs">
              {report.conflicts.length}
            </span>
          </div>
        </div>

        {/* Conflicts List */}
        {report.conflicts.length > 0 && (
          <div className="space-y-3">
            <div className="text-[11px] font-semibold text-ck-fg-1 flex items-center gap-1.5">
              <AlertCircle className="h-3.5 w-3.5 text-accent" />
              AST Control Merge Conflicts:
            </div>

            {report.conflicts.map((c) => {
              const currentChoice = resolvedMap[c.control_id];
              return (
                <div
                  key={c.control_id}
                  className="border border-ck-hairline-strong bg-ck-bg-0 p-3 space-y-2"
                >
                  <div className="flex items-center justify-between border-b border-ck-hairline pb-1.5">
                    <span className="font-semibold text-ck-fg-1">
                      Control: <span className="ck-hash">{c.control_id}</span> (
                      {c.field})
                    </span>
                    {currentChoice && (
                      <Badge variant="success" className="text-[10px]">
                        <Check className="h-2.5 w-2.5 mr-1" />
                        Resolved (
                        {currentChoice === "ours" ? "Local" : "Upstream"})
                      </Badge>
                    )}
                  </div>

                  <div className="grid grid-cols-2 gap-2 text-[11px]">
                    <div className="border-r border-ck-hairline pr-2">
                      <div className="text-[10px] uppercase font-semibold text-ck-fg-mute mb-1">
                        LOCAL (YOUR WORKSPACE)
                      </div>
                      <div className="bg-ck-bg-1 p-2 border border-ck-hairline text-ck-fg-1 text-[11px] line-clamp-3">
                        {c.local_summary}
                      </div>
                    </div>

                    <div className="pl-1">
                      <div className="text-[10px] uppercase font-semibold text-ck-fg-mute mb-1">
                        UPSTREAM (INCOMING)
                      </div>
                      <div className="bg-ck-bg-1 p-2 border border-ck-hairline text-ck-fg-1 text-[11px] line-clamp-3">
                        {c.upstream_summary}
                      </div>
                    </div>
                  </div>

                  <div className="flex justify-end gap-2 pt-1">
                    <Button
                      size="sm"
                      variant={currentChoice === "ours" ? "default" : "outline"}
                      onClick={() => handleChoice(c.control_id, "ours")}
                    >
                      Keep Local
                    </Button>
                    <Button
                      size="sm"
                      variant={
                        currentChoice === "theirs" ? "default" : "outline"
                      }
                      onClick={() => handleChoice(c.control_id, "theirs")}
                    >
                      Accept Upstream
                    </Button>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
