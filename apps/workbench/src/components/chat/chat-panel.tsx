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
  LensMode,
} from "@/lib/oscal-types";

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
        "Welcome to **Mizan Compliance Workbench**. I am your OSCAL compliance kernel copilot connected directly to the native `mizan` CLI binary. I can compute real topological blast radiuses, validate FedRAMP PMO baselines, perform 3-way GitOps AST merges, and inspect active workspaces.",
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

    try {
      let assistantMsg: ChatMessage;
      const lower = promptToSend.toLowerCase();

      if (
        lower.includes("blast") ||
        lower.includes("radius") ||
        lower.includes("ac-1") ||
        lower.includes("ac-2")
      ) {
        const target = lower.includes("ac-2") ? "ac-2" : (selectedControl || "ac-1");
        const res = await fetch("/api/cli", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            command: "blast-radius",
            args: ["examples/sample-catalog.json", "--target", target],
          }),
        });
        const json = await res.json();
        const report = (json.success ? json.data : null) as BlastRadiusReport | null;

        assistantMsg = {
          id: String(Date.now() + 1),
          role: "assistant",
          content: report
            ? `Executed live \`mizan blast-radius\` on \`examples/sample-catalog.json\` for target \`${target}\`:\n\n- Risk exposure score: **${report.risk_exposure_score ?? 0}**\n- Critical path: **${report.is_critical_path ? "YES" : "NO"}**\n- Direct dependents: **${report.direct_dependents?.length ?? 0}**\n- Transitive dependents: **${report.transitive_dependents?.length ?? 0}**`
            : `Failed to compute blast radius: ${json.error || "Unknown error"}`,
          toolInvocations: report
            ? [
                {
                  toolName: "compute_blast_radius",
                  args: { target, file: "examples/sample-catalog.json" },
                  result: report,
                },
              ]
            : undefined,
        };
      } else if (
        lower.includes("fedramp") ||
        lower.includes("pmo") ||
        lower.includes("audit")
      ) {
        const res = await fetch("/api/cli", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            command: "fedramp",
            args: ["validate", "examples/sample-catalog.json", "--baseline", "moderate"],
          }),
        });
        const json = await res.json();
        const report = (json.success ? json.data : null) as FedrampReport | null;

        const isCompliant = report?.passed ?? report?.is_compliant;
        assistantMsg = {
          id: String(Date.now() + 1),
          role: "assistant",
          content: report
            ? `Executed live \`mizan fedramp validate\` on \`examples/sample-catalog.json\` against **FedRAMP Moderate** baseline:\n\n- Compliance verdict: **${isCompliant ? "COMPLIANT" : "NON-COMPLIANT"}**\n- Rules evaluated: **${report.total_rules_checked ?? report.rule_count_evaluated ?? 0}**\n- Violations: **${report.failed_rules ?? report.violation_count ?? report.findings?.length ?? 0}**`
            : `Failed to evaluate FedRAMP baseline: ${json.error || "Unknown error"}`,
          toolInvocations: report
            ? [
                {
                  toolName: "validate_fedramp",
                  args: { baseline: "moderate", file: "examples/sample-catalog.json" },
                  result: report,
                },
              ]
            : undefined,
        };
      } else if (lower.includes("sync") || lower.includes("merge")) {
        const res = await fetch("/api/cli", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            command: "sync",
            args: [
              "--base",
              "examples/sample-catalog.json",
              "--upstream",
              "examples/resolved-catalog.json",
              "--local",
              "examples/sample-catalog.json",
              "--strategy",
              "manual",
            ],
          }),
        });
        const json = await res.json();
        const report = (json.success ? json.data : null) as MergeReport | null;

        assistantMsg = {
          id: String(Date.now() + 1),
          role: "assistant",
          content: report
            ? `Executed live \`mizan sync\` 3-Way AST Merge across Base, Upstream, and Local:\n\n- Merge clean: **${report.is_clean ? "YES" : "NO"}**\n- Controls merged: **${report.controls_merged ?? 0}**\n- Conflicts: **${report.conflicts?.length ?? 0}**`
            : `Failed to execute 3-way sync: ${json.error || "Unknown error"}`,
          toolInvocations: report
            ? [
                {
                  toolName: "sync_and_merge",
                  args: { strategy: "manual" },
                  result: report,
                },
              ]
            : undefined,
        };
      } else {
        const res = await fetch("/api/cli", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            command: "inspect",
            args: ["examples/sample-catalog.json"],
          }),
        });
        const json = await res.json();
        assistantMsg = {
          id: String(Date.now() + 1),
          role: "assistant",
          content: json.success
            ? `Executed query against local OSCAL kernel substrate for prompt: "${promptToSend}"\n\n\`\`\`json\n${JSON.stringify(json.data, null, 2)}\n\`\`\``
            : `Kernel query returned: ${json.error || "Unable to inspect local document."}`,
        };
      }

      setMessages((prev) => [...prev, assistantMsg]);
    } catch (err: unknown) {
      const errMsg = err instanceof Error ? err.message : String(err);
      setMessages((prev) => [
        ...prev,
        {
          id: String(Date.now() + 1),
          role: "assistant",
          content: `Kernel execution error: ${errMsg}`,
        },
      ]);
    } finally {
      setLoading(false);
    }
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
          Live Native CLI Substrate
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
