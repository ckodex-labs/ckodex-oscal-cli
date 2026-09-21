"use client";

import * as React from "react";
import ReactMarkdown from "react-markdown";
import { BlastRadiusCard } from "../generative-ui/blast-radius-card";
import { FedrampBadgeCard } from "../generative-ui/fedramp-badge-card";
import { MergeConflictCard } from "../generative-ui/merge-conflict-card";
import { ControlCard } from "../generative-ui/control-card";
import {
  BlastRadiusReport,
  FedrampReport,
  MergeReport,
  ControlDetail,
} from "@/lib/oscal-types";
import { Bot, User } from "lucide-react";

export interface ChatMessage {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  toolInvocations?: Array<{
    toolName: string;
    args: unknown;
    result?: unknown;
  }>;
}

interface MessageItemProps {
  message: ChatMessage;
}

export function MessageItem({ message }: MessageItemProps) {
  const isUser = message.role === "user";

  return (
    <div
      className={`flex gap-3 py-3 ${isUser ? "bg-ck-bg-1/40 px-3 border-y border-ck-hairline" : "px-3"}`}
    >
      <div className="flex-shrink-0 mt-0.5">
        {isUser ? (
          <div className="h-6 w-6 rounded-[2px] bg-ck-bg-2 border border-ck-hairline flex items-center justify-center text-ck-fg-1">
            <User className="h-3.5 w-3.5" />
          </div>
        ) : (
          <div className="h-6 w-6 rounded-[2px] bg-accent text-white flex items-center justify-center shadow-sm">
            <Bot className="h-3.5 w-3.5" />
          </div>
        )}
      </div>

      <div className="flex-1 min-w-0 space-y-2">
        <div className="flex items-center gap-2 font-mono text-[10px] text-ck-fg-mute">
          <span className="font-semibold text-ck-fg-1 uppercase">
            {isUser ? "Architect / Author" : "Atlas Copilot"}
          </span>
        </div>

        {/* Text Content */}
        {message.content && (
          <div className="font-sans text-xs text-ck-fg-1 leading-relaxed prose dark:prose-invert max-w-none">
            <ReactMarkdown>{message.content}</ReactMarkdown>
          </div>
        )}

        {/* Generative UI Tool Execution Outputs */}
        {message.toolInvocations?.map((inv, idx) => {
          if (!inv.result) return null;

          switch (inv.toolName) {
            case "compute_blast_radius":
              return (
                <BlastRadiusCard
                  key={idx}
                  report={inv.result as BlastRadiusReport}
                />
              );
            case "validate_fedramp":
              return (
                <FedrampBadgeCard
                  key={idx}
                  report={inv.result as FedrampReport}
                />
              );
            case "sync_and_merge":
              return (
                <MergeConflictCard
                  key={idx}
                  report={inv.result as MergeReport}
                />
              );
            case "query_control":
              return (
                <ControlCard key={idx} control={inv.result as ControlDetail} />
              );
            default:
              return (
                <div
                  key={idx}
                  className="p-2 border border-ck-hairline bg-ck-bg-0 font-mono text-xs"
                >
                  <span className="font-semibold">{inv.toolName}:</span>
                  <pre className="mt-1 text-[10px] overflow-x-auto text-ck-fg-mute">
                    {JSON.stringify(inv.result, null, 2)}
                  </pre>
                </div>
              );
          }
        })}
      </div>
    </div>
  );
}
