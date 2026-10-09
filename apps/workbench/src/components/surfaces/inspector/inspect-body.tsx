"use client";

import * as React from "react";
import { StatGrid, StatTile } from "@/components/kit";
import type { InspectReport } from "@/components/generative-ui/engine-types";

function shortDate(s?: string | null) {
  if (!s) return "none";
  const d = new Date(s);
  return Number.isNaN(d.getTime()) ? s : d.toISOString().slice(0, 10);
}

/** Body of an inspect result: metadata, counts, controls by family. */
export function InspectBody({ report }: { report: InspectReport }) {
  const st = report.stats ?? {};
  const fams = Object.entries(st.controls_by_family ?? {});
  return (
    <div className="space-y-4">
      <div className="space-y-1">
        <p className="text-base text-ck-fg-1">{report.title ?? "Untitled document"}</p>
        <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm">
          <dt className="text-ck-fg-mute">Kind</dt>
          <dd className="text-ck-fg-2">{report.kind}</dd>
          <dt className="text-ck-fg-mute">File</dt>
          <dd className="min-w-0 break-all font-mono text-xs text-ck-fg-2">{report.file}</dd>
          <dt className="text-ck-fg-mute">Version</dt>
          <dd className="font-mono text-xs text-ck-fg-2">
            {report.version ?? "none"} <span className="text-ck-fg-mute">(OSCAL {report.oscal_version ?? "unknown"})</span>
          </dd>
          <dt className="text-ck-fg-mute">UUID</dt>
          <dd className="min-w-0 break-all font-mono text-xs text-ck-fg-2">{report.uuid ?? "none"}</dd>
          <dt className="text-ck-fg-mute">Last modified</dt>
          <dd className="font-mono text-xs text-ck-fg-2">{shortDate(report.last_modified)}</dd>
        </dl>
      </div>
      <StatGrid>
        <StatTile label="Controls" value={st.total_controls ?? 0} />
        <StatTile label="Groups" value={st.total_groups ?? 0} />
        <StatTile label="Parameters" value={st.total_params ?? 0} />
        <StatTile label="Roles" value={st.total_roles ?? 0} />
        <StatTile label="Parties" value={st.total_parties ?? 0} />
        <StatTile label="Components" value={st.total_components ?? 0} />
      </StatGrid>
      {fams.length > 0 && (
        <div>
          <p className="ck-eyebrow mb-1.5">Controls by family</p>
          <ul className="flex flex-wrap gap-1.5">
            {fams.map(([f, n]) => (
              <li
                key={f}
                className="inline-flex items-baseline gap-1.5 rounded-sm border border-ck-hairline bg-ck-bg-0 px-1.5 py-0.5 text-xs"
              >
                <span className="font-mono text-ck-fg-1">{f}</span>
                <span className="font-mono text-ck-fg-mute ck-num">{n}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
