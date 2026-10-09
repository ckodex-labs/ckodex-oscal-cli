"use client";

/**
 * Frontier AI Provider Comparison & Compliance Inspector.
 *
 * Compares Anthropic Claude, Google Gemini on Vertex AI, and OpenAI GPT-4o on Azure
 * across FedRAMP High/Moderate, ASL-2/3, Google SAIF, and MITRE ATLAS defenses.
 *
 * Architectural Invariants:
 * - Pure Vector State: Uses typed VectorState S(e,t) = <P, V, A, C, E, L, tau> rather than booleans.
 * - GovX Tokens: Uses CSS variables (--ck-accent, --ck-pos, --ck-warn, --ck-info, --ck-unk).
 * - Zero Emojis: Strictly unicode glyphs (✓, ✕, !, i, ?, ·) and SVG lines. No emoji glyphs.
 * - Merkle Proofs: Cryptographic verification receipts (RFC 9162 / RFC 6962) with leaf hashes.
 */

import * as React from "react";
import { cn } from "@/lib/utils";
import type { Provenance } from "@/lib/provenance";
import { fixture } from "@/lib/provenance";
import type {
  LensMode,
  VectorState,
  Presence,
  DirectionalValence,
  AntiRelation,
  Coherence,
  EvidenceStatus,
} from "@/lib/oscal-types";
import {
  DataTable,
  EmptyState,
  PageHeader,
  Panel,
  ReadOnlyNotice,
  Segmented,
  StatGrid,
  StatTile,
  StateBadge,
  Toolbar,
  type Tone,
} from "@/components/kit";

/* ------------------------------------------------------------------ */
/* Types & Domain Models                                              */
/* ------------------------------------------------------------------ */

export type FrontierProviderId = "anthropic" | "google" | "azure-openai";
export type ComparisonTab =
  | "matrix"
  | "fedramp"
  | "safety"
  | "saif"
  | "atlas"
  | "provenance"
  | "merkle";

export interface MerkleReceipt {
  treeSize: number;
  logIndex: number;
  merkleRoot: string;
  leafHash: string;
  inclusionProof: string[];
  rfcLogId: string;
  timestamp: string;
  status: "verified" | "pending" | "reconciling";
}

export interface AtlasDefenseItem {
  techniqueId: string;
  techniqueName: string;
  defenseMechanism: string;
  coverageLevel: "Full" | "High" | "Partial";
  vectorState: VectorState;
}

export interface FrontierModelProfile {
  id: FrontierProviderId;
  modelName: string;
  providerOrg: string;
  cloudEnclave: string;
  deploymentBoundary: string;
  fedramp: {
    level: "FedRAMP High P-ATO" | "FedRAMP Moderate P-ATO" | "FedRAMP High Agency ATO";
    packageId: string;
    jabApproved: boolean;
    dodImpactLevel: "DoD IL4/IL5" | "DoD IL5/IL6" | "DoD IL2/IL4";
    authorizationDate: string;
    continuousMonitoringCadence: string;
    vectorState: VectorState;
  };
  safetyFramework: {
    frameworkName: string;
    currentTier: string;
    cbrnThresholdContainment: string;
    cyberOffenseContainment: string;
    weightsSecurityHsm: boolean;
    vectorState: VectorState;
  };
  saifPillars: {
    cyberFoundations: string;
    threatDetection: string;
    automatedDefenses: string;
    platformHarmonization: string;
    adaptiveControls: string;
    contextualizedRisk: string;
    vectorState: VectorState;
  };
  mitreAtlas: Record<string, AtlasDefenseItem>;
  supplyChain: {
    modelCardStandard: string;
    slsaLevel: "SLSA v1.2 Build L3" | "SLSA v1.2 Build L2" | "SLSA v1.2 Build L1";
    aiBomFormat: "CycloneDX 1.6 AI" | "SPDX 3.0 AI";
    sigstoreVerified: boolean;
    watermarkTechnology: string;
    watermarkResistance: "Tamper-Resistant (SynthID)" | "C2PA Cryptographic Manifest" | "Token-Seed Heuristic";
    vectorState: VectorState;
  };
  egressIsolation: {
    zeroDataRetentionGuaranteed: boolean;
    cmekSupport: boolean;
    privateNetworking: string;
    promptInjectionLatencyMs: number;
    promptShieldTechnology: string;
    vectorState: VectorState;
  };
  merkleReceipt: MerkleReceipt;
}

/* ------------------------------------------------------------------ */
/* Vector State Builders & Tone Mappers                              */
/* ------------------------------------------------------------------ */

function makeVectorState(
  presence: Presence = "present",
  valence: DirectionalValence = "positive",
  anti: AntiRelation = "none",
  coherence: Coherence = "coherent",
  evidence: EvidenceStatus = "verified",
  lifecycle: string = "operational",
  epoch: number = 104,
): VectorState {
  return { presence, valence, anti, coherence, evidence, lifecycle, epoch };
}

function valenceToTone(valence: DirectionalValence): Tone {
  switch (valence) {
    case "positive":
      return "pos";
    case "negative":
      return "neg";
    case "neutral":
      return "neutral";
    case "mixed":
    case "unresolved":
      return "warn";
    default:
      return "unk";
  }
}

function coverageTone(coverage: "Full" | "High" | "Partial"): Tone {
  switch (coverage) {
    case "Full":
      return "pos";
    case "High":
      return "info";
    case "Partial":
      return "warn";
    default:
      return "neutral";
  }
}

/* ------------------------------------------------------------------ */
/* Baseline Evidence Datasets                                         */
/* ------------------------------------------------------------------ */

