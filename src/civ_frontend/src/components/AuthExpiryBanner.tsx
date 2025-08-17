import React from "react";
import { useAuth } from "@/context/useAuth";
import { Button } from "@/components/ui/button";

const AuthExpiryBanner: React.FC = () => {
  const { sessionInvalid, sessionError, attemptSilentRefresh, login, logout, silentRefreshing } = useAuth();
  if (!sessionInvalid) return null;
  return (
    <div className="fixed top-0 left-0 right-0 z-50 bg-red-600 text-white text-sm px-4 py-2 flex flex-col gap-1 md:flex-row md:items-center md:justify-between shadow">
      <div className="font-medium">
        Session expired / invalid delegation. {silentRefreshing ? "Attempting silent refresh..." : "Action required."}
      </div>
      {sessionError && <div className="opacity-80 max-w-[50ch] truncate" title={sessionError}>{sessionError}</div>}
      <div className="flex gap-2 mt-1 md:mt-0">
        <Button size="sm" variant="secondary" disabled={silentRefreshing} onClick={() => attemptSilentRefresh()}>Retry</Button>
        <Button size="sm" variant="outline" className="bg-white/10 hover:bg-white/20" onClick={() => login()}>Re-Login</Button>
        <Button size="sm" variant="destructive" onClick={() => logout()}>Logout</Button>
      </div>
    </div>
  );
};

export default AuthExpiryBanner;
