import { Actor, HttpAgent, type ActorSubclass, type Identity } from "@dfinity/agent";
import { IDL } from "@dfinity/candid";
import type {
  Asset,
  Heir,
  AssetInput,
  AssetDistribution,
  HeirInput,
  Timer,
} from "@/types/backend";

import { idlFactory as civ_backend_idlFactory, canisterId as civ_backend_canisterId } from "@/../../declarations/civ_backend";
import { getCanisterId } from "@/lib/canisters";

// Define a local service interface to match the canister, avoiding stale generated type mismatches
type ServiceAsset = {
  id: bigint;
  name: string;
  asset_type: string;
  value: bigint;
  description: string;
  created_at: bigint;
  updated_at: bigint;
};

type ServiceHeir = {
  id: bigint;
  name: string;
  relationship: string;
  email: string;
  phone: string;
  address: string;
  created_at: bigint;
  updated_at: bigint;
};

type ServiceDistribution = {
  asset_id: bigint;
  heir_id: bigint;
  percentage: number;
};

type ServiceResult = { Ok: null } | { Err: unknown };

// Integrity report returned by backend check_integrity query
export interface IntegrityReport {
  asset_count: bigint; // backend uses u64
  distribution_count: bigint;
  over_allocated_assets: Array<bigint>;
  fully_allocated_assets: Array<bigint>;
  partially_allocated_assets: Array<bigint>;
  unallocated_assets: Array<bigint>;
  issues: Array<string>;
}

interface Service {
  // optional get_user for connectivity check
  get_user?: () => Promise<unknown>;
  add_asset: (asset: { name: string; asset_type: string; value: bigint; description: string }) => Promise<ServiceResult>;
  update_asset: (id: bigint, asset: { name: string; asset_type: string; value: bigint; description: string }) => Promise<ServiceResult>;
  remove_asset: (id: bigint) => Promise<ServiceResult>;
  list_assets: () => Promise<ServiceAsset[]>;

  add_heir: (heir: { name: string; relationship: string; email: string; phone: string; address: string }) => Promise<ServiceResult>;
  update_heir: (id: bigint, heir: { name: string; relationship: string; email: string; phone: string; address: string }) => Promise<ServiceResult>;
  remove_heir: (id: bigint) => Promise<ServiceResult>;
  list_heirs: () => Promise<ServiceHeir[]>;

  assign_distributions: (dists: ServiceDistribution[]) => Promise<ServiceResult>;
  get_timer: () => Promise<bigint>;
  reset_timer: () => Promise<ServiceResult>;
  check_integrity: () => Promise<IntegrityReport>;
}

const CANISTER_ID = civ_backend_canisterId;
const idlFactory = civ_backend_idlFactory;


const backendCanisterId = getCanisterId("civ_backend", CANISTER_ID as unknown as string | undefined);
if (!backendCanisterId) {
  console.error("[API] civ_backend canisterId is undefined. Checked process.env (CANISTER_ID_CIV_BACKEND, CANISTER_ID) and generated declarations. Ensure .env is loaded and declarations are generated.");
  throw new Error("civ_backend canisterId is undefined");
}
const isBrowser = typeof window !== "undefined";
const host = isBrowser ? window.location.host : "";
const hostname = isBrowser ? window.location.hostname : "";
const isLocalDev = /\.localhost:4943$/.test(host);
const isIC = hostname.endsWith(".ic0.app") || hostname.endsWith(".icp0.io") || (typeof process !== "undefined" && process.env && process.env.DFX_NETWORK === "ic");
const agentHost = isLocalDev
  ? window.location.origin
  : isIC
    ? "https://icp-api.io"
    : (backendCanisterId ? `http://${backendCanisterId}.localhost:4943` : "http://127.0.0.1:4943");
// TODO: FIXME: use something other than this depricated httpAgent @gaurisingh73
let actor: ActorSubclass<Service>;
let lastRootKeyHash: string | null = null;
const ROOT_KEY_HASH_SS_KEY = "__IC_ROOT_KEY_HASH";