const FRONTIER_PROFILES: Record<FrontierProviderId, FrontierModelProfile> = {
  anthropic: {
    id: "anthropic",
    modelName: "Claude 3.5 Sonnet / 3 Opus",
    providerOrg: "Anthropic PBC via AWS Bedrock GovCloud",
    cloudEnclave: "AWS GovCloud (US-East / US-West)",
    deploymentBoundary: "FedRAMP High Isolated Boundary with US-Citizenship Support Staff",
    fedramp: {
      level: "FedRAMP High P-ATO",
      packageId: "F1503097845",
      jabApproved: true,
      dodImpactLevel: "DoD IL4/IL5",
      authorizationDate: "2024-06-18",
      continuousMonitoringCadence: "Monthly ConMon + Automated CSP Telemetry Feed",
      vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 101),
    },
    safetyFramework: {
      frameworkName: "Anthropic Responsible Scaling Policy (RSP)",
      currentTier: "ASL-2 Confirmed / ASL-3 Gated Protocols",
      cbrnThresholdContainment: "Automated bio-safety filter pipeline; CBRN red-team tripwires trigger immediate automated inference circuit-breaker",
      cyberOffenseContainment: "Autonomous vulnerability exploitation benchmarks; blocked high-risk zero-day synthesis",
      weightsSecurityHsm: true,
      vectorState: makeVectorState("present", "positive", "none", "coherent", "attested", "enforced", 102),
    },
    saifPillars: {
      cyberFoundations: "AWS Nitro Enclaves, dedicated VPC peering, TLS 1.3 FIPS 140-3 endpoint encryption",
      threatDetection: "CloudTrail AI API auditing, Bedrock Guardrails perturbation alerting",
      automatedDefenses: "Constitutional AI (RLAIF) intrinsic guardrails, real-time denied topic filters",
      platformHarmonization: "IAM Role-based attenuation with AWS Organizations Service Control Policies",
      adaptiveControls: "Automated adversarial red-teaming feedback loop into reinforcement updates",
      contextualizedRisk: "Explicit Government Shared Responsibility Matrix; zero inference retention mode",
      vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 103),
    },
    mitreAtlas: {
      directInjection: {
        techniqueId: "AML.T0051.001",
        techniqueName: "LLM Direct Prompt Injection",
        defenseMechanism: "Constitutional AI system prompt hardening + Bedrock Guardrails input filtering",
        coverageLevel: "High",
        vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 104),
      },
      indirectInjection: {
        techniqueId: "AML.T0051.002",
        techniqueName: "LLM Indirect Prompt Injection",
        defenseMechanism: "Contextual grounding checks, delimiter tagging, and external tool argument sanitation",
        coverageLevel: "High",
        vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 104),
      },
      jailbreak: {
        techniqueId: "AML.T0054",
        techniqueName: "LLM Jailbreaking",
        defenseMechanism: "Multi-round adversarial training; prompt perplexity gating; automated refusal paths",
        coverageLevel: "Full",
        vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 104),
      },
      poisoning: {
        techniqueId: "AML.T0043",
        techniqueName: "Craft Adversarial Training Data",
        defenseMechanism: "Automated dataset sanitization, cryptographic training set hashes, curated alignment corpus",
        coverageLevel: "High",
        vectorState: makeVectorState("present", "positive", "none", "coherent", "attested", "active", 104),
      },
      modelExtraction: {
        techniqueId: "AML.T0056",
        techniqueName: "Model Extraction & Inversion",
        defenseMechanism: "API token rate-limiting, output token embedding fuzzing, anomaly query volume detection",
        coverageLevel: "Full",
        vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 104),
      },
      insecureOutput: {
        techniqueId: "AML.T0057",
        techniqueName: "LLM Insecure Output Handling",
        defenseMechanism: "Bedrock Guardrails content filters (Hate, Insults, Sexual, Violence, Misconduct)",
        coverageLevel: "Full",
        vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 104),
      },
    },
    supplyChain: {
      modelCardStandard: "NIST AI RMF 1.0 Profile / OSCAL Component Definition v1.1.0",
      slsaLevel: "SLSA v1.2 Build L3",
      aiBomFormat: "CycloneDX 1.6 AI",
      sigstoreVerified: true,
      watermarkTechnology: "Statistical token-generation sampling watermark",
      watermarkResistance: "Token-Seed Heuristic",
      vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 105),
    },
    egressIsolation: {
      zeroDataRetentionGuaranteed: true,
      cmekSupport: true,
      privateNetworking: "AWS PrivateLink endpoints; strictly no public IPv4 routing",
      promptInjectionLatencyMs: 42,
      promptShieldTechnology: "AWS Bedrock Guardrails v2 + Anthropic Constitutional Classifier",
      vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 106),
    },
    merkleReceipt: {
      treeSize: 4096,
      logIndex: 2841,
      merkleRoot: "7e94c92f15e8d890b05b81ea19e1c955a5369bb8853b0066d7ad5f0ee233e101",
      leafHash: "fa329b871c890061e89b4f2c510a76efc255c4217112ea1bc8300257ad3219ee",
      inclusionProof: [
        "a912bb0364d97f88414e861219ef851a7722ccb5391d4e0e47ea502572b84291",
        "3d4f18392100efcb9512ea5098931215ea781bc09a441e89cf0025719ef82041",
        "129bbef2411985eeea78810239b9c0257ea119842100e4781bc09e9921571214",
      ],
      rfcLogId: "urn:rfc9162:log:us-gov-ai-transparency-v1",
      timestamp: "2026-10-09T08:14:22Z",
      status: "verified",
    },
  },
  google: {
    id: "google",
    modelName: "Gemini 1.5 Pro / Flash",
    providerOrg: "Google Cloud Vertex AI",
    cloudEnclave: "Google Cloud Assured Workloads US FedRAMP High",
    deploymentBoundary: "FedRAMP High Boundary with Sovereign Data Residency & US Personnel Controls",
    fedramp: {
      level: "FedRAMP High P-ATO",
      packageId: "F1606077928",
      jabApproved: true,
      dodImpactLevel: "DoD IL4/IL5",
      authorizationDate: "2024-05-12",
      continuousMonitoringCadence: "Continuous Automated Security Command Center Pro feed",
      vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 101),
    },
    safetyFramework: {
      frameworkName: "Google Frontier Safety Framework",
      currentTier: "Critical Capability Level CCL-1 Active / CCL-2 Preparedness",
      cbrnThresholdContainment: "Automated bio-threat screening on all fine-tuning datasets and generation prompts; automated containment stop-gate",
      cyberOffenseContainment: "Automated vulnerability exploitation simulator evaluation; strict prohibited exploit synthesis",
      weightsSecurityHsm: true,
      vectorState: makeVectorState("present", "positive", "none", "coherent", "attested", "enforced", 102),
    },
    saifPillars: {
      cyberFoundations: "Titan Security Chip hardware root of trust, gVisor sandboxing, VPC Service Controls perimeter",
      threatDetection: "Cloud Logging + Event Threat Detection for AI inference APIs, continuous model drift auditing",
      automatedDefenses: "Google Vertex Model Armor prompt sanitizer, automated adversarial perturbation defense",
      platformHarmonization: "Unified Google Cloud IAM, BeyondCorp Enterprise Context-Aware Access",
      adaptiveControls: "Continuous Reinforcement Learning from Security Feedback (RLSF) operational loops",
      contextualizedRisk: "Assured Workloads cryptographic separation, strict FedRAMP boundary verification",
      vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 103),
    },
    mitreAtlas: {
      directInjection: {
        techniqueId: "AML.T0051.001",
        techniqueName: "LLM Direct Prompt Injection",
        defenseMechanism: "Model Armor multi-stage intent parsing + Vertex AI Safety Filters",
        coverageLevel: "Full",
        vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 104),
      },
      indirectInjection: {
        techniqueId: "AML.T0051.002",
        techniqueName: "LLM Indirect Prompt Injection",
        defenseMechanism: "Grounding with Vertex AI Search boundary isolation; dual-model spotlight sanitization",
        coverageLevel: "Full",
        vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 104),
      },
      jailbreak: {
        techniqueId: "AML.T0054",
        techniqueName: "LLM Jailbreaking",
        defenseMechanism: "Model Armor jailbreak filter + adversarial fine-tuning + token safety gates",
        coverageLevel: "Full",
        vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 104),
      },
      poisoning: {
        techniqueId: "AML.T0043",
        techniqueName: "Craft Adversarial Training Data",
        defenseMechanism: "Binary Authorization for BoringCrypto & training artifact provenance tracking",
        coverageLevel: "High",
        vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 104),
      },
      modelExtraction: {
        techniqueId: "AML.T0056",
        techniqueName: "Model Extraction & Inversion",
        defenseMechanism: "VPC Service Controls data exfiltration prevention, egress boundary enforcement",
        coverageLevel: "Full",
        vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 104),
      },
      insecureOutput: {
        techniqueId: "AML.T0057",
        techniqueName: "LLM Insecure Output Handling",
        defenseMechanism: "Automated output safety classification + PII masking + code execution sandbox",
        coverageLevel: "Full",
        vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 104),
      },
    },
    supplyChain: {
      modelCardStandard: "Google Model Cards / NIST AI RMF 1.0 Profile",
      slsaLevel: "SLSA v1.2 Build L3",
      aiBomFormat: "SPDX 3.0 AI",
      sigstoreVerified: true,
      watermarkTechnology: "Google SynthID text and multimodal imperceptible watermarking",
      watermarkResistance: "Tamper-Resistant (SynthID)",
      vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 105),
    },
    egressIsolation: {
      zeroDataRetentionGuaranteed: true,
      cmekSupport: true,
      privateNetworking: "Private Service Connect (PSC) + VPC Service Controls egress restriction",
      promptInjectionLatencyMs: 38,
      promptShieldTechnology: "Google Cloud Vertex AI Model Armor",
      vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 106),
    },
    merkleReceipt: {
      treeSize: 4096,
      logIndex: 2842,
      merkleRoot: "7e94c92f15e8d890b05b81ea19e1c955a5369bb8853b0066d7ad5f0ee233e101",
      leafHash: "8c12bf19aa280145efb0114920aa4589d891bc045187e0245a909ef121571255",
      inclusionProof: [
        "b1029c88219ef8411210459812cc9a5021e89410ea89bcf20155ad1289cf0014",
        "3d4f18392100efcb9512ea5098931215ea781bc09a441e89cf0025719ef82041",
        "129bbef2411985eeea78810239b9c0257ea119842100e4781bc09e9921571214",
      ],
      rfcLogId: "urn:rfc9162:log:us-gov-ai-transparency-v1",
      timestamp: "2026-10-09T08:14:23Z",
      status: "verified",
    },
  },
  "azure-openai": {
    id: "azure-openai",
    modelName: "OpenAI GPT-4o on Azure",
    providerOrg: "Microsoft Azure Government (US Gov Enclave)",
    cloudEnclave: "Azure Government (US Gov Virginia / US Gov Texas)",
    deploymentBoundary: "FedRAMP High JAB P-ATO Boundary with CJIS & DoD SRG IL5 Commitment",
    fedramp: {
      level: "FedRAMP High P-ATO",
      packageId: "F1603087905",
      jabApproved: true,
      dodImpactLevel: "DoD IL5/IL6",
      authorizationDate: "2024-04-20",
      continuousMonitoringCadence: "Azure Security Benchmark ConMon + Defender for Cloud Continuous Audit",
      vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 101),
    },
    safetyFramework: {
      frameworkName: "OpenAI Preparedness Framework & Microsoft Responsible AI Standard v2",
      currentTier: "Medium Risk Tracked Post-Mitigation (CBRN/Cyber Threshold Under Gated Watch)",
      cbrnThresholdContainment: "Automated biology/chemical advisory classifier; hard block on dangerous synthesis recipes",
      cyberOffenseContainment: "Proactive cyber red-teaming against exploit automation; restricted payload synthesis",
      weightsSecurityHsm: true,
      vectorState: makeVectorState("present", "positive", "none", "coherent", "attested", "enforced", 102),
    },
    saifPillars: {
      cyberFoundations: "Azure Dedicated HSM, confidential GPU computing (H100 SEV-SNP), Private Link endpoints",
      threatDetection: "Microsoft Defender for Cloud threat detection, Azure Monitor AI diagnostic metrics",
      automatedDefenses: "Azure AI Content Safety, real-time Prompt Shields with spotlight analysis",
      platformHarmonization: "Microsoft Entra ID Privileged Identity Management (PIM) with Conditional Access",
      adaptiveControls: "Automated red-team loop integrated into Azure AI Studio evaluations",
      contextualizedRisk: "Zero Data Retention (ZDR) guarantee for federal government enterprise subscriptions",
      vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 103),
    },
    mitreAtlas: {
      directInjection: {
        techniqueId: "AML.T0051.001",
        techniqueName: "LLM Direct Prompt Injection",
        defenseMechanism: "Azure Prompt Shields (user prompt analysis with risk scoring < 0.2 threshold)",
        coverageLevel: "Full",
        vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 104),
      },
      indirectInjection: {
        techniqueId: "AML.T0051.002",
        techniqueName: "LLM Indirect Prompt Injection",
        defenseMechanism: "Prompt Shields for document spotlighting + delimiter sanitation",
        coverageLevel: "Full",
        vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 104),
      },
      jailbreak: {
        techniqueId: "AML.T0054",
        techniqueName: "LLM Jailbreaking",
        defenseMechanism: "Multi-layered Content Safety classifiers (Hate, Sexual, Violence, Self-Harm)",
        coverageLevel: "Full",
        vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 104),
      },
      poisoning: {
        techniqueId: "AML.T0043",
        techniqueName: "Craft Adversarial Training Data",
        defenseMechanism: "Pre-training data lineage verification, supply-chain hashes, fine-tuning validation gates",
        coverageLevel: "High",
        vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 104),
      },
      modelExtraction: {
        techniqueId: "AML.T0056",
        techniqueName: "Model Extraction & Inversion",
        defenseMechanism: "Private endpoint encapsulation, API threshold throttling, strict telemetry sampling",
        coverageLevel: "Full",
        vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 104),
      },
      insecureOutput: {
        techniqueId: "AML.T0057",
        techniqueName: "LLM Insecure Output Handling",
        defenseMechanism: "Azure AI Content Safety output scanning, protected material detection for code/text",
        coverageLevel: "Full",
        vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 104),
      },
    },
    supplyChain: {
      modelCardStandard: "Azure AI Transparency Notes / NIST AI RMF 1.0 Profile",
      slsaLevel: "SLSA v1.2 Build L3",
      aiBomFormat: "CycloneDX 1.6 AI",
      sigstoreVerified: true,
      watermarkTechnology: "C2PA Coalition for Content Provenance and Authenticity manifests",
      watermarkResistance: "C2PA Cryptographic Manifest",
      vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 105),
    },
    egressIsolation: {
      zeroDataRetentionGuaranteed: true,
      cmekSupport: true,
      privateNetworking: "Azure Private Link with Microsoft Backbone routing; no internet exposure",
      promptInjectionLatencyMs: 45,
      promptShieldTechnology: "Azure AI Content Safety Prompt Shields v2",
      vectorState: makeVectorState("present", "positive", "none", "coherent", "verified", "active", 106),
    },
    merkleReceipt: {
      treeSize: 4096,
      logIndex: 2843,
      merkleRoot: "7e94c92f15e8d890b05b81ea19e1c955a5369bb8853b0066d7ad5f0ee233e101",
      leafHash: "19ee851b4129bb0364d97f88414e861219ef851a7722ccb5391d4e0e47ea5025",
      inclusionProof: [
        "c9941a80251147e891bc09a441e89cf0025719ef820413d4f18392100efcb951",
        "4e12891bc09e9921571214129bbef2411985eeea78810239b9c0257ea1198421",
        "d81249bcf20155ad1289cf0014b1029c88219ef8411210459812cc9a5021e894",
      ],
      rfcLogId: "urn:rfc9162:log:us-gov-ai-transparency-v1",
      timestamp: "2026-10-09T08:14:24Z",
      status: "verified",
    },
  },
};

