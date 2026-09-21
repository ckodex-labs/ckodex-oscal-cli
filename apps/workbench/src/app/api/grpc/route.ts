import { NextRequest, NextResponse } from "next/server";
import { callOscalCli } from "@/lib/mcp-client";

export async function POST(req: NextRequest) {
  try {
    const body = await req.json();
    const { command, args = [] } = body;

    // Execute via oscal-cli gRPC transport client
    const output = await callOscalCli([command, ...args, "--format", "json"]);
    return NextResponse.json({ success: true, data: JSON.parse(output) });
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err);
    return NextResponse.json(
      { success: false, error: message },
      { status: 500 },
    );
  }
}
