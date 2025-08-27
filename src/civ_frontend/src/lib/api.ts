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

interface ServiceUser {
  user: string;
  assets: ServiceAsset[];
  heirs: ServiceHeir[];
  distributions: ServiceDistribution[];
  timer_expiry: bigint;
  distributed: boolean;
  last_timer_reset: bigint;
}

interface Service {
  // optional get_user for connectivity check
  get_user?: () => Promise<ServiceUser | null>;
  add_asset: (asset: { name: string; asset_type: string; value?: bigint | null; description: string; kind?: number | string; token_canister?: string | null; token_id?: bigint | null; file_path?: string | null; holding_mode?: number | string | null; nft_standard?: number | string | null; chain_wrapped?: number | string | null; }) => Promise<ServiceResult>;
  update_asset: (id: bigint, asset: { name: string; asset_type: string; value?: bigint | null; description: string; kind?: number | string; token_canister?: string | null; token_id?: bigint | null; file_path?: string | null; holding_mode?: number | string | null; nft_standard?: number | string | null; chain_wrapped?: number | string | null; }) => Promise<ServiceResult>;
  remove_asset: (id: bigint) => Promise<ServiceResult>;
  list_assets: () => Promise<ServiceAsset[]>;

  add_heir: (heir: { name: string; relationship: string; email: string; phone: string; address: string; salt?: string | null; adhaarnum?: string | null }) => Promise<ServiceResult>;
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
// Running the frontend dev server (npm run dev) will set hostname to 'localhost' or '127.0.0.1'.
// Detect that and prefer the replica base host (127.0.0.1:4943) so the dev server can talk
// to a dfx-deployed backend without needing the frontend to be deployed as a canister.
const isFrontendLocalhost = isBrowser && (hostname === 'localhost' || hostname === '127.0.0.1' || hostname === '0.0.0.0');
const isIC = hostname.endsWith(".ic0.app") || hostname.endsWith(".icp0.io") || (typeof process !== "undefined" && process.env && process.env.DFX_NETWORK === "ic");
const agentHost = isLocalDev
  ? window.location.origin
  : isIC
    ? "https://icp-api.io"
    : (isFrontendLocalhost
      ? "http://127.0.0.1:4943"
      : (backendCanisterId ? `http://${backendCanisterId}.localhost:4943` : "http://127.0.0.1:4943"));
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

// Documents & Claims API (mock branches implemented)
export async function listDocuments(): Promise<{ id: number; filename: string; size: number; uploaded_at: number }[]> {
  if (USE_MOCK) return mockStore.documents.slice().map(d => ({ ...d }));
  // fallback: not implemented for canister yet
  return [];
}

export async function addDocument(file: { filename: string; size: number; data?: Uint8Array }): Promise<boolean> {
  if (USE_MOCK) {
    // simulate an upload delay and chunk processing
    const id = genId();
    mockStore.documents.push({ id, filename: file.filename, size: file.size, uploaded_at: Date.now() });
    mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: `Document uploaded: ${file.filename}` });
    // simulate processing delay
    await new Promise((res) => setTimeout(res, 400 + Math.random() * 800));
    return true;
  }
  return false;
}

export async function findClaim(code: string): Promise<{ code: string; heirId: number; assets: number[] } | null> {
  if (USE_MOCK) {
    const found = mockStore.claims.find(c => c.code === code);
    return found ? { ...found } : null;
  }
  return null;
}

export async function redeemClaim(code: string, secret?: string): Promise<{ success: boolean; assets?: number[]; reason?: string }> {
  if (USE_MOCK) {
    const idx = mockStore.claims.findIndex(c => c.code === code);
    if (idx === -1) return { success: false, reason: 'Claim not found' };
    const claim = mockStore.claims[idx];
    // For demo, require secret verification if heir has a stored hashed secret (adhaarnum)
    const heir = mockStore.heirs.find(h => h.id === claim.heirId);
    if (!heir) return { success: false, reason: 'Heir not found' };
    if (heir.adhaarnum) {
      if (!secret) return { success: false, reason: 'Secret required' };
      const v = await verifyClaimSecret(code, secret);
      if (!v.ok) return { success: false, reason: v.reason };
    }
    // Simulate an external withdrawal operation by enqueueing a retry job (mock)
    enqueueRetry('withdraw_claim', { code, heirId: claim.heirId, assets: claim.assets });
    // Remove the claim entry to prevent double spend in demo
    mockStore.claims.splice(idx, 1);
    mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: `Claim redeemed (enqueued withdraw): ${code}` });
    return { success: true, assets: claim.assets.slice() };
  }
  return { success: false, reason: 'Not implemented' };
}

