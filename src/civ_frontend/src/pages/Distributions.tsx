import { useEffect, useState } from "react";
import DistributionsManager from "@/components/DistributionsManager";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { useNavigate } from "react-router-dom";
import { PieChart, ArrowLeft } from "lucide-react";

type Asset = { id: number; name: string; value: number };
type Heir = { id: number; name: string };

export default function DistributionsPage() {
  const [assets, setAssets] = useState<Asset[]>([]);
  const [heirs, setHeirs] = useState<Heir[]>([]);
  const [loading, setLoading] = useState(true);
  const navigate = useNavigate();

  useEffect(() => {
    (async () => {
      setLoading(true);
      try {
        const api = await import("@/lib/api");
        const [a, h] = await Promise.all([api.listAssets(), api.listHeirs()]);
        setAssets(a);
        setHeirs(h);
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  if (loading) {
    return (
      <div className="container mx-auto px-4 py-8">
        <Card className="shadow-card">
          <CardContent className="p-6 text-muted-foreground">Loading distributions...</CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background">
      <header className="border-b bg-card shadow-card">
        <div className="container mx-auto px-4 py-4 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <PieChart className="w-5 h-5 text-primary" />
            <h1 className="text-lg font-semibold">Distributions</h1>
          </div>
          <Button variant="outline" size="sm" onClick={() => navigate("/dashboard")}>
            <ArrowLeft className="w-4 h-4 mr-2" /> Back to Dashboard
          </Button>
        </div>
      </header>
      <main className="container mx-auto px-4 py-8">
        {assets.length > 0 && heirs.length > 0 ? (
          <DistributionsManager assets={assets} heirs={heirs} />
        ) : (
          <Card className="shadow-card">
            <CardContent className="p-6 text-muted-foreground">Add assets and heirs to configure distributions.</CardContent>
          </Card>
        )}
      </main>
    </div>
  );
}