async function computeRootKeyHash(agent: HttpAgent): Promise<string | null> {
  try {
    // rootKey is Uint8Array after fetchRootKey
    // Ensure we fetched it
    // @ts-expect-error accessing internal rootKey (not in public types)
    const rootKey: Uint8Array | undefined = agent.rootKey;
    if (!rootKey || !(rootKey instanceof Uint8Array) || rootKey.length === 0) return null;
    if (typeof crypto !== 'undefined' && crypto.subtle) {
      const copy = new Uint8Array(rootKey); // ensure plain Uint8Array
      const digest = await crypto.subtle.digest('SHA-256', copy);
      return Array.from(new Uint8Array(digest)).map(b => b.toString(16).padStart(2, '0')).join('');
    }
    return Array.from(rootKey).map(b => b.toString(16).padStart(2, '0')).join('');
  } catch (e) {
    console.warn('[API] computeRootKeyHash failed', e);
    return null;
  }
}

export function getLastRootKeyHash(): string | null {
  return lastRootKeyHash;
}

async function internalCreateActor(identity?: Identity) {
  const agent = HttpAgent.createSync({ host: agentHost, identity });
  if (agentHost.includes("localhost") || agentHost.includes("127.0.0.1")) {
    await agent.fetchRootKey();
    const hash = await computeRootKeyHash(agent);
    const stored = sessionStorage.getItem(ROOT_KEY_HASH_SS_KEY);
    if (hash) {
      if (stored && stored !== hash) {
        console.warn('[API] Root key hash changed (replica restart?) old=', stored, 'new=', hash);
      }
      sessionStorage.setItem(ROOT_KEY_HASH_SS_KEY, hash);
      lastRootKeyHash = hash;
    }
  }
  actor = Actor.createActor<Service>(idlFactory as unknown as IDL.InterfaceFactory, { agent, canisterId: backendCanisterId });
  const principalText = identity ? (await identity.getPrincipal()).toText() : 'anonymous';
  console.log(`API actor created: host=${agentHost}, canisterId=${backendCanisterId}, principal=${principalText}, rootKeyHash=${lastRootKeyHash}`);
}

function createActor(identity?: Identity) {
  // wrap async but keep external sync signature for existing callers
  void internalCreateActor(identity);
}

// Initialize with anonymous identity
createActor();

export function setApiIdentity(identity: Identity | null) {
  createActor(identity ?? undefined);
}


async function withRetry<T>(fn: () => Promise<T>, retries = 2): Promise<T> {
  let lastError;
  for (let i = 0; i <= retries; i++) {
    try {
      return await fn();
    } catch (err) {
      lastError = err;
      if (i === retries) throw err;
      await new Promise(res => setTimeout(res, 500 * (i + 1)));
    }
  }
  throw lastError;
}


export async function listAssets(): Promise<Asset[]> {
  const result = await withRetry(() => actor.list_assets());
  // Map bigint fields to number for UI layer
  return (Array.isArray(result) ? result : []).map((a: ServiceAsset) => ({
    id: Number(a.id),
    name: a.name,
    asset_type: a.asset_type,
    value: Number(a.value),
    description: a.description,
    created_at: Number(a.created_at),
    updated_at: Number(a.updated_at),
  }));
}

export async function listHeirs(): Promise<Heir[]> {
  const result = await withRetry(() => actor.list_heirs());
  return (Array.isArray(result) ? result : []).map((h: ServiceHeir) => ({
    id: Number(h.id),
    name: h.name,
    relationship: h.relationship,
    email: h.email,
    phone: h.phone,
    address: h.address,
    created_at: Number(h.created_at),
    updated_at: Number(h.updated_at),
  }));
}

// Note: backend currently returns [asset_id_text, heir_id] without percentage; leaving as stub
export async function listDistributions(): Promise<AssetDistribution[]> {
  return [];
}

export async function addAsset(asset: AssetInput): Promise<boolean> {
  console.log(`addAsset called: name=${asset.name}, type=${asset.asset_type}, value=${asset.value}`);
  const assetToSend = { ...asset, value: BigInt(asset.value) };
  const result = await withRetry(() => actor.add_asset(assetToSend));
  const ok = 'Ok' in result;
  if (ok) {
    console.log("onAssetAdded callback triggered");
    try {
      const t = await withRetry(() => actor.get_timer());
      console.log(`Timer value after asset added: ${Number(t)}`);
    } catch (e) {
      console.log("Timer fetch after asset add failed");
    }
  }
  return ok;
}

