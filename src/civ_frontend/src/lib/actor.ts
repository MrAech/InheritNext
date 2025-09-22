// Frontend actor wrappers for backend endpoints.
// This file detects local vs production canister IDs, wires Internet Identity (AuthClient)
// when available, and exposes safe wrappers with basic error handling.
import { Actor, HttpAgent } from "@dfinity/agent";
import { idlFactory as backend_idl } from "../../../declarations/civ_backend/civ_backend.did.js";
import { Principal } from "@dfinity/principal";
import { AuthClient } from "@dfinity/auth-client";

// Helper to resolve the backend canister id.
// Priority: VITE_BACKEND_CANISTER_ID env var (injected by dfx or .env),
// then window.__CIV_BACKEND_CANISTER_ID (set by hosting pages), otherwise undefined.
function resolveBackendCanisterId(): string | undefined {
  // For dfx workflows prefer process.env-based injection. Use globalThis.process to
  // avoid TypeScript errors when node types are not present. This keeps compatibility
  // with dfx which injects env vars into the dev/build step.
  const proc: any = (globalThis as any).process;
  if (
    proc &&
    proc.env &&
    proc.env.VITE_BACKEND_CANISTER_ID &&
    proc.env.VITE_BACKEND_CANISTER_ID !== "undefined"
  ) {
    return proc.env.VITE_BACKEND_CANISTER_ID;
  }
  // Window override for hosting scenarios (optional)
  // @ts-ignore
  if (
    typeof window !== "undefined" &&
    (window as any).__CIV_BACKEND_CANISTER_ID
  ) {
    // @ts-ignore
    return (window as any).__CIV_BACKEND_CANISTER_ID as string;
  }
  return undefined;
}

// Create an agent for anonymous usage first. When the user authenticates we will
// create a new agent with the user's identity.
function createAnonymousAgent(): HttpAgent {
  // For local development the default agent is fine; in prod the host must be set
  const host =
    typeof window !== "undefined" && window.location.hostname !== "localhost"
      ? window.location.origin
      : undefined;
  if (host) {
    return new HttpAgent({ host });
  }
  return new HttpAgent();
}

// Lazy actor creation: we recreate the actor when the identity changes (after login)
let currentAgent: HttpAgent | null = null;
let backendActor: any = null;

function makeActor(agent: HttpAgent) {
  const canisterId = resolveBackendCanisterId();
  if (!canisterId)
    throw new Error(
      "Backend canister id not configured. Set VITE_BACKEND_CANISTER_ID in .env or window.__CIV_BACKEND_CANISTER_ID",
    );
  return Actor.createActor(backend_idl, { agent, canisterId });
}

// Initialize anonymous actor
currentAgent = createAnonymousAgent();
try {
  backendActor = makeActor(currentAgent);
} catch (e) {
  backendActor = null;
}

// Public API: authenticate with Internet Identity (AuthClient) and recreate the actor with identity
export async function authenticate(): Promise<void> {
  const authClient = await AuthClient.create();
  if (await authClient.isAuthenticated()) {
    const identity = authClient.getIdentity();
    currentAgent = new HttpAgent({ identity });
    // For local development env we should fetch the root key (only for dev)
    // Prefer process.env.NODE_ENV (dfx-friendly). Use globalThis.process to avoid TS errors
    const proc: any = (globalThis as any).process;
    const nodeEnv = proc && proc.env ? proc.env.NODE_ENV : undefined;
    if (!nodeEnv || nodeEnv !== "production") {
      try {
        await (currentAgent as any).fetchRootKey();
      } catch (_) {
        /* ignore */
      }
    }
    backendActor = makeActor(currentAgent);
    return;
  }
  // Not authenticated; start login flow
  await authClient.login({
    onSuccess: async () => {
      await authenticate();
    },
  });
}

// Ensure actor is ready (fallback to anonymous actor if not authenticated)
function ensureActor() {
  if (!backendActor) {
    if (!currentAgent) currentAgent = createAnonymousAgent();
    backendActor = makeActor(currentAgent);
  }
  return backendActor;
}

// Robust wrappers with basic error handling
export async function recordTokenApproval(
  tokenCanisterStr: string,
  assetType: string,
  approvedAmount: bigint,
  approvalExpiry: bigint | null,
  autoRenew: boolean,
) {
  try {
    const tokenCanister = Principal.fromText(tokenCanisterStr);
    const actor = ensureActor();
    return await actor.record_token_approval(
      tokenCanister,
      assetType,
      approvedAmount,
      approvalExpiry,
      autoRenew,
    );
  } catch (err) {
    console.error("recordTokenApproval failed", err);
    throw err;
  }
}

export async function registerVaultedNFT(
  collectionCanisterStr: string,
  tokenId: string,
  assignedHeirHash: string,
) {
  try {
    const collection = Principal.fromText(collectionCanisterStr);
    const actor = ensureActor();
    return await actor.register_vaulted_nft(
      collection,
      tokenId,
      assignedHeirHash,
    );
  } catch (err) {
    console.error("registerVaultedNFT failed", err);
    throw err;
  }
}

// Query backend for ledger decimals for an asset_type (returns number or null)
export async function getLedgerDecimals(
  assetType: string,
): Promise<number | null> {
  try {
    const actor = ensureActor();
    const res = await actor.get_ledger_decimals(assetType);
    // actor returns Option<nat8> i.e. null or number
    if (res === null || res === undefined) return null;
    return Number(res);
  } catch (err) {
    console.error("getLedgerDecimals failed", err);
    return null;
  }
}

// Small helper for UI to test whether user is authenticated
export async function isAuthenticated(): Promise<boolean> {
  const authClient = await AuthClient.create();
  return authClient.isAuthenticated();
}

// Expose current canister id for UI
export function getBackendCanisterId(): string | undefined {
  return resolveBackendCanisterId();
}