export async function listAuditLog(limit = 50): Promise<{ id: number; ts: number; msg: string }[]> {
  if (USE_MOCK) {
    return mockStore.auditLog.slice(-limit).map(e => ({ ...e }));
  }
  return [];
}

export async function listDistributions(): Promise<AssetDistribution[]> {
  if (USE_MOCK) {
    // return the mockStore.distributions in backend type shape
    return mockStore.distributions.map(d => ({ asset_id: Number(d.asset_id), heir_id: Number(d.heir_id), percentage: d.percentage }));
  }
  return [];
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

// ---------------------------------------------------------------------------
// Mock mode (in-memory) — used for frontend demo when a backend canister is
// unavailable. Set USE_MOCK = true to force mock storage and behavior.
// ---------------------------------------------------------------------------
const USE_MOCK = true;

type MockAsset = {
  id: number;
  name: string;
  asset_type: string;
  value: number;
  description: string;
  created_at: number;
  updated_at: number;
  decimals?: number;
  kind?: string;
  token_canister?: string | null;
  token_id?: number | null;
  file_path?: string | null;
  holding_mode?: string | null;
  nft_standard?: string | null;
  chain_wrapped?: string | null;
  approval_required?: boolean;
};

type MockHeir = {
  id: number;
  name: string;
  relationship?: string;
  email?: string;
  phone?: string;
  address?: string;
  salt?: string;
  adhaarnum?: string;
  created_at: number;
  updated_at: number;
};

const mockStore = {
  assets: [] as MockAsset[],
  heirs: [] as MockHeir[],
  distributions: [] as ServiceDistribution[],
  documents: [] as { id: number; filename: string; size: number; uploaded_at: number }[],
  claims: [] as { code: string; heirId: number; assets: number[] }[],
  claimAttempts: {} as Record<string, { attempts: number; blockedUntil?: number }>,
  retryQueue: [] as { id: number; op: string; payload?: unknown; attempt: number; nextRun: number }[],
  auditLog: [] as { id: number; ts: number; msg: string }[],
  timerExpiry: -1 as number, // -1 not started, 0 expired, >0 seconds remaining
  nextId: 1,
};

function genId() { return mockStore.nextId++; }

// Seed a couple of demo entries to show UI activity
if (USE_MOCK) {
  mockStore.assets.push({ id: genId(), name: 'Demo Token', asset_type: 'Fungible', value: 1000, description: 'Mock fungible token', created_at: Date.now(), updated_at: Date.now(), decimals: 8, approval_required: false });
  mockStore.heirs.push({ id: genId(), name: 'Alice Example', email: 'alice@example.com', phone: '555-0100', address: '123 Demo St', created_at: Date.now(), updated_at: Date.now() });
  mockStore.heirs.push({ id: genId(), name: 'Bob Example', email: 'bob@example.com', phone: '555-0110', address: '456 Demo Ave', created_at: Date.now(), updated_at: Date.now() });
  mockStore.claims.push({ code: 'CLAIM-ABC-123', heirId: mockStore.heirs[0].id, assets: [mockStore.assets[0].id] });
  // background ticker to append audit log and process retry queue
  setInterval(() => {
    // append an audit entry
    mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: `Background heartbeat ${new Date().toLocaleTimeString()}` });
    // process retry queue: pop due items and simulate success/failure with backoff
    const now = Date.now();
    const due = mockStore.retryQueue.filter(r => r.nextRun <= now).slice(0, 5);
    for (const item of due) {
      // simulate an operation - 70% chance of success
      const ok = Math.random() < 0.7;
      if (ok) {
        mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: `Retry op succeeded: ${item.op} id=${item.id}` });
        // remove the item from queue
        const idx = mockStore.retryQueue.findIndex(q => q.id === item.id);
        if (idx !== -1) mockStore.retryQueue.splice(idx, 1);
      } else {
        // exponential backoff schedule
        item.attempt = (item.attempt || 0) + 1;
        const delay = Math.min(60000, 1000 * Math.pow(2, item.attempt));
        item.nextRun = Date.now() + delay;
        mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: `Retry op failed (will retry): ${item.op} id=${item.id} attempt=${item.attempt}` });
      }
    }
    // simple GC: prune audit log to 100 entries
    if (mockStore.auditLog.length > 100) mockStore.auditLog.splice(0, mockStore.auditLog.length - 100);
  }, 10000);
}