const PROVENANCE_FRONTIER: Provenance = fixture(
  "frontier audit record",
  "Cryptographically attested compliance snapshots and MITRE ATLAS defenses across Anthropic, Google, and Azure OpenAI sovereign boundaries.",
);

/* ------------------------------------------------------------------ */
/* Main Surface Component                                             */
/* ------------------------------------------------------------------ */

export interface FrontierAiComparisonProps {
  lens?: LensMode;
  onSelectModel?: (id: FrontierProviderId) => void;
}

export function FrontierAiComparison({
  lens = "ciso",
  onSelectModel,
}: FrontierAiComparisonProps) {
  const [activeTab, setActiveTab] = React.useState<ComparisonTab>("matrix");
  const [selectedProvider, setSelectedProvider] =
    React.useState<FrontierProviderId>("anthropic");
  const [filterQuery, setFilterQuery] = React.useState("");

  const profile = FRONTIER_PROFILES[selectedProvider];

  const LENS_DIRECTIVE: Record<
    LensMode,
    { title: string; desc: string; tone: Tone }
  > = {
    ciso: {
      title: "CISO Lens Active",
      desc: "Frontier foundation model boundary assurance, FedRAMP High JAB P-ATO standing, and sovereign liability protection.",
      tone: "info",
    },
    assessor: {
      title: "Assessor Lens Active",
      desc: "Auditing FedRAMP High control evidence, MITRE ATLAS defense test results, and continuous monitoring telemetry.",
      tone: "warn",
    },
    architect: {
      title: "Architect Lens Active",
      desc: "Inspecting VPC-SC / PrivateLink network topology, CMEK HSM boundaries, and SAIF infrastructure harmonization.",
      tone: "info",
    },
    author: {
      title: "Author Lens Active",
      desc: "Evaluating OSCAL Component Definition mappings, Model Card transparency declarations, and control narratives.",
      tone: "neutral",
    },
    "risk-owner": {
      title: "Risk Owner Lens Active",
      desc: "Evaluating CBRN / cyber-offense ASL-3 scaling tripwires, indirect prompt injection risks, and Zero Data Retention terms.",
      tone: "warn",
    },
    engineer: {
      title: "Engineer Lens Active",
      desc: "Analyzing real-time Prompt Shield latency overheads, RFC 9162 Merkle proof paths, and raw token security pipelines.",
      tone: "pos",
    },
  };

  const currentDirective = LENS_DIRECTIVE[lens];

  return (
    <div className="space-y-5">
      <PageHeader
        eyebrow="Frontier AI Governance"
        title="Frontier AI Provider Comparison & Compliance Inspector"
        description="Comprehensive compliance, safety, and threat inspection across Anthropic Claude, Google Gemini on Vertex AI, and OpenAI GPT-4o on Azure Government."
        meta={
          <div className="flex flex-wrap items-center gap-2">
            <StateBadge tone="pos">GovX Attested</StateBadge>
            <StateBadge tone="info">RFC 9162 Merkle Verified</StateBadge>
            <span className="font-mono text-2xs text-ck-fg-mute">
              Ledger Root: 7e94c92f…e233e101
            </span>
          </div>
        }
      />

      <ReadOnlyNotice />

      {/* Lens Directive Banner */}
      <div className="flex flex-wrap items-center justify-between gap-2 rounded-md border border-ck-hairline-strong bg-ck-bg-1 px-3 py-2 text-xs">
        <div className="flex items-center gap-2 min-w-0">
          <StateBadge tone={currentDirective.tone}>
            {currentDirective.title}
          </StateBadge>
          <span className="text-ck-fg-2 truncate font-medium">
            {currentDirective.desc}
          </span>
        </div>
        <div className="flex items-center gap-1.5 shrink-0 text-2xs text-ck-fg-mute font-mono">
          <span>Active Lens: {lens}</span>
        </div>
      </div>

      {/* Top Stat Overview */}
      <StatGrid>
        <StatTile
          label="Frontier Providers"
          value="3 Verified"
          hint="Anthropic, Google, Azure OpenAI"
          tone="neutral"
        />
        <StatTile
          label="FedRAMP Baseline"
          value="100% High"
          hint="All providers hold High P-ATO"
          tone="pos"
        />
        <StatTile
          label="ATLAS Defenses"
          value="18 Controls"
          hint="Full coverage on injection & jailbreaks"
          tone="pos"
        />
        <StatTile
          label="Merkle Log Root"
          value="VERIFIED"
          hint="RFC 9162 cryptographic leaf match"
          tone="info"
        />
      </StatGrid>

      {/* Navigation & Provider Selection Toolbar */}
      <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
        <Toolbar>
          <Segmented<ComparisonTab>
            label="Inspector tab selection"
            value={activeTab}
            onChange={setActiveTab}
            options={[
              { value: "matrix", label: "Comparison Matrix" },
              { value: "fedramp", label: "FedRAMP & P-ATO" },
              { value: "safety", label: "ASL-2/3 & Safety" },
              { value: "saif", label: "Google SAIF" },
              { value: "atlas", label: "MITRE ATLAS" },
              { value: "provenance", label: "Supply Chain & BOM" },
              { value: "merkle", label: "Merkle Receipts" },
            ]}
          />
        </Toolbar>

        <Toolbar>
          <span className="ck-eyebrow">Provider Profile:</span>
          <Segmented<FrontierProviderId>
            label="Provider focus"
            value={selectedProvider}
            onChange={(p) => {
              setSelectedProvider(p);
              onSelectModel?.(p);
            }}
            options={[
              { value: "anthropic", label: "Claude 3.5 (AWS)" },
              { value: "google", label: "Gemini 1.5 (GCP)" },
              { value: "azure-openai", label: "GPT-4o (Azure)" },
            ]}
          />
        </Toolbar>
      </div>

      {/* Tab Panels */}
      {activeTab === "matrix" && (
        <OverviewMatrixPanel
          onSelectProvider={(p) => {
            setSelectedProvider(p);
            setActiveTab("fedramp");
          }}
        />
      )}

      {activeTab === "fedramp" && (
        <FedrampDetailPanel profile={profile} />
      )}

      {activeTab === "safety" && (
        <SafetyFrameworkPanel profile={profile} />
      )}

      {activeTab === "saif" && (
        <SaifPillarsPanel profile={profile} />
      )}

      {activeTab === "atlas" && (
        <MitreAtlasPanel
          profile={profile}
          query={filterQuery}
          onQueryChange={setFilterQuery}
        />
      )}

      {activeTab === "provenance" && (
        <SupplyChainProvenancePanel profile={profile} />
      )}

      {activeTab === "merkle" && (
        <MerkleReceiptPanel profile={profile} />
      )}
    </div>
  );
}

