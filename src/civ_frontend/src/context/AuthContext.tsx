import React, { createContext, useContext, useEffect, useState } from "react";
import { AuthClient } from "@dfinity/auth-client";
import { Actor, ActorSubclass, Identity } from "@dfinity/agent";
import { _SERVICE } from "@/../../declarations/civ_backend/civ_backend.did";
import { createActor } from "@/../../declarations/civ_backend";
import { resetTimer } from "@/lib/api";

const network = process.env.DFX_NETWORK || "local";
const canisterId = process.env.CANISTER_ID_CIV_BACKEND;

interface AuthContextType {
  authClient: AuthClient | null;
  isAuthenticated: boolean;
  identity: Identity | null;
  actor: ActorSubclass<_SERVICE> | null;
  login: () => Promise<void>;
  logout: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | null>(null);

export const AuthProvider = ({ children }: { children: React.ReactNode }) => {
  const [authClient, setAuthClient] = useState<AuthClient | null>(null);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [identity, setIdentity] = useState<Identity | null>(null);
  const [actor, setActor] = useState<ActorSubclass<_SERVICE> | null>(null);

  useEffect(() => {
    AuthClient.create().then(async (client) => {
      setAuthClient(client);
      const isAuthenticated = await client.isAuthenticated();
      setIsAuthenticated(isAuthenticated);
      if (isAuthenticated) {
        const identity = client.getIdentity();
        setIdentity(identity);
        const actor = createActor(canisterId, {
          agentOptions: { identity },
        });
        setActor(actor);
      }
    });
  }, []);

  const login = async () => {
    if (!authClient) return;
    const identityProvider =
      network === "ic"
        ? "https://identity.ic0.app"
        : `http://${process.env.CANISTER_ID_INTERNET_IDENTITY}.localhost:4943/`;

    await authClient.login({
      identityProvider,
      onSuccess: async () => {
        const isAuthenticated = await authClient.isAuthenticated();
        setIsAuthenticated(isAuthenticated);
        if (isAuthenticated) {
          const identity = authClient.getIdentity();
          setIdentity(identity);
          const actor = createActor(canisterId, {
            agentOptions: { identity },
          });
          setActor(actor);
          await resetTimer();
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
      }}
    >
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
};
