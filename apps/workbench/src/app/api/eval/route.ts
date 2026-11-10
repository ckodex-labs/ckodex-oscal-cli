import { NextRequest, NextResponse } from "next/server";
import { callOscalCli } from "@/lib/mcp-client";
import {
  MAX_BODY_BYTES,
  bridgeError,
  guardBridgeRequest,
  readBoundedJsonBody,
} from "@/lib/bridge-guard";
import fs from "fs";
import os from "os";
import path from "path";

export async function POST(req: NextRequest) {
  const rejected = guardBridgeRequest(req);
  if (rejected) return rejected;

  const parsedBody = await readBoundedJsonBody(req, MAX_BODY_BYTES);
  if (!parsedBody.ok) return parsedBody.response;

  let tempDir: string | null = null;
  try {
    const body = (parsedBody.value ?? {}) as Record<string, unknown>;
    const { rule, payload } = body;

    if (
      typeof rule !== "string" ||
      !/^[A-Za-z0-9._:-]+$/.test(rule) ||
      rule.startsWith("-")
    ) {
      return bridgeError(400, "Invalid rule identifier");
    }
    if (
      typeof payload !== "string" ||
      Buffer.byteLength(payload, "utf8") > MAX_BODY_BYTES
    ) {
      return bridgeError(400, "Payload must be a string under 1 MB");
    }
    if (!rule || !payload) {
      return NextResponse.json(
        { success: false, error: "Missing required 'rule' or 'payload'" },
        { status: 400 },
      );
    }

    tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "mizan-eval-"));
    const tempFilePath = path.join(tempDir, "input.json");
    fs.writeFileSync(tempFilePath, payload, { encoding: "utf-8", mode: 0o600 });

    const output = await callOscalCli([
      "policy",
      "rulepack",
      "eval",
      "-r",
      rule,
      "-i",
      tempFilePath,
      "--format",
      "json",
    ]);

    let parsed: unknown;
    try {
      parsed = JSON.parse(output);
    } catch {
      parsed = { raw: output };
    }

    return NextResponse.json({ success: true, data: parsed });
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err);
    return NextResponse.json(
      { success: false, error: message },
      { status: 500 },
    );
  } finally {
    if (tempDir) {
      try {
        fs.rmSync(tempDir, { recursive: true, force: true });
      } catch {
        // ignore cleanup error
      }
    }
  }
}
