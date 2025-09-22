import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
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
  Wallet,
} from "lucide-react";
import { useToast } from "@/hooks/use-toast";
import { useAuth } from "@/context/AuthContext";
import Header from "@/components/Header";
import { SettingsDialog } from "@/components/SettingsDialog";
import { useSettings } from "@/context/SettingsContext";

const Dashboard = () => {
  const { logout } = useAuth();
  const [timerResetOpen, setTimerResetOpen] = useState(false);
  const [totalAssets, setTotalAssets] = useState(2850000);
  const [loginTime] = useState<Date>(() => {
    const stored = localStorage.getItem("loginTime");
    return stored ? new Date(stored) : new Date();
  });
  const [lastReset, setLastReset] = useState<Date>(loginTime);
  const [timeRemaining, setTimeRemaining] = useState<string>("");
  const [assets, setAssets] = useState<any[]>([]);
  const [heirs, setHeirs] = useState<any[]>([]);
  const navigate = useNavigate();
  const { toast } = useToast();
  const { formatCurrency } = useSettings();

  useEffect(() => {
    const updateCountdown = () => {
      const now = new Date();
      const expiryDate = new Date(lastReset);
      expiryDate.setMonth(expiryDate.getMonth() + 1);

      const diff = expiryDate.getTime() - now.getTime();

      if (diff > 0) {
        const days = Math.floor(diff / (1000 * 60 * 60 * 24));
        const hours = Math.floor(
          (diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60),
        );
        const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));

        setTimeRemaining(`${days}d ${hours}h ${minutes}m`);
      } else {
        setTimeRemaining("Expired");
      }
    };

    updateCountdown();
    const interval = setInterval(updateCountdown, 60000);

    return () => clearInterval(interval);
  }, [lastReset]);

  const handleSignOut = async () => {
    await logout();
    toast({
      title: "Signed out",
      description: "You have been successfully signed out.",
    });
    navigate("/");
  };

  const handleTimerReset = () => {
    setLastReset(new Date());
    setTimerResetOpen(false);
    toast({
      title: "Timer Reset",
      description: "Dashboard timer has been successfully reset.",
    });
  };

  const goToHeirView = () => {
    navigate("/heir-view");
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
              {/* Header component shows principal and sign-in controls */}
              <div className="ml-2">
                <Header />
              </div>
            </div>
          </div>
        </div>
      </header>

      <main className="container mx-auto px-4 py-8">
        {/* Timer Status */}
        <div className="mb-8">
          <Card className="bg-gradient-primary text-primary-foreground border-0">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-4">
                  <Clock className="w-8 h-8" />
                  <div>
                    <h3 className="text-lg font-semibold">
                      Inheritance Access Timer
                    </h3>
                    <p className="text-primary-foreground/80">
                      Time remaining: {timeRemaining}
                    </p>
                    <p className="text-xs text-primary-foreground/60">
                      Last Reset: {lastReset.toLocaleString()}
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
        </div>

        {/* Total Assets Overview */}
        <div className="grid gap-6 md:grid-cols-3 mb-8">
          <Card className="shadow-card">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">
                Total Assets
              </CardTitle>
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
              <p className="text-xs text-muted-foreground">Heirs</p>
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
            <AssetDistribution
              assets={assets}
              heirs={heirs.map((h) => ({ id: h.gov_id_hash, name: h.name }))}
            />
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