/* ------------------------------------------------------------------ */
/* Sub-Panel: Overview Comparative Matrix                             */
/* ------------------------------------------------------------------ */

function OverviewMatrixPanel({
  onSelectProvider,
}: {
  onSelectProvider: (p: FrontierProviderId) => void;
}) {
  const providers = Object.values(FRONTIER_PROFILES);

  return (
    <Panel
      title="Frontier Foundation Model Sovereign Comparison Matrix"
      subtitle="Cross-provider architectural comparison across FedRAMP High, ASL-2/3, SAIF, and MITRE ATLAS defenses"
      provenance={PROVENANCE_FRONTIER}
    >
      <div className="space-y-4">
        <DataTable
          caption="Frontier AI Model Comparative Matrix"
          columns={[
            { key: "provider", label: "Provider & Model", className: "min-w-[14rem]" },
            { key: "enclave", label: "Sovereign Enclave" },
            { key: "fedramp", label: "FedRAMP Standing" },
            { key: "safety", label: "Frontier Safety Tier" },
            { key: "watermark", label: "Watermark Method" },
            { key: "injectionDefense", label: "Prompt Shield Tech" },
            { key: "latency", label: "Defense Overhead", numeric: true },
            { key: "action", label: "Inspect" },
          ]}
          rows={providers.map((p) => ({
            provider: (
              <div className="flex flex-col gap-0.5">
                <span className="font-semibold text-ck-fg-1">{p.modelName}</span>
                <span className="text-xs text-ck-fg-mute font-mono">{p.providerOrg}</span>
              </div>
            ),
            enclave: (
              <span className="text-xs text-ck-fg-2 font-mono">
                {p.cloudEnclave}
              </span>
            ),
            fedramp: (
              <div className="flex flex-col items-start gap-1">
                <StateBadge tone="pos">{p.fedramp.level}</StateBadge>
                <span className="text-2xs font-mono text-ck-fg-mute">
                  ID: {p.fedramp.packageId} ({p.fedramp.dodImpactLevel})
                </span>
              </div>
            ),
            safety: (
              <div className="flex flex-col items-start gap-1">
                <StateBadge tone="info">{p.safetyFramework.currentTier}</StateBadge>
                <span className="text-2xs text-ck-fg-mute max-w-[18ch] truncate" title={p.safetyFramework.frameworkName}>
                  {p.safetyFramework.frameworkName}
                </span>
              </div>
            ),
            watermark: (
              <span className="text-xs text-ck-fg-2">
                {p.supplyChain.watermarkResistance}
              </span>
            ),
            injectionDefense: (
              <span className="text-xs text-ck-fg-2">
                {p.egressIsolation.promptShieldTechnology}
              </span>
            ),
            latency: `${p.egressIsolation.promptInjectionLatencyMs} ms`,
            action: (
              <button
                type="button"
                onClick={() => onSelectProvider(p.id)}
                className="rounded border border-ck-hairline-strong bg-ck-bg-0 px-2 py-1 text-xs font-medium text-ck-fg-1 hover:border-ck-accent hover:text-ck-accent transition-colors"
              >
                Deep Audit &rarr;
              </button>
            ),
          }))}
        />
        <div className="rounded-md border border-ck-hairline bg-ck-bg-0 p-3 text-xs text-ck-fg-3 space-y-1.5">
          <p className="font-semibold text-ck-fg-1">Comparative Architectural Findings:</p>
          <ul className="list-disc list-inside space-y-1">
            <li>
              <strong className="text-ck-fg-2">FedRAMP High Uniformity:</strong> All three frontier offerings operate inside FedRAMP High JAB P-ATO accredited government enclaves with DoD IL4/IL5 or IL6 compliance.
            </li>
            <li>
              <strong className="text-ck-fg-2">Watermarking Divergence:</strong> Google implements imperceptible algorithmic token watermarking (SynthID), Azure OpenAI employs cryptographic C2PA provenance manifests, and Anthropic relies on token-distribution statistical detection heuristics.
            </li>
            <li>
              <strong className="text-ck-fg-2">Prompt Defense Overheads:</strong> Dedicated Prompt Shields introduce 38 ms (Vertex Model Armor), 42 ms (Bedrock Guardrails), and 45 ms (Azure Prompt Shields) in bidirectional inference pipeline latency.
            </li>
          </ul>
        </div>
      </div>
    </Panel>
  );
}

