import { useState } from "react";
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
  id: string;
  name: string;
  value: number;
}

interface Heir {
  id: string;
  name: string;
}

interface AssetDistribution {
  id: string;
  assetId: string;
  heirId: string;
  percentage: number;
}

interface AssetDistributionProps {
  assets: Asset[];
  heirs: Heir[];
}

const AssetDistribution = ({ assets, heirs }: AssetDistributionProps) => {
  const [distributions, setDistributions] = useState<AssetDistribution[]>([]);
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [selectedAsset, setSelectedAsset] = useState("");
  const [selectedHeir, setSelectedHeir] = useState("");
  const [percentageInput, setPercentageInput] = useState("");
  const { toast } = useToast();

  const handleAddDistribution = () => {
    const percentage = Number(percentageInput);

    if (!selectedAsset || !selectedHeir || !percentageInput || percentage <= 0) {
      toast({
        title: "Invalid input",
        description: "Please select an asset, heir, and enter a valid percentage.",
        variant: "destructive",
      });
      return;
    }

    const existingAssetDistributions = distributions.filter(d => d.assetId === selectedAsset);
    const totalPercentage = existingAssetDistributions.reduce((sum, d) => sum + d.percentage, 0);

    if (totalPercentage + percentage > 100) {
      toast({
        title: "Percentage exceeded",
        description: `This asset already has ${totalPercentage}% distributed. Cannot exceed 100%.`,
        variant: "destructive",
      });
      return;
    }

    const newDistribution: AssetDistribution = {
      id: Date.now().toString(),
      assetId: selectedAsset,
      heirId: selectedHeir,
      percentage
    };

    setDistributions([...distributions, newDistribution]);
    setSelectedAsset("");
    setSelectedHeir("");
    setPercentageInput("");
    setIsDialogOpen(false);

    toast({
      title: "Distribution added",
      description: "Asset distribution has been successfully added.",
    });
  };

  const handlePercentageChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;

    if (value === "" || (!isNaN(Number(value)) && Number(value) >= 0 && Number(value) <= 100)) {
      setPercentageInput(value);
    }
  };

  const handleRemoveDistribution = (distributionId: string) => {
    setDistributions(distributions.filter(d => d.id !== distributionId));
    toast({
      title: "Distribution removed",
      description: "Asset distribution has been removed.",
    });
  };

  const getAssetDistributions = (assetId: string) => {
    return distributions.filter(d => d.assetId === assetId);
  };

  const getDistributionTotal = (assetId: string) => {
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

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h3 className="text-lg font-semibold">Asset Distribution to Heirs</h3>
          <p className="text-muted-foreground">Define how each asset will be distributed among heirs</p>
        </div>
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
                      <SelectItem key={asset.id} value={asset.id}>
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
                      <SelectItem key={heir.id} value={heir.id}>
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
                    Remaining: {100 - getDistributionTotal(selectedAsset)}%
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
                              {100 - totalDistributed}% remaining to be distributed
                            </p>
                          </div>
                        )}
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