// Enqueue an operation into retry queue (used by mock helpers when simulating async tasks)
export function enqueueRetry(op: string, payload?: unknown) {
  const id = genId();
  mockStore.retryQueue.push({ id, op, payload, attempt: 0, nextRun: Date.now() + 1000 });
  mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: `Enqueued retry op: ${op} id=${id}` });
  return id;
}

// Verify secret for a claim code by hashing and comparing to stored heir adhaarnum (mocked)
export async function verifyClaimSecret(code: string, secret: string): Promise<{ ok: boolean; reason?: string }> {
  if (!USE_MOCK) return { ok: false, reason: 'Not implemented' };
  const claim = mockStore.claims.find(c => c.code === code);
  if (!claim) return { ok: false, reason: 'Claim not found' };
  const heir = mockStore.heirs.find(h => h.id === claim.heirId);
  if (!heir) return { ok: false, reason: 'Heir not found' };
  // rate-limit: check attempts
  const rec = mockStore.claimAttempts[code] ?? { attempts: 0 };
  if (rec.blockedUntil && rec.blockedUntil > Date.now()) return { ok: false, reason: 'Temporarily blocked due to too many attempts' };
  // compute sha256 hex of salt+secret if salt present, otherwise of secret alone if heir.adhaarnum stored as hash
  const enc = new TextEncoder();
  const combined = (heir.salt ? (heir.salt + secret) : secret);
  const hashBuf = await crypto.subtle.digest('SHA-256', enc.encode(combined));
  const hex = Array.from(new Uint8Array(hashBuf)).map(b => b.toString(16).padStart(2, '0')).join('');
  const matches = heir.adhaarnum ? (heir.adhaarnum === hex) : false;
  // update attempts
  rec.attempts = (rec.attempts || 0) + (matches ? 0 : 1);
  if (!matches) {
    if (rec.attempts >= 3) {
      rec.blockedUntil = Date.now() + 60_000; // block 1 minute in demo
    }
    mockStore.claimAttempts[code] = rec;
    mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: `Claim verify failed for ${code} attempt=${rec.attempts}` });
    return { ok: false, reason: 'Invalid secret' };
  }
  // reset attempts on success
  if (mockStore.claimAttempts[code]) delete mockStore.claimAttempts[code];
  mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: `Claim verify succeeded for ${code}` });
  return { ok: true };
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
  if (USE_MOCK) {
    return mockStore.assets.map(a => ({
      id: a.id,
      name: a.name,
      asset_type: a.asset_type,
      value: a.value,
      decimals: a.decimals ?? 0,
      description: a.description,
      created_at: a.created_at,
      updated_at: a.updated_at,
    } as Asset));
  }
  const result = await withRetry(() => actor.list_assets());
  // Map bigint fields to number for UI layer
  return (Array.isArray(result) ? result : []).map((a: ServiceAsset) => ({
    id: Number(a.id),
    name: a.name,
    asset_type: a.asset_type,
    value: Number(a.value),
    // candid may return opt nat8; map missing to 0 sentinel
    decimals: (() => {
      const raw: unknown = (a as unknown) && (a as unknown as Record<string, unknown>)['decimals'];
      if (typeof raw === 'number') return raw as number;
      if (typeof raw === 'bigint') return Number(raw as bigint);
      return 0;
    })(),
    description: a.description,
    created_at: Number(a.created_at),
    updated_at: Number(a.updated_at),
  }));
}

