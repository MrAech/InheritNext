import React, { createContext, useEffect, useState } from "react";
import { AuthClient } from "@dfinity/auth-client";
import { Actor, ActorSubclass, Identity } from "@dfinity/agent";
import { _SERVICE } from "@/../../declarations/civ_backend/civ_backend.did";
import { createActor, canisterId as civBackendCanisterId } from "@/../../declarations/civ_backend";
import { resetTimer, setApiIdentity, getLastRootKeyHash } from "@/lib/api";
import { validateBackendSession } from "@/lib/authHealth";
import { setDistributionsIdentity } from "@/lib/distribution";
import { canisterId as internetIdentityCanisterId } from "@/../../declarations/internet_identity";

type EnvMap = { [k: string]: string | undefined };

const network = process.env.DFX_NETWORK || "local";
const canisterId = process.env.CANISTER_ID_CIV_BACKEND || process.env.CANISTER_ID || process.env.VITE_CANISTER_ID_CIV_BACKEND || civBackendCanisterId;
if (!canisterId) {
  console.error("[Auth] civ_backend canisterId is undefined. Checked process.env (CANISTER_ID_CIV_BACKEND, CANISTER_ID) and generated declarations.");
  throw new Error("civ_backend canisterId is undefined");
}
const iiCanisterId = process.env.VITE_CANISTER_ID_INTERNET_IDENTITY || process.env.CANISTER_ID_INTERNET_IDENTITY || internetIdentityCanisterId;

interface AuthContextType {
  authClient: AuthClient | null;
  isAuthenticated: boolean;
  identity: Identity | null;
  actor: ActorSubclass<_SERVICE> | null;
  login: () => Promise<void>;
  logout: () => Promise<void>;
  sessionInvalid: boolean;
  sessionError: string | null;
  silentRefreshing: boolean;
  attemptSilentRefresh: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | null>(null);

export const AuthProvider = ({ children }: { children: React.ReactNode }) => {
  const [authClient, setAuthClient] = useState<AuthClient | null>(null);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [identity, setIdentity] = useState<Identity | null>(null);
  const [actor, setActor] = useState<ActorSubclass<_SERVICE> | null>(null);
  const [sessionInvalid, setSessionInvalid] = useState(false);
  const [sessionError, setSessionError] = useState<string | null>(null);
  const [silentRefreshing, setSilentRefreshing] = useState(false);

  useEffect(() => {
    AuthClient.create().then(async (client) => {
      setAuthClient(client);
      const isAuthenticated = await client.isAuthenticated();
      setIsAuthenticated(isAuthenticated);
      if (isAuthenticated) {
        const identity = client.getIdentity();
        setIdentity(identity);
        const actor = createActor(canisterId, { agentOptions: { identity } });
        setActor(actor);
        setApiIdentity(identity);
        setDistributionsIdentity(identity);
        validateBackendSession().then(h => {
          if (!h.ok && h.needsRelogin) {
            console.warn("[Auth] Detected invalid delegation on load");
            setSessionInvalid(true);
            setSessionError(h.error || 'Invalid delegation');
          }
        }).catch(() => { /* ignore */ });
        // Root key fingerprint logging (local dev only)
        try {
          const rk = getLastRootKeyHash();
          if (rk) {
            const prev = sessionStorage.getItem('__IC_ROOT_KEY_HASH_PREV');
            if (prev && prev !== rk) {
              console.warn('[Auth] Root key changed since last session; forcing delegation refresh may be required');
            }
            sessionStorage.setItem('__IC_ROOT_KEY_HASH_PREV', rk);
          }
        } catch { /* ignore */ }
      }
    });
  }, []);

  // Window focus revalidation & root key change detection
  useEffect(() => {
    const onFocus = async () => {
      if (!authClient) return;
      if (await authClient.isAuthenticated()) {
        try {
          const health = await validateBackendSession();
          if (!health.ok && health.needsRelogin) {
            console.warn('[Auth] Focus validation detected invalid delegation');
            setSessionInvalid(true);
            setSessionError(health.error || 'Invalid delegation');
          }
          const rk = getLastRootKeyHash();
          const stored = sessionStorage.getItem('__IC_ROOT_KEY_HASH_PREV');
          if (rk && stored && rk !== stored) {
            console.warn('[Auth] Root key changed during session; marking session invalid');
            setSessionInvalid(true);
            setSessionError('Replica root key changed. Please re-login.');
          }
        } catch (e) {
          // ignore
        }
      }
    };
    window.addEventListener('focus', onFocus);
    return () => window.removeEventListener('focus', onFocus);
  }, [authClient]);

  const login = async () => {
    if (!authClient) return;
    const identityProvider =
      network === "ic"
        ? "https://identity.ic0.app"
        : (iiCanisterId ? `http://${iiCanisterId}.localhost:4943/` : "http://localhost:4943/");

    await authClient.login({
      identityProvider,
      onSuccess: async () => {
        const isAuthenticated = await authClient.isAuthenticated();
        setIsAuthenticated(isAuthenticated);
        if (isAuthenticated) {
          const identity = authClient.getIdentity();
          setIdentity(identity);
          const actor = createActor(canisterId, { agentOptions: { identity } });
          setActor(actor);
          setApiIdentity(identity);
          setDistributionsIdentity(identity);
          await resetTimer();
          const health = await validateBackendSession();
          if (!health.ok && health.needsRelogin) {
            console.warn("[Auth] Delegation invalid after login");
            setSessionInvalid(true);
            setSessionError(health.error || 'Invalid delegation');
          } else {
            setSessionInvalid(false);
            setSessionError(null);
          }
        }
      },
    });
  };

  const logout = async () => {
    if (!authClient) return;
    await authClient.logout();
    setIsAuthenticated(false);
    setIdentity(null);
    setActor(null);
    setApiIdentity(null);
    setDistributionsIdentity(null);
    setSessionInvalid(false);
    setSessionError(null);
  };

  const attemptSilentRefresh = async () => {
    if (!authClient || !isAuthenticated || silentRefreshing) return;
    setSilentRefreshing(true);
    try {
      const currentIdentity = authClient.getIdentity();
      setApiIdentity(currentIdentity);
      setDistributionsIdentity(currentIdentity);
      const health = await validateBackendSession();
      if (health.ok) {
        setSessionInvalid(false);
        setSessionError(null);
      } else if (!health.ok && !health.needsRelogin) {
        setSessionError(health.error || 'Unknown error');
      } else {
        setSessionError(health.error || 'Invalid delegation');
      }
    } catch (e) {
      setSessionError(String(e));
    } finally {
      setSilentRefreshing(false);
    }
  };

  return (
    <AuthContext.Provider
      value={{
        authClient,
        isAuthenticated,
        identity,
        actor,
        login,
        logout,
        sessionInvalid,
        sessionError,
        silentRefreshing,
        attemptSilentRefresh,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
};

export default AuthContext;
