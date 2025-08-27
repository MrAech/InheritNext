import { useEffect, useState } from "react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { listAssets, updateAsset } from "@/lib/api";
import type { Asset, AssetInput } from "@/types/backend";
type ExtendedAsset = Asset & { approval_required?: boolean; holding_mode?: string };

export default function EscrowPage() {
  const [assets, setAssets] = useState<ExtendedAsset[]>([]);

  useEffect(() => {
    (async () => {
      const a = await listAssets();
      setAssets(a);
    })();
  }, []);

  const release = async (id: number) => {
    const payload = { holding_mode: 'Released' } as unknown as AssetInput;
    await updateAsset(id, payload);
    const a = await listAssets();
    setAssets(a);
  };

  return (
    <div className="container mx-auto px-4 py-8">
      <h1 className="text-2xl font-semibold mb-4">Escrow</h1>
      <div className="grid gap-6 md:grid-cols-3">
        <Card className="shadow-card md:col-span-2">
          <CardContent>
            <div className="space-y-3">
              {assets.map(a => (
                <div key={a.id} className="flex items-center justify-between p-2 border rounded">
                  <div>
                    <div className="font-medium">{a.name}</div>
                    <div className="text-sm text-muted-foreground">{a.holding_mode ?? 'N/A'}</div>
                  </div>
                  <div className="space-x-2">
                    <Button size="sm" onClick={() => void release(a.id)}>Release</Button>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
        <Card className="shadow-card">
          <CardContent>
            <h3 className="text-sm font-medium mb-2">Escrow Tips</h3>
            <div className="space-y-2 text-sm text-muted-foreground">
              <div>Assets in escrow can be released to heirs automatically when the timer expires.</div>
              <div>Use Distributions to configure how assets are split among heirs.</div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
