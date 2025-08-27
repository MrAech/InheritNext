import { AssetsList } from "@/components/AssetsList";
import { Card, CardContent } from "@/components/ui/card";

export default function AssetsPage() {
  return (
    <div className="container mx-auto px-4 py-8">
      <h1 className="text-2xl font-semibold mb-4">Assets</h1>
          <div className="grid gap-6 md:grid-cols-3">
            <Card className="shadow-card md:col-span-2">
              <CardContent>
                <AssetsList onTotalChange={() => {}} />
              </CardContent>
            </Card>
            <Card className="shadow-card">
              <CardContent>
                <h3 className="text-sm font-medium mb-2">Quick Actions</h3>
                <div className="space-y-2 text-sm text-muted-foreground">
                  <div>Add a new asset to start the inheritance timer.</div>
                  <div>Use the Distributions page to assign heirs their shares.</div>
                  <div>Upload related documents in Documents.</div>
                </div>
              </CardContent>
            </Card>
          </div>
    </div>
  );
}