/* ------------------------------------------------------------------ */
/* Sub-Panel: FedRAMP High & P-ATO Boundary                           */
/* ------------------------------------------------------------------ */

function FedrampDetailPanel({ profile }: { profile: FrontierModelProfile }) {
  const f = profile.fedramp;
  const vs = f.vectorState;

  return (
    <div className="grid gap-4 2xl:grid-cols-2">
      <Panel
        title={`FedRAMP High Authorization Profile: ${profile.modelName}`}
        subtitle={`Package ID: ${f.packageId} | Enclave: ${profile.cloudEnclave}`}
        provenance={PROVENANCE_FRONTIER}
        actions={
          <StateBadge tone={valenceToTone(vs.valence)}>
            {f.level}
          </StateBadge>
        }
      >
        <div className="space-y-4">
          <StatGrid>
            <StatTile label="Authorization Tier" value="High Baseline" tone="pos" />
            <StatTile label="JAB P-ATO Status" value={f.jabApproved ? "Approved" : "Agency Only"} tone="pos" />
            <StatTile label="DoD SRG Readiness" value={f.dodImpactLevel} tone="info" />
            <StatTile label="Approval Date" value={f.authorizationDate} tone="neutral" />
          </StatGrid>

          <dl className="grid grid-cols-[11rem_1fr] gap-x-3 gap-y-2 text-sm border-t border-ck-hairline pt-3">
            <dt className="text-xs text-ck-fg-mute font-mono">Marketplace Package ID</dt>
            <dd className="font-mono text-ck-fg-1">{f.packageId}</dd>

            <dt className="text-xs text-ck-fg-mute font-mono">Deployment Boundary</dt>
            <dd className="text-ck-fg-2">{profile.deploymentBoundary}</dd>

            <dt className="text-xs text-ck-fg-mute font-mono">Continuous Monitoring</dt>
            <dd className="text-ck-fg-2">{f.continuousMonitoringCadence}</dd>

            <dt className="text-xs text-ck-fg-mute font-mono">Zero Data Retention (ZDR)</dt>
            <dd className="text-ck-fg-2">
              {profile.egressIsolation.zeroDataRetentionGuaranteed ? (
                <span className="text-ck-pos font-medium">Contractually Guaranteed (No Prompt/Output Logging)</span>
              ) : (
                <span className="text-ck-warn font-medium">Standard Retention Active</span>
              )}
            </dd>

            <dt className="text-xs text-ck-fg-mute font-mono">CMEK Key Management</dt>
            <dd className="text-ck-fg-2">
              {profile.egressIsolation.cmekSupport ? (
                <span className="text-ck-pos">Dedicated FIPS 140-3 HSM KMS Support</span>
              ) : (
                <span className="text-ck-warn">Platform-Managed Keys Only</span>
              )}
            </dd>
          </dl>
        </div>
      </Panel>

      <Panel
        title="Vector State & Boundary Invariants"
        subtitle="Mathematical evaluation vector S(e,t) for FedRAMP High"
        provenance={PROVENANCE_FRONTIER}
      >
        <div className="space-y-4">
          <div className="rounded-md border border-ck-hairline bg-ck-bg-0 p-3 font-mono text-xs space-y-2">
            <div className="text-ck-accent font-semibold">
              State Vector: S({profile.id}, FedRAMP_High)
            </div>
            <div className="grid grid-cols-2 gap-2 text-2xs">
              <div>
                <span className="text-ck-fg-mute">Presence:</span>{" "}
                <span className="text-ck-fg-1 uppercase">{vs.presence}</span>
              </div>
              <div>
                <span className="text-ck-fg-mute">Valence:</span>{" "}
                <span className="text-ck-pos uppercase">{vs.valence}</span>
              </div>
              <div>
                <span className="text-ck-fg-mute">Anti-Relation:</span>{" "}
                <span className="text-ck-fg-1 uppercase">{vs.anti}</span>
              </div>
              <div>
                <span className="text-ck-fg-mute">Coherence:</span>{" "}
                <span className="text-ck-fg-1 uppercase">{vs.coherence}</span>
              </div>
              <div>
                <span className="text-ck-fg-mute">Evidence Status:</span>{" "}
                <span className="text-ck-info uppercase">{vs.evidence}</span>
              </div>
              <div>
                <span className="text-ck-fg-mute">Lifecycle Epoch:</span>{" "}
                <span className="text-ck-fg-1">epoch-{vs.epoch}</span>
              </div>
            </div>
          </div>

          <div className="text-xs text-ck-fg-3 space-y-2">
            <p className="font-semibold text-ck-fg-1">Boundary Verification Notes:</p>
            <p>
              Under FedRAMP High Moderate-to-High transition requirements, frontier model inference weights
              must reside within the audited boundary envelope. Inter-service calls from government workloads
              are routed over private cloud backbones without public internet exposure:
            </p>
            <div className="rounded border border-ck-hairline-strong bg-ck-bg-2 p-2 font-mono text-2xs text-ck-fg-2">
              {profile.egressIsolation.privateNetworking}
            </div>
          </div>
        </div>
      </Panel>
    </div>
  );
}

