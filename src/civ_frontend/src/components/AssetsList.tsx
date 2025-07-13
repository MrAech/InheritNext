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

interface Asset {
  id: string;
  name: string;
  type: string;
  value: number;
  description: string;
}

interface AssetsListProps {
  onTotalChange: (total: number) => void;
  onAssetsChange?: (assets: Asset[]) => void;
}


// FIXME: Hardcoded for now connect with backend 

const AssetsList = ({ onTotalChange, onAssetsChange }: AssetsListProps) => {
  const [assets, setAssets] = useState<Asset[]>([
    {
      id: "1",
      name: "Primary Residence",
      type: "Real Estate",
      value: 850000,
      description: "Family home in Beverly Hills"
    },
    {
      id: "2",
      name: "Investment Portfolio",
      type: "Stocks",
      value: 1200000,
      description: "Diversified stock portfolio"
    },
    {
      id: "3",
      name: "Commercial Property",
      type: "Real Estate",
      value: 650000,
      description: "Downtown office building"
    },
    {
      id: "4",
      name: "Classic Car Collection",
      type: "Collectibles",
      value: 150000,
      description: "Vintage automobiles"
    }
  ]);

  const [editingAsset, setEditingAsset] = useState<Asset | null>(null);
  const [isAddingAsset, setIsAddingAsset] = useState(false);
  const { toast } = useToast();
  const { formatCurrency } = useSettings();

  // Initialize the parent component with assets data
  useEffect(() => {
    calculateTotal(assets);
  }, []);

  const calculateTotal = (assetList: Asset[]) => {
    const total = assetList.reduce((sum, asset) => sum + asset.value, 0);
    onTotalChange(total);
    onAssetsChange?.(assetList);
    return total;
  };

  const handleUpdateAsset = (updatedAsset: Asset) => {
    const newAssets = assets.map(asset =>
      asset.id === updatedAsset.id ? updatedAsset : asset
    );
    setAssets(newAssets);
    calculateTotal(newAssets);
    setEditingAsset(null);
    toast({
      title: "Asset Updated",
      description: `${updatedAsset.name} has been successfully updated.`,
    });
  };

  const handleRemoveAsset = (assetId: string) => {
    const assetToRemove = assets.find(a => a.id === assetId);
    const newAssets = assets.filter(asset => asset.id !== assetId);
    setAssets(newAssets);
    calculateTotal(newAssets);
    toast({
      title: "Asset Removed",
      description: `${assetToRemove?.name} has been removed from your portfolio.`,
      variant: "destructive",
    });
  };

  const handleAddAsset = (newAsset: Omit<Asset, 'id'>) => {
    const asset: Asset = {
      ...newAsset,
      id: Date.now().toString()
    };
    const newAssets = [...assets, asset];
    setAssets(newAssets);
    calculateTotal(newAssets);
    setIsAddingAsset(false);
    toast({
      title: "Asset Added",
      description: `${asset.name} has been added to your portfolio.`,
    });
  };

  const getAssetIcon = (type: string) => {
    switch (type) {
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

      <div className="grid gap-4 md:grid-cols-2">
        {assets.map((asset) => (
          <Card key={asset.id} className="shadow-card hover:shadow-elegant transition-shadow">
            <CardHeader className="pb-3">
              <div className="flex items-start justify-between">
                <div className="flex items-center space-x-3">
                  <div className="p-2 bg-primary/10 rounded-lg text-primary">
                    {getAssetIcon(asset.type)}
                  </div>
                  <div>
                    <CardTitle className="text-lg">{asset.name}</CardTitle>
                    <CardDescription>{asset.description}</CardDescription>
                  </div>
                </div>
                <Badge variant="secondary">{asset.type}</Badge>
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-2xl font-bold text-primary">
                    {formatCurrency(asset.value)}
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
    </div>
  );
};

interface AssetFormDialogProps {
  asset?: Asset;
  onSubmit: (asset: Asset | Omit<Asset, 'id'>) => void;
  onCancel: () => void;
  isEditing?: boolean;
}

const AssetFormDialog = ({ asset, onSubmit, onCancel, isEditing = false }: AssetFormDialogProps) => {
  const [formData, setFormData] = useState({
    name: asset?.name || "",
    type: asset?.type || "",
    value: asset?.value || 0,
    description: asset?.description || ""
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (isEditing && asset) {
      onSubmit({ ...asset, ...formData });
    } else {
      onSubmit(formData);
    }
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
          <Label htmlFor="type">Asset Type</Label>
          <Select
            value={formData.type}
            onValueChange={(value) => setFormData(prev => ({ ...prev, type: value }))}
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
            value={formData.value}
            onChange={(e) => setFormData(prev => ({ ...prev, value: Number(e.target.value) }))}
            placeholder="Enter current value"
            required
          />
        </div>
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