export async function listHeirs(): Promise<Heir[]> {
  if (USE_MOCK) {
    return mockStore.heirs.map(h => ({
      id: h.id,
      name: h.name,
      relationship: h.relationship ?? '',
      email: h.email ?? '',
      phone: h.phone ?? '',
      address: h.address ?? '',
      created_at: h.created_at,
      updated_at: h.updated_at,
    } as Heir));
  }
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



export async function addAsset(asset: AssetInput): Promise<boolean> {
  // Build a defensive payload and always include `decimals` as a nat8 number.
  const payload = {
    name: asset.name,
    asset_type: asset.asset_type,
    // value is server-managed; frontend must not send it.
    description: asset.description,
    kind: asset.kind ?? undefined,
    nft_standard: asset.nft_standard ?? undefined,
    chain_wrapped: asset.chain_wrapped ?? undefined,
    token_canister: asset.token_canister ?? undefined,
    token_id: asset.token_id !== undefined && asset.token_id !== null ? BigInt(asset.token_id as number) : undefined,
    file_path: asset.file_path ?? undefined,
    holding_mode: asset.holding_mode ?? undefined,
  } as unknown as Parameters<Service['add_asset']>[0];
  console.log(`addAsset called: Payload=${payload}`);
  // Log the payload to help debug invalid argument errors reported by the canister
  // (will show in browser console when running dev server)
  try {
    // avoid logging BigInt directly across some browsers: stringify safely
    const safePayload: Record<string, unknown> = { ...payload, token_id: payload.token_id ? (payload.token_id as bigint).toString() : undefined };
    console.debug('[API] add_asset payload:', safePayload);
  } catch (e) {
    console.debug('[API] add_asset payload logging failed', e);
  }
  // Show the exact payload we will send to the canister (BigInt values stringified)
  try {
    console.debug('[API] add_asset serialized payload (sent):', safeSerialize(payload));
  } catch (e) {
    console.debug('[API] failed to serialize payload for debug log', e);
  }
  if (USE_MOCK) {
    const id = genId();
    mockStore.assets.push({
      id,
      name: payload.name,
      asset_type: payload.asset_type,
      value: 1000 + Math.floor(Math.random() * 10000),
      description: (payload.description as string) || '',
      created_at: Date.now(),
      updated_at: Date.now(),
      decimals: 6,
      kind: payload.kind as string | undefined,
      token_canister: payload.token_canister as string | null,
      token_id: payload.token_id ? Number(payload.token_id as bigint) : undefined,
      file_path: payload.file_path as string | null,
      holding_mode: payload.holding_mode as string | undefined,
      nft_standard: payload.nft_standard as string | undefined,
      chain_wrapped: payload.chain_wrapped as string | undefined,
      approval_required: false,
    });
    mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: `Asset added: ${payload.name}` });
    return true;
  }
  const result = await withRetry(() => actor.add_asset(payload));
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
  const payload = {
    name: asset.name,
    asset_type: asset.asset_type,
    description: asset.description,
    kind: asset.kind ?? undefined,
    nft_standard: asset.nft_standard ?? undefined,
    chain_wrapped: asset.chain_wrapped ?? undefined,
    token_canister: asset.token_canister ?? undefined,
    token_id: asset.token_id !== undefined && asset.token_id !== null ? BigInt(asset.token_id as number) : undefined,
    file_path: asset.file_path ?? undefined,
    holding_mode: asset.holding_mode ?? undefined,
  } as unknown as Parameters<Service['update_asset']>[1];
  try {
    const safePayload: Record<string, unknown> = { ...payload, token_id: payload.token_id ? (payload.token_id as bigint).toString() : undefined };
    console.debug('[API] update_asset payload:', BigInt(id).toString(), safePayload);
  } catch (e) {
    console.debug('[API] update_asset payload logging failed', e);
  }
  if (USE_MOCK) {
    const idx = mockStore.assets.findIndex(a => a.id === id);
    if (idx === -1) return false;
    const existing = mockStore.assets[idx];
    mockStore.assets[idx] = { ...existing, ...payload, updated_at: Date.now() } as MockAsset;
    mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: `Asset updated: ${payload.name ?? existing.name}` });
    return true;
  }
  const result = await withRetry(() => actor.update_asset(BigInt(id), payload));
  return 'Ok' in result;
}

