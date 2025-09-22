import React, { useState } from "react";
import { registerVaultedNFT } from "../lib/actor";

export default function Vaults() {
  const [collection, setCollection] = useState("");
  const [tokenId, setTokenId] = useState("");
  const [heirHash, setHeirHash] = useState("");

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    // Input validation
    if (!collection.trim() || !tokenId.trim() || !heirHash.trim()) {
      alert("Please fill in all fields");
      return;
    }
    
    try {
      await registerVaultedNFT(collection, tokenId, heirHash);
      alert("Vaulted NFT registered (frontend demo)");
    } catch (error) {
      console.error("Failed to register vaulted NFT:", error);
      alert("Failed to register vaulted NFT. Please try again.");
    }
  };

  return (
    <div className="p-4">
      <h2 className="text-xl font-bold">Register Vaulted NFT</h2>
      <form onSubmit={submit} className="mt-4">
        <div>
          <label>Collection Canister (principal)</label>
          <input
            className="border p-1 w-full"
            value={collection}
            onChange={(e) => setCollection(e.target.value)}
          />
        </div>
        <div className="mt-2">
          <label>Token ID</label>
          <input
            className="border p-1 w-full"
            value={tokenId}
            onChange={(e) => setTokenId(e.target.value)}
          />
        </div>
        <div className="mt-2">
          <label>Assigned Heir Hash</label>
          <input
            className="border p-1 w-full"
            value={heirHash}
            onChange={(e) => setHeirHash(e.target.value)}
          />
        </div>
        <button
          className="mt-3 bg-blue-500 text-white px-3 py-1 rounded"
          type="submit"
        >
          Register Vault
        </button>
      </form>
    </div>
  );
}
