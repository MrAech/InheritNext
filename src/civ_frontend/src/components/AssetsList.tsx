import { useState, useEffect } from "react";
import { useAuth } from "@/context/AuthContext";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  Edit,
  Trash2,
  Plus,
  Building,
  Car,
  Banknote,
  TrendingUp,
  Home,
} from "lucide-react";

import { useToast } from "@/hooks/use-toast";
import { useSettings } from "@/context/SettingsContext";
import { getLedgerDecimals } from "@/lib/actor";

interface Asset {
  asset_id: string;
  asset_type: string;
  approved: boolean;
  value: bigint;
  name: string;
  description: string;
}

interface AssetsListProps {
  onTotalChange: (total: number) => void;
  onAssetsChange?: (assets: Asset[]) => void;
}

const AssetsList = ({ onTotalChange, onAssetsChange }: AssetsListProps) => {
  const { actor } = useAuth();
  const { isAuthenticated, login, logout } = useAuth();
  const [assets, setAssets] = useState<Asset[]>([]);
  const [loading, setLoading] = useState(false);

  const [editingAsset, setEditingAsset] = useState<Asset | null>(null);
  const [isAddingAsset, setIsAddingAsset] = useState(false);
  const { toast } = useToast();
  const { formatCurrency } = useSettings();

  // Fetch assets from canister on mount
  useEffect(() => {
    if (!actor) return;
    setLoading(true);
    (async () => {
      try {
        const res = await actor.get_user_state();
        const userState = res.length > 0 ? res[0] : null;
        const assetList: Asset[] = userState ? userState.assets : [];
        setAssets(assetList);
        calculateTotal(assetList);
      } catch (e) {
        console.error("get_user_state failed", e);
      } finally {
        setLoading(false);
      }
    })();
    // eslint-disable-next-line
  }, [actor]);

  const calculateTotal = (assetList: Asset[]) => {
    const total = assetList.reduce(
      (sum, asset) => sum + Number(asset.value),
      0,
    );
    onTotalChange(total);
    onAssetsChange?.(assetList);
    return total;
  };

  const handleUpdateAsset = async (updatedAsset: Asset) => {
    if (!actor) return;
    setLoading(true);
    try {
      // Call canister update
      await actor.update_asset(
        updatedAsset.asset_id,
        updatedAsset.asset_type,
        updatedAsset.name,
        updatedAsset.value,
        updatedAsset.description,
        updatedAsset.approved,
      );
      setEditingAsset(null);
      // Refetch
      const res = await actor.get_user_state();
      const userState = res.length > 0 ? res[0] : null;
      const assetList: Asset[] = userState ? userState.assets : [];
      setAssets(assetList);
      calculateTotal(assetList);
      toast({
        title: "Asset Updated",
        description: `${updatedAsset.name} has been successfully updated.`,
      });
    } catch {
      toast({
        title: "Error",
        description: "Failed to update asset.",
        variant: "destructive",
      });
    }
    setLoading(false);
  };

  const handleRemoveAsset = async (asset_id: string) => {
    if (!actor) return;
    setLoading(true);
    try {
      // Call canister remove
      await actor.remove_asset(asset_id);
      // Refetch
      const res = await actor.get_user_state();
      const userState = res.length > 0 ? res[0] : null;
      const assetList: Asset[] = userState ? userState.assets : [];
      setAssets(assetList);
      calculateTotal(assetList);
      toast({
        title: "Asset Removed",
        description: `Asset has been removed from your portfolio.`,
        variant: "destructive",
      });
    } catch {
      toast({
        title: "Error",
        description: "Failed to remove asset.",
        variant: "destructive",
      });
    }
    setLoading(false);
  };

  const handleAddAsset = async (newAsset: {
    asset_type: string;
    name: string;
    value: number;
    description: string;
  }) => {
    if (!actor) return;
    setLoading(true);
    try {
      // Call canister add_asset
      await actor.add_asset({
        asset_id: Date.now().toString(),
        asset_type: newAsset.asset_type,
        name: newAsset.name,
        value: BigInt(Math.floor(newAsset.value)),
        description: newAsset.description,
        approved: false,
      });
      setIsAddingAsset(false);
      // Refetch
      const res = await actor.get_user_state();
      const userState = Array.isArray(res) && res.length ? res[0] : null;
      const assetList: Asset[] =
        userState && userState.assets ? userState.assets : [];
      setAssets(assetList);
      calculateTotal(assetList);
      toast({
        title: "Asset Added",
        description: `${newAsset.name} has been added to your portfolio.`,
      });
    } catch {
      toast({
        title: "Error",
        description: "Failed to add asset.",
        variant: "destructive",
      });
    }
    setLoading(false);
  };

  const getAssetIcon = (asset_type: string) => {
    switch (asset_type) {
      case "Real Estate":
        return <Home className="w-5 h-5" />;
      case "Stocks":
        return <TrendingUp className="w-5 h-5" />;
      case "Collectibles":
        return <Car className="w-5 h-5" />;
      default:
        return <Banknote className="w-5 h-5" />;
    }
  };

  // NOTE: Kept for fallback sake

  // const formatCurrency = (amount: number) => {
  //   return new Intl.NumberFormat('en-US', {
  //     style: 'currency',
  //     currency: 'USD',
  //     minimumFractionDigits: 0,
  //     maximumFractionDigits: 0,
  //   }).format(amount);
  // };

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <p className="text-muted-foreground">
          Manage your personal assets for inheritance
        </p>
        <div className="flex items-center space-x-2">
          {!isAuthenticated ? (
            <Button size="sm" onClick={() => login()} disabled={loading}>
              Sign In
            </Button>
          ) : (
            <Button
              size="sm"
              variant="ghost"
              onClick={() => logout()}
              disabled={loading}
            >
              Sign Out
            </Button>
          )}

          <Dialog open={isAddingAsset} onOpenChange={setIsAddingAsset}>
            <DialogTrigger asChild>
              <Button
                size="sm"
                className="bg-gradient-success"
                disabled={loading || !isAuthenticated}
              >
                <Plus className="w-4 h-4 mr-2" />
                Add Asset
              </Button>
            </DialogTrigger>
            <AssetFormDialog
              onSubmit={handleAddAsset}
              onCancel={() => setIsAddingAsset(false)}
            />
          </Dialog>
        </div>
      </div>

      {loading ? (
        <div className="text-center py-8">Loading assets...</div>
      ) : (
        <div className="grid gap-4 md:grid-cols-2">
          {assets.map((asset) => (
            <Card
              key={asset.asset_id}
              className="shadow-card hover:shadow-elegant transition-shadow"
            >
              <CardHeader className="pb-3">
                <div className="flex items-start justify-between">
                  <div className="flex items-center space-x-3">
                    <div className="p-2 bg-primary/10 rounded-lg text-primary">
                      {getAssetIcon(asset.asset_type)}
                    </div>
                    <div>
                      <CardTitle className="text-lg">{asset.name}</CardTitle>
                      <CardDescription>{asset.description}</CardDescription>
                    </div>
                  </div>
                  <Badge variant="secondary">{asset.asset_type}</Badge>
                </div>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <span className="text-2xl font-bold text-primary">
                      {formatCurrency(Number(asset.value))}
                    </span>
                  </div>
                  <div className="flex space-x-2">
                    <Dialog
                      open={editingAsset?.asset_id === asset.asset_id}
                      onOpenChange={(open) => !open && setEditingAsset(null)}
                    >
                      <DialogTrigger asChild>
                        <Button
                          variant="outline"
                          size="sm"
                          className="flex-1"
                          onClick={() => setEditingAsset(asset)}
                        >
                          <Edit className="w-4 h-4 mr-2" />
                          Update
                        </Button>
                      </DialogTrigger>
                      {editingAsset && (
                        <AssetFormDialog
                          asset={editingAsset}
                          onSubmit={handleUpdateAsset}
                          onCancel={() => setEditingAsset(null)}
                          isEditing
                        />
                      )}
                    </Dialog>
                    <Button
                      variant="outline"
                      size="sm"
                      className="flex-1 text-destructive hover:bg-destructive hover:text-destructive-foreground"
                      onClick={() => handleRemoveAsset(asset.asset_id)}
                    >
                      <Trash2 className="w-4 h-4 mr-2" />
                      Remove
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
};

interface AssetFormDialogProps {
  asset?: Asset;
  // accept a generic form shape; the parent will normalize
  onSubmit: (
    asset:
      | Asset
      | {
        asset_type: string;
        name: string;
        value: number;
        description: string;
      },
  ) => void;
  onCancel: () => void;
  isEditing?: boolean;
}

const AssetFormDialog = ({
  asset,
  onSubmit,
  onCancel,
  isEditing = false,
}: AssetFormDialogProps) => {
  const [formData, setFormData] = useState<{
    name: string;
    type: string;
    value: number | bigint;
    description: string;
  }>({
    name: asset?.name || "",
    type: asset?.asset_type || "",
    value: asset ? Number(asset.value) : 0,
    description: asset?.description || "",
  });
  const [decimals, setDecimals] = useState<number | null>(null);

  // Fetch ledger decimals when asset type changes
  useEffect(() => {
    let mounted = true;
    const t = formData.type;
    if (!t) {
      setDecimals(null);
      return;
    }
    (async () => {
      try {
        const d = await getLedgerDecimals(t);
        if (mounted) setDecimals(d);
      } catch (e) {
        if (mounted) setDecimals(null);
      }
    })();
    return () => {
      mounted = false;
    };
  }, [formData.type]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const normalized = {
      asset_type: String(formData.type),
      name: String(formData.name),
      value: Number(formData.value),
      description: String(formData.description),
    };
    if (isEditing && asset) {
      // merge and convert value to bigint for submission if caller expects Asset
      const human = normalized.value;
      const smallest =
        decimals && decimals > 0
          ? BigInt(Math.round(human * Math.pow(10, decimals)))
          : BigInt(Math.floor(human));
      const merged: Asset = {
        ...asset,
        asset_type: normalized.asset_type,
        name: normalized.name,
        value: smallest,
        description: normalized.description,
      };
      onSubmit(merged);
    } else {
      const human = normalized.value;
      const smallest =
        decimals && decimals > 0
          ? BigInt(Math.round(human * Math.pow(10, decimals)))
          : BigInt(Math.floor(human));
      onSubmit({
        asset_type: normalized.asset_type,
        name: normalized.name,
        value: Number(smallest),
        description: normalized.description,
      });
    }
  };

  return (
    <DialogContent className="sm:max-w-md">
      <DialogHeader>
        <DialogTitle>
          {isEditing ? "Update Asset" : "Add New Asset"}
        </DialogTitle>
        <DialogDescription>
          {isEditing
            ? "Modify the asset details below."
            : "Enter the details for your new asset."}
        </DialogDescription>
      </DialogHeader>
      <form onSubmit={handleSubmit} className="space-y-4">
        <div className="space-y-2">
          <Label htmlFor="name">Asset Name</Label>
          <Input
            id="name"
            value={formData.name}
            onChange={(e) =>
              setFormData((prev) => ({ ...prev, name: e.target.value }))
            }
            placeholder="Enter asset name"
            required
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="type">Asset Type</Label>
          <Select
            value={formData.type}
            onValueChange={(value) =>
              setFormData((prev) => ({ ...prev, type: value }))
            }
          >
            <SelectTrigger>
              <SelectValue placeholder="Select asset type" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="Real Estate">Real Estate</SelectItem>
              <SelectItem value="Stocks">Stocks</SelectItem>
              <SelectItem value="Collectibles">Collectibles</SelectItem>
              <SelectItem value="Cash">Cash</SelectItem>
              <SelectItem value="Bonds">Bonds</SelectItem>
            </SelectContent>
          </Select>
        </div>
        <div className="space-y-2">
          <Label htmlFor="value">Current Value ($)</Label>
          <Input
            id="value"
            type="number"
            value={Number(formData.value)}
            onChange={(e) =>
              setFormData((prev) => ({
                ...prev,
                value: Number(e.target.value),
              }))
            }
            placeholder="Enter current value"
            required
          />
          {decimals !== null && (
            <div className="text-xs text-muted-foreground">
              Ledger decimals: {decimals} — value will be scaled on submit.
            </div>
          )}
        </div>
        <div className="space-y-2">
          <Label htmlFor="description">Description</Label>
          <Input
            id="description"
            value={formData.description}
            onChange={(e) =>
              setFormData((prev) => ({ ...prev, description: e.target.value }))
            }
            placeholder="Enter asset description"
            required
          />
        </div>
        <DialogFooter>
          <Button type="button" variant="outline" onClick={onCancel}>
            Cancel
          </Button>
          <Button type="submit" className="bg-gradient-primary">
            {isEditing ? "Update Asset" : "Add Asset"}
          </Button>
        </DialogFooter>
      </form>
    </DialogContent>
  );
};

export { AssetsList };