export async function removeAsset(id: number): Promise<boolean> {
  if (USE_MOCK) {
    const idx = mockStore.assets.findIndex(a => a.id === id);
    if (idx === -1) return false;
    mockStore.assets.splice(idx, 1);
    mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: `Asset removed: ${id}` });
    return true;
  }
  const result = await withRetry(() => actor.remove_asset(BigInt(id)));
  return 'Ok' in result;
}

export async function addHeir(heir: HeirInput): Promise<boolean> {
  if (USE_MOCK) {
    const id = genId();
    mockStore.heirs.push({ id, name: heir.name, relationship: heir.relationship ?? '', email: heir.email ?? '', phone: heir.phone ?? '', address: heir.address ?? '', salt: heir.salt ?? undefined, adhaarnum: heir.adhaarnum ?? undefined, created_at: Date.now(), updated_at: Date.now() });
    mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: `Heir added: ${heir.name}` });
    return true;
  }
  const result = await withRetry(() => actor.add_heir(heir));
  return 'Ok' in result;
}

export async function updateHeir(id: number, heir: HeirInput): Promise<boolean> {
  if (USE_MOCK) {
    const idx = mockStore.heirs.findIndex(h => h.id === id);
    if (idx === -1) return false;
    mockStore.heirs[idx] = { ...mockStore.heirs[idx], ...heir, updated_at: Date.now() } as MockHeir;
    mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: `Heir updated: ${heir.name}` });
    return true;
  }
  const result = await withRetry(() => actor.update_heir(BigInt(id), heir));
  return 'Ok' in result;
}

export async function removeHeir(id: number): Promise<boolean> {
  if (USE_MOCK) {
    const idx = mockStore.heirs.findIndex(h => h.id === id);
    if (idx === -1) return false;
    mockStore.heirs.splice(idx, 1);
    mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: `Heir removed: ${id}` });
    return true;
  }
  const result = await withRetry(() => actor.remove_heir(BigInt(id)));
  return 'Ok' in result;
}

export async function assignDistributions(distributions: AssetDistribution[]): Promise<boolean> {
  console.log(`assignDistributions called: count=${distributions.length}`);
  type IncomingDistribution = { asset_id?: number; assetId?: number; heir_id?: number; heirId?: number; percentage: number };
  const payload: ServiceDistribution[] = (distributions as IncomingDistribution[]).map((d) => ({
    asset_id: BigInt((d.asset_id ?? d.assetId) ?? 0),
    heir_id: BigInt((d.heir_id ?? d.heirId) ?? 0),
    percentage: Number(d.percentage),
  }));
  if (USE_MOCK) {
      // Merge distributions per-asset: keep other assets' distributions intact.
      // Payload contains entries for potentially multiple assets; group by asset_id.
      const byAsset = new Map<number, ServiceDistribution[]>();
      for (const p of payload) {
        const aid = Number(p.asset_id);
        const list = byAsset.get(aid) ?? [];
        list.push(p);
        byAsset.set(aid, list);
      }
      // Remove existing entries for assets present in incoming payload and replace with new ones
      mockStore.distributions = mockStore.distributions.filter(d => !byAsset.has(Number(d.asset_id)));
      for (const [aid, rows] of byAsset.entries()) {
        for (const r of rows) mockStore.distributions.push(r);
      }
      mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: `Distributions assigned (${payload.length})` });
      return true;
  }
  const result = await withRetry(() => actor.assign_distributions(payload));
  console.log(`assignDistributions result: ok=${'Ok' in result}`);
  return 'Ok' in result;
}

export async function timerStatus(): Promise<Timer> {
  if (USE_MOCK) {
    return mockStore.timerExpiry as unknown as Timer;
  }
  const res = await withRetry(() => actor.get_timer());
  // Backend returns int (i64) as bigint; convert to number for UI
  const value = Number(res);
  console.log(`Timer value: ${value}`);
  return value;
}

