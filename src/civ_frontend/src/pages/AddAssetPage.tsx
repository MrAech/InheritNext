import React, { useState, useEffect } from "react";
import { Card, CardHeader, CardTitle, CardContent, CardDescription } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectTrigger, SelectValue, SelectContent, SelectItem } from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipTrigger, TooltipContent } from "@/components/ui/tooltip";
import { Badge } from "@/components/ui/badge";

import { useOverlayManager } from "@/context/OverlayManagerContext";
import { useNavigate } from "react-router-dom";
import { addAsset as addAssetAPI } from "../lib/api";

const AddAssetPage: React.FC = () => {
  const { setOverlay, setOverlayProps } = useOverlayManager();
  const navigate = useNavigate();
  const [formData, setFormData] = useState({
    name: "",
    asset_type: "",
    value: "",
    description: "",
  });
  const [showConfirm, setShowConfirm] = useState(false);

  const handleChange = (field: string, value: string) => {
    setFormData(prev => ({ ...prev, [field]: value }));
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setShowConfirm(true);
  };

  const handleConfirm = async () => {
    const { name, asset_type, value, description } = formData;
    await addAssetAPI({
      name,
      asset_type,
      value: BigInt(value),
      description,
    });
    setShowConfirm(false);
    navigate("/dashboard");
  };

  return (
    <div className="flex justify-center items-center min-h-screen bg-background">
      <Card className="max-w-xl w-full p-6 shadow-elegant">
        <CardHeader>
          <CardTitle>Add New Asset</CardTitle>
          <CardDescription>
            Enter asset details. Tokenization and KYC are simulated for evaluator mode.
          </CardDescription>
        </CardHeader>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <Tooltip>
              <TooltipTrigger asChild>
                <Label htmlFor="name">Asset Name</Label>
              </TooltipTrigger>
              <TooltipContent>Enter the name of your asset (e.g., House, Stocks).</TooltipContent>
            </Tooltip>
            <Input
              id="name"
              value={formData.name}
              onChange={e => handleChange("name", e.target.value)}
              placeholder="Enter asset name"
              required
            />
          </div>
          <div className="space-y-2">
            <Tooltip>
              <TooltipTrigger asChild>
                <Label htmlFor="type">Asset Type</Label>
              </TooltipTrigger>
              <TooltipContent>Select the type of asset you want to add.</TooltipContent>
            </Tooltip>
            <Select
              value={formData.asset_type}
              onValueChange={value => handleChange("asset_type", value)}
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
            <Tooltip>
              <TooltipTrigger asChild>
                <Label htmlFor="value">Current Value ($)</Label>
              </TooltipTrigger>
              <TooltipContent>Enter the estimated value(will be auto calculated) of your asset.</TooltipContent>
            </Tooltip>
            <Input
              id="value"
              type="number"
              value={formData.value}
              onChange={e => handleChange("value", e.target.value)}
              placeholder="Enter current value"
              required
            />
          </div>
          <div className="space-y-2">
            <Tooltip>
              <TooltipTrigger asChild>
                <Label htmlFor="description">Description</Label>
              </TooltipTrigger>
              <TooltipContent>Provide a brief description of the asset.</TooltipContent>
            </Tooltip>
            <Input
              id="description"
              value={formData.description}
              onChange={e => handleChange("description", e.target.value)}
              placeholder="Enter asset description"
              required
            />
          </div>
          <Button
            type="submit"
            className="bg-green-600 w-full"
          >
            Add Asset
          </Button>
        </form>
      </Card>
      {/* Confirmation Modal */}
      {showConfirm && (
        <div className="fixed inset-0 z-[110] flex items-center justify-center bg-black/40">
          <Card className="max-w-md p-6 shadow-elegant">
            <h2 className="text-xl font-bold mb-4">Confirm Asset Addition</h2>
            <div className="mb-4 space-y-2">
              <div><strong>Name:</strong> {formData.name}</div>
              <div><strong>Type:</strong> {formData.asset_type}</div>
              <div><strong>Value:</strong> {formData.value}</div>
              <div><strong>Description:</strong> {formData.description}</div>
            </div>
            <div className="flex gap-2 justify-end">
              <Button size="sm" variant="outline" onClick={() => setShowConfirm(false)}>
                Cancel
              </Button>
              <Button size="sm" variant="default" onClick={handleConfirm}>
                Confirm & Add
              </Button>
            </div>
          </Card>
        </div>
      )}
    </div>
  );
};

export default AddAssetPage;