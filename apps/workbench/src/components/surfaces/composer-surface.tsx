"use client";

import * as React from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

interface ComposerSurfaceProps {
  selectedControlId: string;
  onSelectControl: (id: string) => void;
  onPublishControl?: (id: string) => void;
}

interface ControlConfig {
  id: string;
  title: string;
  family: string;
  baseline: string;
  version: string;
  lastModified: string;
  params: Record<string, string>;
  paramLabels: Record<string, string>;
  presets: Record<string, string[]>;
  renderText: (
    p: Record<string, string>,
    onChipClick: (key: string, val: string) => void,
  ) => React.ReactNode;
  affectedComponents: string[];
}

function renderParamChip(
  paramKey: string,
  paramVal: string,
  onChipClick: (key: string, val: string) => void,
  punctuation?: string,
) {
  return (
    <span className="inline-block whitespace-nowrap">
      <button
        type="button"
        onClick={() => onChipClick(paramKey, paramVal)}
        className="font-mono text-xs bg-ck-bg-0 border border-ck-hairline-strong px-2 py-0.5 font-semibold text-ck-fg-1 hover:border-ck-accent hover:text-ck-accent transition-colors shadow-xs"
        title={`Click to tailor parameter ${paramKey}`}
      >
        {paramVal}
      </button>
      {punctuation && (
        <span className="text-ck-fg-1 font-serif">{punctuation}</span>
      )}
    </span>
  );
}

