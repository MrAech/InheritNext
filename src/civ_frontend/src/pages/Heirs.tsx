import { HeirsList } from "@/components/HeirsList";
import { Card, CardContent } from "@/components/ui/card";

export default function HeirsPage() {
  return (
    <div className="container mx-auto px-4 py-8">
      <h1 className="text-2xl font-semibold mb-4">Heirs</h1>
          <div className="grid gap-6 md:grid-cols-3">
            <Card className="shadow-card md:col-span-2">
              <CardContent>
                <HeirsList />
              </CardContent>
            </Card>
            <Card className="shadow-card">
              <CardContent>
                <h3 className="text-sm font-medium mb-2">Heir Setup Tips</h3>
                <div className="space-y-2 text-sm text-muted-foreground">
                  <div>Set a secret for each heir to enable secure claim flows.</div>
                  <div>Assign relationships and contact info for verification.</div>
                </div>
              </CardContent>
            </Card>
          </div>
    </div>
  );
}
