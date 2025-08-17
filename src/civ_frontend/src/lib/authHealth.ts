// Utility to validate backend session / delegation health.
// Detects common signature/delegation errors and signals caller to force re-login.
import { checkBackendConnection } from "@/lib/api";

export interface AuthHealth {
  ok: boolean;
  principal?: string;
  error?: string;
  needsRelogin: boolean; // true if delegation / signature invalid
}

const SIGNATURE_ERROR_SNIPPETS = [
  "Invalid delegation",
  "Invalid basic signature",
  "certificate verification failed",
  "threshold signature",
  "signature could not be verified",
];

export async function validateBackendSession(): Promise<AuthHealth> {
  try {
    const res = await checkBackendConnection();
    if (res.ok) {
      return { ok: true, principal: res.principal, needsRelogin: false };
    }
    const err = res.error || "Unknown error";
    const needs = SIGNATURE_ERROR_SNIPPETS.some(s => err.includes(s));
    return { ok: false, principal: res.principal, error: err, needsRelogin: needs };
  } catch (e) {
    const msg = String(e);
    const needs = SIGNATURE_ERROR_SNIPPETS.some(s => msg.includes(s));
    return { ok: false, error: msg, needsRelogin: needs };
  }
}

export function isSignatureOrDelegationError(msg: string | undefined): boolean {
  if (!msg) return false;
  return SIGNATURE_ERROR_SNIPPETS.some(s => msg.includes(s));
}
