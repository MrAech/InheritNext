import React, { createContext, useContext, useState, ReactNode } from "react";
import { addAsset as addAssetAPI, addHeir as addHeirAPI } from "../lib/api";

type Asset = {
  name: string;
  asset_type: string;
  value: string;
  description: string;
};

type Heir = {
  name: string;
  relationship: string;
  email: string;
  phone: string;
  address: string;
};

interface SimulatedDataContextType {
  assets: Asset[];
  heirs: Heir[];
  addAsset: (asset: Asset) => void;
  addHeir: (heir: Heir) => void;
  reset: () => void;
}

const SimulatedDataContext = createContext<SimulatedDataContextType | undefined>(undefined);

export const useSimulatedData = () => {
  const context = useContext(SimulatedDataContext);
  if (!context) throw new Error("useSimulatedData must be used within SimulatedDataProvider");
  return context;
};

export const SimulatedDataProvider = ({ children }: { children: ReactNode }) => {
  const [assets, setAssets] = useState<Asset[]>([]);
  const [heirs, setHeirs] = useState<Heir[]>([]);


  const addAsset = async (asset: Asset) => {
    const assetInput = {
      name: asset.name,
      asset_type: asset.asset_type,
      value: BigInt(asset.value),
      description: asset.description,
    };
    const success = await addAssetAPI(assetInput);
    if (success) setAssets(prev => [...prev, asset]);
  };

  const addHeir = async (heir: Heir) => {
    const heirInput = {
      name: heir.name,
      relationship: heir.relationship,
      email: heir.email,
      phone: heir.phone,
      address: heir.address,
    };
    const success = await addHeirAPI(heirInput);
    if (success) setHeirs(prev => [...prev, heir]);
  };
  const reset = () => {
    setAssets([]);
    setHeirs([]);
  };

  return (
    <SimulatedDataContext.Provider value={{ assets, heirs, addAsset, addHeir, reset }}>
      {children}
    </SimulatedDataContext.Provider>
  );
};