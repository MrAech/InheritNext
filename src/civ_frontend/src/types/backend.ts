export interface Asset {
  id: number;
  name: string;
  asset_type: string;
  value: number;
  description: string;
  created_at: number;
  updated_at: number;
}

export interface AssetInput {
  name: string;
  asset_type: string;
  value: bigint;
  description: string;
}

export interface Heir {
  id: number;
  name: string;
  relationship: string;
  email: string;
  phone: string;
  address: string;
  created_at: number;
  updated_at: number;
}

export interface HeirInput {
  name: string;
  relationship: string;
  email: string;
  phone: string;
  address: string;
}

export interface AssetDistribution {
  asset_id: number;
  heir_id: number;
  percentage: number;
}

export interface CivError {
  AssetNotFound?: null;
  InvalidHeirPercentage?: null;
  AssetExists?: null;
  HeirExists?: null;
  DistributionAssetNotFound?: null;
  DistributionHeirNotFound?: null;
  HeirNotFound?: null;
  Other?: string;
  UserNotFound?: null;
}

export type Result = { Ok?: null; Err?: CivError };

export interface User {
  user: string;
  assets: Asset[];
  heirs: Heir[];
  distributions: AssetDistribution[];
  timer: number;
}

export type Timer = number;
