export interface Asset {
  id: number;
  name: string;
  asset_type: string;
  kind?: "Fungible" | "NFT" | "ChainWrapped" | "Document" | string;
  value: number;
  decimals: number;
  token_canister?: string | null;
  token_id?: number | null;
  holding_mode?: "Escrow" | "Approval" | null;
  nft_standard?: string | null;
  chain_wrapped?: string | null;
  file_path?: string | null;
  description: string;
  created_at: number;
  updated_at: number;
}

export interface AssetInput {
  name: string;
  asset_type: string;
  kind?: "Fungible" | "NFT" | "ChainWrapped" | "Document" | string;
  description: string;
  token_canister?: string | null;
  token_id?: number | null;
  file_path?: string | null;
  holding_mode?: "Escrow" | "Approval" | null;
  nft_standard?: string | null;
  chain_wrapped?: string | null;
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
  // Optional Aadhaar-related fields: salt and hashed aadhaar value
  salt?: string | null;
  adhaarnum?: string | null;
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