const CONTROLS_DATA: Record<string, ControlConfig> = {
  "AC-2": {
    id: "AC-2",
    title: "Account Management",
    family: "Access Control",
    baseline: "NIST SP 800-53 r5 · MER-MOD Profile",
    version: "1.4.2",
    lastModified: "2026-08-16T09:12:40Z",
    params: {
      p1: "developer accounts, system service accounts",
      p2: "System Owner & Security Officer",
      p3: "30 days",
      p4: "quarterly",
    },
    paramLabels: {
      p1: "ac-2_prm_1 (permitted account types)",
      p2: "ac-2_prm_2 (approving authority roles)",
      p3: "ac-2_prm_3 (inactive account disable period)",
      p4: "ac-2_prm_4 (account review frequency)",
    },
    presets: {
      p1: [
        "developer accounts, system service accounts",
        "human interactive accounts only",
        "ephemeral machine workload identities",
      ],
      p2: [
        "System Owner & Security Officer",
        "Automated HR Provisioner & Team Lead",
        "Privileged Access Review Board",
      ],
      p3: ["15 days", "30 days", "45 days", "90 days"],
      p4: ["monthly", "quarterly", "semi-annually", "annually"],
    },
    affectedComponents: ["iam-reconciler", "idp-core", "shared-cred-rotator"],
    renderText: (params, onChipClick) => (
      <div className="font-sans text-sm leading-relaxed text-ck-fg-2 space-y-3">
        <p>
          Meridian permits accounts of the following types:{" "}
          {renderParamChip("p1", params.p1, onChipClick, ".")} Each account is
          bound to a person or a workload identity in the HR or deployment
          system of record.
        </p>
        <p>
          Account requests are approved by{" "}
          {renderParamChip("p2", params.p2, onChipClick)} before creation.
          Accounts inactive for {renderParamChip("p3", params.p3, onChipClick)}{" "}
          are disabled automatically by the IAM reconciler, and a compliance
          review of all accounts runs every{" "}
          {renderParamChip("p4", params.p4, onChipClick, ".")}
        </p>
      </div>
    ),
  },
  "AC-3": {
    id: "AC-3",
    title: "Access Enforcement",
    family: "Access Control",
    baseline: "NIST SP 800-53 r5 · MER-MOD Profile",
    version: "1.4.2",
    lastModified: "2026-08-16T09:12:40Z",
    params: {
      p1: "attribute-based access control (ABAC) and SPIFFE x509 claims",
      p2: "Envoy gateway & kernel eBPF filter",
    },
    paramLabels: {
      p1: "ac-3_prm_1 (mandatory access control mechanism)",
      p2: "ac-3_prm_2 (enforcement points)",
    },
    presets: {
      p1: [
        "attribute-based access control (ABAC) and SPIFFE x509 claims",
        "role-based access control (RBAC) with hardware tokens",
      ],
      p2: [
        "Envoy gateway & kernel eBPF filter",
        "API Gateway & zero-trust service mesh",
      ],
    },
    affectedComponents: ["mesh-sidecar", "ebpf-enforcer", "spire-agent"],
    renderText: (params, onChipClick) => (
      <div className="font-sans text-sm leading-relaxed text-ck-fg-2 space-y-3">
        <p>
          The system enforces approved authorizations for logical access to
          information and system resources in accordance with applicable access
          control policies using{" "}
          {renderParamChip("p1", params.p1, onChipClick, ".")}
        </p>
        <p>
          All inter-service invocations are intercepted and verified at the{" "}
          {renderParamChip("p2", params.p2, onChipClick, ".")}
        </p>
      </div>
    ),
  },
  "AC-6": {
    id: "AC-6",
    title: "Least Privilege",
    family: "Access Control",
    baseline: "NIST SP 800-53 r5 · MER-MOD Profile",
    version: "1.4.2",
    lastModified: "2026-08-16T09:12:40Z",
    params: {
      p1: "just-in-time capability leases bounded to 60 minutes",
      p2: "ephemeral signed tokens with cryptographically enforced scopes",
    },
    paramLabels: {
      p1: "ac-6_prm_1 (privileged access duration)",
      p2: "ac-6_prm_2 (privilege attenuation mechanism)",
    },
    presets: {
      p1: [
        "just-in-time capability leases bounded to 60 minutes",
        "ephemeral leases bounded to 15 minutes with dual-approver",
      ],
      p2: [
        "ephemeral signed tokens with cryptographically enforced scopes",
        "hardware security key attestation",
      ],
    },
    affectedComponents: ["lease-issuer", "vault-pki", "auth-webhook"],
    renderText: (params, onChipClick) => (
      <div className="font-sans text-sm leading-relaxed text-ck-fg-2 space-y-3">
        <p>
          The system employs the principle of least privilege, allowing only
          authorized access for users and processes necessary to accomplish
          assigned tasks via{" "}
          {renderParamChip("p1", params.p1, onChipClick, ".")}
        </p>
        <p>
          Attenuated permissions are granted exclusively using{" "}
          {renderParamChip("p2", params.p2, onChipClick, ".")}
        </p>
      </div>
    ),
  },
  "IA-2": {
    id: "IA-2",
    title: "Multi-factor Authentication",
    family: "Identification & Auth",
    baseline: "NIST SP 800-53 r5 · MER-MOD Profile",
    version: "1.4.2",
    lastModified: "2026-08-16T09:12:40Z",
    params: {
      p1: "FIDO2 / WebAuthn hardware security keys",
      p2: "12 hours with step-up verification on privileged mutations",
    },
    paramLabels: {
      p1: "ia-2_prm_1 (approved MFA authenticators)",
      p2: "ia-2_prm_2 (session lifetime)",
    },
    presets: {
      p1: [
        "FIDO2 / WebAuthn hardware security keys",
        "FIDO2 Level 3 Keys + biometric attestation",
      ],
      p2: [
        "8 hours with step-up verification",
        "12 hours with step-up verification on privileged mutations",
      ],
    },
    affectedComponents: ["idp-core", "fido-verifier", "session-manager"],
    renderText: (params, onChipClick) => (
      <div className="font-sans text-sm leading-relaxed text-ck-fg-2 space-y-3">
        <p>
          The system implements multifactor authentication for all interactive
          sessions using {renderParamChip("p1", params.p1, onChipClick, ".")}
        </p>
        <p>
          Sessions persist for a maximum duration of{" "}
          {renderParamChip("p2", params.p2, onChipClick, ".")}
        </p>
      </div>
    ),
  },
  "SC-7": {
    id: "SC-7",
    title: "Boundary Protection",
    family: "System & Comms",
    baseline: "NIST SP 800-53 r5 · MER-MOD Profile",
    version: "1.4.2",
    lastModified: "2026-08-16T09:12:40Z",
    params: {
      p1: "isolated micro-segmented Calico eBPF network interfaces",
      p2: "mTLS sidecar proxy with explicit egress domain whitelisting",
    },
    paramLabels: {
      p1: "sc-7_prm_1 (managed external interfaces)",
      p2: "sc-7_prm_2 (egress filter rules)",
    },
    presets: {
      p1: [
        "isolated micro-segmented Calico eBPF network interfaces",
        "dual-homed bastion firewalls with hardware acceleration",
      ],
      p2: [
        "mTLS sidecar proxy with explicit egress domain whitelisting",
        "default-deny egress with Sigstore Rekor attestation only",
      ],
    },
    affectedComponents: ["calico-ebpf", "egress-proxy", "boundary-controller"],
    renderText: (params, onChipClick) => (
      <div className="font-sans text-sm leading-relaxed text-ck-fg-2 space-y-3">
        <p>
          The system monitors and controls communications at external boundaries
          of the system and key internal boundaries through{" "}
          {renderParamChip("p1", params.p1, onChipClick, ".")}
        </p>
        <p>
          All outbound traffic requires transit through the{" "}
          {renderParamChip("p2", params.p2, onChipClick, ".")}
        </p>
      </div>
    ),
  },
  "SI-4": {
    id: "SI-4",
    title: "System Monitoring",
    family: "System & Info Integrity",
    baseline: "NIST SP 800-53 r5 · MER-MOD Profile",
    version: "1.4.2",
    lastModified: "2026-08-16T09:12:40Z",
    params: {
      p1: "OpenTelemetry continuous kernel tracepoints & auditd daemon",
      p2: "15 seconds for anomaly detection and automated isolation",
    },
    paramLabels: {
      p1: "si-4_prm_1 (monitoring collectors)",
      p2: "si-4_prm_2 (alert latency threshold)",
    },
    presets: {
      p1: [
        "OpenTelemetry continuous kernel tracepoints & auditd daemon",
        "eBPF Tetragon runtime telemetry + Prometheus metrics",
      ],
      p2: [
        "5 seconds for anomaly detection and automated isolation",
        "15 seconds for anomaly detection and automated isolation",
      ],
    },
    affectedComponents: ["tetragon-daemon", "otel-collector", "falco-engine"],
    renderText: (params, onChipClick) => (
      <div className="font-sans text-sm leading-relaxed text-ck-fg-2 space-y-3">
        <p>
          The system monitors for unauthorized network connections, malicious
          execution attempts, and abnormal data movements using{" "}
          {renderParamChip("p1", params.p1, onChipClick, ".")}
        </p>
        <p>
          Security incidents trigger automated quarantine within{" "}
          {renderParamChip("p2", params.p2, onChipClick, ".")}
        </p>
      </div>
    ),
  },
};

