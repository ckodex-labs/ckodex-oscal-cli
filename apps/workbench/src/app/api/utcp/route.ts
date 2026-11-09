/**
 * Universal Tool Calling Protocol (UTCP v1.0.1) Discovery & Execution Endpoint (/api/utcp)
 *
 * Compatible with:
 * - https://github.com/universal-tool-calling-protocol
 * - https://github.com/universal-tool-calling-protocol/utcp-specification
 * - https://github.com/universal-tool-calling-protocol/utcp-mcp
 * - https://github.com/universal-tool-calling-protocol/rs-utcp
 * - https://github.com/universal-tool-calling-protocol/typescript-utcp
 * - https://github.com/universal-tool-calling-protocol/go-utcp
 * - https://github.com/universal-tool-calling-protocol/python-utcp
 */

import { NextRequest, NextResponse } from "next/server";
import fs from "fs";
import path from "path";
import { callOscalCli } from "@/lib/mcp-client";
import { bridgeError as fail, guardBridgeRequest } from "@/lib/bridge-guard";

const PROJECT_ROOT = path.resolve(process.cwd(), "../..");

const VALID_TRANSPORTS = new Set(["cli", "http", "grpc", "mcp", "sse", "text"]);

const ALLOWED_UTCP_TOOLS = new Set([
  "query_control",
  "inspect_document",
  "compute_blast_radius",
  "validate_fedramp",
  "evaluate_rego_policy",
  "export_baseline_catalog",
  "verify_slsa_provenance",
  "query_cas_store",
  "evaluate_policy_rulepack",
  "fabric_status",
  "fabric_validate_spiffe",
  "search_utcp_tools",
]);

const FILE_PATH_KEYS = new Set([
  "file",
  "catalog_file",
  "ssp_file",
  "policy_file",
  "evidence_file",
  "base_file",
  "upstream_file",
  "local_file",
  "sbom_file",
  "workload_file",
  "report_file",
  "existing_poam_file",
  "input_dir",
]);

const NON_FILE_KEYS = new Set([
  "spiffe_id",
  "statement_json",
  "input_data",
  "base_url",
  "query",
  "q",
  "target",
  "control_id",
  "rule_id",
  "jurisdiction",
  "baseline",
  "transport",
  "digest",
  "trust_domain",
  "query_package",
]);

function looksLikeFilePath(key: string, val: string): boolean {
  if (
    FILE_PATH_KEYS.has(key) ||
    key.endsWith("_file") ||
    key.endsWith("_path") ||
    key.endsWith("_dir")
  ) {
    return true;
  }
  if (NON_FILE_KEYS.has(key)) {
    return false;
  }
  const lower = val.toLowerCase();
  return (
    val.includes("..") ||
    val.includes("/") ||
    val.includes("\\") ||
    lower.endsWith(".json") ||
    lower.endsWith(".yaml") ||
    lower.endsWith(".yml") ||
    lower.endsWith(".xml") ||
    lower.endsWith(".rego") ||
    lower.endsWith(".md")
  );
}

function resolveSafePath(rawPath: string): string | null {
  if (!rawPath || rawPath.includes("\0") || rawPath.includes("..")) {
    return null;
  }
  const resolved = path.resolve(PROJECT_ROOT, rawPath);
  if (
    resolved !== PROJECT_ROOT &&
    !resolved.startsWith(PROJECT_ROOT + path.sep)
  ) {
    return null;
  }
  const rel = path.relative(PROJECT_ROOT, resolved);
  if (rel.startsWith("..") || path.isAbsolute(rel)) {
    return null;
  }
  let repoReal = PROJECT_ROOT;
  try {
    if (fs.existsSync(PROJECT_ROOT)) {
      repoReal = fs.realpathSync(PROJECT_ROOT);
    }
  } catch {
    return null;
  }
  let current = PROJECT_ROOT;
  const parts = rel.split(path.sep).filter(Boolean);
  for (const part of parts) {
    current = path.join(/*turbopackIgnore: true*/ current, part);
    let stat: fs.Stats | undefined;
    try {
      stat = fs.lstatSync(/*turbopackIgnore: true*/ current);
    } catch {
      stat = undefined;
    }
    if (stat) {
      if (stat.isSymbolicLink()) {
        return null;
      }
      try {
        const realCurrent = fs.realpathSync(/*turbopackIgnore: true*/ current);
        if (
          realCurrent !== repoReal &&
          !realCurrent.startsWith(repoReal + path.sep)
        ) {
          return null;
        }
      } catch {
        return null;
      }
    }
  }
  return resolved;
}