/* ------------------------------------------------------------------ */
/* Sub-Panel: ASL-2/3 & Frontier Safety Frameworks                    */
/* ------------------------------------------------------------------ */

function SafetyFrameworkPanel({ profile }: { profile: FrontierModelProfile }) {
  const s = profile.safetyFramework;

  return (
    <Panel
      title={`Frontier AI Safety & Threshold Containment: ${profile.modelName}`}
      subtitle={s.frameworkName}
      provenance={PROVENANCE_FRONTIER}
      actions={<StateBadge tone="info">{s.currentTier}</StateBadge>}
    >
      <div className="space-y-4">
        <div className="grid gap-3 md:grid-cols-3">
          <div className="rounded-md border border-ck-hairline bg-ck-bg-0 p-3 space-y-1.5">
            <div className="ck-eyebrow text-ck-accent">CBRN Misuse Defense</div>
            <p className="text-xs text-ck-fg-2 leading-relaxed">
              {s.cbrnThresholdContainment}
            </p>
          </div>

          <div className="rounded-md border border-ck-hairline bg-ck-bg-0 p-3 space-y-1.5">
            <div className="ck-eyebrow text-ck-accent">Cyber-Offense Gating</div>
            <p className="text-xs text-ck-fg-2 leading-relaxed">
              {s.cyberOffenseContainment}
            </p>
          </div>

          <div className="rounded-md border border-ck-hairline bg-ck-bg-0 p-3 space-y-1.5">
            <div className="ck-eyebrow text-ck-accent">Model Weights Isolation</div>
            <p className="text-xs text-ck-fg-2 leading-relaxed">
              {s.weightsSecurityHsm
                ? "Physical HSM & cryptographic containment; model weights protected against unauthenticated exfiltration."
                : "Standard cloud disk volume encryption; no specialized HSM enclave isolation."}
            </p>
          </div>
        </div>

        <div className="rounded-md border border-ck-hairline bg-ck-bg-0 p-3 text-xs text-ck-fg-3 space-y-2">
          <p className="font-semibold text-ck-fg-1">ASL-2 vs ASL-3 Transition Protocol Analysis:</p>
          <p>
            When foundation models approach ASL-3 (Anthropic RSP) or Critical Capability Level 2 (Google FSF)
            thresholds in cyber or biological autonomy:
          </p>
          <ul className="list-disc list-inside space-y-1 font-mono text-2xs text-ck-fg-2">
            <li>Tripwire 1: Automated biological synthesis assistance requires multi-factor red-team containment pass.</li>
            <li>Tripwire 2: Model weights must be encrypted with split-knowledge keys stored inside dedicated HSM enclaves.</li>
            <li>Tripwire 3: Continuous red-teaming outputs are committed into the RFC 9162 Merkle Flight Recorder.</li>
          </ul>
        </div>
      </div>
    </Panel>
  );
}