export async function resetTimer(): Promise<boolean> {
  if (USE_MOCK) {
    mockStore.timerExpiry = 3600 * 24 * 7; // 1 week
    mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: 'Timer reset (mock)' });
    return true;
  }
  const result: ServiceResult = await withRetry(() => actor.reset_timer());
  return 'Ok' in result;
}

// Fetch integrity report and map bigint arrays to numbers for convenience
// Provide defensive conversion to avoid runtime errors if unexpected types slip through.
function safeBigIntToNumber(val: unknown, label: string): number {
  if (typeof val === 'number') return val;
  if (typeof val === 'bigint') {
    // Clamp if exceeds JS safe integer
    const max = BigInt(Number.MAX_SAFE_INTEGER);
    if (val > max) {
      console.warn(`[Integrity] ${label} value ${val.toString()} exceeds MAX_SAFE_INTEGER; clamping.`);
      return Number.MAX_SAFE_INTEGER;
    }
    return Number(val);
  }
  // Attempt parse if string
  if (typeof val === 'string') {
    try {
      const asBig = BigInt(val);
      return safeBigIntToNumber(asBig, label);
    } catch {
      console.warn(`[Integrity] Could not parse string for ${label}:`, val);
      return 0;
    }
  }
  console.warn(`[Integrity] Unexpected type for ${label}:`, typeof val, val);
  return 0;
}

type IntegrityNumericContainer = bigint[] | BigUint64Array | BigInt64Array;
interface RawIntegrityLike {
  asset_count: bigint | number | string;
  distribution_count: bigint | number | string;
  over_allocated_assets: IntegrityNumericContainer | unknown;
  fully_allocated_assets: IntegrityNumericContainer | unknown;
  partially_allocated_assets: IntegrityNumericContainer | unknown;
  unallocated_assets: IntegrityNumericContainer | unknown;
  issues: string[] | unknown;
}

export async function checkIntegrity(): Promise<{
  assetCount: number;
  distributionCount: number;
  overAllocated: number[];
  fullyAllocated: number[];
  partiallyAllocated: number[];
  unallocated: number[];
  issues: string[];
}> {
  let raw: IntegrityReport;
  try {
    if (USE_MOCK) {
      // Synthesize a simple integrity report from mockStore
      raw = {
        asset_count: BigInt(mockStore.assets.length),
        distribution_count: BigInt(mockStore.distributions.length),
        over_allocated_assets: [],
        fully_allocated_assets: [],
        partially_allocated_assets: [],
        unallocated_assets: mockStore.assets.map(a => BigInt(a.id)),
        issues: [],
      } as unknown as IntegrityReport;
    } else {
      raw = await withRetry(() => actor.check_integrity());
    }
  } catch (e) {
    console.error('[Integrity] fetch failed', e);
    throw e;
  }

  const rawLike: RawIntegrityLike = raw as unknown as RawIntegrityLike;

  const mapIds = (arr: unknown): number[] => {
    if (Array.isArray(arr)) {
      return arr.map((v, i) => safeBigIntToNumber(v, `id[${i}]`));
    }
    // Handle typed arrays produced by candid (BigUint64Array / BigInt64Array)
    if (arr instanceof BigUint64Array || arr instanceof BigInt64Array) {
      const out: number[] = [];
      let idx = 0;
      for (const v of arr as Iterable<bigint>) {
        out.push(safeBigIntToNumber(v, `id[${idx}]`));
        idx++;
      }
      return out;
    }
    // Fallback: generic iterable detection
    if (arr && typeof (arr as { length?: unknown }) === 'object' && Symbol.iterator in (arr as object)) {
      try {
        const pseudo = Array.from(arr as Iterable<unknown>);
        return pseudo.map((v, i) => safeBigIntToNumber(v, `id[${i}]`));
      } catch { /* ignore */ }
    }
    console.warn('[Integrity] Unexpected id list container', arr);
    return [];
  };

  const report = {
    assetCount: safeBigIntToNumber(rawLike.asset_count, 'asset_count'),
    distributionCount: safeBigIntToNumber(rawLike.distribution_count, 'distribution_count'),
    overAllocated: mapIds(rawLike.over_allocated_assets),
    fullyAllocated: mapIds(rawLike.fully_allocated_assets),
    partiallyAllocated: mapIds(rawLike.partially_allocated_assets),
    unallocated: mapIds(rawLike.unallocated_assets),
    issues: Array.isArray(rawLike.issues) ? rawLike.issues.slice() : [],
  };
  console.log('[Integrity] report', report);
  return report;
}

