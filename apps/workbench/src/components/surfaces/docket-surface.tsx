"use client";

import * as React from "react";
import { PoamItem } from "@/lib/atlas-data";
import { Badge } from "@/components/ui/badge";

interface DocketSurfaceProps {
  poams: PoamItem[];
  onSelectPoamControl?: (controlId: string) => void;
}

export function DocketSurface({
  poams,
  onSelectPoamControl,
}: DocketSurfaceProps) {
  const [sortBy, setSortBy] = React.useState<"pressure" | "deadline">(
    "pressure",
  );

  const sortedPoams = [...poams].sort((a, b) => {
    if (sortBy === "pressure") {
      return (b.pressure || 0) - (a.pressure || 0);
    }
    return a.dueN - b.dueN;
  });

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex flex-wrap items-baseline justify-between border-b border-ck-hairline pb-2 gap-2">
        <div className="flex items-baseline gap-3 min-w-0">
          <h1 className="font-serif text-2xl font-normal tracking-tight text-ck-fg-1 whitespace-nowrap shrink-0">
            The Docket
          </h1>
          <span className="text-xs text-ck-fg-mute font-mono hidden md:inline">
            The one honest queue. Sorted by pressure, deadline, and graph blast
            radius.
          </span>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          <Badge
            variant="outline"
            className="font-mono text-[10px] uppercase text-accent font-semibold whitespace-nowrap shrink-0"
          >
            {poams.filter((p) => p.dueN < 0).length} Overdue Past SLA
          </Badge>
        </div>
      </div>

      {/* Toolbar */}
      <div className="flex items-center justify-between border border-ck-hairline bg-ck-bg-1 p-2 shadow-sm font-mono text-xs">
        <span className="text-ck-fg-2 text-xs">
          {poams.length} open items · 2 owners past SLA
        </span>

        <div className="flex items-center border border-ck-hairline bg-ck-bg-0 p-0.5">
          <button
            onClick={() => setSortBy("pressure")}
            className={`px-2 py-1 text-[11px] font-mono transition-colors ${
              sortBy === "pressure"
                ? "bg-ck-fg-1 text-ck-bg-0"
                : "text-ck-fg-mute"
            }`}
          >
            pressure
          </button>
          <button
            onClick={() => setSortBy("deadline")}
            className={`px-2 py-1 text-[11px] font-mono transition-colors ${
              sortBy === "deadline"
                ? "bg-ck-fg-1 text-ck-bg-0"
                : "text-ck-fg-mute"
            }`}
          >
            deadline
          </button>
        </div>
      </div>

      {/* POA&M Table */}
      <div className="border border-ck-hairline-strong bg-ck-bg-1 overflow-x-auto shadow-sm font-mono text-xs">
        <table className="w-full text-left border-collapse">
          <thead className="bg-ck-bg-2 border-b border-ck-hairline text-ck-fg-mute uppercase text-[10px]">
            <tr>
              <th className="p-2.5">Item</th>
              <th className="p-2.5">Finding Description</th>
              <th className="p-2.5">Owner</th>
              <th className="p-2.5">Deadline</th>
              <th className="p-2.5">Age</th>
              <th className="p-2.5">Blast Radius</th>
              <th className="p-2.5 text-right">Pressure</th>
            </tr>
          </thead>
          <tbody>
            {sortedPoams.map((p) => {
              const isOverdue = p.dueN < 0;
              const pressurePct = Math.round((p.pressure || 0) * 100);

              return (
                <tr
                  key={p.id}
                  onClick={() => {
                    if (onSelectPoamControl && p.ids[0]) {
                      onSelectPoamControl(p.ids[0]);
                    }
                  }}
                  className="border-b border-ck-hairline cursor-pointer hover:bg-ck-bg-2 transition-colors"
                  title="Click to inspect primary affected control on The Atlas"
                >
                  <td className="p-2.5 font-bold text-ck-fg-1">{p.id}</td>
                  <td className="p-2.5 font-sans text-sm text-ck-fg-1">
                    {p.t}
                    <span className="font-mono text-[10px] text-ck-fg-mute block">
                      Controls: {p.ids.join(", ")}
                    </span>
                  </td>
                  <td className="p-2.5 text-ck-fg-2">{p.own}</td>
                  <td className="p-2.5">
                    {isOverdue ? (
                      <span className="bg-ck-fg-1 text-ck-bg-0 px-1.5 py-0.5 font-bold text-[10px]">
                        {p.due}
                      </span>
                    ) : (
                      <span className="text-ck-fg-2">{p.due}</span>
                    )}
                  </td>
                  <td className="p-2.5 text-ck-fg-mute">{p.age} d</td>
                  <td className="p-2.5 text-ck-fg-1 font-semibold">
                    {p.blast} impls
                  </td>
                  <td className="p-2.5 text-right">
                    <div className="flex items-center justify-end gap-2">
                      <div className="w-16 h-2 bg-ck-bg-0 border border-ck-hairline overflow-hidden">
                        <div
                          className={`h-full ${isOverdue ? "bg-accent" : "bg-ck-fg-1"}`}
                          style={{ width: `${pressurePct}%` }}
                        />
                      </div>
                      <span className="font-bold text-xs">{pressurePct}%</span>
                    </div>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      <p className="font-mono text-[11px] text-ck-fg-mute">
        Blast radius = dependent implementations of every control the gap
        undermines, computed from the OSCAL graph. Click any row to focus it on
        The Atlas.
      </p>
    </div>
  );
}
