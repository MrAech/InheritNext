import React from "react";
import { useAuth } from "@/context/AuthContext";
import { Button } from "@/components/ui/button";
// Simple inline avatar fallback

const Header = () => {
  const { isAuthenticated, identity, login, logout } = useAuth() as any;
  const principal = identity
    ? identity.getPrincipal
      ? identity.getPrincipal().toText()
      : undefined
    : undefined;

  return (
    <div className="flex items-center space-x-4">
      <div className="flex items-center gap-3">
        <div className="w-8 h-8 rounded-full bg-primary/20 flex items-center justify-center text-sm font-semibold">
          {principal ? principal.slice(0, 2).toUpperCase() : "AN"}
        </div>
        <div>
          <div className="text-sm font-semibold">
            {principal ? principal.slice(0, 8) + "..." : "Anonymous"}
          </div>
          <div className="text-xs text-muted-foreground">
            {isAuthenticated ? "Signed in" : "Guest"}
          </div>
        </div>
      </div>
      <div className="ml-4">
        {!isAuthenticated ? (
          <Button size="sm" onClick={() => login()}>
            Sign In
          </Button>
        ) : (
          <Button size="sm" variant="ghost" onClick={() => logout()}>
            Sign Out
          </Button>
        )}
      </div>
    </div>
  );
};

export default Header;
