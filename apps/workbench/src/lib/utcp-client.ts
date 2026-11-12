/**
 * TypeScript Universal Tool Calling Protocol (UTCP) Client Adapter
 * Compatible with `@utcp/sdk` (`https://github.com/universal-tool-calling-protocol/typescript-utcp`)
 * and `utcp-specification` (`https://github.com/universal-tool-calling-protocol/utcp-specification`).
 */

export interface UtcpCallTemplate {
  call_template_type: "cli" | "http" | "grpc" | "mcp" | "sse" | "text";
  name?: string;
  allowed_communication_protocols?: string[];
  command?: string;
  command_name?: string;
  args?: string[];
  commands?: Array<{ command: string; append_to_final_output?: boolean }>;
  param_style?: string;
  output_format?: string;
  url?: string;
  http_method?: string;
  content_type?: string;
  host?: string;
  port?: number;
  service_name?: string;
  method_name?: string;
  use_ssl?: boolean;
  grpc_service?: string;
  grpc_method?: string;
  mcp_server?: string;
  mcp_tool_name?: string;
  config?: Record<string, unknown>;
  file_path?: string;
  base_path?: string;
  headers?: Record<string, string>;
}

export interface UtcpToolProvider {
  provider_type: "cli" | "http" | "grpc" | "mcp" | "sse" | "text";
  url?: string;
  http_method?: string;
  command_name?: string;
  grpc_service?: string;
  grpc_method?: string;
  mcp_tool_name?: string;
}

export interface UtcpTool {
  name: string;
  description: string;
  tags: string[];
  read_only: boolean;
  inputs: Record<string, unknown>;
  outputs: Record<string, unknown>;
  tool_call_template: UtcpCallTemplate;
  tool_provider: UtcpToolProvider;
}

export interface UtcpManual {
  manual_version: string;
  utcp_version: string;
  version: string;
  name: string;
  description: string;
  info?: {
    title: string;
    version: string;
    description?: string;
  };
  allowed_communication_protocols?: string[];
  homepage: string;
  specification_url: string;
  supported_repositories: string[];
  tools: UtcpTool[];
  manual_digest: string;
}

export interface UtcpCallReceipt<T = unknown> {
  receipt_id: string;
  tool_name: string;
  transport: string;
  timestamp: string;
  input_digest: string;
  output_digest: string;
  status: string;
  output: T;
}

function getBasePath(): string {
  return process.env.NEXT_PUBLIC_BASE_PATH ?? "";
}

function withBasePath(endpoint: string): string {
  const base = getBasePath();
  if (
    !base ||
    !endpoint.startsWith("/") ||
    endpoint === base ||
    endpoint.startsWith(`${base}/`)
  ) {
    return endpoint;
  }
  return `${base}${endpoint}`;
}

/**
 * 3-Tier UTCP Manual Discovery:
 * 1. Tauri IPC (`get_utcp_manual`) when running inside the Mizan Desktop application.
 * 2. Local Next.js HTTP Bridge (`GET /api/utcp`).
 * 3. Static Build-Time Snapshot (`GET /snapshot/utcp-manual.json`) for air-gapped / GitHub Pages deployment.
 */
export async function fetchMizanUtcpManual(
  endpoint = "/api/utcp",
  transport = "http",
): Promise<UtcpManual> {
  if (typeof window !== "undefined" && window.__TAURI__?.core?.invoke) {
    try {
      return await window.__TAURI__.core.invoke<UtcpManual>("get_utcp_manual", {
        transport,
        baseUrl: "http://localhost:3000",
        base_url: "http://localhost:3000",
      });
    } catch {
      // Fall through to HTTP bridge / static snapshot
    }
  }

  const resolvedEndpoint = withBasePath(endpoint);
  const skipStaticApiProbe =
    process.env.NEXT_PUBLIC_STATIC_EXPORT === "true" && endpoint === "/api/utcp";
  if (!skipStaticApiProbe) {
    try {
      const sep = resolvedEndpoint.includes("?") ? "&" : "?";
      const res = await fetch(
        `${resolvedEndpoint}${sep}transport=${encodeURIComponent(transport)}`,
        {
          method: "GET",
          headers: { Accept: "application/json" },
          signal: AbortSignal.timeout(8_000),
        },
      );
      if (res.ok) {
        return (await res.json()) as UtcpManual;
      }
    } catch {
      // Fall through to static snapshot fallback
    }
  }

  const snapshotUrl = `${getBasePath()}/snapshot/utcp-manual.json`;
  const snapRes = await fetch(snapshotUrl, {
    method: "GET",
    headers: { Accept: "application/json" },
    signal: AbortSignal.timeout(8_000),
  });
  if (!snapRes.ok) {
    throw new Error(
      `Failed to fetch UTCP manual from live bridge (${resolvedEndpoint}) or snapshot (${snapshotUrl}: ${snapRes.status})`,
    );
  }
  return (await snapRes.json()) as UtcpManual;
}

/**
 * Multi-Tier UTCP Tool Invocation:
 * 1. Tauri IPC (`call_utcp_tool`) when running inside the Mizan Desktop application.
 * 2. Local Next.js HTTP Bridge (`POST /api/utcp?tool=<toolName>`).
 */
export async function invokeMizanUtcpTool<T = unknown>(
  toolName: string,
  inputs: Record<string, unknown> = {},
  endpoint = "/api/utcp",
): Promise<UtcpCallReceipt<T>> {
  const normalizedTool = toolName.includes(".")
    ? (toolName.split(".").pop() ?? toolName)
    : toolName;

  if (typeof window !== "undefined" && window.__TAURI__?.core?.invoke) {
    return window.__TAURI__.core.invoke<UtcpCallReceipt<T>>("call_utcp_tool", {
      toolName: normalizedTool,
      tool_name: normalizedTool,
      inputs,
    });
  }

  const resolvedEndpoint = withBasePath(endpoint);
  const sep = resolvedEndpoint.includes("?") ? "&" : "?";
  const res = await fetch(
    `${resolvedEndpoint}${sep}tool=${encodeURIComponent(normalizedTool)}`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ tool: normalizedTool, inputs }),
      signal: AbortSignal.timeout(30_000),
    },
  );
  if (!res.ok) {
    const errText = await res.text();
    throw new Error(`UTCP call failed (${res.status}): ${errText}`);
  }
  return (await res.json()) as UtcpCallReceipt<T>;
}
