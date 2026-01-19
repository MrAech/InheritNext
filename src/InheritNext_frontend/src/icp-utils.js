import { HttpAgent } from "@icp-sdk/core/agent";
import { createActor } from "./icp-declarations/InheritNext_backend/InheritNext_backend";

export const createBackendActor = async (identity) => {
  const network = process.env.DFX_NETWORK || "local";
  const isLocal = network !== "ic";

  // If running locally (port 4943) or on mainnet (ic0.app), we can use relative paths.
  // If running via Vite (port 3000), we must proxy or point to localhost:4943.
  let host;
  if (isLocal) {
    // If we are ALREADY on the replica URL, leave host undefined to use relative paths.
    // If we are on Vite , force the replica URL.
    const isServedFromReplica =
      window.location.port === "4943" || window.location.port === "8000";
    host = isServedFromReplica ? undefined : "http://127.0.0.1:4943";
  } else {
    host = "https://icp-api.io";
  }

  const agent = await HttpAgent.create({
    identity,
    host,
  });

  if (isLocal) {
    try {
      await agent.fetchRootKey();
    } catch (err) {
      console.warn(
        'Unable to fetch root key. Check to ensure "dfx start" is running',
      );
      console.error(err);
    }
  }

  const canisterId = process.env.CANISTER_ID_INHERITNEXT_BACKEND;

  return createActor(canisterId, {
    agent,
  });
};

export const calculateDmsStatus = (vault) => {
  if (!vault) return null;

  const last = Number(vault.dms.last_heartbeat / 1000000n);
  const interval = Number(vault.dms.heartbeat_interval / 1000000n);
  const nextDue = last + interval;
  const now = Date.now();
  const diff = nextDue - now;
  const daysRemaining = Math.ceil(diff / (1000 * 60 * 60 * 24));

  return {
    nextDueDate: new Date(nextDue).toLocaleDateString(),
    daysRemaining,
    isOverdue: diff < 0,
  };
};
