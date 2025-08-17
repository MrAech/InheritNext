import { useMemo, useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Share2, Plus, Trash2 } from "lucide-react";
import { useToast } from "@/hooks/use-toast";
import { AssetDistributionChart } from "@/components/AssetDistributionChart";
// import { assignDistributions as assignDistributionsApi } from "@/lib/api";
import { useEffect } from "react";
import { getAssetDistributions as getAssetDistributionsApi, setAssetDistributions as setAssetDistributionsApi, deleteAssetDistribution as deleteAssetDistributionApi } from "@/lib/distribution";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger
} from "@/components/ui/dialog";

interface Asset {
  id: number;
  name: string;
  value: number;
}

interface Heir {
  id: number;
  name: string;
}

interface LocalDistribution {
  id: string;
  assetId: number;
  heirId: number;
  percentage: number;
}

interface AssetDistributionProps {
  assets: Asset[];
  heirs: Heir[];
}

const AssetDistribution = ({ assets, heirs }: AssetDistributionProps) => {
  const [distributions, setDistributions] = useState<LocalDistribution[]>([]); // Always mirrors backend snapshot
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  // Load existing distributions for all assets on mount
  useEffect(() => {
    (async () => {
      try {
        const all: LocalDistribution[] = [];
        for (const a of assets) {
          const d = await getAssetDistributionsApi(a.id);
          all.push(...d.map((x) => ({
            id: `${a.id}-${x.heir_id}-${x.percentage}-${Date.now()}`,
            assetId: a.id,
            heirId: Number(x.heir_id),
            percentage: x.percentage,
          })));
          console.log(`[AssetDistribution][init] Loaded distributions for asset ${a.id}:`, d);
        }
        setDistributions(all);
        console.log(`[AssetDistribution][init] Aggregated distributions:`, all);
      } catch {
        // ignore for now
      }
    })();
  }, [assets]);

  // Debug: expose a helper to dump current backend + local state
  const dumpBackendState = async () => {
    console.groupCollapsed("[AssetDistribution][dump] Backend & Local State");
    try {
      for (const a of assets) {
        try {
          const remote = await getAssetDistributionsApi(a.id);
          console.log(`Asset ${a.id} (${a.name}) remote distributions:`, remote);
        } catch (e) {
          console.warn(`Asset ${a.id} remote fetch failed`, e);
        }
      }
      console.log("Local state distributions:", distributions);
    } finally {
      console.groupEnd();
    }
  };
  const [selectedAsset, setSelectedAsset] = useState("");
  const [selectedHeir, setSelectedHeir] = useState("");
  const [percentageInput, setPercentageInput] = useState("");
  const { toast } = useToast();

  const refreshAll = async () => {
    const all: LocalDistribution[] = [];
    for (const a of assets) {
      try {
        const d = await getAssetDistributionsApi(a.id);
        all.push(...d.map((x) => ({
          id: `${a.id}-${x.heir_id}-${x.percentage}-${Date.now()}`,
          assetId: a.id,
          heirId: Number(x.heir_id),
          percentage: x.percentage,
        })));
      } catch (e) {
        console.warn(`[AssetDistribution][refreshAll] failed asset ${a.id}`, e);
      }
    }
    setDistributions(all);
  };

  const handleAddDistribution = async () => {
    const percentage = Number(percentageInput);

    if (!selectedAsset || !selectedHeir || !percentageInput || percentage <= 0) {
      toast({
        title: "Invalid input",
        description: "Please select an asset, heir, and enter a valid percentage.",
        variant: "destructive",
      });
      return;
    }

    const assetIdNum = Number(selectedAsset);
    const heirIdNum = Number(selectedHeir);
    const existingAssetDistributions = distributions.filter(d => d.assetId === assetIdNum);
    const totalPercentage = existingAssetDistributions.reduce((sum, d) => sum + d.percentage, 0);

    if (totalPercentage + percentage > 100) {
      toast({
        title: "Percentage exceeded",
        description: `This asset already has ${totalPercentage}% distributed. Cannot exceed 100%.`,
        variant: "destructive",
      });
      return;
    }

    // Build new list for that asset including this one and attempt immediate backend save
    const existing = distributions.filter(d => d.assetId === assetIdNum);
    const updatedForAsset = [...existing, { id: Date.now().toString(), assetId: assetIdNum, heirId: heirIdNum, percentage }];
    const body = updatedForAsset.map(l => ({ asset_id: l.assetId, heir_id: l.heirId, percentage: l.percentage }));
    try {
      console.log('[AssetDistribution][add] saving', body);
      const ok = await setAssetDistributionsApi(assetIdNum, body);
      if (!ok) throw new Error('Backend rejected add');
      await refreshAll();
      setSelectedAsset("");
      setSelectedHeir("");
      setPercentageInput("");
      setIsDialogOpen(false);
      toast({ title: "Distribution added", description: "Saved to backend." });
    } catch (e) {
      toast({ title: "Add failed", description: String(e), variant: "destructive" });
    }
  };

  const handlePercentageChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;

    if (value === "" || (!isNaN(Number(value)) && Number(value) >= 0 && Number(value) <= 100)) {
      setPercentageInput(value);
    }
  };

  const handleRemoveDistribution = async (distributionId: string) => {
    const target = distributions.find(d => d.id === distributionId);
    if (!target) return;
    try {
      console.log('[AssetDistribution][delete] attempt', target);
      const ok = await deleteAssetDistributionApi(target.assetId, target.heirId);
      if (!ok) throw new Error('Backend rejected delete');
      await refreshAll();
      toast({ title: "Distribution removed", description: "Deleted on backend." });
    } catch (e) {
      toast({ title: "Delete failed", description: String(e), variant: "destructive" });
    }
  };

  const getAssetDistributions = (assetId: number) => {
    return distributions.filter(d => d.assetId === assetId);
  };

  const getDistributionTotal = (assetId: number) => {
    return getAssetDistributions(assetId).reduce((sum, d) => sum + d.percentage, 0);
  };

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 0,
      maximumFractionDigits: 0,
    }).format(amount);
  };

  const assetsNeedingCompletion = useMemo(() => {
    const map = new Map<number, number>();
    for (const d of distributions) {
      map.set(d.assetId, (map.get(d.assetId) || 0) + d.percentage);
    }
    return assets
      .filter(a => (map.get(a.id) || 0) !== 100)
      .map(a => ({ asset: a, total: map.get(a.id) || 0 }));
  }, [assets, distributions]);

  const handleSaveAssignments = async () => {
    if (distributions.length === 0) {
      toast({ title: "Nothing to save", description: "Add at least one distribution.", variant: "destructive" });
      return;
    }

    // Validate 100% per asset
    if (assetsNeedingCompletion.length > 0) {
      const names = assetsNeedingCompletion.map(x => `${x.asset.name} (${x.total}%)`).join(", ");
      toast({
        title: "Incomplete distribution",
        description: `The following assets must sum to 100%: ${names}`,
        variant: "destructive",
      });
      return;
    }

    // Save per asset using setAssetDistributions
    try {
      const byAsset = new Map<number, LocalDistribution[]>();
      for (const d of distributions) {
        byAsset.set(d.assetId, [...(byAsset.get(d.assetId) || []), d]);
      }
      for (const [assetId, list] of byAsset.entries()) {
        const body = list.map(l => ({ asset_id: assetId, heir_id: l.heirId, percentage: l.percentage }));
        console.log(`[AssetDistribution][saveAll] Saving asset ${assetId} body=`, body);
        const ok = await setAssetDistributionsApi(assetId, body);
        console.log(`[AssetDistribution][saveAll] Result asset ${assetId}: ok=${ok}`);
        if (!ok) {
          throw new Error(`Failed to save for asset ${assetId}`);
        }
      }
      toast({ title: "Distributions saved", description: "Assignments have been saved to the backend." });
      await refreshAll();
      await dumpBackendState();
    } catch (e) {
      toast({ title: "Save error", description: String(e), variant: "destructive" });
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h3 className="text-lg font-semibold">Asset Distribution to Heirs</h3>
          <p className="text-muted-foreground">Define how each asset will be distributed among heirs</p>
        </div>
        <div className="flex items-center gap-2">
          <Button size="sm" variant="outline" onClick={handleSaveAssignments}>
            Save Assignments
          </Button>
          <Button size="sm" variant="outline" onClick={dumpBackendState}>
            Debug Dump
          </Button>
          <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
            <DialogTrigger asChild>
              <Button size="sm" className="bg-gradient-success">
                <Plus className="w-4 h-4 mr-2" />
                Add Distribution
              </Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-md">
              <DialogHeader>
                <DialogTitle>Add Asset Distribution</DialogTitle>
                <DialogDescription>
                  Specify how much percentage of an asset goes to which heir.
                </DialogDescription>
              </DialogHeader>
              <div className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="asset">Select Asset</Label>
                  <Select value={selectedAsset} onValueChange={setSelectedAsset}>
                    <SelectTrigger>
                      <SelectValue placeholder="Choose an asset" />
                    </SelectTrigger>
                    <SelectContent>
                      {assets.map((asset) => (
                        <SelectItem key={asset.id} value={String(asset.id)}>
                          {asset.name} - {formatCurrency(asset.value)}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <Label htmlFor="heir">Select Heir</Label>
                  <Select value={selectedHeir} onValueChange={setSelectedHeir}>
                    <SelectTrigger>
                      <SelectValue placeholder="Choose an heir" />
                    </SelectTrigger>
                    <SelectContent>
                      {heirs.map((heir) => (
                        <SelectItem key={heir.id} value={String(heir.id)}>
                          {heir.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <Label htmlFor="percentage">Percentage (%)</Label>
                  <Input
                    id="percentage"
                    type="number"
                    min="0"
                    max="100"
                    value={percentageInput}
                    onChange={handlePercentageChange}
                    placeholder="Enter percentage"
                  />
                  {selectedAsset && (
                    <p className="text-xs text-muted-foreground">
                      Remaining: {100 - getDistributionTotal(Number(selectedAsset))}%
                    </p>
                  )}
                </div>
              </div>
              <DialogFooter>
                <Button variant="outline" onClick={() => setIsDialogOpen(false)}>
                  Cancel
                </Button>
                <Button onClick={handleAddDistribution} className="bg-gradient-primary">
                  Add Distribution
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        </div>
      </div>

      <div className="grid gap-4">
        {assets.map((asset) => {
          const assetDistributions = getAssetDistributions(asset.id);
          const totalDistributed = getDistributionTotal(asset.id);
          const isComplete = totalDistributed === 100;

          return (
            <Card key={asset.id} className="shadow-card">
              <CardHeader className="pb-3">
                <div className="flex items-center justify-between">
                  <div>
                    <CardTitle className="text-lg">{asset.name}</CardTitle>
                    <CardDescription>
                      Total Value: {formatCurrency(asset.value)}
                    </CardDescription>
                  </div>
                  <Badge variant={isComplete ? "secondary" : "destructive"}>
                    {totalDistributed}% Distributed
                  </Badge>
                </div>
              </CardHeader>
              <CardContent>
                <div className="grid lg:grid-cols-2 gap-6">
                  {/* Distribution List */}
                  <div>
                    {assetDistributions.length > 0 ? (
                      <div className="space-y-3">
                        {assetDistributions.map((distribution) => {
                          const heir = heirs.find(h => h.id === distribution.heirId);
                          const inheritanceValue = (Number(asset.value) * distribution.percentage) / 100;

                          return (
                            <div key={distribution.id} className="flex items-center justify-between p-3 bg-muted/50 rounded-lg">
                              <div className="flex items-center gap-3">
                                <Share2 className="w-4 h-4 text-primary" />
                                <div>
                                  <p className="font-medium">{heir?.name}</p>
                                  <p className="text-sm text-muted-foreground">
                                    {distribution.percentage}% • {formatCurrency(inheritanceValue)}
                                  </p>
                                </div>
                              </div>
                              <Button
                                variant="outline"
                                size="sm"
                                onClick={() => handleRemoveDistribution(distribution.id)}
                                className="text-destructive hover:bg-destructive hover:text-destructive-foreground"
                              >
                                <Trash2 className="w-4 h-4" />
                              </Button>
                            </div>
                          );
                        })}
                        {totalDistributed < 100 && (
                          <div className="text-center py-2">
                            <Separator className="mb-2" />
                            <p className="text-sm text-muted-foreground">
                              {100 - totalDistributed}% remains to be distributed
                            </p>
                          </div>
                        )}
                        <div className="flex justify-end">
                          <Button size="sm" variant="outline" onClick={async () => {
                            const list = getAssetDistributions(asset.id);
                            if (getDistributionTotal(asset.id) > 100) {
                              toast({ title: "Invalid", description: "Total exceeds 100%", variant: "destructive" });
                              return;
                            }
                            try {
                              const body = list.map(l => ({ asset_id: asset.id, heir_id: l.heirId, percentage: l.percentage }));
                              console.log(`[AssetDistribution][saveOne] Saving for asset ${asset.id}:`, body);
                              const ok = await setAssetDistributionsApi(asset.id, body);
                              console.log(`[AssetDistribution][saveOne] Result for asset ${asset.id}: ok=${ok}`);
                              if (ok) {
                                await refreshAll();
                                toast({ title: "Saved", description: `Saved distributions for ${asset.name}.` });
                              } else {
                                toast({ title: "Save failed", description: `Failed to save for ${asset.name}.`, variant: "destructive" });
                              }
                              await dumpBackendState();
                            } catch (e) {
                              console.error(`[AssetDistribution][saveOne] Error for asset ${asset.id}:`, e);
                              toast({ title: "Save error", description: String(e), variant: "destructive" });
                            }
                          }}>Save {asset.name}</Button>
                        </div>
                      </div>
                    ) : (
                      <div className="text-center py-6 text-muted-foreground">
                        <Share2 className="w-12 h-12 mx-auto mb-2 opacity-50" />
                        <p>No distributions set for this asset</p>
                        <p className="text-sm">Use the "Add Distribution" button to get started</p>
                      </div>
                    )}
                  </div>

                  {/* Pie Chart */}
                  <div>
                    <AssetDistributionChart
                      asset={asset}
                      heirs={heirs}
                      distributions={distributions}
                    />
                  </div>
                </div>
              </CardContent>
            </Card>
          );
        })}
      </div>

      {assets.length === 0 && (
        <Card className="shadow-card">
          <CardContent className="text-center py-8">
            <Share2 className="w-16 h-16 mx-auto mb-4 text-muted-foreground opacity-50" />
            <h3 className="text-lg font-semibold mb-2">No assets available</h3>
            <p className="text-muted-foreground">
              Add some assets first to start distributing them to heirs.
            </p>
          </CardContent>
        </Card>
      )}
    </div>
  );
};

export { AssetDistribution };
