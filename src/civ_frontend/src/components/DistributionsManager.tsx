import { useCallback, useEffect, useMemo, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { Share2, Plus, Trash2 } from "lucide-react";
import { useToast } from "@/hooks/use-toast";
import { AssetDistributionChart } from "@/components/AssetDistributionChart";
import { getAssetDistributions, setAssetDistributions } from "@/lib/distribution";

type Asset = { id: number; name: string; value: number };
type Heir = { id: number; name: string };

type Row = { heirId: number; percentage: number };

function formatCurrency(amount: number) {
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  }).format(amount);
}

function AssetEditor({ asset, heirs }: { asset: Asset; heirs: Heir[] }) {
  const { toast } = useToast();
  const [rows, setRows] = useState<Row[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [newHeirId, setNewHeirId] = useState<string>("");
  const [newPct, setNewPct] = useState<string>("");

  const load = useCallback(async () => {
    setLoading(true);
    try {
      console.debug("[Dist] load", { assetId: asset.id });
      const list = await getAssetDistributions(asset.id);
      const mapped = list.map(d => ({ heirId: d.heir_id, percentage: d.percentage }));
      console.debug("[Dist] load ->", mapped);
      setRows(mapped);
    } finally {
      setLoading(false);
    }
  }, [asset.id]);

  useEffect(() => { load(); }, [load]);

  const total = useMemo(() => rows.reduce((s, r) => s + r.percentage, 0), [rows]);

  const addRow = async () => {
    const pct = Number(newPct);
    const heirId = Number(newHeirId);
    if (!heirId || !pct || pct <= 0) {
      toast({ title: "Invalid input", description: "Pick an heir and a valid percentage.", variant: "destructive" });
      return;
    }
    if (rows.some(r => r.heirId === heirId)) {
      toast({ title: "Duplicate heir", description: "This heir already has a share.", variant: "destructive" });
      return;
    }
    if (total + pct > 100) {
      toast({ title: "Exceeded 100%", description: `Remaining ${100 - total}%`, variant: "destructive" });
      return;
    }
    const next = [...rows, { heirId, percentage: pct }];
    console.debug("[Dist] addRow -> persist", { assetId: asset.id, next });
    setSaving(true);
    try {
      const ok = await setAssetDistributions(asset.id, next.map(r => ({ asset_id: asset.id, heir_id: r.heirId, percentage: r.percentage })));
      console.debug("[Dist] addRow result", { ok });
      if (ok) {
        setRows(next);
        setNewHeirId("");
        setNewPct("");
        toast({ title: "Saved", description: `Added heir to ${asset.name}.` });
      } else {
        toast({ title: "Save failed", description: `Failed to add for ${asset.name}.`, variant: "destructive" });
      }
    } catch (e) {
      console.error("[Dist] addRow error", e);
      toast({ title: "Save error", description: String(e), variant: "destructive" });
    } finally {
      setSaving(false);
    }
  };

  const removeRow = async (heirId: number) => {
    const next = rows.filter(r => r.heirId !== heirId);
    console.debug("[Dist] removeRow -> persist", { assetId: asset.id, heirId, next });
    setSaving(true);
    try {
      const ok = await setAssetDistributions(asset.id, next.map(r => ({ asset_id: asset.id, heir_id: r.heirId, percentage: r.percentage })));
      console.debug("[Dist] removeRow result", { ok });
      if (ok) {
        setRows(next);
        toast({ title: "Removed", description: `Removed heir from ${asset.name}.` });
      } else {
        toast({ title: "Remove failed", description: `Failed to remove for ${asset.name}.`, variant: "destructive" });
      }
    } catch (e) {
      console.error("[Dist] removeRow error", e);
      toast({ title: "Remove error", description: String(e), variant: "destructive" });
    } finally {
      setSaving(false);
    }
  };

  const updatePctLocal = (heirId: number, pct: number) => setRows(rows.map(r => r.heirId === heirId ? { ...r, percentage: pct } : r));

  const persistPct = async (heirId: number) => {
    const target = rows.find(r => r.heirId === heirId);
    if (!target) return;
    const othersTotal = rows.filter(r => r.heirId !== heirId).reduce((s, r) => s + r.percentage, 0);
    if (othersTotal + target.percentage > 100) {
      toast({ title: "Exceeded 100%", description: `Remaining ${100 - othersTotal}%`, variant: "destructive" });
      return;
    }
    const next = [...rows];
    console.debug("[Dist] persistPct -> persist", { assetId: asset.id, heirId, next });
    setSaving(true);
    try {
      const ok = await setAssetDistributions(asset.id, next.map(r => ({ asset_id: asset.id, heir_id: r.heirId, percentage: r.percentage })));
      console.debug("[Dist] persistPct result", { ok });
      if (!ok) {
        toast({ title: "Save failed", description: `Failed to update ${asset.name}.`, variant: "destructive" });
      }
    } catch (e) {
      console.error("[Dist] persistPct error", e);
      toast({ title: "Save error", description: String(e), variant: "destructive" });
    } finally {
      setSaving(false);
    }
  };

  const save = async () => {
    if (rows.length === 0) {
      toast({ title: "Nothing to save", description: "Add at least one row.", variant: "destructive" });
      return;
    }
    if (total !== 100) {
      toast({ title: "Must total 100%", description: `Currently ${total}%`, variant: "destructive" });
      return;
    }
    setSaving(true);
    try {
      const ok = await setAssetDistributions(asset.id, rows.map(r => ({ asset_id: asset.id, heir_id: r.heirId, percentage: r.percentage })));
      if (ok) {
        toast({ title: "Saved", description: `Saved distributions for ${asset.name}.` });
        await load();
      } else {
        toast({ title: "Save failed", description: `Failed to save for ${asset.name}.`, variant: "destructive" });
      }
    } catch (e) {
      toast({ title: "Save error", description: String(e), variant: "destructive" });
    } finally {
      setSaving(false);
    }
  };

  return (
    <Card className="shadow-card">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="text-lg">{asset.name}</CardTitle>
            <CardDescription>Total Value: {formatCurrency(asset.value)}</CardDescription>
          </div>
          <Badge variant={total === 100 ? "secondary" : "destructive"}>{total}% Distributed</Badge>
        </div>
      </CardHeader>
      <CardContent>
        <div className="grid lg:grid-cols-2 gap-6">
          <div>
            {loading ? (
              <div className="text-muted-foreground">Loading...</div>
            ) : rows.length ? (
              <div className="space-y-3">
                {rows.map(r => {
                  const heir = heirs.find(h => h.id === r.heirId);
                  const inheritanceValue = (Number(asset.value) * r.percentage) / 100;
                  return (
                    <div key={r.heirId} className="flex items-center justify-between p-3 bg-muted/50 rounded-lg">
                      <div className="flex items-center gap-3">
                        <Share2 className="w-4 h-4 text-primary" />
                        <div>
                          <p className="font-medium">{heir?.name}</p>
                          <p className="text-sm text-muted-foreground">{r.percentage}% • {formatCurrency(inheritanceValue)}</p>
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        <Input type="number" value={r.percentage} min={0} max={100} className="w-20"
                          disabled={saving}
                          onChange={(e) => updatePctLocal(r.heirId, Number(e.target.value))}
                          onBlur={() => void persistPct(r.heirId)} />
                        <Button variant="outline" size="sm" disabled={saving} className="text-destructive hover:bg-destructive hover:text-destructive-foreground" onClick={() => void removeRow(r.heirId)}>
                          <Trash2 className="w-4 h-4" />
                        </Button>
                      </div>
                    </div>
                  );
                })}
                {total < 100 && (
                  <div className="text-center py-2">
                    <Separator className="mb-2" />
                    <p className="text-sm text-muted-foreground">{100 - total}% remaining to be distributed</p>
                  </div>
                )}
                {/* Removed explicit Save button; changes persist immediately */}
              </div>
            ) : (
              <div className="text-center py-6 text-muted-foreground">
                <Share2 className="w-12 h-12 mx-auto mb-2 opacity-50" />
                <p>No distributions set for this asset</p>
                <p className="text-sm">Use the form below to get started</p>
              </div>
            )}
            <div className="mt-4 grid grid-cols-3 gap-2 items-end">
              <div className="col-span-2">
                <Label>Heir</Label>
                <Select value={newHeirId} onValueChange={setNewHeirId} disabled={saving}>
                  <SelectTrigger><SelectValue placeholder="Choose an heir" /></SelectTrigger>
                  <SelectContent>
                    {heirs.filter(h => !rows.some(r => r.heirId === h.id)).map(h => (
                      <SelectItem key={h.id} value={String(h.id)}>{h.name}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div>
                <Label>Percentage %</Label>
                <Input type="number" value={newPct} min={0} max={100} onChange={e => setNewPct(e.target.value)} placeholder="%" disabled={saving} />
              </div>
              <div className="col-span-3 flex justify-end">
                <Button size="sm" className="bg-gradient-success" onClick={() => void addRow()} disabled={saving}>
                  <Plus className="w-4 h-4 mr-2" /> Add
                </Button>
              </div>
            </div>
          </div>
          <div>
            <AssetDistributionChart asset={asset} heirs={heirs} distributions={rows.map(r => ({ id: `${asset.id}-${r.heirId}`, assetId: asset.id, heirId: r.heirId, percentage: r.percentage }))} />
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

function DistributionsOverview({ assets, heirs }: { assets: Asset[]; heirs: Heir[] }) {
  const [data, setData] = useState<Record<number, Row[]>>({});
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      setLoading(true);
      const map: Record<number, Row[]> = {};
      for (const a of assets) {
        const d = await getAssetDistributions(a.id);
        map[a.id] = d.map(x => ({ heirId: x.heir_id, percentage: x.percentage }));
      }
      setData(map);
      setLoading(false);
    })();
  }, [assets]);

  if (loading) return null;

  return (
    <Card className="shadow-card">
      <CardHeader>
        <CardTitle>All Distributions</CardTitle>
        <CardDescription>Overview across all assets</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {assets.map(a => {
          const rows = data[a.id] || [];
          const total = rows.reduce((s, r) => s + r.percentage, 0);
          return (
            <div key={a.id} className="p-4 border rounded-lg">
              <div className="flex items-center justify-between mb-2">
                <div className="font-medium">{a.name}</div>
                <Badge variant={total === 100 ? "secondary" : "destructive"}>{total}%</Badge>
              </div>
              {rows.length ? rows.map(r => {
                const heir = heirs.find(h => h.id === r.heirId);
                return (
                  <div key={`${a.id}-${r.heirId}`} className="flex justify-between text-sm py-1">
                    <span>{heir?.name || `Heir #${r.heirId}`}</span>
                    <span>{r.percentage}%</span>
                  </div>
                );
              }) : (
                <div className="text-sm text-muted-foreground">No distributions</div>
              )}
            </div>
          );
        })}
      </CardContent>
    </Card>
  );
}

export function DistributionsManager({ assets, heirs }: { assets: Asset[]; heirs: Heir[] }) {
  return (
    <div className="space-y-6">
      <div className="grid gap-4">
        {assets.map(a => (
          <AssetEditor key={a.id} asset={a} heirs={heirs} />
        ))}
      </div>
      <DistributionsOverview assets={assets} heirs={heirs} />
    </div>
  );
}

export default DistributionsManager;