export function ComposerSurface({
  selectedControlId,
  onSelectControl,
  onPublishControl,
}: ComposerSurfaceProps) {
  const currentControl =
    CONTROLS_DATA[selectedControlId] || CONTROLS_DATA["AC-2"];

  const [controlParams, setControlParams] = React.useState<
    Record<string, Record<string, string>>
  >({
    "AC-2": { ...CONTROLS_DATA["AC-2"].params },
    "AC-3": { ...CONTROLS_DATA["AC-3"].params },
    "AC-6": { ...CONTROLS_DATA["AC-6"].params },
    "IA-2": { ...CONTROLS_DATA["IA-2"].params },
    "SC-7": { ...CONTROLS_DATA["SC-7"].params },
    "SI-4": { ...CONTROLS_DATA["SI-4"].params },
  });

  const [alters, setAlters] = React.useState([
    {
      id: "ac-2_alt_1",
      word: "EXCL",
      target: "AC-2(9)",
      label: "Previous Logon Notification",
      note: "Excluded · Handled at identity provider session level",
      excluded: true,
      hasWarning: true,
      warningText:
        "consequence · orphans 1 implementation (shared-cred rotator) · invalidates assessment ar-2189",
    },
    {
      id: "ac-2_alt_2",
      word: "INCL",
      target: "AC-2(12)",
      label: "Account Monitoring For Inactivity",
      note: "Included · Automated audit notifications via IAM reconciler",
      excluded: false,
      hasWarning: false,
      warningText: "",
    },
    {
      id: "ac-2_alt_3",
      word: "INCL",
      target: "AC-2(13)",
      label: "Disable Accounts For High-Risk Roles",
      note: "Included · Ephemeral capability lease revocation",
      excluded: false,
      hasWarning: false,
      warningText: "",
    },
  ]);

  const [activeParamChip, setActiveParamChip] = React.useState<string | null>(
    null,
  );
  const [paramInputVal, setParamInputVal] = React.useState("");
  const [derivedAdopted, setDerivedAdopted] = React.useState(false);
  const [ownerAssigned, setOwnerAssigned] = React.useState(false);
  const [isPublished, setIsPublished] = React.useState(false);
  const [previewTab, setPreviewTab] = React.useState<"diff" | "json" | "yaml">(
    "diff",
  );
  const [committedVersion, setCommittedVersion] = React.useState("1.4.2");
  const [commitSuccess, setCommitSuccess] = React.useState(false);

  const activeParams =
    controlParams[selectedControlId] || currentControl.params;

  const handleChipClick = (key: string, currentVal: string) => {
    setActiveParamChip(key);
    setParamInputVal(currentVal);
  };

  const handleChipSave = (valToSave?: string) => {
    const finalVal = valToSave ?? paramInputVal;
    if (activeParamChip) {
      setControlParams((prev) => ({
        ...prev,
        [selectedControlId]: {
          ...prev[selectedControlId],
          [activeParamChip]: finalVal,
        },
      }));
      setActiveParamChip(null);
    }
  };

  const handleToggleAlter = (id: string) => {
    setAlters((prev) =>
      prev.map((al) => {
        if (al.id === id) {
          const nextExcl = !al.excluded;
          return {
            ...al,
            excluded: nextExcl,
            word: nextExcl ? "EXCL" : "INCL",
          };
        }
        return al;
      }),
    );
  };

  const handleCommitResolution = () => {
    const nextVer = "1.4.3";
    setCommittedVersion(nextVer);
    setCommitSuccess(true);
    setTimeout(() => setCommitSuccess(false), 3000);
  };

  // Generated OSCAL representation
  const oscalJson = JSON.stringify(
    {
      profile: {
        uuid: "7b79a832-7f28-4e11-b0db-1f2fa9d88001",
        metadata: {
          title: "Meridian Moderate Baseline (MER-MOD)",
          "last-modified": new Date().toISOString(),
          version: committedVersion,
          "oscal-version": "1.2.3",
        },
        imports: [
          {
            href: "urn:oscal:catalog:nist-sp-800-53-r5",
            "include-controls": [
              {
                "with-child-controls": "yes",
                matching: [{ pattern: selectedControlId.toLowerCase() }],
              },
            ],
          },
        ],
        modify: {
          "set-parameters": Object.entries(activeParams).map(([k, v]) => ({
            "param-id": `${selectedControlId.toLowerCase()}_prm_${k.replace("p", "")}`,
            values: [v],
          })),
          alters: alters.map((a) => ({
            "control-id": a.target
              .toLowerCase()
              .replace("(", "-")
              .replace(")", ""),
            action: a.excluded ? "exclude" : "include",
          })),
        },
      },
    },
    null,
    2,
  );

  const oscalYaml = `profile:
  uuid: 7b79a832-7f28-4e11-b0db-1f2fa9d88001
  metadata:
    title: Meridian Moderate Baseline (MER-MOD)
    last-modified: '${new Date().toISOString()}'
    version: '${committedVersion}'
    oscal-version: '1.2.3'
  imports:
    - href: urn:oscal:catalog:nist-sp-800-53-r5
      include-controls:
        - with-child-controls: yes
          matching:
            - pattern: ${selectedControlId.toLowerCase()}
  modify:
    set-parameters:${Object.entries(activeParams)
      .map(
        ([k, v]) => `
      - param-id: ${selectedControlId.toLowerCase()}_prm_${k.replace("p", "")}
        values:
          - "${v}"`,
      )
      .join("")}
    alters:${alters
      .map(
        (a) => `
      - control-id: ${a.target.toLowerCase().replace("(", "-").replace(")", "")}
        action: ${a.excluded ? "exclude" : "include"}`,
      )
      .join("")}`;

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex flex-wrap items-baseline justify-between border-b border-ck-hairline pb-2 gap-2">
        <div className="flex items-baseline gap-3 min-w-0">
          <h1 className="font-serif text-2xl font-normal tracking-tight text-ck-fg-1 whitespace-nowrap shrink-0">
            The Composer
          </h1>
          <span className="text-xs text-ck-fg-mute font-mono hidden md:inline">
            A document you would sign — parameters are the only machinery
            showing.
          </span>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          <Badge
            variant="outline"
            className="font-mono text-[10px] uppercase text-green-700 dark:text-green-400 whitespace-nowrap shrink-0"
          >
            {currentControl.baseline} · v{committedVersion}
          </Badge>
        </div>
      </div>

      {/* Control Selector Bar */}
      <div className="flex items-center justify-between border border-ck-hairline bg-ck-bg-1 p-2 shadow-xs font-mono text-xs">
        <div className="flex items-center gap-2">
          <span className="text-[10px] uppercase text-ck-fg-mute font-semibold whitespace-nowrap">
            Selected Control:
          </span>
          <select
            value={selectedControlId}
            onChange={(e) => {
              onSelectControl(e.target.value);
              setActiveParamChip(null);
            }}
            className="border border-ck-hairline-strong bg-ck-bg-0 text-ck-fg-1 p-1 text-xs font-mono font-semibold cursor-pointer"
          >
            {Object.keys(CONTROLS_DATA).map((id) => (
              <option key={id} value={id}>
                {id} · {CONTROLS_DATA[id].title}
              </option>
            ))}
          </select>
        </div>
        <div className="flex items-center gap-4 text-ck-fg-mute text-[11px] whitespace-nowrap shrink-0">
          <span>
            Family:{" "}
            <strong className="text-ck-fg-1">{currentControl.family}</strong>
          </span>
          <span>·</span>
          <span>85 controls in catalog · 11 cross-framework mappings</span>
        </div>
      </div>

      {/* Main Authoring Article & Profile Tailoring Diff */}
      <div className="grid grid-cols-1 lg:grid-cols-[1fr_420px] gap-4 items-start min-w-0">
        {/* Left Article */}
        <article className="border border-ck-hairline-strong bg-ck-bg-1 p-6 space-y-4 shadow-xs min-w-0">
          <div className="flex flex-wrap items-baseline justify-between border-b border-ck-hairline pb-2 gap-2">
            <div className="flex items-baseline gap-3 min-w-0">
              <span className="font-mono font-bold text-lg text-ck-fg-1 whitespace-nowrap shrink-0">
                {currentControl.id}
              </span>
              <h2 className="font-serif text-2xl font-normal text-ck-fg-1 whitespace-nowrap">
                {currentControl.title}
              </h2>
            </div>
            <span className="font-mono text-[10px] text-ck-fg-mute uppercase whitespace-nowrap shrink-0">
              last-modified {currentControl.lastModified}
            </span>
          </div>

          <p className="font-mono text-[11px] text-ck-fg-mute">
            catalog NIST SP 800-53 r5 · tailored by profile MER-MOD · version{" "}
            {committedVersion}
          </p>

          {/* Render Narrative with Interactive Chips */}
          {currentControl.renderText(activeParams, handleChipClick)}

          {/* Active Chip Inline Editor */}
          {activeParamChip && (
            <div className="border border-ck-accent bg-ck-bg-0 p-3 space-y-3 font-mono text-xs shadow-sm animate-in fade-in duration-150">
              <div className="flex items-center justify-between">
                <span className="text-[10px] uppercase font-bold text-ck-accent">
                  Tailoring Parameter:{" "}
                  {currentControl.paramLabels[activeParamChip] ||
                    activeParamChip}
                </span>
                <span className="text-[10px] text-ck-fg-mute">
                  Instant Live Preview
                </span>
              </div>

              {/* Quick Presets */}
              {currentControl.presets[activeParamChip] && (
                <div className="flex flex-wrap gap-1.5 items-center">
                  <span className="text-[10px] text-ck-fg-mute uppercase mr-1">
                    Presets:
                  </span>
                  {currentControl.presets[activeParamChip].map((preset) => (
                    <button
                      key={preset}
                      type="button"
                      onClick={() => {
                        setParamInputVal(preset);
                        handleChipSave(preset);
                      }}
                      className="px-2 py-0.5 text-[10px] border border-ck-hairline-strong bg-ck-bg-1 hover:border-ck-fg-1 text-ck-fg-1 transition-colors"
                    >
                      {preset}
                    </button>
                  ))}
                </div>
              )}

              <input
                type="text"
                value={paramInputVal}
                onChange={(e) => setParamInputVal(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleChipSave();
                  if (e.key === "Escape") setActiveParamChip(null);
                }}
                autoFocus
                className="w-full border border-ck-hairline-strong bg-ck-bg-1 text-ck-fg-1 p-2 text-xs font-mono focus:border-ck-accent focus:outline-none"
              />
              <div className="flex gap-2">
                <Button
                  size="sm"
                  variant="default"
                  onClick={() => handleChipSave()}
                  className="h-7 text-xs font-mono bg-ck-accent text-white hover:bg-ck-accent/90"
                >
                  Set Parameter
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => setActiveParamChip(null)}
                  className="h-7 text-xs font-mono"
                >
                  Cancel (Esc)
                </Button>
              </div>
            </div>
          )}

          {/* AI Derived Assertion */}
          {!derivedAdopted ? (
            <div className="border border-dashed border-ck-hairline-strong bg-ck-bg-2 p-3 space-y-2 font-mono text-xs">
              <div className="flex items-center gap-2">
                <span className="text-ck-accent font-bold">⇝</span>
                <span className="font-semibold text-ck-fg-1">
                  Drafted · Not Yet an Assertion
                </span>
                <span className="text-[10px] text-ck-fg-mute ml-auto">
                  model: claude-sonnet-4 · conf: 0.72
                </span>
              </div>
              <p className="font-sans text-xs text-ck-fg-2">
                Shared and group account credentials are rotated whenever
                membership changes, and membership itself is reviewed at the
                same cadence as individual accounts.
              </p>
              <div className="flex gap-2">
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => setDerivedAdopted(true)}
                  className="h-6 text-[10px] font-mono"
                >
                  Adopt as Mine
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => setDerivedAdopted(false)}
                  className="h-6 text-[10px] font-mono text-ck-fg-mute hover:text-ck-fg-1"
                >
                  Discard
                </Button>
              </div>
            </div>
          ) : (
            <p className="font-sans text-xs text-ck-fg-2 border-l-2 border-ck-accent pl-2">
              <span className="font-mono text-ck-accent font-bold">⇝ </span>
              Shared and group account credentials are rotated whenever
              membership changes, and membership itself is reviewed at the same
              cadence as individual accounts.
            </p>
          )}

          {/* Governance & Responsible Role Gate */}
          <div className="border-t border-ck-hairline pt-4 space-y-2 font-mono text-xs">
            {!ownerAssigned ? (
              <div className="flex items-center justify-between bg-ck-bg-2 p-2.5 border border-ck-hairline">
                <div className="flex items-center gap-2">
                  <span className="bg-ck-fg-1 text-ck-bg-0 font-bold px-1.5 py-0.5 text-[10px]">
                    ⊭ PUBLISH BLOCKED
                  </span>
                  <span className="text-ck-fg-2 text-[11px]">
                    Constraint oscal-implemented-requirement-responsible-role
                    missing
                  </span>
                </div>
                <Button
                  size="sm"
                  variant="default"
                  onClick={() => setOwnerAssigned(true)}
                  className="h-7 text-xs font-mono"
                >
                  Assign Owner · T. Mori
                </Button>
              </div>
            ) : (
              <div className="flex items-center justify-between bg-ck-bg-2 p-2.5 border border-ck-hairline">
                <div className="flex items-center gap-2">
                  <span className="text-green-700 dark:text-green-400 font-bold text-[10px]">
                    [OK] CONSTRAINT HOLDS
                  </span>
                  <span className="text-ck-fg-2 text-[11px]">
                    Responsible-role · T. Mori (Platform Engineering)
                  </span>
                </div>
                {!isPublished ? (
                  <Button
                    size="sm"
                    variant="default"
                    onClick={() => {
                      setIsPublished(true);
                      if (onPublishControl) onPublishControl(selectedControlId);
                    }}
                    className="h-7 text-xs font-mono"
                  >
                    Publish {selectedControlId}
                  </Button>
                ) : (
                  <Badge
                    variant="outline"
                    className="font-mono text-[10px] uppercase text-green-700 dark:text-green-400"
                  >
                    PUBLISHED · RECEIPT WRITTEN
                  </Badge>
                )}
              </div>
            )}
          </div>
        </article>

        {/* Right Resolution & Diff Sidebar */}
        <aside className="border border-ck-hairline-strong bg-ck-bg-1 p-4 space-y-4 shadow-xs font-mono text-xs min-w-0">
          <div className="flex items-center justify-between border-b border-ck-hairline pb-2">
            <div>
              <span className="text-[10px] uppercase font-bold text-ck-fg-mute block">
                Profile Tailoring · MER-MOD
              </span>
              <span className="text-xs font-bold text-ck-fg-1">
                Interactive OSCAL Engine
              </span>
            </div>
            <div className="flex items-center gap-1">
              <button
                type="button"
                onClick={() => setPreviewTab("diff")}
                className={`px-2 py-0.5 text-[10px] font-mono border ${
                  previewTab === "diff"
                    ? "bg-ck-fg-1 text-ck-bg-0 border-ck-fg-1"
                    : "bg-ck-bg-0 text-ck-fg-mute border-ck-hairline"
                }`}
              >
                Diff
              </button>
              <button
                type="button"
                onClick={() => setPreviewTab("json")}
                className={`px-2 py-0.5 text-[10px] font-mono border ${
                  previewTab === "json"
                    ? "bg-ck-fg-1 text-ck-bg-0 border-ck-fg-1"
                    : "bg-ck-bg-0 text-ck-fg-mute border-ck-hairline"
                }`}
              >
                JSON
              </button>
              <button
                type="button"
                onClick={() => setPreviewTab("yaml")}
                className={`px-2 py-0.5 text-[10px] font-mono border ${
                  previewTab === "yaml"
                    ? "bg-ck-fg-1 text-ck-bg-0 border-ck-fg-1"
                    : "bg-ck-bg-0 text-ck-fg-mute border-ck-hairline"
                }`}
              >
                YAML
              </button>
            </div>
          </div>

          {/* Alterations Toggles */}
          <div className="space-y-2 border border-ck-hairline bg-ck-bg-0 p-3">
            <span className="text-[10px] uppercase font-bold text-ck-fg-mute block mb-1">
              Baseline Alterations (Alters)
            </span>
            {alters.map((al) => (
              <div key={al.id} className="space-y-1">
                <div className="flex items-center gap-2 min-w-0">
                  <button
                    type="button"
                    onClick={() => handleToggleAlter(al.id)}
                    className={`px-1.5 py-0.5 text-[9px] font-mono font-bold border transition-colors shrink-0 ${
                      al.excluded
                        ? "bg-ck-fg-1 text-ck-bg-0 border-ck-fg-1"
                        : "bg-ck-bg-1 text-ck-fg-mute border-ck-hairline-strong hover:text-ck-fg-1"
                    }`}
                  >
                    {al.word}
                  </button>
                  <span className="font-semibold text-ck-fg-1 text-[11px] whitespace-nowrap shrink-0">
                    {al.target}
                  </span>
                  <span className="text-[10px] text-ck-fg-mute truncate ml-auto">
                    {al.label}
                  </span>
                </div>
                {al.hasWarning && al.excluded && (
                  <div className="border-l-2 border-ck-accent bg-ck-bg-2 p-1.5 text-[10px] text-ck-fg-2">
                    {al.warningText}
                  </div>
                )}
              </div>
            ))}
          </div>

          {/* Main Tab Content */}
          {previewTab === "diff" && (
            <div className="space-y-2">
              <div className="border border-ck-hairline bg-ck-bg-0 p-2.5">
                <span className="text-[10px] uppercase text-ck-fg-mute block font-bold">
                  Catalog Baseline (SP 800-53 r5)
                </span>
                <p className="text-[11px] text-ck-fg-mute line-through">
                  Inactive account disabling period: 90 days
                </p>
                <p className="text-[11px] text-ck-fg-mute line-through">
                  Permitted account types: general user, administrator, guest
                </p>
              </div>

              <div className="border border-ck-hairline bg-ck-bg-0 p-2.5">
                <span className="text-[10px] uppercase text-green-700 dark:text-green-400 block font-bold">
                  Tailored Profile (MER-MOD)
                </span>
                <p className="text-[11px] text-green-700 dark:text-green-400 font-semibold">
                  Inactive account disabling period:{" "}
                  {activeParams.p3 || "30 days"}
                </p>
                <p className="text-[11px] text-green-700 dark:text-green-400 font-semibold">
                  Permitted types:{" "}
                  {activeParams.p1 ||
                    "developer accounts, system service accounts"}
                </p>
              </div>
            </div>
          )}

          {previewTab === "json" && (
            <div className="border border-ck-hairline bg-ck-bg-0 p-2 max-h-56 overflow-auto">
              <pre className="text-[10px] font-mono leading-relaxed text-ck-fg-1">
                {oscalJson}
              </pre>
            </div>
          )}

          {previewTab === "yaml" && (
            <div className="border border-ck-hairline bg-ck-bg-0 p-2 max-h-56 overflow-auto">
              <pre className="text-[10px] font-mono leading-relaxed text-ck-fg-1">
                {oscalYaml}
              </pre>
            </div>
          )}

          {/* Resolution Stats & Commit */}
          <div className="border border-ck-hairline bg-ck-bg-2 p-2.5 space-y-2">
            <div className="flex items-center justify-between text-[11px]">
              <span className="text-ck-fg-mute font-mono">
                resolve · oscal-cli (pinned)
              </span>
              <span className="text-green-700 dark:text-green-400 font-bold font-mono">
                14 ms · 3 alters
              </span>
            </div>
            <div className="text-[11px] text-ck-fg-2">
              <strong className="text-ck-fg-1">Impact Analysis:</strong> Affects{" "}
              {currentControl.affectedComponents.length} components:{" "}
              {currentControl.affectedComponents
                .map((c) => `\`${c}\``)
                .join(", ")}
              .
            </div>
            <Button
              size="sm"
              variant="outline"
              onClick={handleCommitResolution}
              className="w-full h-7 text-xs font-mono border-ck-hairline-strong hover:bg-ck-bg-0"
            >
              {commitSuccess
                ? "[OK] Profile Committed to Git"
                : `Commit → Version 1.4.3`}
            </Button>
          </div>
        </aside>
      </div>
    </div>
  );
}
