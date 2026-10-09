"use client";

/**
 * Root Fabric & Identity.
 *
 * Engine coverage (verified against src/cli/fabric.rs and by running the
 * binary): the only `mizan fabric` subcommand that is offline, read-only,
 * deterministic and emits JSON is `fabric spiffe <id>`, a syntactic parse of
 * a SPIFFE ID plus a comparison with a trust domain compiled into the engine.
 * That output is read from the snapshot id `fabric-spiffe-parse` when the
 * snapshot generator captures it. Everything else on this surface is FIXTURE.
 */

import * as React from "react";
import { useSnapshot } from "@/lib/engine";
import { fixture } from "@/lib/provenance";
import {
  DataTable,
  EmptyState,
  PageHeader,
  Panel,
  ReadOnlyNotice,
  StateBadge,
} from "@/components/kit";
import type { SpiffeParseReport } from "@/components/generative-ui/engine-types";

const SPIFFE_SNAPSHOT_ID = "fabric-spiffe-parse";

const AUTHORITY_PATH: { level: string; example: string; note: string }[] = [
  { level: "Root Fabric", example: "root", note: "Origin of all standing authority" },
  { level: "Tenant", example: "tenant-a", note: "Isolation and quota boundary" },
  { level: "Namespace", example: "ns-prod", note: "Grouping inside a tenant" },
  { level: "Workspace", example: "ws-compliance", note: "Team working area" },
  { level: "Plane", example: "control", note: "Control or data plane" },
  { level: "Environment", example: "env-prod", note: "Deployment stage" },
  { level: "Project", example: "proj-ssp", note: "Unit that owns documents" },
  { level: "Resource", example: "ssp document", note: "The governed object" },
];

const EXAMPLE_TENANTS = [
  { id: "default", name: "Default local workspace", tier: "Community", quota: 10, jurisdiction: "us" },
  { id: "tenant-a", name: "Example Tenant A", tier: "Enterprise", quota: 100, jurisdiction: "us" },
  { id: "tenant-b", name: "Example Tenant B", tier: "Enterprise", quota: 500, jurisdiction: "ca" },
];

const EXAMPLE_WORKLOADS = [
  { id: "spiffe://example.org/ns/prod/sa/auditor", ns: "prod", sa: "auditor" },
  { id: "spiffe://example.org/ns/prod/sa/pipeline", ns: "prod", sa: "pipeline" },
  { id: "spiffe://example.org/ns/dev/sa/author", ns: "dev", sa: "author" },
];

const ENGINE_COVERAGE: { cmd: string; tone: "pos" | "warn" | "neutral"; status: string; why: string }[] = [
  {
    cmd: "fabric spiffe <id>",
    tone: "pos",
    status: "Shown",
    why: "Offline, deterministic JSON. Parses the ID and compares its trust domain with one compiled into the engine. It does not verify an SVID or contact SPIRE.",
  },
  {
    cmd: "fabric status",
    tone: "warn",
    status: "Not shown",
    why: "Prints fixed strings (OPERATIONAL, ACTIVE, READY) without checking anything, plus the local tenant registry.",
  },
  {
    cmd: "fabric tenant list",
    tone: "warn",
    status: "Not shown",
    why: "Reads a local tenant registry that differs per machine and is printed in no stable order.",
  },
  {
    cmd: "fabric datastore stats | list",
    tone: "warn",
    status: "Not shown",
    why: "Ignores --format json and prints hard-coded rows.",
  },
  {
    cmd: "fabric oidc <token>",
    tone: "neutral",
    status: "Not shown",
    why: "Decodes a JWT against the current clock, so output is not reproducible; prints text only.",
  },
  {
    cmd: "fabric tenant create",
    tone: "neutral",
    status: "Not offered",
    why: "Mutates the local registry; the Workbench bridge is read-only.",
  },
];

function SpiffePanel() {
  const snap = useSnapshot<SpiffeParseReport>(SPIFFE_SNAPSHOT_ID);
  const provenance =
    snap.provenance ??
    fixture(
      "not captured",
      `Snapshot id '${SPIFFE_SNAPSHOT_ID}' is not in the manifest yet.`,
    );
  const r = snap.data;
  return (
    <Panel
      title="SPIFFE ID parse"
      subtitle="Syntactic parse by the engine. No SVID, signature or SPIRE server is involved."
      provenance={provenance}
    >
      {snap.loading ? (
        <p className="text-sm text-ck-fg-3">Loading snapshot.</p>
      ) : !r ? (
        <EmptyState kind="empty" title="No captured fabric output">
          The snapshot does not contain <code className="font-mono text-xs">{SPIFFE_SNAPSHOT_ID}</code>. When the
          snapshot generator runs <code className="font-mono text-xs">mizan --format json fabric spiffe</code>, its
          output appears here.
        </EmptyState>
      ) : (
        <div className="space-y-3">
          <p className="break-all font-mono text-sm text-ck-fg-1">{r.spiffe_id}</p>
          <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1.5 text-sm">
            <dt className="text-ck-fg-mute">Trust domain</dt>
            <dd className="min-w-0 break-all font-mono text-ck-fg-1">{r.trust_domain}</dd>
            <dt className="text-ck-fg-mute">Path</dt>
            <dd className="min-w-0 break-all font-mono text-ck-fg-1">{r.path}</dd>
            <dt className="text-ck-fg-mute">Namespace</dt>
            <dd className="font-mono text-ck-fg-1">{r.namespace ?? "none"}</dd>
            <dt className="text-ck-fg-mute">Service account</dt>
            <dd className="font-mono text-ck-fg-1">{r.service_account ?? "none"}</dd>
            <dt className="text-ck-fg-mute">Trust domain match</dt>
            <dd>
              {r.valid_for_trust_domain ? (
                <StateBadge tone="pos">Matches engine domain</StateBadge>
              ) : (
                <StateBadge tone="neutral">Differs from engine domain</StateBadge>
              )}
            </dd>
          </dl>
          <p className="text-xs text-ck-fg-3">
            &quot;Match&quot; only compares the trust-domain string with the one compiled into the engine. It is not
            identity verification.
          </p>
        </div>
      )}
    </Panel>
  );
}

