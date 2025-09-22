import { useState } from "react";
import { Principal } from "@dfinity/principal";
import { useAuth } from "@/context/AuthContext";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { useToast } from "@/hooks/use-toast";

const HeirView = () => {
  const { actor } = useAuth();
  const { toast } = useToast();
  const [govIdHash, setGovIdHash] = useState("");
  const [results, setResults] = useState<any[] | null>(null);
  const [ownerPrincipal, setOwnerPrincipal] = useState<Principal | null>(null);
  const [loading, setLoading] = useState(false);

  const handleLookup = async () => {
    if (!actor) return;
    if (!govIdHash) {
      toast({
        title: "Input required",
        description: "Enter a gov id hash to lookup.",
      });
      return;
    }
    setLoading(true);
    try {
      const userState = await actor.get_user_state();
      const state = userState.length > 0 ? userState[0] : null;
      if (!state) {
        toast({
          title: "No data",
          description: "No user state available on canister.",
          variant: "destructive",
        });
        setResults([]);
        setLoading(false);
        return;
      }

      // normalize heirs (security_question_hash may be []|[string])
      const heirs = state.heirs.map((h: any) => ({
        name: h.name,
        gov_id_hash: h.gov_id_hash,
        security_question_hash:
          h.security_question_hash && h.security_question_hash.length > 0
            ? h.security_question_hash[0]
            : "",
      }));

      // find heir names that match the provided gov id
      const matchingNames = heirs
        .filter((h: any) => h.gov_id_hash === govIdHash)
        .map((h: any) => h.name);

      if (matchingNames.length === 0) {
        setResults([]);
        toast({
          title: "No matches",
          description: "No heirs found for this government id hash.",
        });
        setLoading(false);
        return;
      }

      // distributions: backend Distribution uses heir_name or heir identifier depending on implementation
      const distributions = state.distributions || [];
      const assets = state.assets || [];

      const matchedDistributions = distributions
        .filter((d: any) => matchingNames.includes(d.heir_name))
        .map((d: any) => {
          const asset = assets.find((a: any) => a.asset_id === d.asset_id);
          const percent =
            typeof d.percent === "number"
              ? d.percent
              : d.percent
                ? Number(d.percent)
                : 0;
          const value = asset ? Number(asset.value || 0) : 0;
          const inheritedValue = Math.floor((value * percent) / 100);
          return {
            assetId: d.asset_id,
            assetName: asset ? asset.name : d.asset_id,
            percent,
            value,
            inheritedValue,
          };
        });

      let ownerP: Principal | null = null;
      if (state && state.profile && state.profile.user_principal) {
        try {
          const raw = state.profile.user_principal;
          if (typeof raw === "string") {
            ownerP = Principal.fromText(raw);
          } else {
            // attempt to stringify
            ownerP = Principal.fromText(String(raw));
          }
        } catch (_e) {
          ownerP = null;
        }
      }
      setOwnerPrincipal(ownerP);
      setResults(matchedDistributions);
    } catch (e) {
      console.error(e);
      toast({
        title: "Error",
        description: "Failed to lookup heir distributions.",
        variant: "destructive",
      });
    }
    setLoading(false);
  };

  return (
    <div className="min-h-screen p-4 flex items-start justify-center">
      <div className="w-full max-w-3xl">
        <Card>
          <CardHeader>
            <CardTitle>Heir Claim View</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div>
                <label className="block text-sm mb-2">Government ID Hash</label>
                <Input
                  value={govIdHash}
                  onChange={(e) =>
                    setGovIdHash((e.target as HTMLInputElement).value)
                  }
                  placeholder="Enter gov id hash to view assigned assets"
                />
              </div>
              <div className="flex gap-2">
                <Button onClick={handleLookup} disabled={loading}>
                  {loading ? "Looking..." : "Lookup"}
                </Button>
                <Button
                  variant="outline"
                  onClick={() => {
                    setGovIdHash("");
                    setResults(null);
                  }}
                >
                  Clear
                </Button>
              </div>

              {results && (
                <div>
                  <h4 className="font-semibold mb-2">Assigned Assets</h4>
                  {results.length === 0 ? (
                    <p className="text-muted-foreground">
                      No assets assigned to this government id.
                    </p>
                  ) : (
                    <div className="grid gap-3">
                      {results.map((r, i) => (
                        <div
                          key={i}
                          className="p-3 bg-muted/50 rounded-lg flex items-center justify-between"
                        >
                          <div>
                            <div className="font-medium">{r.assetName}</div>
                            <div className="text-sm text-muted-foreground">
                              {r.percent}% • Value: {r.value} • Inherited:{" "}
                              {r.inheritedValue}
                            </div>
                          </div>
                          <div>
                            <Button
                              size="sm"
                              onClick={async () => {
                                if (!actor) return;
                                try {
                                  // Call heir-facing endpoint: heir_claim_asset(owner_principal, asset_id, heir_gov_id)
                                  // owner principal is available on state.profile.user_principal when we looked up the owner
                                  if (!ownerPrincipal) {
                                    toast({
                                      title: "Error",
                                      description: "Owner principal not found.",
                                      variant: "destructive",
                                    });
                                    return;
                                  }
                                  const res = await actor.heir_claim_asset(
                                    ownerPrincipal,
                                    r.assetId,
                                    govIdHash,
                                    [], // no security answer provided
                                  );
                                  toast({
                                    title: "Claim submitted",
                                    description: `Requested heir claim for ${r.assetName}`,
                                  });
                                  await handleLookup();
                                } catch (e) {
                                  console.error(e);
                                  toast({
                                    title: "Error",
                                    description: "Failed to submit heir claim.",
                                    variant: "destructive",
                                  });
                                }
                              }}
                            >
                              Claim
                            </Button>
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};

export default HeirView;
