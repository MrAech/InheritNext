import type { AssetDistribution } from "@/types/backend";
import { Actor, HttpAgent, type ActorSubclass } from "@dfinity/agent";
import { IDL } from "@dfinity/candid";
import { idlFactory as civ_backend_idlFactory, canisterId as civ_backend_canisterId } from "@/../../declarations/civ_backend";
import type { Identity } from "@dfinity/agent";
import { getCanisterId } from "@/lib/canisters";

// Minimal service for distribution endpoints
interface DistService {
  // Newer endpoints
  get_asset_distributions?: (asset_id: bigint) => Promise<Array<{ asset_id: bigint; heir_id: bigint; percentage: number }>>;
  set_asset_distributions?: (asset_id: bigint, list: Array<{ asset_id: bigint; heir_id: bigint; percentage: number }>) => Promise<{ Ok: null } | { Err: unknown }>;
  delete_distribution?: (asset_id: bigint, heir_id: bigint) => Promise<{ Ok: null } | { Err: unknown }>;
  // Legacy/fallback endpoints (may or may not exist depending on declarations)
  list_distributions?: () => Promise<Array<{ asset_id: bigint; heir_id: bigint; percentage: number }>>;
  get_distribution?: () => Promise<Array<[string, bigint]>>;
  assign_distributions?: (list: Array<{ asset_id: bigint; heir_id: bigint; percentage: number }>) => Promise<{ Ok: null } | { Err: unknown }>;
}


const envObj: Record<string, string | undefined> | undefined =
  typeof process !== "undefined" && typeof process.env !== "undefined"
    ? (process.env as unknown as Record<string, string | undefined>)
    : undefined;

const backendCanisterId: string | undefined = getCanisterId("civ_backend", civ_backend_canisterId as unknown as string | undefined);
const isBrowser = typeof window !== "undefined";
const host = isBrowser ? window.location.host : "";
const hostname = isBrowser ? window.location.hostname : "";
const isLocalDev = /\.localhost:4943$/.test(host);
const isFrontendLocalhost = isBrowser && (hostname === 'localhost' || hostname === '127.0.0.1' || hostname === '0.0.0.0');
const isIC = hostname.endsWith(".ic0.app") || hostname.endsWith(".icp0.io") || (typeof process !== "undefined" && process.env && process.env.DFX_NETWORK === "ic");
const agentHost = isLocalDev
  ? window.location.origin
  : isIC
    ? "https://icp-api.io"
    : (isFrontendLocalhost
      ? "http://127.0.0.1:4943"
      : (backendCanisterId ? `http://${backendCanisterId}.localhost:4943` : "http://127.0.0.1:4943"));

let actor: ActorSubclass<DistService>;

function makeActor(identity?: Identity) {
  const agent = HttpAgent.createSync({ host: agentHost, identity });
  if (agentHost.includes("localhost") || agentHost.includes("127.0.0.1")) {
    void agent.fetchRootKey();
  }
  if (!backendCanisterId) {
    console.error("[API] civ_backend canisterId is undefined. Checked process.env and declarations. Ensure .env has CANISTER_ID_CIV_BACKEND or declarations are generated.");
    throw new Error("civ_backend canisterId is undefined");
  }
  actor = Actor.createActor<DistService>(civ_backend_idlFactory as IDL.InterfaceFactory, { agent, canisterId: backendCanisterId });
  console.log(`Distributions actor created: host=${agentHost}, canisterId=${backendCanisterId}`);
}

makeActor();

export function setDistributionsIdentity(identity: Identity | null) {
  makeActor(identity ?? undefined);
}

