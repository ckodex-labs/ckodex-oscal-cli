"use client";

import * as React from "react";
import {
  ShieldCheck,
  Terminal,
  FileCode,
  CheckCircle2,
  AlertTriangle,
  Play,
  PackageCheck,
  Download,
} from "lucide-react";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

export function CicdSbomPanel() {
  const [activeSubTab, setActiveSubTab] = React.useState<
    "rulepacks" | "cicd" | "sbom"
  >("rulepacks");
  const [selectedRule, setSelectedRule] =
    React.useState<string>("cis-k8s-5.2.1");
  const [evaluating, setEvaluating] = React.useState(false);
  const [evalResult, setEvalResult] = React.useState<{
    passed: boolean;
    message: string;
    findings: string[];
  }>({
    passed: false,
    message:
      "Container 'production-api' specifies privileged: true in securityContext",
    findings: ["Container production-api has privileged: true"],
  });

  const rules = [
    {
      id: "cis-k8s-5.2.1",
      name: "Disallow Privileged Containers",
      framework: "CIS K8s 1.8 / FedRAMP AC-6",
      severity: "High",
      desc: "Privileged containers share host capabilities and must be blocked.",
      samplePayload: JSON.stringify(
        {
          apiVersion: "v1",
          kind: "Pod",
          spec: {
            containers: [
              {
                name: "production-api",
                securityContext: {
                  privileged: true,
                  runAsNonRoot: false,
                },
              },
            ],
          },
        },
        null,
        2,
      ),
    },
    {
      id: "cis-k8s-5.2.6",
      name: "Require Read-Only Root Filesystem",
      framework: "CIS K8s 1.8 / FedRAMP SI-4",
      severity: "Medium",
      desc: "Containers must run with read-only root filesystems to prevent binary modification.",
      samplePayload: JSON.stringify(
        {
          apiVersion: "v1",
          kind: "Pod",
          spec: {
            containers: [
              {
                name: "worker-proc",
                securityContext: {
                  readOnlyRootFilesystem: false,
                },
              },
            ],
          },
        },
        null,
        2,
      ),
    },
    {
      id: "fedramp-ac-2",
      name: "Enforce Non-Root Execution",
      framework: "FedRAMP High / NIST AC-2",
      severity: "High",
      desc: "Containers must enforce runAsNonRoot: true for non-privileged execution context.",
      samplePayload: JSON.stringify(
        {
          apiVersion: "v1",
          kind: "Pod",
          spec: {
            containers: [
              {
                name: "database-proxy",
                securityContext: {
                  runAsNonRoot: true,
                  readOnlyRootFilesystem: true,
                },
              },
            ],
          },
        },
        null,
        2,
      ),
    },
    {
      id: "itsg33-boundary-isolation",
      name: "CCCS Sovereign Boundary Isolation",
      framework: "CCCS ITSG-33 / PBMM SC-7",
      severity: "High",
      desc: "Enforce ingress and egress network isolation rules for Protected B workloads.",
      samplePayload: JSON.stringify(
        {
          apiVersion: "networking.k8s.io/v1",
          kind: "NetworkPolicy",
          spec: {
            ingress: [
              {
                from: [{ podSelector: { matchLabels: { role: "frontend" } } }],
              },
            ],
          },
        },
        null,
        2,
      ),
    },
  ];

  const currentRuleObj = rules.find((r) => r.id === selectedRule) || rules[0];

  const handleRunEval = async (isGood: boolean) => {
    setEvaluating(true);
    let payload = currentRuleObj.samplePayload;

    if (isGood) {
      if (selectedRule === "cis-k8s-5.2.1") {
        payload = JSON.stringify(
          {
            apiVersion: "v1",
            kind: "Pod",
            spec: {
              containers: [
                {
                  name: "production-api",
                  securityContext: {
                    privileged: false,
                    runAsNonRoot: true,
                  },
                },
              ],
            },
          },
          null,
          2,
        );
      } else if (selectedRule === "cis-k8s-5.2.6") {
        payload = JSON.stringify(
          {
            apiVersion: "v1",
            kind: "Pod",
            spec: {
              containers: [
                {
                  name: "worker-proc",
                  securityContext: {
                    readOnlyRootFilesystem: true,
                  },
                },
              ],
            },
          },
          null,
          2,
        );
      } else if (selectedRule === "fedramp-ac-2") {
        payload = JSON.stringify(
          {
            apiVersion: "v1",
            kind: "Pod",
            spec: {
              containers: [
                {
                  name: "core-service",
                  securityContext: {
                    runAsNonRoot: true,
                  },
                },
              ],
            },
          },
          null,
          2,
        );
      } else {
        payload = JSON.stringify(
          {
            apiVersion: "networking.k8s.io/v1",
            kind: "NetworkPolicy",
            spec: {
              ingress: [
                {
                  from: [
                    { podSelector: { matchLabels: { role: "frontend" } } },
                  ],
                },
              ],
            },
          },
          null,
          2,
        );
      }
    }

    try {
      const res = await fetch("/api/eval", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ rule: selectedRule, payload }),
      });
      const json = await res.json();
      if (json.success && json.data) {
        setEvalResult({
          passed: Boolean(json.data.passed),
          message: json.data.passed
            ? "Evaluated by Regorus engine · All compliance invariants satisfied."
            : json.data.findings?.[0] ||
              `Policy violation detected by rule '${selectedRule}'`,
          findings: json.data.findings || [],
        });
      }
    } catch (err) {
      console.error("Evaluation failed:", err);
    } finally {
      setEvaluating(false);
    }
  };

  return (
    <div className="space-y-4">
      {/* Sub-Navigation Header */}
      <div className="flex items-center justify-between border-b border-ck-hairline pb-2">
        <div className="flex items-center gap-2">
          <Button
            size="sm"
            variant={activeSubTab === "rulepacks" ? "default" : "outline"}
            onClick={() => setActiveSubTab("rulepacks")}
            className="h-7 text-xs font-mono"
          >
            Regorus Policy Rulepacks
          </Button>
          <Button
            size="sm"
            variant={activeSubTab === "cicd" ? "default" : "outline"}
            onClick={() => setActiveSubTab("cicd")}
            className="h-7 text-xs font-mono"
          >
            SARIF &amp; GitLab Exporters
          </Button>
          <Button
            size="sm"
            variant={activeSubTab === "sbom" ? "default" : "outline"}
            onClick={() => setActiveSubTab("sbom")}
            className="h-7 text-xs font-mono"
          >
            CycloneDX SBOM Ingestion
          </Button>
        </div>
        <Badge
          variant="outline"
          className="font-mono text-[10px] text-green-700 dark:text-green-400"
        >
          Engine: Regorus 0.3.4 + OSCAL 1.2
        </Badge>
      </div>

      {activeSubTab === "rulepacks" && (
        <div className="grid grid-cols-12 gap-3">
          {/* Rule Selector List */}
          <div className="col-span-5 space-y-2">
            <span className="text-[11px] font-mono text-ck-fg-mute uppercase tracking-wider block">
              Verified Compliance Rules
            </span>
            {rules.map((rule) => {
              const isSelected = rule.id === selectedRule;
              return (
                <div
                  key={rule.id}
                  onClick={() => setSelectedRule(rule.id)}
                  className={`p-2.5 border cursor-pointer transition-colors ${
                    isSelected
                      ? "border-ck-hairline-strong bg-ck-bg-2 shadow-[2px_2px_0_var(--ck-fg-1)]"
                      : "border-ck-hairline bg-ck-bg-1 hover:border-ck-hairline-strong"
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <span className="font-mono text-xs font-semibold text-ck-fg-1">
                      {rule.id}
                    </span>
                    <Badge
                      variant="outline"
                      className="text-[9px] font-mono uppercase"
                    >
                      {rule.severity}
                    </Badge>
                  </div>
                  <div className="text-xs text-ck-fg-1 font-medium mt-1">
                    {rule.name}
                  </div>
                  <div className="text-[10px] text-ck-fg-mute font-mono mt-0.5">
                    {rule.framework}
                  </div>
                </div>
              );
            })}
          </div>

          {/* Interactive Evaluation Sandbox */}
          <div className="col-span-7 space-y-3">
            <Card className="border-ck-hairline-strong bg-ck-bg-1">
              <CardHeader className="p-3 border-b border-ck-hairline">
                <div className="flex items-center justify-between">
                  <CardTitle className="text-xs font-mono flex items-center gap-1.5">
                    <Terminal className="h-3.5 w-3.5 text-accent" />
                    Rego Workload Evaluation: {selectedRule}
                  </CardTitle>
                  <div className="flex items-center gap-1.5">
                    <Button
                      size="sm"
                      onClick={() => handleRunEval(false)}
                      disabled={evaluating}
                      className="h-6 text-[10px] font-mono bg-red-600 hover:bg-red-700 text-white"
                    >
                      <Play className="h-2.5 w-2.5 mr-1" />
                      Eval Violation
                    </Button>
                    <Button
                      size="sm"
                      onClick={() => handleRunEval(true)}
                      disabled={evaluating}
                      className="h-6 text-[10px] font-mono bg-green-700 hover:bg-green-800 text-white"
                    >
                      <CheckCircle2 className="h-2.5 w-2.5 mr-1" />
                      Eval Passing
                    </Button>
                  </div>
                </div>
                <CardDescription className="text-[11px] text-ck-fg-mute text-balance">
                  {currentRuleObj.desc}
                </CardDescription>
              </CardHeader>
              <CardContent className="p-3 space-y-2">
                <pre className="p-2 bg-ck-bg-0 border border-ck-hairline font-mono text-[10px] text-ck-fg-1 overflow-x-auto max-h-40">
                  {currentRuleObj.samplePayload}
                </pre>

                {/* Live Result Output */}
                <div
                  className={`p-2 border font-mono text-xs ${
                    evalResult.passed
                      ? "border-green-700 bg-green-950/20 text-green-700 dark:text-green-400"
                      : "border-red-700 bg-red-950/20 text-red-700 dark:text-red-400"
                  }`}
                >
                  <div className="flex items-center gap-1.5 font-semibold">
                    {evalResult.passed ? (
                      <CheckCircle2 className="h-3.5 w-3.5" />
                    ) : (
                      <AlertTriangle className="h-3.5 w-3.5" />
                    )}
                    Status:{" "}
                    {evalResult.passed
                      ? "ALLOWED / COMPLIANT"
                      : "DENIED / VIOLATION DETECTED"}
                  </div>
                  <div className="text-[11px] mt-1 opacity-90">
                    {evalResult.message}
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      )}

      {activeSubTab === "cicd" && (
        <div className="grid grid-cols-2 gap-3">
          <Card className="border-ck-hairline-strong bg-ck-bg-1">
            <CardHeader className="p-3 border-b border-ck-hairline">
              <CardTitle className="text-xs font-mono flex items-center gap-1.5">
                <FileCode className="h-3.5 w-3.5 text-accent" />
                OASIS SARIF v2.1.0 Exporter
              </CardTitle>
              <CardDescription className="text-[11px]">
                Target: GitHub Code Scanning, SonarQube &amp; VS Code SARIF
                Viewer
              </CardDescription>
            </CardHeader>
            <CardContent className="p-3 space-y-2 font-mono text-[11px]">
              <div className="p-2 bg-ck-bg-0 border border-ck-hairline text-ck-fg-1">
                <div>
                  $ mizan export sarif -i catalog.json -o mizan-sarif.json
                </div>
                <div className="text-green-700 dark:text-green-400 mt-1">
                  ✓ Exported 9 controls to SARIF v2.1.0 schema with rule-level
                  NIST URIs
                </div>
              </div>
              <Button
                size="sm"
                variant="outline"
                className="w-full h-7 text-xs font-mono"
              >
                <Download className="h-3 w-3 mr-1.5" />
                Download SARIF v2.1.0 Sample
              </Button>
            </CardContent>
          </Card>

          <Card className="border-ck-hairline-strong bg-ck-bg-1">
            <CardHeader className="p-3 border-b border-ck-hairline">
              <CardTitle className="text-xs font-mono flex items-center gap-1.5">
                <ShieldCheck className="h-3.5 w-3.5 text-accent" />
                GitLab Security Scanner Report v15.0.0
              </CardTitle>
              <CardDescription className="text-[11px]">
                Target: GitLab CI/CD Security &amp; Compliance Pipeline Gates
              </CardDescription>
            </CardHeader>
            <CardContent className="p-3 space-y-2 font-mono text-[11px]">
              <div className="p-2 bg-ck-bg-0 border border-ck-hairline text-ck-fg-1">
                <div>
                  $ mizan export gitlab -i catalog.json -o
                  gl-security-report.json
                </div>
                <div className="text-green-700 dark:text-green-400 mt-1">
                  ✓ Formatted scanner ID: mizan-compliance-scanner (v15.0.0)
                </div>
              </div>
              <Button
                size="sm"
                variant="outline"
                className="w-full h-7 text-xs font-mono"
              >
                <Download className="h-3 w-3 mr-1.5" />
                Download GitLab Report Sample
              </Button>
            </CardContent>
          </Card>
        </div>
      )}

      {activeSubTab === "sbom" && (
        <Card className="border-ck-hairline-strong bg-ck-bg-1">
          <CardHeader className="p-3 border-b border-ck-hairline">
            <CardTitle className="text-xs font-mono flex items-center gap-1.5">
              <PackageCheck className="h-3.5 w-3.5 text-accent" />
              CycloneDX &amp; SPDX SBOM to OSCAL Component Definition
            </CardTitle>
            <CardDescription className="text-[11px]">
              Ingests third-party bill of materials and automatically maps
              software components to NIST SA-11 and SI-2 controls.
            </CardDescription>
          </CardHeader>
          <CardContent className="p-3 space-y-3 font-mono text-[11px]">
            <div className="grid grid-cols-3 gap-2">
              <div className="border border-ck-hairline bg-ck-bg-0 p-2 text-center">
                <span className="text-[10px] text-ck-fg-mute block">
                  Direct Dependencies
                </span>
                <span className="text-sm font-semibold text-ck-fg-1">
                  24 Packages
                </span>
              </div>
              <div className="border border-ck-hairline bg-ck-bg-0 p-2 text-center">
                <span className="text-[10px] text-ck-fg-mute block">
                  Target OSCAL Object
                </span>
                <span className="text-sm font-semibold text-ck-fg-1">
                  component-definition
                </span>
              </div>
              <div className="border border-ck-hairline bg-ck-bg-0 p-2 text-center">
                <span className="text-[10px] text-ck-fg-mute block">
                  Linked Controls
                </span>
                <span className="text-sm font-semibold text-green-700 dark:text-green-400">
                  SA-11, SI-2
                </span>
              </div>
            </div>

            <div className="p-2 bg-ck-bg-0 border border-ck-hairline text-ck-fg-1">
              <div>
                $ mizan sbom import -i cyclonedx.json -o
                oscal-component-definition.json
              </div>
              <div className="text-green-700 dark:text-green-400 mt-1">
                ✓ Generated OSCAL Component Definition (UUID:
                7a82b94e-5c61-4fa2-9382-3f81e6b01429)
              </div>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
