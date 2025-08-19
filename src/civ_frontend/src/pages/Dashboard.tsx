import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { AssetsList } from "@/components/AssetsList";
import { HeirsList } from "@/components/HeirsList";
import { TimerResetDialog } from "@/components/TimerResetDialog";
import DistributionsManager from "@/components/DistributionsManager";
import {
  DollarSign,
  TrendingUp,
  Users,
  Clock,
  LogOut,
  RefreshCw,
  PieChart,
  Wallet
} from "lucide-react";
import { useToast } from "@/hooks/use-toast";
import { useAuth } from "@/context/useAuth";
import type { Asset, Heir, AssetDistribution as BackendDistribution } from "@/types/backend";
import { SettingsDialog } from "@/components/SettingsDialog";
import { useSettings } from "@/context/SettingsContext";

import { timerStatus, resetTimer } from "@/lib/api";
import IntegritySummary from "@/components/IntegritySummary";
import { Link } from "react-router-dom";

const Dashboard = () => {
  const { logout, identity } = useAuth();
  const [totalAssets, setTotalAssets] = useState(2850000);
  const [timerResetOpen, setTimerResetOpen] = useState(false);
  const [lastReset, setLastReset] = useState<Date | null>(null);
  const [timeRemaining, setTimeRemaining] = useState<number | null>(null);
  const [timerStatusLabel, setTimerStatusLabel] = useState<"not_started" | "running" | "expired">("not_started");
  const [timerLoading, setTimerLoading] = useState(false);
  const [timerError, setTimerError] = useState<string | null>(null);
  const [assets, setAssets] = useState<Asset[]>([]);
  const [heirs, setHeirs] = useState<Heir[]>([]);
  const [distributionWarning, setDistributionWarning] = useState<string | null>(null);
  const [initialLoading, setInitialLoading] = useState(true);
  const navigate = useNavigate();
  const { toast } = useToast();
  const { formatCurrency } = useSettings();


  // Initial data fetch for assets, heirs, and last reset when component mounts or identity changes
  useEffect(() => {
    let cancelled = false;
    setInitialLoading(true);
    (async () => {
      try {
        const api = await import("@/lib/api");
        const [a, h, user] = await Promise.all([
          api.listAssets(),
          api.listHeirs(),
          api.getUserWithTimer ? api.getUserWithTimer() : null,
        ]);
        if (!cancelled) {
          setAssets(a);
          setHeirs(h);
          if (user && user.last_timer_reset && user.last_timer_reset > 0) {
            setLastReset(new Date(user.last_timer_reset * 1000));
          } else {
            setLastReset(null);
          }
        }
      } catch (e) {
        if (!cancelled) console.warn("[Dashboard] initial fetch failed", e);
      } finally {
        if (!cancelled) setInitialLoading(false);
      }
    })();
    return () => { cancelled = true; };
  }, [identity]);

  useEffect(() => {
    let prevStatus: "not_started" | "running" | "expired" = timerStatusLabel;
    let lastBackendTimer: number | null = null;

    const fetchTimer = async () => {
      setTimerLoading(true);
      setTimerError(null);
      try {
        const timer = await timerStatus();
        lastBackendTimer = timer;
        if (assets.length === 0 || timer === -1) {
          setTimeRemaining(null);
          setTimerStatusLabel("not_started");
          setDistributionWarning(null);
        } else if (timer === 0) {
          setTimeRemaining(0);
          setTimerStatusLabel("expired");
          setDistributionWarning("Timer expired! Assets will be auto-distributed.");
          if (prevStatus !== "expired") {
            toast({
              title: "Timer Expired",
              description: "Timer expired! Assets have been auto-distributed (placeholder logic).",
              variant: "destructive",
            });
          }
        } else {
          setTimeRemaining(timer);
          setTimerStatusLabel("running");
          setDistributionWarning(null);
        }
        prevStatus = timer === 0 ? "expired" : timer === -1 ? "not_started" : "running";
      } catch (err: unknown) {
        setTimerError("Failed to fetch timer status.");
      } finally {
        setTimerLoading(false);
      }
    };

    fetchTimer();
    const interval = setInterval(fetchTimer, 60000);

    // Real-time countdown with drift correction
    let driftCounter = 0;
    const countdownInterval = setInterval(() => {
      setTimeRemaining(prev => {
        if (typeof prev === "number" && prev > 0) {
          const next = prev - 1;
          // Every 10 seconds, check for drift with backend
          driftCounter++;
          if (driftCounter % 10 === 0 && lastBackendTimer !== null) {
            const expected = lastBackendTimer - driftCounter;
            if (Math.abs(next - expected) > 2) {
              // Drift detected, force backend sync
              fetchTimer();
              driftCounter = 0;
              return expected > 0 ? expected : 0;
            }
          }
          return next;
        }
        return prev;
      });
    }, 1000);

    return () => {
      clearInterval(interval);
      clearInterval(countdownInterval);
    };
  }, [assets, toast]);

  const handleSignOut = async () => {
    await logout();
    toast({
      title: "Signed out",
      description: "You have been successfully signed out.",
    });
    navigate("/");
  };

  const handleTimerReset = async () => {
    setTimerLoading(true);
    setTimerError(null);
    try {
      const ok = await resetTimer();
      if (ok) {
        const timer = await timerStatus();
        setTimeRemaining(typeof timer === "number" && timer >= 0 ? timer : 0);
        try {
          const api = await import("@/lib/api");
          const [assetsData, heirsData, user] = await Promise.all([
            api.listAssets(),
            api.listHeirs(),
            api.getUserWithTimer ? api.getUserWithTimer() : null,
          ]);
          setAssets(await assetsData);
          setHeirs(await heirsData);
          if (user && user.last_timer_reset && user.last_timer_reset > 0) {
            setLastReset(new Date(user.last_timer_reset * 1000));
          } else {
            setLastReset(null);
          }
        } catch (err) {
          toast({
            title: "Reload Error",
            description: "Failed to reload assets/heirs after reset.",
            variant: "destructive",
          });
        }
        setDistributionWarning(null);
        toast({
          title: "Timer Reset",
          description: "Dashboard timer has been successfully reset.",
        });
      } else {
        setTimerError("Failed to reset timer.");
        toast({
          title: "Timer Error",
          description: "Failed to reset timer.",
          variant: "destructive",
        });
      }
    } catch (err: unknown) {
      setTimerError("Failed to reset timer.");
      toast({
        title: "Timer Error",
        description: "Failed to reset timer.",
        variant: "destructive",
      });
    } finally {
      setTimerLoading(false);
      setTimerResetOpen(false);
    }
  };

  // NOTE: kept for fallback sake well for when i brick it 

  // const formatCurrency = (amount: number) => {
  //   return new Intl.NumberFormat('en-US', {
  //     style: 'currency',
  //     currency: 'USD',
  //     minimumFractionDigits: 0,
  //     maximumFractionDigits: 0,
  //   }).format(amount);
  // };



  //   TODO: test with backend 
  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b bg-card shadow-card">
        <div className="container mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-4">
              <div className="w-20 sm:w-28 md:w-32 lg:w-56 flex flex-col items-center justify-center shrink-0">
                <img
                  src="/favicons/internext.png"
                  alt="InheritNext Logo"
                  className="w-full h-auto object-contain max-w-[80%] max-h-[75%]"
                />
              </div>
              {/* <div>
                <h1 className="text-2xl font-bold text-foreground">InheritNext</h1>
                <p className="text-sm text-muted-foreground">Inheritance Management System</p>
              </div> */}
            </div>
            <div className="flex items-center space-x-4">
              <SettingsDialog />
              <Link to="/distributions">
                <Button variant="outline" size="sm" className="hidden sm:flex">
                  <PieChart className="w-4 h-4 mr-2" />
                  Distributions
                </Button>
              </Link>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setTimerResetOpen(true)}
                className="hidden sm:flex"
              >
                <RefreshCw className="w-4 h-4 mr-2" />
                Reset Timer
              </Button>
              <Button variant="outline" size="sm" onClick={handleSignOut}>
                <LogOut className="w-4 h-4 mr-2" />
                Sign Out
              </Button>
            </div>
          </div>
        </div>
      </header>

      <main className="container mx-auto px-4 py-8">
        {/* Timer Status */}
        <div className="mb-8">
          <Card
            className={
              timerStatusLabel === "expired"
                ? "bg-gradient-to-r from-red-500 to-red-700 text-white border-0"
                : timerStatusLabel === "running"
                ? "bg-gradient-to-r from-green-500 to-blue-500 text-white border-0"
                : "bg-gradient-to-r from-gray-400 to-gray-600 text-white border-0"
            }
          >
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-4">
                  <Clock className="w-8 h-8" />
                  <div>
                    <h3 className="text-lg font-semibold">Inheritance Access Timer</h3>
                    {timerStatusLabel === "not_started" && (
                      <>
                        <span className="inline-block px-2 py-1 rounded bg-gray-700 text-xs font-bold mr-2">Not Started</span>
                        <p className="text-primary-foreground/80">
                          Timer will start automatically when you add your first distribution.
                        </p>
                      </>
                    )}
                    {timerStatusLabel === "running" && (
                      <>
                        <span className="inline-block px-2 py-1 rounded bg-green-700 text-xs font-bold mr-2">Running</span>
                        <p className="text-primary-foreground/80">
                          Time remaining: {formatDuration(timeRemaining)}
                        </p>
                      </>
                    )}
                    {timerStatusLabel === "expired" && (
                      <>
                        <span className="inline-block px-2 py-1 rounded bg-red-700 text-xs font-bold mr-2">Expired</span>
                        <p className="text-primary-foreground/80">
                          Timer expired! Assets will be auto-distributed.
                        </p>
                      </>
                    )}
                    <p className="text-xs text-primary-foreground/60">
                      Last Reset: {lastReset ? lastReset.toLocaleString() : "N/A"}
                    </p>
                  </div>
          
                  {/* Timer Expiry Notification History (placeholder) */}
                  <div className="mb-8">
                    <Card className="shadow-card">
                      <CardHeader>
                        <CardTitle className="text-sm font-medium flex items-center gap-2">
                          <Clock className="h-4 w-4" />
                          Timer Expiry Notifications
                        </CardTitle>
                        <CardDescription>
                          This is a placeholder for persistent notification history. In the future, timer expiry and auto-distribution events will be listed here.
                        </CardDescription>
                      </CardHeader>
                      <CardContent>
                        <div className="text-muted-foreground text-sm">
                          No timer expiry events recorded yet.
                        </div>
                      </CardContent>
                    </Card>
                  </div>
                </div>
                <Button
                  variant="secondary"
                  onClick={() => setTimerResetOpen(true)}
                  className="sm:hidden"
                >
                  <RefreshCw className="w-4 h-4" />
                </Button>
              </div>
              <Button
                variant="outline"
                size="sm"
                onClick={() => {
                  setTimerLoading(true);
                  import("@/lib/api").then(api =>
                    api.timerStatus().then(timer => {
                      setTimeRemaining(timer);
                      setTimerLoading(false);
                    }).catch(() => setTimerLoading(false))
                  );
                }}
                className="ml-4"
                disabled={timerLoading}
              >
                Sync Timer
              </Button>
            </CardContent>
          </Card>
        </div>

        {/* Total Assets Overview */}
        <div className="grid gap-6 md:grid-cols-4 mb-8">
          <Card className="shadow-card">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Total Assets</CardTitle>
              <Wallet className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-primary">
                {formatCurrency(totalAssets)}
              </div>
              <p className="text-xs text-muted-foreground">
                Personal asset value
              </p>
            </CardContent>
          </Card>

          <Card className="shadow-card">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Total Asset</CardTitle>
              <TrendingUp className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{assets.length}</div>
              {/* <p className="text-xs text-muted-foreground">
                Total asset items
              </p> */}
            </CardContent>
          </Card>

          <Card className="shadow-card">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Total Heirs</CardTitle>
              <Users className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{heirs.length}</div>
              <p className="text-xs text-muted-foreground">
                Heirs
              </p>
            </CardContent>
          </Card>

          <IntegritySummary />
        </div>

        {/* Assets Section */}
        <div className="mb-8">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-xl font-semibold flex items-center gap-2">
              <DollarSign className="w-5 h-5 text-primary" />
              Personal Assets
            </h2>
            <Badge variant="secondary">Updated 2 hours ago</Badge>
          </div>
          {initialLoading ? (
            <div className="text-center text-muted-foreground py-8">Loading assets...</div>
          ) : (
            <AssetsList
              onTotalChange={setTotalAssets}
              onAssetsChange={setAssets}
              onAssetAdded={async () => {
                console.log("onAssetAdded callback triggered");
                setTimerLoading(true);
                try {
                  // Fetch latest assets before checking timer
                  const assetsData = await import("@/lib/api").then(m => m.listAssets());
                  setAssets(await assetsData);
                  let timer = await timerStatus();
                  if (typeof timer === "bigint") {
                    timer = Number(timer);
                  }
                  console.log("Timer value after asset added:", timer);
                  if ((assetsData.length === 0) || timer === -1) {
                    setTimeRemaining(null);
                    setDistributionWarning(null);
                  } else {
                    setTimeRemaining(typeof timer === "number" && timer >= 0 ? timer : 0);
                    if (timer === 0) {
                      setDistributionWarning("Timer expired! Assets will be auto-distributed.");
                    } else {
                      setDistributionWarning(null);
                    }
                  }
                } catch {
                  setTimerError("Failed to fetch timer status.");
                } finally {
                  setTimerLoading(false);
                }
              }}
            />
          )}
        </div>

        {/* Heirs Section */}
        <div>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-xl font-semibold flex items-center gap-2">
              <Users className="w-5 h-5 text-primary" />
              Heirs & Asset Distribution
            </h2>
            <Badge variant="secondary">{heirs.length} Active Heirs</Badge>
          </div>
          {initialLoading ? (
            <div className="text-center text-muted-foreground py-8">Loading heirs...</div>
          ) : (
            <HeirsList onHeirsChange={setHeirs} />
          )}
        </div>

        {/* Distributions Manager */}
        {assets.length > 0 && heirs.length > 0 && (
          <div className="mb-8">
            <DistributionsManager assets={assets} heirs={heirs} />
          </div>
        )}
      </main>

      <TimerResetDialog
        open={timerResetOpen}
        onOpenChange={setTimerResetOpen}
        onConfirm={handleTimerReset}
      />
    </div>
  );
};

function formatDuration(seconds: number | null): string {
  if (seconds == null || seconds < 0) return "N/A";
  if (seconds === 0) return "Expired";
  const d = Math.floor(seconds / 86400);
  const h = Math.floor((seconds % 86400) / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;
  const parts = [];
  if (d > 0) parts.push(`${d}d`);
  if (h > 0 || d > 0) parts.push(`${h}h`);
  if (m > 0 || h > 0 || d > 0) parts.push(`${m}m`);
  parts.push(`${s}s`);
  return parts.join(" ");
}

/*
  TODO: Add tests for timer UI and logic (placeholder for unit/integration tests)
*/

export default Dashboard;
