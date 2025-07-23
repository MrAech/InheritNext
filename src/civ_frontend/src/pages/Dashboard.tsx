import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { AssetsList } from "@/components/AssetsList";
import { HeirsList } from "@/components/HeirsList";
import { TimerResetDialog } from "@/components/TimerResetDialog";
import { AssetDistribution } from "@/components/AssetDistribution";
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
import { useAuth } from "@/context/AuthContext";
import { SettingsDialog } from "@/components/SettingsDialog";
import { useSettings } from "@/context/SettingsContext";

import { timerStatus, resetTimer, assignDistributions } from "@/lib/api";

const Dashboard = () => {
  const { logout } = useAuth();
  const [totalAssets, setTotalAssets] = useState(2850000);
  const [timerResetOpen, setTimerResetOpen] = useState(false);
  const [lastReset, setLastReset] = useState<Date | null>(null);
  const [timeRemaining, setTimeRemaining] = useState<string>("");
  const [timerLoading, setTimerLoading] = useState(false);
  const [timerError, setTimerError] = useState<string | null>(null);
  const [assets, setAssets] = useState<any[]>([]);
  const [heirs, setHeirs] = useState<any[]>([]);
  const [distributionWarning, setDistributionWarning] = useState<string | null>(null);
  const navigate = useNavigate();
  const { toast } = useToast();
  const { formatCurrency } = useSettings();

  // Fetch timer status from backend
  useEffect(() => {
    let interval: ReturnType<typeof setInterval>;
    const fetchTimer = async () => {
      setTimerLoading(true);
      setTimerError(null);
      try {
        const timer = await timerStatus();
        if (assets.length === 0 || timer === -1) {
          setTimeRemaining("");
          setDistributionWarning(null);
        } else {
          setTimeRemaining(timer ? `${timer} seconds` : "Expired");
          if (timer === 0) {
            setDistributionWarning("Timer expired! Assets will be auto-distributed.");
            try {
              for (const asset of assets) {
                const distributions = asset.distributions || [];
                if (distributions.length > 0) {
                  await assignDistributions(distributions);
                }
              }
              toast({
                title: "Auto Distribution",
                description: "Assets have been auto-distributed due to timer expiry.",
              });
            } catch (err) {
              toast({
                title: "Distribution Error",
                description: "Failed to auto-distribute assets.",
                variant: "destructive",
              });
            }
          } else {
            setDistributionWarning(null);
          }
        }
      } catch (err: any) {
        setTimerError("Failed to fetch timer status.");
      } finally {
        setTimerLoading(false);
      }
    };

    fetchTimer();
    interval = setInterval(fetchTimer, 60000);

    return () => clearInterval(interval);
  }, [assets]);

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
        setTimeRemaining(timer ? `${timer} seconds` : "Expired");
        // Reload assets and heirs from backend after reset
        try {
          const assetsData = await import("@/lib/api").then(m => m.listAssets());
          setAssets(await assetsData);
          const heirsData = await import("@/lib/api").then(m => m.listHeirs());
          setHeirs(await heirsData);
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
    } catch (err: any) {
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

  // NOTE: kept for fallback sake

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
          {timeRemaining === "" ? (
            <Card className="bg-gradient-primary text-primary-foreground border-0">
              <CardContent className="p-6">
                <div className="flex items-center space-x-4">
                  <Clock className="w-8 h-8" />
                  <div>
                    <h3 className="text-lg font-semibold">Inheritance Access Timer</h3>
                    <p className="text-primary-foreground/80">
                      Timer will start automatically when you add your first asset.
                    </p>
                  </div>
                </div>
              </CardContent>
            </Card>
          ) : (
            <Card className="bg-gradient-primary text-primary-foreground border-0">
              <CardContent className="p-6">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-4">
                    <Clock className="w-8 h-8" />
                    <div>
                      <h3 className="text-lg font-semibold">Inheritance Access Timer</h3>
                      <p className="text-primary-foreground/80">
                        Time remaining: {timeRemaining}
                      </p>
                      <p className="text-xs text-primary-foreground/60">
                        Last Reset: {lastReset ? lastReset.toLocaleString() : "N/A"}
                      </p>
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
              </CardContent>
            </Card>
          )}
        </div>

        {/* Total Assets Overview */}
        <div className="grid gap-6 md:grid-cols-3 mb-8">
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
                  setTimeRemaining("");
                  setDistributionWarning(null);
                } else {
                  setTimeRemaining(timer ? `${timer} seconds` : "Expired");
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
          <HeirsList onHeirsChange={setHeirs} />
        </div>

        {/* Asset Distribution  */}
        {assets.length > 0 && heirs.length > 0 && (
          <div className="mb-8">
            <AssetDistribution assets={assets} heirs={heirs} />
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

export default Dashboard;
