
import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useAuth } from "@/context/useAuth";
import { validateBackendSession } from "@/lib/authHealth";

interface ProtectedRouteProps {
  children: React.ReactNode;
}

const ProtectedRoute = ({ children }: ProtectedRouteProps) => {
  const { isAuthenticated, authClient, attemptSilentRefresh } = useAuth();
  const navigate = useNavigate();
  const [checking, setChecking] = useState(false);
  const [checkedOnce, setCheckedOnce] = useState(false);

  useEffect(() => {
    if (authClient && !isAuthenticated) {
      navigate("/", { replace: true });
    }
  }, [isAuthenticated, authClient, navigate]);

  useEffect(() => {
    let cancelled = false;
    const run = async () => {
      if (!authClient || !isAuthenticated || checkedOnce) return;
      setChecking(true);
      const health = await validateBackendSession();
      if (cancelled) return;
      setChecking(false);
      setCheckedOnce(true);
      if (!health.ok && health.needsRelogin) {
        console.warn("[Auth] Invalid delegation detected; attempting silent refresh", health.error);
        await attemptSilentRefresh();
      }
    };
    void run();
    return () => { cancelled = true; };
  }, [authClient, isAuthenticated, checkedOnce, attemptSilentRefresh, navigate]);

  // Show loading state while checking authentication
  if (!authClient || checking) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4"></div>
          <p className="text-muted-foreground">{checking ? "Verifying session..." : "Initializing authentication..."}</p>
        </div>
      </div>
    );
  }

  // Only render children if authenticated
  return isAuthenticated ? <>{children}</> : null;
};

export default ProtectedRoute;
