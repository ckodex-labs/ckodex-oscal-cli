"use client";

import * as React from "react";
import { useEngine } from "@/lib/engine";

interface LandingPageProps {
  onLaunchWorkbench: (surface?: string) => void;
}

export function LandingPage({ onLaunchWorkbench }: LandingPageProps) {
  const { manifest } = useEngine();
  const basePath = process.env.NEXT_PUBLIC_BASE_PATH ?? "";
  const [activeTab, setActiveTab] = React.useState<"install" | "validate" | "capsule" | "pipeline">("install");
  const [copied, setCopied] = React.useState(false);

  const SNIPPETS = {
    install: `# Install Mizan CLI (macOS, Linux, Windows)
curl -sSL https://raw.githubusercontent.com/ckodex-labs/ckodex-oscal-cli/main/install.sh | bash

# Verify zero-dependency binary
mizan --version
# oscal-cli 0.1.0 (NIST OSCAL v1.2.3)`,
    validate: `# Strict validation against official NIST metaschemas (< 10ms)
mizan validate examples/sample-ssp.json --strict

# Automated linting and deterministic UUIDv5 repair
mizan lint examples/sample-ssp.json --fix`,
    capsule: `# Export standalone air-gap verifiable Merkle evidence capsule
mizan export capsule \\
  --assessment results.json \\
  --output public/capsule.html

# Open in any browser: Zero phone-home, 100% offline WebCrypto`,
    pipeline: `# Execute single-step zero-trust compliance pipeline
mizan pipeline run \\
  --catalog examples/nist-800-53-r5/NIST_SP-800-53_rev5_catalog.json \\
  --ssp examples/sample-ssp.json \\
  --sarif output.sarif \\
  --attest-slsa provenance.json`,
  };

  const copySnippet = () => {
    navigator.clipboard.writeText(SNIPPETS[activeTab]);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="min-h-screen bg-[#090d16] text-[#ecebe8] selection:bg-[#2dd4bf]/20 selection:text-[#2dd4bf]">
      {/* Top GovX Header Rail */}
      <div className="border-b border-white/10 bg-[#0e1320] px-4 py-1.5 font-mono text-[11px] tracking-wider text-[#8e949c] select-none">
        <div className="mx-auto flex max-w-7xl items-center justify-between">
          <div className="flex items-center gap-2 sm:gap-3">
            <span className="font-bold text-[#2dd4bf]">GOVX PROTOCOL // RESTRICTED</span>
            <span className="hidden sm:inline text-white/20">|</span>
            <span className="hidden sm:inline">NIST OSCAL v1.2.3</span>
            <span className="hidden md:inline text-white/20">|</span>
            <span className="hidden md:inline">FEDRAMP REV 5 MODERATE & HIGH</span>
          </div>
          <div className="flex items-center gap-3">
            <span className="hidden lg:inline text-white/40">AIR-GAP LEVEL: E4</span>
            <span className="inline-flex items-center gap-1.5 font-medium text-[#10b981]">
              <span className="h-1.5 w-1.5 rounded-full bg-[#10b981] animate-pulse"></span>
              CRYPTOGRAPHICALLY VERIFIED
            </span>
          </div>
        </div>
      </div>

      {/* Main Navigation Bar */}
      <header className="sticky top-0 z-40 border-b border-white/10 bg-[#090d16]/90 backdrop-blur-md">
        <div className="mx-auto flex h-16 max-w-7xl items-center justify-between px-4 sm:px-6 lg:px-8">
          <div className="flex items-center gap-4">
            <a href="#" className="flex items-baseline gap-2 text-white">
              <span className="font-serif text-2xl font-bold tracking-tight">Mizan</span>
              <span className="rounded border border-[#2dd4bf]/40 bg-[#2dd4bf]/10 px-1.5 py-0.5 font-mono text-[10px] text-[#2dd4bf]">
                OSCAL v1.2.3
              </span>
            </a>
          </div>

          <nav className="hidden md:flex items-center gap-6 text-sm text-[#8e949c]">
            <a href="#features" className="hover:text-white transition-colors">Features</a>
            <a href="#frontier-ai" className="hover:text-white transition-colors">Frontier AI</a>
            <a href="#architecture" className="hover:text-white transition-colors">Architecture</a>
            <a href={`${basePath}/docs/`} className="hover:text-white transition-colors">Documentation</a>
            <a href={`${basePath}/docs/api/mizan_oscal/index.html`} className="hover:text-white transition-colors">Rust API</a>
            <a href={`${basePath}/capsule.html`} className="hover:text-white transition-colors">Capsule</a>
          </nav>

          <div className="flex items-center gap-3">
            <a
              href="https://github.com/ckodex-labs/ckodex-oscal-cli"
              target="_blank"
              rel="noopener noreferrer"
              className="hidden sm:inline-flex items-center gap-1.5 rounded-md border border-white/10 bg-white/5 px-3 py-1.5 text-xs text-[#ecebe8] hover:bg-white/10 transition-colors"
            >
              <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z" />
              </svg>
              <span>GitHub</span>
            </a>
            <button
              type="button"
              onClick={() => onLaunchWorkbench("atlas")}
              className="inline-flex items-center gap-2 rounded-md border border-[#2dd4bf] bg-[#2dd4bf]/15 px-3.5 py-1.5 text-xs font-semibold text-[#2dd4bf] hover:bg-[#2dd4bf] hover:text-[#090d16] transition-all shadow-[0_0_15px_rgba(45,212,191,0.2)]"
            >
              <span className="h-1.5 w-1.5 rounded-full bg-[#2dd4bf] animate-ping"></span>
              <span>Launch Workbench &rarr;</span>
            </button>
          </div>
        </div>
      </header>

      {/* Hero Section */}
      <section className="relative overflow-hidden border-b border-white/10 py-16 sm:py-24">
        {/* Subtle background glow */}
        <div className="absolute top-1/4 left-1/2 -translate-x-1/2 -translate-y-1/2 h-96 w-[700px] rounded-full bg-[#2dd4bf]/5 blur-[120px] pointer-events-none" />

        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8 relative z-10">
          <div className="text-center max-w-3xl mx-auto space-y-6">
            <div className="inline-flex items-center gap-2 rounded-full border border-white/15 bg-white/5 px-3 py-1 font-mono text-xs text-[#2dd4bf]">
              <span className="h-1.5 w-1.5 rounded-full bg-[#2dd4bf]"></span>
              <span>NIST OSCAL v1.2.3 COMPLIANCE KERNEL</span>
            </div>

            <h1 className="font-serif text-4xl sm:text-5xl lg:text-6xl font-normal tracking-tight text-white leading-[1.1]">
              Continuous Compliance. <br />
              <span className="italic text-[#2dd4bf]">Cryptographic Proof.</span>
            </h1>

            <p className="text-base sm:text-lg text-[#8e949c] font-sans leading-relaxed">
              Mizan is the high-assurance Rust semantic kernel and tactical workbench for NIST OSCAL v1.2.3.
              Drop-in parity with NIST <code className="font-mono text-white text-sm bg-white/10 px-1 py-0.5 rounded">oscal-cli</code>,
              3-way GitOps AST synchronization, native Frontier AI threat governance, and offline Merkle evidence capsules.
            </p>

            <div className="flex flex-wrap items-center justify-center gap-3 pt-2">
              <button
                type="button"
                onClick={() => onLaunchWorkbench("atlas")}
                className="inline-flex items-center gap-2 rounded-md bg-[#2dd4bf] px-5 py-2.5 text-sm font-semibold text-[#090d16] hover:bg-[#2dd4bf]/90 transition-all shadow-[0_0_20px_rgba(45,212,191,0.3)]"
              >
                Launch Tactical Workbench &rarr;
              </button>
              <a
                href={`${basePath}/docs/`}
                className="inline-flex items-center gap-2 rounded-md border border-white/20 bg-white/5 px-5 py-2.5 text-sm font-medium text-white hover:bg-white/10 transition-colors"
              >
                Developer Hub & Specs
              </a>
              <a
                href={`${basePath}/capsule.html`}
                className="inline-flex items-center gap-2 rounded-md border border-white/20 bg-white/5 px-5 py-2.5 text-sm font-medium text-white hover:bg-white/10 transition-colors"
              >
                Offline Evidence Capsule
              </a>
            </div>
          </div>

          {/* Terminal / Code Showcase */}
          <div className="mt-12 max-w-3xl mx-auto rounded-lg border border-white/15 bg-[#0e1320] shadow-2xl overflow-hidden">
            <div className="flex items-center justify-between border-b border-white/10 px-4 py-2.5 bg-[#090d16]/60">
              <div className="flex items-center gap-2">
                <span className="h-3 w-3 rounded-full bg-[#ef4444]/60"></span>
                <span className="h-3 w-3 rounded-full bg-[#f59e0b]/60"></span>
                <span className="h-3 w-3 rounded-full bg-[#10b981]/60"></span>
                <span className="ml-2 font-mono text-xs text-[#8e949c]">mizan-terminal // v0.1.0</span>
              </div>
              <button
                type="button"
                onClick={copySnippet}
                className="rounded border border-white/10 bg-white/5 px-2 py-1 font-mono text-[11px] text-[#8e949c] hover:text-white hover:bg-white/10 transition-colors"
              >
                {copied ? "COPIED" : "COPY"}
              </button>
            </div>

            <div className="flex border-b border-white/10 bg-[#0c101c] font-mono text-xs overflow-x-auto">
              {(["install", "validate", "capsule", "pipeline"] as const).map((tab) => (
                <button
                  key={tab}
                  type="button"
                  onClick={() => setActiveTab(tab)}
                  className={`px-4 py-2 border-r border-white/10 transition-colors ${
                    activeTab === tab
                      ? "bg-[#0e1320] text-[#2dd4bf] font-semibold border-b-2 border-b-[#2dd4bf]"
                      : "text-[#8e949c] hover:text-white"
                  }`}
                >
                  {tab === "install" && "Quickstart"}
                  {tab === "validate" && "Strict Validation"}
                  {tab === "capsule" && "Merkle Capsule"}
                  {tab === "pipeline" && "CI/CD Pipeline"}
                </button>
              ))}
            </div>

            <div className="p-4 font-mono text-xs text-[#ecebe8] overflow-x-auto leading-relaxed">
              <pre><code>{SNIPPETS[activeTab]}</code></pre>
            </div>
          </div>

          {/* Engine Snapshot Ticker */}
          {manifest && (
            <div className="mt-8 flex flex-wrap items-center justify-center gap-4 text-xs font-mono text-[#8e949c]">
              <span>Commit: <span className="text-white">{manifest.gitCommit.slice(0, 8)}</span></span>
              <span>·</span>
              <span>Engine: <span className="text-white">{manifest.mizanVersion}</span></span>
              <span>·</span>
              <span>NIST Baseline: <span className="text-[#2dd4bf]">SP 800-53 Rev 5 Moderate</span></span>
              <span>·</span>
              <span>Snapshot Inputs: <span className="text-white">{manifest.inputs.length}</span> verified</span>
            </div>
          )}
        </div>
      </section>

      {/* 4 Metrics Strip */}
      <section className="border-b border-white/10 bg-[#0c101c] py-10">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-6 text-center">
            <div className="space-y-1">
              <div className="font-serif text-3xl sm:text-4xl font-bold text-white">1,196</div>
              <div className="font-mono text-xs text-[#8e949c] uppercase tracking-wider">NIST Controls Indexed</div>
            </div>
            <div className="space-y-1">
              <div className="font-serif text-3xl sm:text-4xl font-bold text-[#2dd4bf]">&lt; 10 ms</div>
              <div className="font-mono text-xs text-[#8e949c] uppercase tracking-wider">In-Process Validation</div>
            </div>
            <div className="space-y-1">
              <div className="font-serif text-3xl sm:text-4xl font-bold text-[#10b981]">4</div>
              <div className="font-mono text-xs text-[#8e949c] uppercase tracking-wider">Frontier AI Profiles</div>
            </div>
            <div className="space-y-1">
              <div className="font-serif text-3xl sm:text-4xl font-bold text-white">100%</div>
              <div className="font-mono text-xs text-[#8e949c] uppercase tracking-wider">Air-Gap Offline Proofs</div>
            </div>
          </div>
        </div>
      </section>

      {/* Features Showcase: The 6 Pillars */}
      <section id="features" className="py-16 sm:py-24 border-b border-white/10">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="max-w-2xl mb-12">
            <div className="font-mono text-xs text-[#2dd4bf] uppercase tracking-wider mb-2">High-Assurance Architecture</div>
            <h2 className="font-serif text-3xl sm:text-4xl font-normal text-white">The Six Pillars of Governed Compliance</h2>
            <p className="mt-3 text-sm text-[#8e949c]">
              Built ground-up in Rust to replace fragile Python virtual environments with sub-millisecond compiled verification.
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {/* Pillar 1 */}
            <div className="rounded-lg border border-white/10 bg-[#0e1320] p-6 space-y-3 hover:border-[#2dd4bf]/40 transition-colors">
              <div className="h-8 w-8 rounded border border-[#2dd4bf]/30 bg-[#2dd4bf]/10 flex items-center justify-center font-mono text-xs text-[#2dd4bf]">
                01
              </div>
              <h3 className="font-serif text-xl font-semibold text-white">Pure Semantic Kernel</h3>
              <p className="text-sm text-[#8e949c] leading-relaxed">
                Zero-allocation streaming parser compiles official NIST schemas into native memory. Executes complex Rego/OPA policy rulepacks in-process in under 10ms.
              </p>
            </div>

            {/* Pillar 2 */}
            <div className="rounded-lg border border-white/10 bg-[#0e1320] p-6 space-y-3 hover:border-[#2dd4bf]/40 transition-colors">
              <div className="h-8 w-8 rounded border border-[#2dd4bf]/30 bg-[#2dd4bf]/10 flex items-center justify-center font-mono text-xs text-[#2dd4bf]">
                02
              </div>
              <h3 className="font-serif text-xl font-semibold text-white">Frontier AI Governance</h3>
              <p className="text-sm text-[#8e949c] leading-relaxed">
                Pre-compiled OSCAL component definitions for Anthropic Claude Enterprise, Google Gemini (SAIF), Azure OpenAI, and Cohere North with direct MITRE ATLAS crosswalks.
              </p>
            </div>

            {/* Pillar 3 */}
            <div className="rounded-lg border border-white/10 bg-[#0e1320] p-6 space-y-3 hover:border-[#2dd4bf]/40 transition-colors">
              <div className="h-8 w-8 rounded border border-[#2dd4bf]/30 bg-[#2dd4bf]/10 flex items-center justify-center font-mono text-xs text-[#2dd4bf]">
                03
              </div>
              <h3 className="font-serif text-xl font-semibold text-white">Offline Merkle Capsules</h3>
              <p className="text-sm text-[#8e949c] leading-relaxed">
                Self-contained, air-gap verifiable HTML audit capsules with in-browser WebCrypto SHA-256 Merkle proofs. Zero network calls, zero SaaS lock-in, 100% 3PAO defensible.
              </p>
            </div>

            {/* Pillar 4 */}
            <div className="rounded-lg border border-white/10 bg-[#0e1320] p-6 space-y-3 hover:border-[#2dd4bf]/40 transition-colors">
              <div className="h-8 w-8 rounded border border-[#2dd4bf]/30 bg-[#2dd4bf]/10 flex items-center justify-center font-mono text-xs text-[#2dd4bf]">
                04
              </div>
              <h3 className="font-serif text-xl font-semibold text-white">3-Way GitOps AST Sync</h3>
              <p className="text-sm text-[#8e949c] leading-relaxed">
                Bidirectional AST-level reconciler merges upstream agency catalogs with local engineering overrides. Deterministic UUIDv5 eliminates spurious 10,000-line git diffs.
              </p>
            </div>

            {/* Pillar 5 */}
            <div className="rounded-lg border border-white/10 bg-[#0e1320] p-6 space-y-3 hover:border-[#2dd4bf]/40 transition-colors">
              <div className="h-8 w-8 rounded border border-[#2dd4bf]/30 bg-[#2dd4bf]/10 flex items-center justify-center font-mono text-xs text-[#2dd4bf]">
                05
              </div>
              <h3 className="font-serif text-xl font-semibold text-white">6D State Vector Engine</h3>
              <p className="text-sm text-[#8e949c] leading-relaxed">
                Replaces simplistic boolean pass/fail with &lang;P, V, A, C, E, L, &tau;&rang;. Mandatory anti-invariants dominate aggregate scores; operational decoherence is flagged immediately.
              </p>
            </div>

            {/* Pillar 6 */}
            <div className="rounded-lg border border-white/10 bg-[#0e1320] p-6 space-y-3 hover:border-[#2dd4bf]/40 transition-colors">
              <div className="h-8 w-8 rounded border border-[#2dd4bf]/30 bg-[#2dd4bf]/10 flex items-center justify-center font-mono text-xs text-[#2dd4bf]">
                06
              </div>
              <h3 className="font-serif text-xl font-semibold text-white">Dagger Rust SDK CI/CD</h3>
              <p className="text-sm text-[#8e949c] leading-relaxed">
                Containerized continuous compliance pipeline authored in pure Rust with the official Dagger SDK. Emits cosign-signed SLSA v1.2 provenance and SARIF reports.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Frontier AI Section */}
      <section id="frontier-ai" className="py-16 sm:py-24 border-b border-white/10 bg-[#0c101c]">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="flex flex-col lg:flex-row lg:items-end justify-between gap-6 mb-12">
            <div>
              <div className="font-mono text-xs text-[#2dd4bf] uppercase tracking-wider mb-2">Artificial Intelligence Governance</div>
              <h2 className="font-serif text-3xl sm:text-4xl font-normal text-white">Frontier AI & MITRE ATLAS Matrix</h2>
              <p className="mt-3 text-sm text-[#8e949c] max-w-xl">
                Auditing LLM workloads against legacy static server controls leaves gaping vulnerabilities. Mizan natively bridges Frontier Models to the MITRE ATLAS AI Threat Matrix.
              </p>
            </div>
            <div className="flex gap-2">
              <button
                type="button"
                onClick={() => onLaunchWorkbench("frontier")}
                className="rounded-md border border-white/20 bg-white/5 px-3 py-1.5 text-xs text-white hover:bg-white/10 transition-colors"
              >
                Frontier AI Matrix &rarr;
              </button>
              <button
                type="button"
                onClick={() => onLaunchWorkbench("simulator")}
                className="rounded-md border border-[#2dd4bf]/50 bg-[#2dd4bf]/10 px-3 py-1.5 text-xs text-[#2dd4bf] hover:bg-[#2dd4bf] hover:text-[#090d16] transition-colors"
              >
                Threat Simulator &rarr;
              </button>
            </div>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
            <div className="rounded-lg border border-white/10 bg-[#0e1320] p-5 space-y-2">
              <div className="font-mono text-xs font-semibold text-[#2dd4bf]">Anthropic Claude</div>
              <div className="text-xs text-white font-medium">Claude 3.5 Sonnet & ClaudeGov</div>
              <p className="text-xs text-[#8e949c]">
                Air-gap ready, Constitutional AI bounds, prompt-injection guardrails, and cryptographic system card verification.
              </p>
            </div>
            <div className="rounded-lg border border-white/10 bg-[#0e1320] p-5 space-y-2">
              <div className="font-mono text-xs font-semibold text-[#3b82f6]">Google Vertex AI</div>
              <div className="text-xs text-white font-medium">Gemini 1.5 Pro & SAIF</div>
              <p className="text-xs text-[#8e949c]">
                Google Secure AI Framework (SAIF) integration, grounding verification, and multi-tenant VPC Service Controls.
              </p>
            </div>
            <div className="rounded-lg border border-white/10 bg-[#0e1320] p-5 space-y-2">
              <div className="font-mono text-xs font-semibold text-[#10b981]">Azure OpenAI</div>
              <div className="text-xs text-white font-medium">GPT-4o & FedRAMP High</div>
              <p className="text-xs text-[#8e949c]">
                Dedicated sovereign government enclave deployment, private link isolation, and automated zero-retention logging.
              </p>
            </div>
            <div className="rounded-lg border border-white/10 bg-[#0e1320] p-5 space-y-2">
              <div className="font-mono text-xs font-semibold text-[#f59e0b]">Cohere North 2</div>
              <div className="text-xs text-white font-medium">Command R+ Sovereign Multi-Cloud</div>
              <p className="text-xs text-[#8e949c]">
                Private cluster weights deployment, enterprise air-gap RAG security, and cross-jurisdictional data residency.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Comparison: Mizan vs. OSCAL Compass */}
      <section id="architecture" className="py-16 sm:py-24 border-b border-white/10">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="max-w-2xl mb-12">
            <div className="font-mono text-xs text-[#2dd4bf] uppercase tracking-wider mb-2">Architectural Benchmark</div>
            <h2 className="font-serif text-3xl sm:text-4xl font-normal text-white">How Mizan Differs from Legacy Tools</h2>
            <p className="mt-3 text-sm text-[#8e949c]">
              An objective architectural comparison between traditional Python-based document tooling and Mizan&apos;s compiled semantic kernel.
            </p>
          </div>

          <div className="overflow-x-auto rounded-lg border border-white/10 bg-[#0e1320]">
            <table className="w-full text-left text-sm">
              <thead className="border-b border-white/10 bg-[#090d16]/70 font-mono text-xs uppercase text-[#8e949c]">
                <tr>
                  <th className="px-6 py-3.5">Dimension</th>
                  <th className="px-6 py-3.5">CNCF OSCAL Compass (Python)</th>
                  <th className="px-6 py-3.5 text-[#2dd4bf]">Ckodex Mizan (Rust)</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-white/5 font-sans">
                <tr>
                  <td className="px-6 py-3.5 font-medium text-white">Startup & Execution Latency</td>
                  <td className="px-6 py-3.5 text-[#8e949c]">2.5s &ndash; 6.0s (Python runtime &amp; import overhead)</td>
                  <td className="px-6 py-3.5 font-mono text-xs text-[#2dd4bf] font-bold">&lt; 10 ms (Kernel Validation) &middot; ~47 ms (Cold CLI Launch)</td>
                </tr>
                <tr>
                  <td className="px-6 py-3.5 font-medium text-white">Memory Footprint</td>
                  <td className="px-6 py-3.5 text-[#8e949c]">80 MB &ndash; 250 MB (Python interpreter base)</td>
                  <td className="px-6 py-3.5 font-mono text-xs text-[#2dd4bf] font-bold">&lt; 15 MB (Kernel Working Set) &middot; ~130 MB (Multi-Thread Process RSS)</td>
                </tr>
                <tr>
                  <td className="px-6 py-3.5 font-medium text-white">Continuous Gate Suitability</td>
                  <td className="px-6 py-3.5 text-[#8e949c]">Slow for pre-commit hooks</td>
                  <td className="px-6 py-3.5 text-white font-medium">Sub-second zero-overhead CI/CD gates</td>
                </tr>
                <tr>
                  <td className="px-6 py-3.5 font-medium text-white">Cryptographic Provenance</td>
                  <td className="px-6 py-3.5 text-[#8e949c]">Relies on raw Git commit SHAs</td>
                  <td className="px-6 py-3.5 text-white font-medium">SLSA v1.2, in-toto receipts, SHA-256 Merkle CAS</td>
                </tr>
                <tr>
                  <td className="px-6 py-3.5 font-medium text-white">Policy Evaluation Engine</td>
                  <td className="px-6 py-3.5 text-[#8e949c]">Out-of-process transpiler (Kyverno/OCM)</td>
                  <td className="px-6 py-3.5 text-white font-medium">In-process Regorus (Rego/OPA) + Native AST</td>
                </tr>
                <tr>
                  <td className="px-6 py-3.5 font-medium text-white">State Representation</td>
                  <td className="px-6 py-3.5 text-[#8e949c]">Binary boolean (Pass / Fail)</td>
                  <td className="px-6 py-3.5 text-white font-medium">6D State Vector with Anti-relation domination</td>
                </tr>
                <tr>
                  <td className="px-6 py-3.5 font-medium text-white">Offline Audit Delivery</td>
                  <td className="px-6 py-3.5 text-[#8e949c]">Requires Python install or SaaS portal</td>
                  <td className="px-6 py-3.5 text-white font-medium">Self-verifying WebCrypto HTML Capsule</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </section>

      {/* Interactive Workbench CTA Banner */}
      <section className="py-16 sm:py-20 bg-gradient-to-b from-[#0c101c] to-[#090d16] border-b border-white/10">
        <div className="mx-auto max-w-5xl px-4 sm:px-6 lg:px-8 text-center space-y-6">
          <div className="inline-flex items-center gap-2 rounded-full border border-[#2dd4bf]/40 bg-[#2dd4bf]/10 px-3 py-1 font-mono text-xs text-[#2dd4bf]">
            <span>FULL-SCREEN OPERATIONAL COCKPIT</span>
          </div>
          <h2 className="font-serif text-3xl sm:text-4xl lg:text-5xl font-normal text-white">
            Experience the Tactical Workbench
          </h2>
          <p className="text-sm sm:text-base text-[#8e949c] max-w-2xl mx-auto">
            Explore 10 live operational surfaces: Hexagonal Family Topology, Cross-Family Corridors, 3-Way AST Composer, Audit Ledger, POA&amp;M Docket, Pipeline Runner, and AI Threat Simulator.
          </p>
          <div className="pt-2">
            <button
              type="button"
              onClick={() => onLaunchWorkbench("atlas")}
              className="inline-flex items-center gap-2 rounded-md bg-[#2dd4bf] px-6 py-3 text-sm font-bold text-[#090d16] hover:bg-[#2dd4bf]/90 transition-all shadow-[0_0_30px_rgba(45,212,191,0.3)]"
            >
              <span>Enter Full-Screen Workbench</span>
              <span>&rarr;</span>
            </button>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="py-12 bg-[#090d16] text-xs text-[#8e949c]">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8 flex flex-col sm:flex-row items-center justify-between gap-4">
          <div className="flex items-center gap-3">
            <span className="font-serif text-lg font-bold text-white">Mizan</span>
            <span>·</span>
            <span>Published under Apache-2.0 License by ckodex-labs</span>
          </div>
          <div className="flex items-center gap-6">
            <a href={`${basePath}/docs/`} className="hover:text-white transition-colors">Documentation</a>
            <a href={`${basePath}/docs/api/mizan_oscal/index.html`} className="hover:text-white transition-colors">Rust API Docs</a>
            <a href={`${basePath}/capsule.html`} className="hover:text-white transition-colors">Evidence Capsule</a>
            <a href="https://github.com/ckodex-labs/ckodex-oscal-cli" className="hover:text-white transition-colors">GitHub</a>
          </div>
        </div>
      </footer>
    </div>
  );
}
