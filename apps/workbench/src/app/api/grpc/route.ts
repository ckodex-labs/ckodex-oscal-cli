/**
 * Local Development HTTP-to-CLI Bridge Adapter (/api/grpc)
 *
 * ARCHITECTURAL CONTRACT:
 * - Local Development Bridge: This endpoint acts as an HTTP-to-CLI bridge adapter for local development.
 *   It translates HTTP POST requests into native `mizan` CLI subprocess execution via `callOscalCli`.
 *   It is NOT a direct gRPC wire protocol or gRPC-Web proxy.
 * - Transport Boundary: Used exclusively in dynamic server environments (e.g., local Next.js dev server).
 * - Offline / Static Export Mode: In static export distributions (`output: "export"`, e.g., GitHub Pages)
 *   or air-gapped web environments, Next.js server-side API routes are unavailable. Callers and frontend
 *   surfaces MUST handle offline fallback gracefully (via in-browser AST evaluation or Tauri IPC)
 *   with explicit valence (e.g., Inferred or Claimed).
 *
 * API CONTRACT:
 * Request:
 *   - command (string, required): The mizan CLI subcommand to execute (e.g. "blast-radius", "fedramp", "sync", "inspect")
 *   - args (string[], optional): Command line flags and positional arguments
 *
 * Response:
 *   - success (boolean): Whether CLI execution succeeded (exit code 0)
 *   - transport (string): "http-cli-bridge" (identifies this adapter contract)
 *   - data (unknown): Parsed JSON payload or raw CLI stdout
 *   - error (string, optional): Error message if execution or parsing failed
 */

import { NextRequest, NextResponse } from "next/server";
import { callOscalCli } from "@/lib/mcp-client";

export async function POST(req: NextRequest) {
  try {
    const body = await req.json();
    const { command, args = [] } = body;

    if (!command || typeof command !== "string") {
      return NextResponse.json(
        {
          success: false,
          transport: "http-cli-bridge",
          error: "Missing or invalid required parameter 'command'",
        },
        { status: 400 },
      );
    }

    const output = await callOscalCli([command, ...args, "--format", "json"]);
    let parsed: unknown;
    try {
      parsed = JSON.parse(output);
    } catch {
      parsed = { raw: output };
    }
    return NextResponse.json({
      success: true,
      transport: "http-cli-bridge",
      data: parsed,
    });
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err);
    return NextResponse.json(
      {
        success: false,
        transport: "http-cli-bridge",
        error: message,
      },
      { status: 500 },
    );
  }
}
