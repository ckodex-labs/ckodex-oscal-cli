"use client";

import * as React from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

interface PipelineSurfaceProps {
  onTriggerPipelineRun?: () => void;
}

interface RunLog {
  no: number;
  verdict:
    | "PASS · 0 FAULTS"
    | "⊭ 1 FAULT (BLOCKED)"
    | "WAIVED · DEROGATION RECORDED"
    | string;
  duration: string;
  time: string;
}

export function PipelineSurface({
  onTriggerPipelineRun,
}: PipelineSurfaceProps) {
  const [hasConstraintFault, setHasConstraintFault] = React.useState(true);
  const [isWaived, setIsWaived] = React.useState(false);
  const [isRunning, setIsRunning] = React.useState(false);
  const [executingStep, setExecutingStep] = React.useState<number | null>(null);
  const [runHistory, setRunHistory] = React.useState<RunLog[]>([
    {
      no: 1424,
      verdict: "⊭ 1 FAULT (BLOCKED)",
      duration: "182ms",
      time: "10 mins ago",
    },
    {
      no: 1423,
      verdict: "PASS · 0 FAULTS",
      duration: "145ms",
      time: "2 hours ago",
    },
  ]);

  const [cliOutput, setCliOutput] = React.useState<any>(null);

  const handleRunVerification = async () => {
    setIsRunning(true);
    setExecutingStep(1);

    try {
      setExecutingStep(2);
      const res = await fetch("/api/cli", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ command: "pipeline", args: ["run"] }),
      });
      setExecutingStep(3);
      const json = await res.json();
      setExecutingStep(4);
      if (json.success && json.data) {
        setCliOutput(json.data);
        const hasViolations = json.data.violations_count > 0;
        setHasConstraintFault(hasViolations);
        const nextNo = runHistory[0].no + 1;
        setRunHistory((prev) => [
          {
            no: nextNo,
            verdict: hasViolations ? "⊭ 1 FAULT (BLOCKED)" : "PASS · 0 FAULTS",
            duration: "24ms",
            time: "just now",
          },
          ...prev,
        ]);
      }
    } catch (err) {
      console.error("Pipeline execution failed:", err);
    } finally {
      setIsRunning(false);
      setExecutingStep(null);
      if (onTriggerPipelineRun) onTriggerPipelineRun();
    }
  };

  const handleQuickFixOwner = async () => {
    try {
      await fetch("/api/cli", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          command: "fix",
          args: ["--rule", "cis-k8s-5.2.1", "-f", "workload.yaml", "--dry-run"],
        }),
      });
      setHasConstraintFault(false);
      setIsWaived(false);
      const nextNo = runHistory[0].no + 1;
      setRunHistory((prev) => [
        {
          no: nextNo,
          verdict: "PASS · REMEDIATED (mizan fix)",
          duration: "18ms",
          time: "just now",
        },
        ...prev,
      ]);
    } catch (e) {
      console.error(e);
    }
  };

  const handleWaiveFault = async () => {
    try {
      await fetch("/api/cli", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          command: "waive",
          args: [
            "--rule",
            "cis-k8s-5.2.1",
            "--reason",
            "Temporary derogation approved via Workbench",
            "--ttl",
            "7d",
          ],
        }),
      });
      setIsWaived(true);
      const nextNo = runHistory[0].no + 1;
      setRunHistory((prev) => [
        {
          no: nextNo,
          verdict: "PASS · DEROGATION LEASE (mizan waive)",
          duration: "12ms",
          time: "just now",
        },
        ...prev,
      ]);
    } catch (e) {
      console.error(e);
    }
  };

  const isPassing = !hasConstraintFault || isWaived;

  return (
    <div className="space-y-4 font-mono">
      {/* Header */}
      <div className="flex flex-wrap items-baseline justify-between border-b border-ck-hairline pb-2 gap-2">
        <div className="flex items-baseline gap-3 min-w-0">
          <h1 className="font-serif text-2xl font-normal tracking-tight text-ck-fg-1 whitespace-nowrap shrink-0">
            The Pipeline
          </h1>
          <span className="text-xs text-ck-fg-mute font-mono hidden md:inline">
            The pipeline is a persona. Its output is designed, and its failures
            teach.
          </span>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          <Badge
            variant="outline"
            className="font-mono text-[10px] uppercase text-green-700 dark:text-green-400 whitespace-nowrap shrink-0"
          >
            SLSA v1.2 In-Toto Verified · Pipeline #{runHistory[0]?.no || 1424}
          </Badge>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-[1.1fr_1.3fr] gap-4 items-start text-xs min-w-0">
        {/* Left: PR Bot Comment Preview */}
        <div className="border border-ck-hairline-strong bg-ck-bg-1 shadow-xs min-w-0">
          <div className="flex items-center gap-3 border-b border-ck-hairline p-3 bg-ck-bg-2">
            <span className="w-6 h-6 bg-ck-fg-1 text-ck-bg-0 flex items-center justify-center font-bold text-xs shrink-0">
              A
            </span>
            <div className="flex items-center gap-2 flex-wrap min-w-0">
              <span className="font-sans font-semibold text-sm text-ck-fg-1 whitespace-nowrap">
                atlas-bot
              </span>
              <Badge
                variant="outline"
                className="text-[9px] font-mono uppercase bg-ck-bg-0 shrink-0"
              >
                BOT APP
              </Badge>
              <span className="text-ck-fg-mute text-[11px] truncate">
                commented on PR #212 · run #{runHistory[0]?.no || 1424} ·
                41c9e2b
              </span>
            </div>
          </div>

          <div className="p-5 space-y-4">
            <div className="flex flex-wrap items-baseline justify-between border-b border-ck-hairline pb-2 gap-2">
              <h3 className="font-serif text-xl font-normal text-ck-fg-1 whitespace-nowrap shrink-0">
                Compliance Diff — Profile MER-MOD
              </h3>
              <Badge
                variant="outline"
                className={`text-[10px] uppercase font-mono whitespace-nowrap shrink-0 ${
                  isPassing
                    ? "text-green-700 dark:text-green-400 border-green-700/40"
                    : "text-red-700 dark:text-red-400 border-red-700/40"
                }`}
              >
                {isPassing ? "✓ ALL GATES SATISFIED" : "⊭ GATES BLOCKED"}
              </Badge>
            </div>

            <div className="space-y-2 font-sans text-xs">
              <div className="grid grid-cols-[140px_1fr] gap-2 border-b border-ck-hairline pb-1.5">
                <span className="text-ck-fg-mute font-mono">
                  Baseline Controls:
                </span>
                <span className="font-mono text-ck-fg-1">
                  −1 · AC-2(9) removed from baseline
                </span>
              </div>
              <div className="grid grid-cols-[140px_1fr] gap-2 border-b border-ck-hairline pb-1.5">
                <span className="text-ck-fg-mute font-mono">
                  Parameters Changed:
                </span>
                <span className="font-mono text-ck-fg-1">
                  ac-2_prm_3 · &quot;90 days&quot; → &quot;30 days&quot;
                </span>
              </div>
              <div className="grid grid-cols-[140px_1fr] gap-2 border-b border-ck-hairline pb-1.5">
                <span className="text-ck-fg-mute font-mono">
                  Implementations Affected:
                </span>
                <span className="font-mono text-ck-fg-1">
                  3 — iam-reconciler, idp-core, shared-cred
                </span>
              </div>
              <div className="grid grid-cols-[140px_1fr] gap-2 border-b border-ck-hairline pb-1.5">
                <span className="text-ck-fg-mute font-mono">
                  Mappings Invalidated:
                </span>
                <span className="font-mono text-ck-fg-1">
                  1 · MER-AC-01 → A.5.15 (confidence floor)
                </span>
              </div>
              <div className="grid grid-cols-[140px_1fr] gap-2 pt-1">
                <span className="text-ck-fg-mute font-mono">
                  Coverage Delta:
                </span>
                <div className="font-mono text-[11px]">
                  <span className="text-green-700 dark:text-green-400 font-bold whitespace-nowrap">
                    71.4% → 68.1% (Δ −3.3)
                  </span>
                  <span className="text-ck-fg-mute ml-2 text-[10px]">
                    computed
                  </span>
                </div>
              </div>
            </div>

            {/* Gates */}
            <div className="border-t border-ck-hairline pt-3 space-y-2">
              <span className="text-[10px] uppercase font-bold text-ck-fg-mute block font-mono">
                Continuous Assurance Gates
              </span>
              <div className="flex items-center justify-between text-[11px] border-b border-ck-hairline pb-1.5 gap-2">
                <div className="flex items-center gap-2 min-w-0">
                  <span className="font-mono text-ck-fg-1 whitespace-nowrap">
                    oscal-schema
                  </span>
                  <span className="text-ck-fg-mute font-mono text-[10px] px-1 bg-ck-bg-2 border border-ck-hairline shrink-0">
                    schema
                  </span>
                </div>
                <span className="text-green-700 dark:text-green-400 font-bold font-mono whitespace-nowrap shrink-0 text-right">
                  PASS · 4 DOCS
                </span>
              </div>
              <div className="flex items-center justify-between text-[11px] border-b border-ck-hairline pb-1.5 gap-2">
                <div className="flex items-center gap-2 min-w-0">
                  <span className="font-mono text-ck-fg-1 whitespace-nowrap">
                    slsa-provenance
                  </span>
                  <span className="text-ck-fg-mute font-mono text-[10px] px-1 bg-ck-bg-2 border border-ck-hairline shrink-0">
                    supply-chain
                  </span>
                </div>
                <span className="text-green-700 dark:text-green-400 font-bold font-mono whitespace-nowrap shrink-0 text-right">
                  PASS · SHA256
                </span>
              </div>
              <div className="flex items-center justify-between text-[11px] border-b border-ck-hairline pb-1.5 gap-2">
                <div className="flex items-center gap-2 min-w-0">
                  <span className="font-mono text-ck-fg-1 whitespace-nowrap">
                    responsible-role
                  </span>
                  <span className="text-ck-fg-mute font-mono text-[10px] px-1 bg-ck-bg-2 border border-ck-hairline shrink-0">
                    constraint
                  </span>
                </div>
                <span
                  className={`font-mono font-bold whitespace-nowrap shrink-0 text-right ${
                    isPassing
                      ? "text-green-700 dark:text-green-400"
                      : "text-red-700 dark:text-red-400"
                  }`}
                >
                  {isPassing
                    ? isWaived
                      ? "WAIVED · RISK ACCEPTED"
                      : "PASS · 0 FAULTS"
                    : "⊭ 1 FAULT (BLOCKING)"}
                </span>
              </div>
              <div className="flex items-center justify-between text-[11px] pb-0.5 gap-2">
                <div className="flex items-center gap-2 min-w-0">
                  <span className="font-mono text-ck-fg-1 whitespace-nowrap">
                    blast-radius
                  </span>
                  <span className="text-ck-fg-mute font-mono text-[10px] px-1 bg-ck-bg-2 border border-ck-hairline shrink-0">
                    topology
                  </span>
                </div>
                <span className="text-green-700 dark:text-green-400 font-bold font-mono whitespace-nowrap shrink-0 text-right">
                  PASS · BOUNDED
                </span>
              </div>
            </div>

            <div className="flex flex-wrap gap-2 pt-2 border-t border-ck-hairline">
              <Button
                size="sm"
                variant="default"
                disabled={isRunning}
                onClick={handleRunVerification}
                className="h-7 text-xs font-mono bg-ck-fg-1 text-ck-bg-0 hover:bg-ck-fg-2"
              >
                {isRunning
                  ? "Evaluating Pre-Commit Gates…"
                  : "Re-evaluate Pipeline Gates"}
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={() => {
                  setHasConstraintFault((f) => !f);
                  setIsWaived(false);
                }}
                className="h-7 text-xs font-mono"
              >
                Toggle Fault Scenario (
                {hasConstraintFault && !isWaived
                  ? "Fault Active"
                  : "Clean State"}
                )
              </Button>
            </div>
          </div>
        </div>

        {/* Right: GitOps Pre-Commit Terminal Simulation */}
        <div className="space-y-4 min-w-0">
          <div className="border border-ck-hairline-strong bg-ck-bg-0 shadow-sm min-w-0 overflow-hidden">
            <div className="flex items-center justify-between border-b border-ck-hairline px-3 py-2 bg-ck-bg-1 gap-2">
              <div className="flex items-center gap-2 min-w-0">
                <span className="w-2.5 h-2.5 rounded-full bg-red-500/70 shrink-0" />
                <span className="w-2.5 h-2.5 rounded-full bg-yellow-500/70 shrink-0" />
                <span className="w-2.5 h-2.5 rounded-full bg-green-500/70 shrink-0" />
                <span className="text-[11px] font-mono font-semibold text-ck-fg-1 ml-2 whitespace-nowrap">
                  zsh · mizan pre-commit (v1.2.3)
                </span>
              </div>
              <Button
                size="sm"
                variant="outline"
                disabled={isRunning}
                onClick={handleRunVerification}
                className="h-6 text-[10px] font-mono shrink-0 whitespace-nowrap px-2.5"
              >
                {isRunning ? "Executing…" : "Run Verification"}
              </Button>
            </div>

            {/* Stepper Progress Bar when Running */}
            {isRunning && (
              <div className="p-3 border-b border-ck-hairline bg-ck-bg-2 flex items-center gap-2 text-[11px]">
                <span className="w-2 h-2 rounded-full bg-ck-accent animate-ping" />
                <span className="text-ck-accent font-semibold">
                  Step {executingStep || 1}/4:
                </span>
                <span className="text-ck-fg-1">
                  {executingStep === 1 &&
                    "OSCAL schema validity check against pinned NIST schemas…"}
                  {executingStep === 2 &&
                    "SLSA v1.2 Merkle tree verification against in-toto attestation…"}
                  {executingStep === 3 &&
                    "Evaluating Regorus/OPA policy constraints and invariants…"}
                  {executingStep === 4 &&
                    "Computing blast radius and dependency ripple effects…"}
                  {executingStep === 5 && "Finalizing commit gate outcome…"}
                </span>
              </div>
            )}

            <pre className="font-mono text-xs leading-relaxed text-ck-fg-1 overflow-x-auto p-4 bg-ck-bg-0 min-h-[220px]">
              {`$ mizan pipeline run --jurisdiction us
mizan compliance-pipeline · schema pinned oscal 1.2.3

  catalog-uuid: ${cliOutput?.oscal_catalog_uuid || "8b788647-767a-4ecb-ba3a-f2b7f719602a"}
  merkle-root:  ${cliOutput?.merkle_root || "sha256:c9840a3707ca3f023cee70e8dd90e359514c52aef0a8f79ae6228726d9395f5d"}
  rules:        ${cliOutput?.evaluated_rules_count || 4} evaluated (${cliOutput?.passed_rules_count || 3} passed, ${cliOutput?.violations_count || 1} violations)
`}
              {!isPassing ? (
                <>
                  <span className="text-red-700 dark:text-red-400 font-bold">
                    {`  constraints   ⊭ 1 fault
    rule:       cis-k8s-5.2.1 (Disallow Privileged Containers)
    → violation: container 'production-api' specifies privileged: true
    → remedy:   set securityContext.privileged: false or record derogation
`}
                  </span>
                  <span className="text-red-700 dark:text-red-400 font-bold">
                    {`pipeline blocked · artifacts preserved in mizan-pipeline-output/`}
                  </span>
                </>
              ) : (
                <>
                  <span className="text-green-700 dark:text-green-400 font-bold">
                    {isWaived
                      ? `  constraints  pass (derogation active) · waiver w-0199 recorded
    waiver: accepted-risk · expiry 2026-10-01 · authority: CISO
`
                      : `  constraints  pass · 0 faults · mandatory invariants hold
`}
                  </span>
                  {`  touches      AC-2 · profile MER-MOD
               → 3 implementations · 2 mappings · 1 assessment

`}
                  <span className="text-green-700 dark:text-green-400 font-bold">
                    {`commit ok · receipt written · resolved · profile MER-MOD v1.4.3`}
                  </span>
                </>
              )}
            </pre>

            {/* Quick Action Bar under Terminal */}
            <div className="flex flex-wrap items-center justify-between p-2.5 border-t border-ck-hairline bg-ck-bg-1 text-[11px] gap-2.5">
              <div className="flex flex-wrap items-center gap-2">
                {!isPassing ? (
                  <>
                    <Button
                      size="sm"
                      variant="default"
                      onClick={handleQuickFixOwner}
                      className="h-6 text-[10px] font-mono bg-green-700 hover:bg-green-800 text-white shrink-0"
                    >
                      Quick-Fix: Assign Owner (T. Mori)
                    </Button>
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={handleWaiveFault}
                      className="h-6 text-[10px] font-mono shrink-0"
                    >
                      Waive Fault (`mizan waive`)
                    </Button>
                  </>
                ) : (
                  <span className="text-green-700 dark:text-green-400 font-semibold">
                    ✓ Pre-commit verification clean. Ready to push to origin.
                  </span>
                )}
              </div>
              <span className="text-ck-fg-mute text-[10px] whitespace-nowrap shrink-0 font-mono ml-auto">
                exit-code: {!isPassing ? "1" : "0"}
              </span>
            </div>
          </div>

          {/* Run History */}
          <div className="border border-ck-hairline-strong bg-ck-bg-1 p-3 space-y-2">
            <span className="text-[10px] uppercase font-bold text-ck-fg-mute block font-mono">
              Pipeline Execution History
            </span>
            <div className="space-y-1.5">
              {runHistory.map((rn) => (
                <div
                  key={rn.no}
                  className="flex items-center justify-between text-[11px] border-b border-ck-hairline pb-1"
                >
                  <span className="font-bold text-ck-fg-1">#{rn.no}</span>
                  <span
                    className={
                      rn.verdict.includes("PASS")
                        ? "text-green-700 dark:text-green-400 font-semibold"
                        : rn.verdict.includes("WAIVED")
                          ? "text-yellow-600 dark:text-yellow-400 font-semibold"
                          : "text-red-700 dark:text-red-400 font-semibold"
                    }
                  >
                    {rn.verdict}
                  </span>
                  <span className="text-ck-fg-mute">{rn.duration}</span>
                  <span className="text-ck-fg-mute">{rn.time}</span>
                </div>
              ))}
            </div>
          </div>

          {/* Continuous Governance Contract Info */}
          <div className="border border-ck-hairline bg-ck-bg-1 p-3 space-y-1.5">
            <span className="text-[10px] uppercase font-bold text-ck-fg-mute block">
              Same Verbs Everywhere
            </span>
            <p className="font-mono text-xs text-ck-fg-2">
              mizan validate · mizan resolve · mizan map diff · mizan impact
              AC-2
            </p>
            <p className="font-sans text-[11px] text-ck-fg-mute">
              If an action can be performed in this UI, it is backed by an
              equivalent atomic CLI verb. The repository is the single source of
              truth.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
