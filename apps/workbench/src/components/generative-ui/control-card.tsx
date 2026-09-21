"use client";

import * as React from "react";
import { ControlDetail } from "@/lib/oscal-types";
import { Badge } from "@/components/ui/badge";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Sliders } from "lucide-react";

interface ControlCardProps {
  control: ControlDetail;
}

export function ControlCard({ control }: ControlCardProps) {
  return (
    <Card className="my-2 border-ck-hairline-strong bg-ck-bg-1 shadow-[3px_3px_0_var(--ck-fg-1)]">
      <CardHeader className="flex flex-row items-center justify-between pb-2 bg-ck-bg-2/50">
        <div>
          <div className="flex items-center gap-2">
            <Badge
              variant="outline"
              className="font-semibold text-ck-fg-1 bg-ck-bg-0"
            >
              {control.id.toUpperCase()}
            </Badge>
            <CardTitle className="text-base font-serif text-ck-fg-1">
              {control.title}
            </CardTitle>
          </div>
          {control.class && (
            <p className="mt-0.5 font-mono text-[11px] text-ck-fg-mute">
              Family / Baseline: {control.class}
            </p>
          )}
        </div>
      </CardHeader>

      <CardContent className="space-y-3 pt-3 font-mono text-xs text-ck-fg-2">
        {control.statement && (
          <div>
            <div className="mb-1 text-[11px] font-semibold text-ck-fg-1 uppercase tracking-wider">
              Control Statement
            </div>
            <div className="border border-ck-hairline bg-ck-bg-0 p-2.5 leading-relaxed text-ck-fg-1 font-sans text-xs">
              {control.statement}
            </div>
          </div>
        )}

        {control.params && control.params.length > 0 && (
          <div>
            <div className="mb-1 text-[11px] font-semibold text-ck-fg-1 flex items-center gap-1">
              <Sliders className="h-3 w-3 text-accent" />
              Parameters &amp; Values:
            </div>
            <div className="space-y-1">
              {control.params.map((p) => (
                <div
                  key={p.id}
                  className="flex items-center justify-between border border-ck-hairline bg-ck-bg-0 px-2 py-1 text-[11px]"
                >
                  <span className="text-ck-fg-mute">{p.id}</span>
                  <span className="font-semibold text-ck-fg-1">
                    {p.values ? p.values.join(", ") : p.label || "(unset)"}
                  </span>
                </div>
              ))}
            </div>
          </div>
        )}

        {control.guidance && (
          <div className="border-t border-ck-hairline pt-2 text-[11px] text-ck-fg-mute">
            <span className="font-semibold text-ck-fg-1">Guidance: </span>
            <span className="font-sans">{control.guidance}</span>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