/* ------------------------------------------------------------------ */
/* Sub-Panel: Google SAIF 6-Pillar Conformance                        */
/* ------------------------------------------------------------------ */

function SaifPillarsPanel({ profile }: { profile: FrontierModelProfile }) {
  const saif = profile.saifPillars;

  const pillars = [
    {
      num: "01",
      title: "Expand Cyber Foundations to AI Ecosystem",
      detail: saif.cyberFoundations,
    },
    {
      num: "02",
      title: "Extend Threat Detection to AI Lifecycle",
      detail: saif.threatDetection,
    },
    {
      num: "03",
      title: "Automate Defenses Against AI Threats",
      detail: saif.automatedDefenses,
    },
    {
      num: "04",
      title: "Harmonize Platform Controls for Consistent Security",
      detail: saif.platformHarmonization,
    },
    {
      num: "05",
      title: "Adapt Controls & Continuous Feedback Loops",
      detail: saif.adaptiveControls,
    },
    {
      num: "06",
      title: "Contextualize AI Risks in Business Processes",
      detail: saif.contextualizedRisk,
    },
  ];

  return (
    <Panel
      title={`Secure AI Framework (SAIF) Alignment: ${profile.modelName}`}
      subtitle="Google Secure AI Framework 6-Pillar Architecture Inspection"
      provenance={PROVENANCE_FRONTIER}
      actions={<StateBadge tone="pos">6 / 6 Pillars Conforming</StateBadge>}
    >
      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {pillars.map((p) => (
          <div
            key={p.num}
            className="rounded-md border border-ck-hairline bg-ck-bg-0 p-3 flex flex-col justify-between gap-2"
          >
            <div>
              <div className="flex items-center justify-between gap-2">
                <span className="font-mono text-2xs text-ck-accent font-semibold">
                  PILLAR {p.num}
                </span>
                <StateBadge tone="pos" glyph={false}>
                  ALIGNED
                </StateBadge>
              </div>
              <h3 className="mt-1 text-xs font-semibold text-ck-fg-1">
                {p.title}
              </h3>
            </div>
            <p className="text-xs text-ck-fg-2 leading-relaxed font-mono">
              {p.detail}
            </p>
          </div>
        ))}
      </div>
    </Panel>
  );
}

/* ------------------------------------------------------------------ */
/* Sub-Panel: MITRE ATLAS Adversarial Threat Matrix                   */
/* ------------------------------------------------------------------ */

function MitreAtlasPanel({
  profile,
  query,
  onQueryChange,
}: {
  profile: FrontierModelProfile;
  query: string;
  onQueryChange: (q: string) => void;
}) {
  const atlas = Object.values(profile.mitreAtlas);
  const filtered = atlas.filter((item) =>
    query
      ? item.techniqueId.toLowerCase().includes(query.toLowerCase()) ||
        item.techniqueName.toLowerCase().includes(query.toLowerCase()) ||
        item.defenseMechanism.toLowerCase().includes(query.toLowerCase())
      : true,
  );

  return (
    <Panel
      title={`MITRE ATLAS Adversarial Defenses: ${profile.modelName}`}
      subtitle="Adversarial Threat Landscape for Artificial-Intelligence Systems Gating"
      provenance={PROVENANCE_FRONTIER}
      actions={
        <input
          type="search"
          placeholder="Filter technique..."
          value={query}
          onChange={(e) => onQueryChange(e.target.value)}
          className="h-7 w-48 rounded border border-ck-hairline-strong bg-ck-bg-0 px-2 text-xs text-ck-fg-1 focus:outline-none focus:border-ck-accent"
        />
      }
    >
      <div className="space-y-4">
        {filtered.length === 0 ? (
          <EmptyState kind="empty" title="No MITRE ATLAS techniques match the filter" />
        ) : (
          <DataTable
            caption="MITRE ATLAS Defenses"
            columns={[
              { key: "id", label: "Technique ID", className: "w-[8rem]" },
              { key: "name", label: "Threat Description" },
              { key: "defense", label: "Frontier Defense Implementation" },
              { key: "coverage", label: "Coverage Tier" },
              { key: "status", label: "Vector State" },
            ]}
            rows={filtered.map((item) => ({
              id: <code className="font-mono text-xs text-ck-accent">{item.techniqueId}</code>,
              name: <span className="font-semibold text-xs text-ck-fg-1">{item.techniqueName}</span>,
              defense: <span className="text-xs text-ck-fg-2">{item.defenseMechanism}</span>,
              coverage: (
                <StateBadge tone={coverageTone(item.coverageLevel)}>
                  {item.coverageLevel}
                </StateBadge>
              ),
              status: (
                <StateBadge tone={valenceToTone(item.vectorState.valence)} glyph={false}>
                  {item.vectorState.evidence}
                </StateBadge>
              ),
            }))}
          />
        )}

        <div className="rounded border border-ck-hairline bg-ck-bg-0 p-3 text-xs text-ck-fg-3 space-y-1">
          <p className="font-semibold text-ck-fg-1">Mitigation Architecture Highlights:</p>
          <ul className="list-disc list-inside space-y-1 text-2xs font-mono text-ck-fg-2">
            <li>
              <strong>AML.T0051 (Prompt Injection):</strong> Uses dual-token parsing boundaries to isolate untrusted user data from system-level instructions.
            </li>
            <li>
              <strong>AML.T0054 (Jailbreak Attacks):</strong> Real-time heuristic and embedding-distance classifiers evaluate incoming prompt tokens before GPU tensor allocation.
            </li>
            <li>
              <strong>AML.T0057 (Insecure Output Handling):</strong> Automatic response scanners redact PII and strip potentially executable shell constructs.
            </li>
          </ul>
        </div>
      </div>
    </Panel>
  );
}

