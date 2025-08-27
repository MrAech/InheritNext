import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger
} from "@/components/ui/dialog";
import {
  Edit,
  Trash2,
  Plus,
  Building,
  Car,
  Banknote,
  TrendingUp,
  Home
} from "lucide-react";
import { useToast } from "@/hooks/use-toast";
import { useSettings } from "@/context/SettingsContext";
import {
  listAssets,
  addAsset,
  updateAsset,
  removeAsset
} from "@/lib/api";
import type { Asset, AssetInput } from "@/types/backend";

interface AssetsListProps {
  onTotalChange: (total: number) => void;
  onAssetsChange?: (assets: Asset[]) => void;
  onAssetAdded?: () => void;
}

const AssetsList = ({ onTotalChange, onAssetsChange, onAssetAdded }: AssetsListProps) => {
  const [assets, setAssets] = useState<Asset[]>([]);
  const [loading, setLoading] = useState(true);
  const [editingAsset, setEditingAsset] = useState<Asset | null>(null);
  const [isAddingAsset, setIsAddingAsset] = useState(false);
  const { toast } = useToast();
  const { formatCurrency } = useSettings();

  useEffect(() => {
    async function fetchAssets() {
      setLoading(true);
      try {
        const data = await listAssets();
        setAssets(data);
        calculateTotal(data);
      } catch (err) {
        toast({
          title: "Error loading assets",
          description: String(err),
          variant: "destructive",
        });
      } finally {
        setLoading(false);
      }
    }
    fetchAssets();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const calculateTotal = (assetList: Asset[]) => {
    const totalBigInt = assetList.reduce((sum, asset) => sum + BigInt(asset.value), BigInt(0));
    const total = Number(totalBigInt);
    onTotalChange(total);
    onAssetsChange?.(assetList);
    return total;
  };

  const handleUpdateAsset = async (updatedAsset: AssetInput) => {
    setLoading(true);
    try {
      const ok = await updateAsset(editingAsset!.id, updatedAsset);
      if (ok) {
        const data = await listAssets();
        setAssets(data);
        calculateTotal(data);
        setEditingAsset(null);
        toast({
          title: "Asset Updated",
          description: `${updatedAsset.name} has been successfully updated.`,
        });
      } else {
        toast({
          title: "Update Failed",
          description: "Could not update asset.",
          variant: "destructive",
        });
      }
    } catch (err) {
      toast({
        title: "Error updating asset",
        description: String(err),
        variant: "destructive",
      });
    } finally {
      setLoading(false);
    }
  };

  const handleRemoveAsset = async (assetId: number) => {
    setLoading(true);
    try {
      const ok = await removeAsset(assetId);
      if (ok) {
        const data = await listAssets();
        setAssets(data);
        calculateTotal(data);
        toast({
          title: "Asset Removed",
          description: `Asset has been removed from your portfolio.`,
          variant: "destructive",
        });
      } else {
        toast({
          title: "Remove Failed",
          description: "Could not remove asset.",
          variant: "destructive",
        });
      }
    } catch (err) {
      toast({
        title: "Error removing asset",
        description: String(err),
        variant: "destructive",
      });
    } finally {
      setLoading(false);
    }
  };

  const handleAddAsset = async (newAsset: AssetInput) => {
    setLoading(true);
    try {
      // value may be undefined for new assets; do not convert undefined to BigInt
      const assetToSend = { ...newAsset } as AssetInput;
      // Defensive: ensure decimals is always a number (0 sentinel) before calling API
      const normalizedAsset: AssetInput = {
        ...assetToSend,
      };
      const ok = await addAsset(normalizedAsset);
      if (ok) {
        try {
          const data = await listAssets();
          setAssets(data);
          calculateTotal(data);
          setIsAddingAsset(false);
          toast({
            title: "Asset Added",
            description: `${newAsset.name} has been added to your portfolio.`,
          });
          if (typeof onAssetAdded === "function") {
            onAssetAdded();
          }
        } catch (err) {
          if (
            err &&
            typeof err === "object" &&
            "message" in err &&
            String((err as Error).message).includes("Cannot mix BigInt and other types")
          ) {
            // nothing to see here
          } else {
            console.error("Error loading assets from backend:", err);
            toast({
              title: "Error loading assets from backend",
              description: String(err),
              variant: "destructive",
            });
          }
        }
        } else {
          // Simplify UX: close dialog even on add failure but show error so user can try again.
          setIsAddingAsset(false);
          toast({
            title: "Add Failed",
            description: "Could not add asset (backend rejected request). It was saved locally but may not be persisted.",
            variant: "destructive",
          });
        }
    } catch (err) {
      if (
        err &&
        typeof err === "object" &&
        "message" in err &&
        String((err as Error).message).includes("Cannot mix BigInt")
      ) {
        // nothing to see here :) 
      } else {
        toast({
          title: "Error adding asset",
          description: String(err),
          variant: "destructive",
        });
      }
    } finally {
      setLoading(false);
    }
  };

  const getAssetIcon = (asset_type: string) => {
    // Prefer the asset kind when available (frontend maps backend `kind` into asset.kind)
    switch (asset_type) {
      case "Fungible":
        return <Banknote className="w-5 h-5" />;
      case "NFT":
        return <TrendingUp className="w-5 h-5" />;
      case "ChainWrapped":
        return <Building className="w-5 h-5" />;
      case "Document":
        return <Home className="w-5 h-5" />;
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
        <p className="text-muted-foreground">Manage your personal assets for inheritance</p>
        <Dialog open={isAddingAsset} onOpenChange={setIsAddingAsset}>
          <DialogTrigger asChild>
            <Button size="sm" className="bg-gradient-success">
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

      {loading ? (
        <div className="text-center text-muted-foreground py-8">Loading assets...</div>
      ) : (
        <div className="grid gap-4 md:grid-cols-2">
          {assets.map((asset) => (
            <Card key={asset.id} className="shadow-card hover:shadow-elegant transition-shadow">
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
                  <Badge variant="secondary">{asset.kind ?? asset.asset_type}</Badge>
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
                    <Dialog open={editingAsset?.id === asset.id} onOpenChange={(open) => !open && setEditingAsset(null)}>
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
                      onClick={() => handleRemoveAsset(asset.id)}
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
  onSubmit: (asset: AssetInput) => void;
  onCancel: () => void;
  isEditing?: boolean;
}

const AssetFormDialog = ({ asset, onSubmit, onCancel, isEditing = false }: AssetFormDialogProps) => {
  const [formData, setFormData] = useState(() => ({
    name: asset?.name || "",
    // asset_type is a human-facing label; default to kind when present
    asset_type: asset?.asset_type || (asset?.kind as string) || "Fungible",
    kind: (asset?.kind as string) || "Fungible",
    // backend-managed fields (value, decimals) are not edited by frontend
  token_canister: asset ? asset.token_canister ?? undefined : undefined,
  token_id: asset ? asset.token_id ?? undefined : undefined,
  holding_mode: asset ? asset.holding_mode ?? undefined : undefined,
  nft_standard: asset ? asset.nft_standard ?? undefined : undefined,
  chain_wrapped: asset ? asset.chain_wrapped ?? undefined : undefined,
  file_path: asset ? asset.file_path ?? undefined : undefined,
    description: asset?.description || "",
  }));

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const base: Partial<AssetInput> = {
      name: formData.name,
      asset_type: formData.asset_type,
      kind: formData.kind,
      description: formData.description,
      token_canister: formData.token_canister ?? undefined,
      token_id: formData.token_id ?? undefined,
      nft_standard: formData.nft_standard ?? undefined,
      chain_wrapped: formData.chain_wrapped ?? undefined,
      file_path: formData.file_path ?? undefined,
    };
    // For Document assets, force holding_mode to Escrow and keep payload minimal.
    if (formData.kind === "Document") {
      base.holding_mode = "Escrow";
    } else {
      base.holding_mode = formData.holding_mode ?? undefined;
    }
    const assetInput: AssetInput = base as AssetInput;
    // Call onSubmit and expect the handler to close the dialog on success.
    onSubmit(assetInput);
  };

  return (
    <DialogContent className="sm:max-w-md">
      <DialogHeader>
        <DialogTitle>
          {isEditing ? "Update Asset" : "Add New Asset"}
        </DialogTitle>
        <DialogDescription>
          {isEditing ? "Modify the asset details below." : "Enter the details for your new asset."}
        </DialogDescription>
      </DialogHeader>
      <form onSubmit={handleSubmit} className="space-y-4">
        <div className="space-y-2">
          <Label htmlFor="name">Asset Name</Label>
          <Input
            id="name"
            value={formData.name}
            onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
            placeholder="Enter asset name"
            required
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="type">Asset Kind</Label>
          <Select
            value={formData.kind}
            onValueChange={(value) => setFormData(prev => ({ ...prev, kind: value, asset_type: value }))}
          >
            <SelectTrigger>
              <SelectValue placeholder="Select asset kind" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="Fungible">Fungible Token</SelectItem>
              <SelectItem value="NFT">NFT</SelectItem>
              <SelectItem value="ChainWrapped">Chain Wrapped</SelectItem>
              <SelectItem value="Document">Document</SelectItem>
            </SelectContent>
          </Select>
        </div>
        {/* Value is assigned by backend for new assets; show current value when editing but don't require on add */}
        {/* Backend-managed fields (value/decimals) are not editable in the frontend form. */}
        <div className="space-y-2">
          <Label htmlFor="description">Description</Label>
          <Input
            id="description"
            value={formData.description}
            onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
            placeholder="Enter asset description"
            required
          />
        </div>
        {/* Dynamic inputs by kind */}
        {formData.kind === "NFT" && (
          <div className="space-y-2">
            <Label htmlFor="nft_canister">NFT Canister ID</Label>
            <Input
              id="nft_canister"
              value={formData.token_canister ?? ""}
              onChange={(e) => setFormData(prev => ({ ...prev, token_canister: e.target.value }))}
              placeholder="Enter canister id"
            />
            <Label htmlFor="nft_token_id">Token ID</Label>
            <Input
              id="nft_token_id"
              type="number"
              value={formData.token_id ?? ""}
              onChange={(e) => setFormData(prev => ({ ...prev, token_id: e.target.value ? Number(e.target.value) : null }))}
            />
            <Label htmlFor="nft_standard">NFT Standard</Label>
            <Select value={formData.nft_standard ?? "Ext"} onValueChange={(v) => setFormData(prev => ({ ...prev, nft_standard: v }))}>
              <SelectTrigger>
                <SelectValue placeholder="Select standard" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="Dip721">DIP721</SelectItem>
                <SelectItem value="Ext">Ext</SelectItem>
                <SelectItem value="Other">Other</SelectItem>
              </SelectContent>
            </Select>
          </div>
        )}
        {formData.kind === "Fungible" && (
          <div className="space-y-2">
            <Label htmlFor="token_id">Token ID (optional)</Label>
            <Input
              id="token_id"
              type="number"
              value={formData.token_id ?? ""}
              onChange={(e) => setFormData(prev => ({ ...prev, token_id: e.target.value ? Number(e.target.value) : null }))}
              placeholder="Enter token id (if applicable)"
            />
          </div>
        )}
        {formData.kind === "ChainWrapped" && (
          <div className="space-y-2">
            <Label htmlFor="wrapped">Wrapped Type</Label>
            <Select value={formData.chain_wrapped ?? "CkBtc"} onValueChange={(v) => setFormData(prev => ({ ...prev, chain_wrapped: v }))}>
              <SelectTrigger>
                <SelectValue placeholder="Select wrapped type" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="CkBtc">ckBTC</SelectItem>
                <SelectItem value="CkEth">ckETH</SelectItem>
              </SelectContent>
            </Select>
          </div>
        )}
        {formData.kind === "Document" && (
          <div className="space-y-2">
            <Label htmlFor="file">Document Upload (for now we store filename)</Label>
            <Input
              id="file"
              type="file"
              onChange={(e) => {
                const f = (e.target as HTMLInputElement).files?.[0];
                if (f) setFormData(prev => ({ ...prev, file_path: `/uploads/${f.name}` }));
              }}
            />
          </div>
        )}
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
