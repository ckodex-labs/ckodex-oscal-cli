"use client";

import * as React from "react";
import { MessageItem, ChatMessage } from "./message-item";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Send, Sparkles, Shield, GitMerge, FileCheck } from "lucide-react";
import {
  BlastRadiusReport,
  FedrampReport,
  MergeReport,
} from "@/lib/oscal-types";

import { LensMode } from "@/lib/oscal-types";

interface ChatPanelProps {
  selectedControl?: string;
  lens?: LensMode;
  onNavigateControl?: (id: string) => void;
}

export function ChatPanel({
  selectedControl,
  lens,
  onNavigateControl,
}: ChatPanelProps = {}) {
  const [messages, setMessages] = React.useState<ChatMessage[]>([
    {
      id: "welcome",
      role: "assistant",
      content:
        "Welcome to **Mizan Compliance Workbench**. I am your OSCAL compliance kernel copilot. I can compute blast radiuses, audit FedRAMP PMO baselines, perform 3-way GitOps AST merges, and draft control implementation prose.",
    },
  ]);
  const [input, setInput] = React.useState("");
  const [loading, setLoading] = React.useState(false);
  const scrollRef = React.useRef<HTMLDivElement>(null);

  const handleSend = async (userPrompt?: string) => {
    const promptToSend = userPrompt || input;
    if (!promptToSend.trim() || loading) return;

    const userMsg: ChatMessage = {
      id: String(Date.now()),
      role: "user",
      content: promptToSend,
    };

    setMessages((prev) => [...prev, userMsg]);
    setInput("");
    setLoading(true);

    // Mock/Simulated agent response with tool invocations
    setTimeout(() => {
      let assistantMsg: ChatMessage;

      const lower = promptToSend.toLowerCase();
      if (
        lower.includes("blast") ||
        lower.includes("radius") ||
        lower.includes("ac-1")
      ) {
        const mockReport: BlastRadiusReport = {
          target_id: "ac-1",
          target_kind: "control",
          risk_exposure_score: 7.8,
          is_critical_path: true,
          documents_analyzed: ["sample-catalog.json", "ssp-prod.json"],
          direct_dependents: ["ac-2", "ac-3", "ia-2"],
          transitive_dependents: ["ia-5", "sc-7", "si-4", "cm-2"],
          downstream_impact_paths: [
            ["ac-1", "ac-2", "ia-2", "ssp-prod.json"],
            ["ac-1", "ac-3", "cm-2", "ssp-prod.json"],
          ],
        };
        assistantMsg = {
          id: String(Date.now() + 1),
          role: "assistant",
          content:
            "I executed a multi-model **Blast Radius Analysis** for target `ac-1` across your active workspace. Here are the propagation metrics and dependent critical paths:",
          toolInvocations: [
            {
              toolName: "compute_blast_radius",
              args: { target: "ac-1" },
              result: mockReport,
            },
          ],
        };
      } else if (
        lower.includes("fedramp") ||
        lower.includes("pmo") ||
        lower.includes("audit")
      ) {
        const mockFedramp: FedrampReport = {
          document_kind: "System Security Plan (SSP)",
          baseline: "moderate",
          passed: false,
          rule_count_evaluated: 85,
          violation_count: 2,
          findings: [
            {
              rule_id: "FEDRAMP-AC-02-01",
              severity: "high",
              title: "Missing account manager designation parameter",
              detail:
                "Parameter ac-02_prm_1 must specify organization-defined account managers.",
              target: "ac-2",
            },
            {
              rule_id: "FEDRAMP-IA-05-01",
              severity: "medium",
              title: "MFA authenticator assurance level unspecified",
              detail:
                "Authenticator management requires AAL2 or AAL3 declaration.",
              target: "ia-5",
            },
          ],
        };
        assistantMsg = {
          id: String(Date.now() + 1),
          role: "assistant",
          content:
            "I evaluated your active SSP against the **FedRAMP Moderate PMO Baseline Rules**. 2 rule violations were detected:",
          toolInvocations: [
            {
              toolName: "validate_fedramp",
              args: { baseline: "moderate" },
              result: mockFedramp,
            },
          ],
        };
      } else if (lower.includes("sync") || lower.includes("merge")) {
        const mockMerge: MergeReport = {
          strategy: "Manual",
          controls_merged: 2,
          added_from_upstream: ["ac-3"],
          preserved_local_additions: ["custom-1"],
          updated_from_upstream: [],
          retained_local_modifications: [],
          conflicts: [
            {
              control_id: "ac-2",
              field: "title/prose",
              local_summary:
                "Account Management (90-day automated key rotation policy)",
              upstream_summary:
                "Account Management (NIST SP 800-53 r5 update with supervisor notification)",
              resolution: "Marked with <<<<<<< LOCAL ======= UPSTREAM >>>>>>>",
            },
          ],
          is_clean: false,
        };
        assistantMsg = {
          id: String(Date.now() + 1),
          role: "assistant",
          content:
            "I ran a **3-Way GitOps AST Merge** across Base, Upstream, and Local. 1 conflict requires your resolution:",
          toolInvocations: [
            {
              toolName: "sync_and_merge",
              args: { strategy: "manual" },
              result: mockMerge,
            },
          ],
        };
      } else {
        assistantMsg = {
          id: String(Date.now() + 1),
          role: "assistant",
          content: `Understood: "${promptToSend}". I have queried the local Metaschema catalog ledger. All controls and schemas are validated according to OSCAL 1.2.3.`,
        };
      }

      setMessages((prev) => [...prev, assistantMsg]);
      setLoading(false);
    }, 600);
  };

  return (
    <div className="flex h-full flex-col bg-ck-bg-0 font-mono text-xs">
      {/* Chat Header */}
      <div className="flex items-center justify-between border-b border-ck-hairline px-4 py-2.5 bg-ck-bg-1/60">
        <div className="flex items-center gap-2">
          <Sparkles className="h-4 w-4 text-accent" />
          <span className="font-serif text-base font-normal text-ck-fg-1">
            Atlas Compliance Copilot
          </span>
        </div>
        <span className="font-mono text-[10px] text-ck-fg-mute uppercase">
          MCP stdio connected
        </span>
      </div>

      {/* Message List */}
      <ScrollArea className="flex-1 p-2">
        <div className="space-y-1">
          {messages.map((m) => (
            <MessageItem key={m.id} message={m} />
          ))}
        </div>
      </ScrollArea>

      {/* Quick Prompts */}
      <div className="flex flex-wrap gap-1.5 border-t border-ck-hairline bg-ck-bg-1/40 px-3 py-2">
        <Button
          variant="outline"
          size="sm"
          className="text-[10px] h-6 bg-ck-bg-0"
          onClick={() => handleSend("Compute blast radius for control ac-1")}
        >
          <Shield className="h-3 w-3 mr-1 text-accent" />
          Blast Radius AC-1
        </Button>
        <Button
          variant="outline"
          size="sm"
          className="text-[10px] h-6 bg-ck-bg-0"
          onClick={() =>
            handleSend("Validate SSP against FedRAMP Moderate baseline")
          }
        >
          <FileCheck className="h-3 w-3 mr-1 text-green-600 dark:text-green-400" />
          FedRAMP Audit
        </Button>
        <Button
          variant="outline"
          size="sm"
          className="text-[10px] h-6 bg-ck-bg-0"
          onClick={() =>
            handleSend("Sync 3-way merge with upstream NIST catalog")
          }
        >
          <GitMerge className="h-3 w-3 mr-1 text-accent" />
          3-Way AST Merge
        </Button>
      </div>

      {/* Input Form */}
      <div className="border-t border-ck-hairline-strong p-3 bg-ck-bg-0">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            handleSend();
          }}
          className="flex items-center gap-2"
        >
          <Input
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Ask Atlas (e.g. 'Audit FedRAMP', 'Blast radius of AC-1')..."
            className="flex-1 font-mono text-xs bg-ck-bg-1 border-ck-hairline-strong focus-visible:ring-accent"
            disabled={loading}
          />
          <Button
            type="submit"
            size="sm"
            variant="ck"
            disabled={loading || !input.trim()}
          >
            <Send className="h-3.5 w-3.5 mr-1" />
            Send
          </Button>
        </form>
      </div>
    </div>
  );
}
