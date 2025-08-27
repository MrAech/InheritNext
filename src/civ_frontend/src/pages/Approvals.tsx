import { useEffect, useState } from "react";
import { Card, CardContent } from "@/components/ui/card";
import { listAssets, updateAsset } from "@/lib/api";
import type { Asset, AssetInput } from "@/types/backend";
type ExtendedAsset = Asset & { approval_required?: boolean; holding_mode?: string };

export default function ApprovalsPage() {
  const [assets, setAssets] = useState<ExtendedAsset[]>([]);

  useEffect(() => {
    (async () => {
      const a = await listAssets();
      setAssets(a);
    })();
  }, []);

  const toggle = async (asset: ExtendedAsset) => {
    const payload = { approval_required: !asset.approval_required } as unknown as AssetInput;
    await updateAsset(asset.id, payload);
    const a = await listAssets();
    setAssets(a);
  };

  return (
    <div className="container mx-auto px-4 py-8">
      <h1 className="text-2xl font-semibold mb-4">Approvals</h1>
      <Card className="shadow-card">
        <CardContent>
          <div className="space-y-3">
            {assets.map(a => (
              <div key={a.id} className="flex items-center justify-between p-2 border rounded">
                <div>
                  <div className="font-medium">{a.name}</div>
                </div>
                <div>
                  <label className="flex items-center gap-2">
                    <input type="checkbox" checked={!!a.approval_required} onChange={() => void toggle(a)} />
                    <span className="text-sm">Approved</span>
                  </label>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