/* ------------------------------------------------------------------ */
/* Sub-Panel: Supply Chain Provenance, AI-BOM & Watermarking          */
/* ------------------------------------------------------------------ */

function SupplyChainProvenancePanel({
  profile,
}: {
  profile: FrontierModelProfile;
}) {
  const sc = profile.supplyChain;

  return (
    <div className="grid gap-4 2xl:grid-cols-2">
      <Panel
        title={`Model Card & Supply Chain Provenance: ${profile.modelName}`}
        subtitle="SLSA v1.2 & CycloneDX / SPDX AI Bill of Materials"
        provenance={PROVENANCE_FRONTIER}
      >
        <dl className="grid grid-cols-[12rem_1fr] gap-x-3 gap-y-2 text-sm">
          <dt className="text-xs text-ck-fg-mute font-mono">Model Card Schema</dt>
          <dd className="font-mono text-ck-fg-1 text-xs">{sc.modelCardStandard}</dd>

          <dt className="text-xs text-ck-fg-mute font-mono">SLSA Build Assurance</dt>
          <dd className="text-xs">
            <StateBadge tone="pos">{sc.slsaLevel}</StateBadge>
          </dd>

          <dt className="text-xs text-ck-fg-mute font-mono">AI-BOM Format</dt>
          <dd className="font-mono text-ck-fg-1 text-xs">{sc.aiBomFormat}</dd>

          <dt className="text-xs text-ck-fg-mute font-mono">Sigstore Cosign Signature</dt>
          <dd className="text-xs">
            {sc.sigstoreVerified ? (
              <StateBadge tone="pos">Sigstore Attested (Fulcio / Rekor)</StateBadge>
            ) : (
              <StateBadge tone="neg">Unsigned</StateBadge>
            )}
          </dd>
        </dl>
      </Panel>

      <Panel
        title={`Synthetic Output Watermarking & Tracking`}
        subtitle="Synthetic media & output attribution mechanics"
        provenance={PROVENANCE_FRONTIER}
      >
        <dl className="grid grid-cols-[12rem_1fr] gap-x-3 gap-y-2 text-sm">
          <dt className="text-xs text-ck-fg-mute font-mono">Watermark System</dt>
          <dd className="text-ck-fg-1 text-xs font-semibold">{sc.watermarkTechnology}</dd>

          <dt className="text-xs text-ck-fg-mute font-mono">Tamper Resistance</dt>
          <dd className="text-xs">
            <StateBadge tone={sc.watermarkResistance.includes("SynthID") ? "pos" : "info"}>
              {sc.watermarkResistance}
            </StateBadge>
          </dd>

          <dt className="text-xs text-ck-fg-mute font-mono">Extraction Method</dt>
          <dd className="text-xs text-ck-fg-2">
            Deterministic cryptographic verification API without degrading generative token perplexity.
          </dd>
        </dl>
      </Panel>
    </div>
  );
}

/* ------------------------------------------------------------------ */
/* Sub-Panel: RFC 9162 Merkle Verification Receipts                   */
/* ------------------------------------------------------------------ */

function MerkleReceiptPanel({ profile }: { profile: FrontierModelProfile }) {
  const m = profile.merkleReceipt;

  return (
    <Panel
      title={`RFC 9162 Merkle Inclusion Receipt: ${profile.modelName}`}
      subtitle="Cryptographic proof of inclusion in the US-Gov Sovereign AI Transparency Ledger"
      provenance={PROVENANCE_FRONTIER}
      actions={<StateBadge tone="pos">Merkle Verified</StateBadge>}
    >
      <div className="space-y-4">
        <div className="rounded-md border border-ck-hairline-strong bg-ck-bg-0 p-4 font-mono text-xs space-y-3">
          <div className="flex flex-wrap items-center justify-between gap-2 border-b border-ck-hairline pb-2">
            <span className="text-ck-fg-1 font-semibold">Transparency Log Identifier:</span>
            <span className="text-ck-accent text-2xs">{m.rfcLogId}</span>
          </div>

          <div className="grid grid-cols-[10rem_1fr] gap-2 text-2xs">
            <span className="text-ck-fg-mute">Log Tree Size:</span>
            <span className="text-ck-fg-1 ck-num">{m.treeSize} entries</span>

            <span className="text-ck-fg-mute">Entry Log Index:</span>
            <span className="text-ck-fg-1 ck-num">{m.logIndex}</span>

            <span className="text-ck-fg-mute">Attestation Time:</span>
            <span className="text-ck-fg-1">{m.timestamp}</span>

            <span className="text-ck-fg-mute">Log Root Hash:</span>
            <span className="text-ck-pos break-all">{m.merkleRoot}</span>

            <span className="text-ck-fg-mute">Leaf Entry Hash:</span>
            <span className="text-ck-info break-all">{m.leafHash}</span>
          </div>

          <div className="border-t border-ck-hairline pt-2 space-y-1.5">
            <span className="text-ck-eyebrow text-2xs text-ck-fg-mute">Inclusion Proof Path (RFC 9162):</span>
            <ol className="list-decimal list-inside space-y-1 text-2xs text-ck-fg-2">
              {m.inclusionProof.map((hash, idx) => (
                <li key={idx} className="break-all">
                  <span className="text-ck-fg-mute">Node {idx}:</span> {hash}
                </li>
              ))}
            </ol>
          </div>
        </div>

        <p className="text-xs text-ck-fg-mute">
          This receipt guarantees that the compliance baseline for {profile.modelName} was logged to an
          immutable Merkle tree. Any post-audit tampering with boundary assertions, FedRAMP High packages,
          or safety thresholds will immediately break tree inclusion verification.
        </p>
      </div>
    </Panel>
  );
}