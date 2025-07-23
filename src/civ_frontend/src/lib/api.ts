import { Actor, HttpAgent } from "@dfinity/agent";
import { IDL } from "@dfinity/candid";
import type {
  Asset,
  Heir,
  AssetInput,
  AssetDistribution,
  HeirInput,
  Timer,
} from "@/types/backend";

import { idlFactory as civ_backend_idlFactory, canisterId as civ_backend_canisterId } from "../../../declarations/civ_backend";

const CANISTER_ID = civ_backend_canisterId;


const idlFactory = civ_backend_idlFactory;

const backendCanisterId = import.meta.env.VITE_CANISTER_ID_CIV_BACKEND || CANISTER_ID;
const agentHost = backendCanisterId
  ? `http://${backendCanisterId}.localhost:4943`
  : "http://localhost:4943";
  // TODO: FIXME: use something other than this depricated httpAgent @gaurisingh73
const agent = new HttpAgent({ host: agentHost });
if (agentHost.includes("localhost")) {
  // Required for local development to validate certificates
  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  agent.fetchRootKey();
}
const actor = Actor.createActor(idlFactory, { agent, canisterId: CANISTER_ID });


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
  return withRetry(() => actor.list_assets()).then((result: any) => Array.isArray(result) ? result : []);
}

export async function listHeirs(): Promise<Heir[]> {
  return withRetry(() => actor.list_heirs()).then((result: any) => Array.isArray(result) ? result : []);
}

export async function listDistributions(): Promise<AssetDistribution[]> {
  return withRetry(() => actor.get_distribution()).then((result: any) => Array.isArray(result) ? result : []);
}

export async function addAsset(asset: AssetInput): Promise<boolean> {
  const assetToSend = { ...asset, value: BigInt(asset.value) };
  return withRetry(() => actor.add_asset(assetToSend)).then((result: any) => {
    if (result && result.Ok !== undefined) return true;
    return false;
  });
}

export async function updateAsset(id: number, asset: AssetInput): Promise<boolean> {
  const assetToSend = { ...asset, value: BigInt(asset.value) };
  return withRetry(() => actor.update_asset(id, assetToSend)).then((result: any) => {
    if (result && result.Ok !== undefined) return true;
    return false;
  });
}

export async function removeAsset(id: number): Promise<boolean> {
  return withRetry(() => actor.remove_asset(id)).then((result: any) => {
    if (result && result.Ok !== undefined) return true;
    return false;
  });
}

export async function addHeir(heir: HeirInput): Promise<boolean> {
  return withRetry(() => actor.add_heir(heir)).then((result: any) => {
    if (result && result.Ok !== undefined) return true;
    return false;
  });
}

export async function updateHeir(id: number, heir: HeirInput): Promise<boolean> {
  return withRetry(() => actor.update_heir(id, heir)).then((result: any) => {
    if (result && result.Ok !== undefined) return true;
    return false;
  });
}

export async function removeHeir(id: number): Promise<boolean> {
  return withRetry(() => actor.remove_heir(id)).then((result: any) => {
    if (result && result.Ok !== undefined) return true;
    return false;
  });
}

export async function assignDistributions(distributions: AssetDistribution[]): Promise<boolean> {
  return withRetry(() => actor.assign_distributions(distributions)).then((result: any) => {
    if (result && result.Ok !== undefined) return true;
    return false;
  });
}

export async function timerStatus(): Promise<Timer> {
  return withRetry(() => actor.get_timer()).then((result: any) => typeof result === "number" ? result : 0);
}

export async function resetTimer(): Promise<boolean> {
  return withRetry(() => actor.reset_timer()).then((result: any) => {
    if (result && result.Ok !== undefined) return true;
    return false;
  });
}