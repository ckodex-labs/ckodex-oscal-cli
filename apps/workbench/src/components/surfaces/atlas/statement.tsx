"use client";

import * as React from "react";
import { cn } from "@/lib/utils";
import type { AtlasParam } from "@/lib/atlas-types";
import { isTruncated, parseStatement } from "./model";

/**
 * Render a projected control statement with parameter insertions as inline
 * chips. `renderParam` lets the caller make chips interactive (Composer);
 * by default chips are static and show the catalog param label.
 */
export function StatementText({
  statement,
  params,
  renderParam,
  className,
}: {
  statement: string;
  params: AtlasParam[];
  renderParam?: (p: AtlasParam, occurrence: number) => React.ReactNode;
  className?: string;
}) {
  const segments = React.useMemo(
    () => parseStatement(statement, params),
    [statement, params],
  );
  let n = 0;
  return (
    <div className={cn("space-y-2", className)}>
      <p className="text-base leading-7 text-ck-fg-1 break-words">
        {segments.map((s, i) =>
          s.kind === "text" ? (
            <React.Fragment key={i}>{s.text}</React.Fragment>
          ) : (
            <React.Fragment key={i}>
              {renderParam ? (
                renderParam(s.param, n++)
              ) : (
                <ParamChip param={s.param} />
              )}
            </React.Fragment>
          ),
        )}
      </p>
      {isTruncated(statement) && (
        <p className="text-xs text-ck-fg-mute">
          Truncated to 600 characters in the snapshot projection (atlas.json).
          The full statement is in the resolved catalog.
        </p>
      )}
    </div>
  );
}

export function ParamChip({ param }: { param: AtlasParam }) {
  return (
    <span
      title={param.id}
      className="mx-0.5 inline rounded-sm border border-ck-hairline-strong bg-ck-bg-2 px-1 py-px text-sm text-ck-fg-2 [box-decoration-break:clone]"
    >
      {param.label}
    </span>
  );
}