export async function updateAsset(id: number, asset: AssetInput): Promise<boolean> {
  const assetToSend = { ...asset, value: BigInt(asset.value) };
  const result = await withRetry(() => actor.update_asset(BigInt(id), assetToSend));
  return 'Ok' in result;
}

export async function removeAsset(id: number): Promise<boolean> {
  const result = await withRetry(() => actor.remove_asset(BigInt(id)));
  return 'Ok' in result;
}

export async function addHeir(heir: HeirInput): Promise<boolean> {
  const result = await withRetry(() => actor.add_heir(heir));
  return 'Ok' in result;
}

export async function updateHeir(id: number, heir: HeirInput): Promise<boolean> {
  const result = await withRetry(() => actor.update_heir(BigInt(id), heir));
  return 'Ok' in result;
}

export async function removeHeir(id: number): Promise<boolean> {
  const result = await withRetry(() => actor.remove_heir(BigInt(id)));
  return 'Ok' in result;
}

export async function assignDistributions(distributions: AssetDistribution[]): Promise<boolean> {
  console.log(`assignDistributions called: count=${distributions.length}`);
  const payload: ServiceDistribution[] = distributions.map((d) => ({
    asset_id: BigInt(d.asset_id),
    heir_id: BigInt(d.heir_id),
    percentage: d.percentage,
  }));
  const result = await withRetry(() => actor.assign_distributions(payload));
  console.log(`assignDistributions result: ok=${'Ok' in result}`);
  return 'Ok' in result;
}

export async function timerStatus(): Promise<Timer> {
  const res = await withRetry(() => actor.get_timer());
  // Backend returns int (i64) as bigint; convert to number for UI
  const value = Number(res);
  console.log(`Timer value: ${value}`);
  return value;
}

export async function resetTimer(): Promise<boolean> {
  const result: ServiceResult = await withRetry(() => actor.reset_timer());
  return 'Ok' in result;
}

// Fetch integrity report and map bigint arrays to numbers for convenience
export async function checkIntegrity(): Promise<{
  assetCount: number;
  distributionCount: number;
  overAllocated: number[];
  fullyAllocated: number[];
  partiallyAllocated: number[];
  unallocated: number[];
  issues: string[];
}> {
  const raw = await withRetry(() => actor.check_integrity());
  const mapIds = (arr: Array<bigint>) => arr.map(n => Number(n));
  const report = {
    assetCount: Number(raw.asset_count),
    distributionCount: Number(raw.distribution_count),
    overAllocated: mapIds(raw.over_allocated_assets),
    fullyAllocated: mapIds(raw.fully_allocated_assets),
    partiallyAllocated: mapIds(raw.partially_allocated_assets),
    unallocated: mapIds(raw.unallocated_assets),
    issues: raw.issues.slice(),
  };
  console.log('[Integrity] report', report);
  return report;
}

// Connectivity helper: verify agent/canister reachability and identity
export async function checkBackendConnection(): Promise<{ ok: boolean; canisterId: string; host: string; principal?: string; error?: string }> {
  try {
    // Identity principal is embedded in the agent used by the actor
    const agentField = (actor as unknown as { _agent?: HttpAgent })._agent;
    const principal = agentField && (await agentField.getPrincipal())?.toText();
    const host = agentHost;
    const canisterId = backendCanisterId;
    // Try a cheap query – list_assets or optional get_user
    let ok = false;
    try {
      const maybeActor = actor as unknown as Record<string, unknown>;
      const hasGetUser = 'get_user' in maybeActor && typeof maybeActor['get_user'] === 'function';
      if (hasGetUser) {
        await (maybeActor['get_user'] as () => Promise<unknown>)();
      } else {
        await listAssets();
      }
      ok = true;
    } catch (e) {
      console.error("[API] connectivity probe failed", e);
      return { ok: false, canisterId, host, principal, error: String(e) };
    }
    console.debug("[API] connectivity ok", { canisterId, host, principal });
    return { ok, canisterId, host, principal };
  } catch (e) {
    return { ok: false, canisterId: backendCanisterId, host: agentHost, error: String(e) };
  }
}