export async function getAssetDistributions(assetId: number): Promise<AssetDistribution[]> {
  console.log(`getAssetDistributions called: assetId=${assetId}`);
  const svc = actor as unknown as DistService;
  if (typeof svc.get_asset_distributions === "function") {
    const res = await svc.get_asset_distributions!(BigInt(assetId));
    const mapped = res.map(r => ({ asset_id: Number(r.asset_id), heir_id: Number(r.heir_id), percentage: r.percentage }));
    console.log(`getAssetDistributions result: count=${mapped.length}`);
    return mapped;
  }
  if (typeof svc.list_distributions === "function") {
    console.warn("[API] get_asset_distributions missing; using list_distributions fallback");
    const all = await svc.list_distributions!();
    const filtered = all.filter(r => Number(r.asset_id) === assetId);
    const mapped = filtered.map(r => ({ asset_id: Number(r.asset_id), heir_id: Number(r.heir_id), percentage: r.percentage }));
    console.log(`getAssetDistributions fallback:list_distributions count=${mapped.length}`);
    return mapped;
  }
  if (typeof svc.get_distribution === "function") {
    console.warn("[API] Neither get_asset_distributions nor list_distributions present; using get_distribution (no percentages)");
    const pairs = await svc.get_distribution!();
    const mapped = pairs
      .filter(([aid]) => Number(aid) === assetId)
      .map(([aid, hid]) => ({ asset_id: Number(aid), heir_id: Number(hid), percentage: 0 }));
    console.log(`getAssetDistributions fallback:get_distribution count=${mapped.length}`);
    return mapped;
  }
  const msg = "No suitable distribution query method found on actor. Regenerate declarations (dfx generate) and redeploy canister.";
  console.error("[API] getAssetDistributions error:", msg);
  throw new Error(msg);
}

export async function setAssetDistributions(assetId: number, list: AssetDistribution[]): Promise<boolean> {
  const payload = list.map(l => ({ asset_id: BigInt(l.asset_id), heir_id: BigInt(l.heir_id), percentage: l.percentage }));
  console.log(`setAssetDistributions called: assetId=${assetId}, count=${list.length}`);
  const svc = actor as unknown as DistService;
  if (typeof svc.set_asset_distributions === "function") {
    const res = await svc.set_asset_distributions!(BigInt(assetId), payload);
    const ok = 'Ok' in res;
    console.log(` result: ok=${ok}`);
    if (ok && typeof window !== 'undefined') {
      window.dispatchEvent(new CustomEvent('integrity:changed'));
    }
    return ok;
  }
  if (typeof svc.assign_distributions === "function") {
    if (typeof svc.list_distributions !== "function") {
      const msg = "set_asset_distributions missing and list_distributions unavailable; cannot safely merge. Please dfx generate & deploy.";
      console.error("[API] setAssetDistributions error:", msg);
      throw new Error(msg);
    }
    console.warn("[API] set_asset_distributions missing; using assign_distributions fallback (strict totals)");
    try {
      const currentAll = await svc.list_distributions!();
      const merged = [
        ...currentAll.filter(d => Number(d.asset_id) !== assetId),
        ...payload,
      ];
      const res = await svc.assign_distributions!(merged);
      const ok = 'Ok' in res;
      console.log(`setAssetDistributions fallback: ok=${ok}`);
      if (ok && typeof window !== 'undefined') {
        window.dispatchEvent(new CustomEvent('integrity:changed'));
      }
      return ok;
    } catch (e) {
      console.error("[API] fallback assign_distributions error", e);
      return false;
    }
  }
  const msg = "No suitable distribution update method found on actor. Regenerate declarations (dfx generate) and redeploy canister.";
  console.error("[API] setAssetDistributions error:", msg);
  throw new Error(msg);
}

export async function deleteAssetDistribution(assetId: number, heirId: number): Promise<boolean> {
  console.log(`deleteAssetDistribution called: assetId=${assetId}, heirId=${heirId}`);
  const svc = actor as unknown as DistService;
  if (typeof svc.delete_distribution === 'function') {
    const res = await svc.delete_distribution(BigInt(assetId), BigInt(heirId));
    const ok = 'Ok' in res;
    console.log(`deleteAssetDistribution result: ok=${ok}`);
    if (ok && typeof window !== 'undefined') {
      window.dispatchEvent(new CustomEvent('integrity:changed'));
    }
    return ok;
  }
  // Fallback emulate via set if available
  if (typeof svc.get_asset_distributions === 'function' && typeof svc.set_asset_distributions === 'function') {
    try {
      const current = await svc.get_asset_distributions(BigInt(assetId));
      const remaining = current.filter(d => Number(d.heir_id) !== heirId);
      const res = await svc.set_asset_distributions(BigInt(assetId), remaining);
      const ok = 'Ok' in res;
      console.log(`deleteAssetDistribution fallback(set) ok=${ok}`);
      if (ok && typeof window !== 'undefined') {
        window.dispatchEvent(new CustomEvent('integrity:changed'));
      }
      return ok;
    } catch (e) {
      console.error('deleteAssetDistribution fallback error', e);
      return false;
    }
  }
  console.error('deleteAssetDistribution: no supported deletion mechanism');
  return false;
}
