import { useState, useEffect } from "react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useToast } from "@/hooks/use-toast";
import { findClaim, redeemClaim, verifyClaimSecret, listAssets } from "@/lib/api";

type Asset = { id: number; name: string; asset_type?: string; value?: number };

/*
  Visual / style improvements for Claim page:
  - clearer headers and descriptions
  - grouped inputs with visible primary actions
  - success indicators and compact asset rows
*/
export default function ClaimPage() {
  const [code, setCode] = useState("");
  const [secret, setSecret] = useState("");
  const [claim, setClaim] = useState<{ code: string; heirId: number; assets: number[] } | null>(null);
  const [assetMap, setAssetMap] = useState<Record<number, Asset>>({});
  const [verified, setVerified] = useState(false);
  const [prefs, setPrefs] = useState<Record<number, string>>({});
  const [loading, setLoading] = useState(false);
  const { toast } = useToast();

  useEffect(() => {
    // reset verification when code changes
    setVerified(false);
    setClaim(null);
    setAssetMap({});
    setPrefs({});
  }, [code]);

  const handleLookup = async () => {
    if (!code.trim()) {
      toast({ title: "Enter code", description: "Please enter a claim code", variant: "destructive" });
      return;
    }
    setLoading(true);
    try {
      const c = await findClaim(code.trim());
      if (!c) {
        toast({ title: "Not found", description: "Claim code not found", variant: "destructive" });
        setClaim(null);
        return;
      }
      setClaim(c);
      // load asset details for display
      const assets = await listAssets();
      const map: Record<number, Asset> = {};
      for (const a of assets) map[a.id] = a;
      setAssetMap(map);
      // initialize prefs with sensible defaults
      const initial: Record<number, string> = {};
      for (const aid of c.assets) initial[aid] = "to_principal";
      setPrefs(initial);
      toast({ title: "Claim found", description: `Claim for heir ${c.heirId} with ${c.assets.length} asset(s).` });
    } catch (e) {
      console.error("lookup error", e);
      toast({ title: "Lookup failed", description: String(e), variant: "destructive" });
    } finally {
      setLoading(false);
    }
  };

  const handleVerify = async () => {
    if (!claim) {
      toast({ title: "Lookup first", description: "Please lookup the claim code first", variant: "destructive" });
      return;
    }
    if (!secret) {
      toast({ title: "Enter secret", description: "Please enter identity secret to verify", variant: "destructive" });
      return;
    }
    setLoading(true);
    try {
      const v = await verifyClaimSecret(claim.code, secret);
      if (!v.ok) {
        toast({ title: "Verification failed", description: v.reason ?? "Invalid secret", variant: "destructive" });
        setVerified(false);
        return;
      }
      toast({ title: "Verified", description: "Identity secret verified" });
      setVerified(true);
    } catch (e) {
      console.error("verify error", e);
      toast({ title: "Verification error", description: String(e), variant: "destructive" });
    } finally {
      setLoading(false);
    }
  };

  const handleConfirmAndRedeem = async () => {
    if (!claim) {
      toast({ title: "Lookup first", description: "Please lookup the claim code first", variant: "destructive" });
      return;
    }
    if (!verified) {
      toast({ title: "Verify first", description: "Please verify your identity secret first", variant: "destructive" });
      return;
    }
    setLoading(true);
    try {
      // For demo: log chosen payout preferences to console so you can observe them during demo
      console.log("Claim payout preferences:", prefs);
      const res = await redeemClaim(claim.code, secret || undefined);
      if (res.success) {
        toast({ title: "Claim redeemed", description: `Assets: ${res.assets?.join(", ")}` });
        console.log("redeem result:", res);
      } else {
        toast({ title: "Redeem failed", description: res.reason ?? "Redeem failed", variant: "destructive" });
        console.error("redeem failed:", res);
      }
    } catch (e) {
      console.error("redeem error", e);
      toast({ title: "Redeem error", description: String(e), variant: "destructive" });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="container mx-auto px-4 py-8 max-w-4xl">
      <h1 className="text-2xl font-semibold mb-4">Claim</h1>

      <Card className="shadow-card mb-6">
        <CardHeader>
          <CardTitle>Lookup Claim</CardTitle>
          <CardDescription>Enter the claim code you received to begin the claim process.</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col sm:flex-row gap-3 items-start">
            <Input
              placeholder="CLAIM-XXXX-YYYY"
              value={code}
              onChange={(e) => setCode(e.target.value)}
              className="flex-1"
            />
            <Button onClick={handleLookup} disabled={loading} className="whitespace-nowrap">
              Lookup
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card className="shadow-card mb-6">
        <CardHeader>
          <CardTitle>Identity Verification</CardTitle>
          <CardDescription>Provide the identity secret (Aadhaar/PAN or passphrase). This is hashed client-side for demo verification.</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            <Input
              placeholder="Identity Secret (Aadhaar / PAN / phrase)"
              value={secret}
              onChange={(e) => setSecret(e.target.value)}
              type="password"
            />
            <div className="flex gap-2 items-center">
              <Button onClick={handleVerify} disabled={loading || !claim}>Verify Secret</Button>
              <Button variant="ghost" onClick={() => { setSecret(""); setVerified(false); }}>Clear</Button>
              {verified && <Badge variant="secondary">Verified</Badge>}
            </div>
            {verified && <div className="text-sm text-green-600">Secret verified — choose payout preferences below and Redeem.</div>}
          </div>
        </CardContent>
      </Card>

      {claim && !verified && (
        <Card className="shadow-card mb-6">
          <CardHeader>
            <CardTitle>Claim found</CardTitle>
            <CardDescription>Claim recognized — verify identity to reveal assets and redeem.</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              <div className="font-medium">Claim: {claim.code}</div>
              <div className="text-sm text-muted-foreground">
                Heir ID: {claim.heirId} — {claim.assets.length} protected asset(s). Verify with your identity secret to reveal details and redeem.
              </div>
            </div>
            <div className="mt-4 flex gap-2">
              <Button variant="ghost" onClick={() => { setClaim(null); setPrefs({}); }}>Cancel</Button>
            </div>
          </CardContent>
        </Card>
      )}

      {claim && verified && (
        <Card className="shadow-card mb-6">
          <CardHeader>
            <CardTitle>Assets to claim</CardTitle>
            <CardDescription>Pick how you want each asset delivered (demo options).</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {claim.assets.map(aid => {
                const asset = assetMap[aid];
                const name = asset ? `${asset.name}` : `Asset ${aid}`;
                const type = asset?.asset_type ?? "Unknown";
                const pref = prefs[aid] ?? "to_principal";
                return (
                  <div key={aid} className="flex items-center justify-between gap-3 p-2 border rounded">
                    <div>
                      <div className="font-medium">{name}</div>
                      <div className="text-xs text-muted-foreground">{type} • id:{aid}</div>
                    </div>
                    <div className="flex items-center gap-3">
                      <select
                        value={pref}
                        onChange={(e) => setPrefs(prev => ({ ...prev, [aid]: e.target.value }))}
                        className="border rounded px-2 py-1 bg-black text-white appearance-none"
                        style={{ WebkitAppearance: 'none', MozAppearance: 'none' }}
                      >
                        <option value="to_principal">Deliver to Principal</option>
                        <option value="to_custody">Deliver to Custody (hold)</option>
                        <option value="ck_withdraw">Deliver + CK Withdraw (if supported)</option>
                      </select>
                    </div>
                  </div>
                );
              })}
            </div>
            <div className="mt-4 flex gap-2">
              <Button onClick={handleConfirmAndRedeem} disabled={!verified || loading}>Confirm & Redeem</Button>
              <Button variant="ghost" onClick={() => { setClaim(null); setPrefs({}); }}>Cancel</Button>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
