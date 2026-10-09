/**
 * Shared request guards for the local dev bridges (/api/cli, /api/eval).
 *
 * These routes spawn the local `mizan` binary, so they must not be reachable
 * from other origins while `next dev` is running (drive-by CSRF).
 *
 * - Content-Type must be application/json. Cross-origin JSON POSTs require a
 *   CORS preflight, which these routes never approve.
 * - When a browser sends an Origin header it must match the Host header.
 */

import { NextRequest, NextResponse } from "next/server";

export function bridgeError(status: number, error: string) {
  return NextResponse.json(
    { success: false, transport: "http-cli-bridge", error },
    { status },
  );
}

/** Returns an error response if the request must be rejected, else null. */
export function guardBridgeRequest(req: NextRequest): NextResponse | null {
  const origin = req.headers.get("origin");
  if (origin) {
    let ok = false;
    try {
      ok = new URL(origin).host === req.headers.get("host");
    } catch {
      ok = false;
    }
    if (!ok) return bridgeError(403, "Cross-origin requests are not accepted");
  }
  const ct = req.headers.get("content-type") ?? "";
  if (!ct.startsWith("application/json")) {
    return bridgeError(415, "Content-Type must be application/json");
  }
  return null;
}