export async function GET(req?: NextRequest) {
  let transport = "http";
  let searchQuery = "";
  let formatMode = "manual";
  let baseUrl = "http://localhost:3000";

  try {
    const sp = req?.nextUrl?.searchParams;
    if (sp) {
      const tParam = (sp.get("transport") ?? "http").trim().toLowerCase();
      if (VALID_TRANSPORTS.has(tParam)) {
        transport = tParam;
      }
      searchQuery = (sp.get("search") ?? sp.get("q") ?? "").trim();
      formatMode = (sp.get("format") ?? "manual").trim().toLowerCase();
      const bParam = (sp.get("base_url") ?? "").trim();
      if (bParam && /^https?:\/\/[^\s"-]+$/.test(bParam)) {
        baseUrl = bParam;
      }
    }
  } catch {
    // Fallback for static build contexts where request URL params are unavailable
  }

  try {
    if (formatMode === "config") {
      const stdout = await callOscalCli([
        "utcp",
        "config",
        "--base-url",
        baseUrl,
        "--format",
        "json",
      ]);
      const cfg = JSON.parse(stdout);
      return NextResponse.json(cfg, {
        status: 200,
        headers: { "Cache-Control": "no-store" },
      });
    }

    const stdout = await callOscalCli([
      "utcp",
      "manual",
      "--transport",
      transport,
      "--base-url",
      baseUrl,
      "--format",
      "json",
    ]);
    const manual = JSON.parse(stdout);

    if (searchQuery && Array.isArray(manual.tools)) {
      const q = searchQuery.toLowerCase();
      const words = q.split(/[^a-z0-9_-]+/).filter(Boolean);
      const filtered = manual.tools.filter((t: Record<string, unknown>) => {
        const name = typeof t.name === "string" ? t.name.toLowerCase() : "";
        const desc =
          typeof t.description === "string" ? t.description.toLowerCase() : "";
        const tags = Array.isArray(t.tags)
          ? t.tags.map((x) => String(x).toLowerCase())
          : [];
        if (
          name.includes(q) ||
          desc.includes(q) ||
          tags.some((tag) => tag.includes(q))
        ) {
          return true;
        }
        return words.some(
          (w) =>
            name.includes(w) ||
            desc.includes(w) ||
            tags.some((tag) => tag.includes(w)),
        );
      });
      return NextResponse.json(
        {
          ...manual,
          query: searchQuery,
          count: filtered.length,
          tools: filtered,
        },
        {
          status: 200,
          headers: { "Cache-Control": "no-store" },
        },
      );
    }

    return NextResponse.json(manual, {
      status: 200,
      headers: { "Cache-Control": "no-store" },
    });
  } catch (err) {
    return NextResponse.json(
      {
        manual_version: "1.0.0",
        utcp_version: "1.0.1",
        version: "1.0",
        name: "mizan-oscal",
        description:
          "Mizan High-Assurance OSCAL v1.2.3 Compliance Kernel & Workbench — Universal Tool Calling Protocol (UTCP) Manual",
        info: {
          title: "mizan-oscal",
          version: "0.2.0",
          description:
            "Mizan High-Assurance OSCAL v1.2.3 Compliance Kernel & Workbench — Universal Tool Calling Protocol (UTCP) Manual",
        },
        allowed_communication_protocols: [
          "cli",
          "http",
          "grpc",
          "mcp",
          "sse",
          "text",
        ],
        homepage: "https://github.com/universal-tool-calling-protocol",
        specification_url:
          "https://github.com/universal-tool-calling-protocol/utcp-specification",
        supported_repositories: [
          "https://github.com/universal-tool-calling-protocol",
          "https://github.com/universal-tool-calling-protocol/utcp-specification",
          "https://github.com/universal-tool-calling-protocol/utcp-mcp",
          "https://github.com/universal-tool-calling-protocol/rs-utcp",
          "https://github.com/universal-tool-calling-protocol/typescript-utcp",
          "https://github.com/universal-tool-calling-protocol/go-utcp",
          "https://github.com/universal-tool-calling-protocol/python-utcp",
        ],
        error: err instanceof Error ? err.message : "static fallback manual",
        tools: [],
      },
      { status: 200, headers: { "Cache-Control": "no-store" } },
    );
  }
}

export async function POST(req: NextRequest) {
  const denied = guardBridgeRequest(req);
  if (denied) return denied;

  let body: Record<string, unknown>;
  try {
    const parsed = await req.json();
    if (
      typeof parsed !== "object" ||
      parsed === null ||
      Array.isArray(parsed)
    ) {
      return fail(400, "JSON body must be an object");
    }
    body = parsed as Record<string, unknown>;
  } catch {
    return fail(400, "invalid JSON body");
  }

  const queryTool = (
    req.nextUrl?.searchParams?.get("tool") ??
    req.nextUrl?.searchParams?.get("name") ??
    ""
  ).trim();

  const rawToolName =
    typeof body.tool === "string" && body.tool.trim() !== ""
      ? body.tool.trim()
      : typeof body.name === "string" && body.name.trim() !== ""
        ? body.name.trim()
        : queryTool;

  // Support provider-prefixed tool names such as "mizan_oscal.fabric_status"
  const toolName = rawToolName.includes(".")
    ? (rawToolName.split(".").pop() ?? rawToolName)
    : rawToolName;

  if (!toolName || !ALLOWED_UTCP_TOOLS.has(toolName)) {
    return fail(
      403,
      `UTCP tool not permitted on read-only HTTP bridge: ${rawToolName || "<empty>"}`,
    );
  }

  let rawArgs: unknown;
  if (body.arguments !== undefined || body.inputs !== undefined) {
    rawArgs = body.arguments ?? body.inputs ?? {};
  } else if (body.tool === undefined && body.name === undefined) {
    // Standard UTCP HTTP POST body containing direct tool input arguments
    rawArgs = body;
  } else {
    rawArgs = {};
  }

  if (
    typeof rawArgs !== "object" ||
    rawArgs === null ||
    Array.isArray(rawArgs)
  ) {
    return fail(400, "`arguments` or `inputs` must be a JSON object");
  }

  const argsRecord = rawArgs as Record<string, unknown>;
  if ("output_file" in argsRecord || "output_dir" in argsRecord) {
    return fail(
      403,
      "Output path parameters (`output_file`, `output_dir`) are forbidden on the read-only UTCP HTTP bridge",
    );
  }

  for (const [k, v] of Object.entries(argsRecord)) {
    if (typeof v === "string" && looksLikeFilePath(k, v)) {
      if (!resolveSafePath(v)) {
        return fail(
          403,
          `File path argument '${k}' ('${v}') is outside the repository or traverses a symlink`,
        );
      }
    }
  }

  try {
    const stdout = await callOscalCli([
      "utcp",
      "call",
      toolName,
      "--args",
      JSON.stringify(argsRecord),
    ]);
    const receipt = JSON.parse(stdout);
    return NextResponse.json(receipt, {
      status: 200,
      headers: { "Cache-Control": "no-store" },
    });
  } catch (err) {
    const message = err instanceof Error ? err.message : "UTCP execution failed";
    return fail(500, message);
  }
}