/**
 * Fetch the current user object, including last_timer_reset.
 * Returns null if user not initialized.
 */
export async function getUserWithTimer(): Promise<{
  user: string;
  last_timer_reset: number | null;
} | null> {
  if (!actor.get_user) return null;
  if (USE_MOCK) {
    return { user: 'demo', last_timer_reset: Date.now() - 3600 * 24 };
  }
  const res = await withRetry(() => actor.get_user!());
  if (!res) return null;
  // Defensive: handle both candid and JS types
  const userObj = res as { user?: string; last_timer_reset?: number | bigint };
  const last = userObj.last_timer_reset;
  return {
    user: userObj.user ?? "",
    last_timer_reset: typeof last === "bigint" ? Number(last) : typeof last === "number" ? last : null,
  };
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

// Helper to produce a safe, serializable copy of objects that may contain BigInt
function safeSerialize(val: unknown): unknown {
  if (val === null || val === undefined) return val;
  if (typeof val === 'bigint') return val.toString();
  if (Array.isArray(val)) return val.map(safeSerialize);
  if (typeof val === 'object') {
    const out: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(val as Record<string, unknown>)) {
      out[k] = safeSerialize(v);
    }
    return out;
  }
  return val;
}

/**
 * Demo helper: create a claim code for a given heir and asset list (mock only).
 * Returns the generated claim code string.
 */
export function createDemoClaim(heirId: number, assets: number[]) {
  if (!USE_MOCK) {
    throw new Error("createDemoClaim is only available in mock mode");
  }
  // generate a short claim code
  const code = `CLAIM-${Math.random().toString(36).slice(2, 8).toUpperCase()}-${Date.now().toString().slice(-4)}`;
  mockStore.claims.push({ code, heirId, assets: assets.slice() });
  mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: `Demo claim created: ${code} for heir ${heirId} assets=[${assets.join(',')}]` });
  return code;
}

/**
 * Execute estate now (mock): processes all assets and enqueues release operations
 * according to their holding_mode/approval flags. Produces audit events and
 * sets the timer to expired (0). This simulates the estate execution flow for demo.
 */
export async function executeEstateNow(): Promise<boolean> {
  if (!USE_MOCK) return false;
  try {
    // mark timer expired
    mockStore.timerExpiry = 0;
    mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: 'Estate execution started (manual trigger)' });

    for (const asset of mockStore.assets) {
      // compute distributions for this asset
      const dists = mockStore.distributions.filter(d => Number(d.asset_id) === asset.id);
      if (!dists || dists.length === 0) {
        mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: `No distributions for asset ${asset.id} (${asset.name})` });
        continue;
      }
      // For each heir distribution, enqueue an op depending on asset holding mode
      for (const d of dists) {
        const heirId = Number(d.heir_id);
        const heir = mockStore.heirs.find(h => h.id === heirId);
        const opPayload = { assetId: asset.id, heirId, percent: d.percentage };
        // decide op kind by checking approval_required flag or holding_mode
        if (asset.approval_required) {
          enqueueRetry('icrc_transfer', { ...opPayload, method: 'icrc_transfer' });
          mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: `Scheduled ICRC transfer for asset ${asset.id} -> heir ${heirId}` });
        } else if (asset.holding_mode === 'custody' || asset.holding_mode === 'Custody') {
          enqueueRetry('custody_release', { ...opPayload });
          mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: `Scheduled custody release for asset ${asset.id} -> heir ${heirId}` });
        } else {
          // default to direct transfer
          enqueueRetry('direct_transfer', { ...opPayload });
          mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: `Scheduled direct transfer for asset ${asset.id} -> heir ${heirId}` });
        }
      }
    }

    mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: 'Estate execution enqueued operations' });
    return true;
  } catch (e) {
    mockStore.auditLog.push({ id: genId(), ts: Date.now(), msg: `Estate execution failed: ${String(e)}` });
    return false;
  }
}