export function FabricSurface() {
  return (
    <>
      <PageHeader
        eyebrow="Governance"
        title="Root Fabric & Identity"
        description="Authority hierarchy and workload identity. Only the SPIFFE parse comes from the engine; the rest of this page is illustrative and describes no real organization."
      />
      <ReadOnlyNotice />

      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <SpiffePanel />

        <Panel
          title="Authority path"
          subtitle="Every governed object sits under one path. Child authority never exceeds its parent."
          provenance={fixture(
            "authority model",
            "Illustrative path. It would be replaced by an authority graph exported by the engine; no read-only export of that graph exists today.",
          )}
        >
          <ol className="space-y-0">
            {AUTHORITY_PATH.map((a, i) => (
              <li key={a.level} className="flex min-w-0 items-baseline gap-3 border-l border-ck-hairline-strong py-1 pl-3" style={{ marginLeft: Math.min(i, 7) * 6 }}>
                <span className="w-24 shrink-0 text-xs text-ck-fg-mute">{a.level}</span>
                <span className="font-mono text-sm text-ck-fg-1">{a.example}</span>
                <span className="hidden min-w-0 truncate text-xs text-ck-fg-3 sm:inline">{a.note}</span>
              </li>
            ))}
          </ol>
        </Panel>
      </div>

      <Panel
        title="Example tenants"
        subtitle="Shape of a tenant record. Quotas are not enforced by the Workbench."
        provenance={fixture(
          "example tenants",
          "Illustrative rows. A real view would read a tenant registry served by a fabric server; the local `mizan fabric tenant list` registry varies per machine and is not captured.",
        )}
      >
        <DataTable
          caption="Example tenants"
          columns={[
            { key: "id", label: "Tenant id" },
            { key: "name", label: "Display name" },
            { key: "tier", label: "Tier" },
            { key: "quota", label: "Quota (GiB)", numeric: true },
            { key: "jurisdiction", label: "Jurisdiction" },
          ]}
          rows={EXAMPLE_TENANTS.map((t) => ({
            id: <span className="font-mono text-ck-fg-1">{t.id}</span>,
            name: t.name,
            tier: t.tier,
            quota: t.quota,
            jurisdiction: <span className="font-mono">{t.jurisdiction}</span>,
          }))}
        />
      </Panel>

      <Panel
        title="Example workload identities"
        subtitle="SPIFFE IDs a workload might present. None of these are verified."
        provenance={fixture(
          "example workloads",
          "Illustrative IDs under example.org. Real input would be X.509-SVIDs issued by a SPIRE agent; the Workbench does not verify SVIDs.",
        )}
      >
        <DataTable
          caption="Example workload identities"
          columns={[
            { key: "id", label: "SPIFFE ID" },
            { key: "ns", label: "Namespace" },
            { key: "sa", label: "Service account" },
            { key: "state", label: "Verification" },
          ]}
          rows={EXAMPLE_WORKLOADS.map((w) => ({
            id: <span className="break-all font-mono text-xs text-ck-fg-1">{w.id}</span>,
            ns: <span className="font-mono">{w.ns}</span>,
            sa: <span className="font-mono">{w.sa}</span>,
            state: <StateBadge tone="unk">Not verified</StateBadge>,
          }))}
        />
      </Panel>

      <section aria-labelledby="fabric-coverage" className="space-y-2">
        <h2 id="fabric-coverage" className="text-sm font-semibold text-ck-fg-1">
          What the engine&apos;s fabric commands do today
        </h2>
        <ul className="divide-y divide-ck-hairline rounded-lg border border-ck-hairline-strong bg-ck-bg-1">
          {ENGINE_COVERAGE.map((c) => (
            <li key={c.cmd} className="flex min-w-0 flex-col gap-1 px-4 py-2.5 sm:flex-row sm:items-baseline sm:gap-3">
              <code className="shrink-0 font-mono text-xs text-ck-fg-1 sm:w-56">mizan {c.cmd}</code>
              <span className="shrink-0">
                <StateBadge tone={c.tone}>{c.status}</StateBadge>
              </span>
              <span className="min-w-0 text-sm text-ck-fg-3">{c.why}</span>
            </li>
          ))}
        </ul>
      </section>
    </>
  );
}
