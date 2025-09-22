import React, { useState } from "react";
import { recordTokenApproval } from "../lib/actor";

export default function Approvals() {
  const [tokenCanister, setTokenCanister] = useState("");
  const [assetType, setAssetType] = useState("");
  const [approvedAmount, setApprovedAmount] = useState("0");

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    await recordTokenApproval(
      tokenCanister,
      assetType,
      BigInt(approvedAmount),
      null,
      false,
    );
    alert("Approval recorded (frontend demo)");
  };

  return (
    <div className="p-4">
      <h2 className="text-xl font-bold">Record Token Approval</h2>
      <form onSubmit={submit} className="mt-4">
        <div>
          <label>Token Canister (principal)</label>
          <input
            className="border p-1 w-full"
            value={tokenCanister}
            onChange={(e) => setTokenCanister(e.target.value)}
          />
        </div>
        <div className="mt-2">
          <label>Asset Type</label>
          <input
            className="border p-1 w-full"
            value={assetType}
            onChange={(e) => setAssetType(e.target.value)}
          />
        </div>
        <div className="mt-2">
          <label>Approved Amount</label>
          <input
            className="border p-1 w-full"
            value={approvedAmount}
            onChange={(e) => setApprovedAmount(e.target.value)}
          />
        </div>
        <button
          className="mt-3 bg-blue-500 text-white px-3 py-1 rounded"
          type="submit"
        >
          Record Approval
        </button>
      </form>
    </div>
  );
}
