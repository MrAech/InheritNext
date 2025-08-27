import { useAuth } from "@/context/useAuth";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";

export default function SettingsPage() {
  const { identity } = useAuth();

  return (
    <div className="container mx-auto px-4 py-8">
      <h1 className="text-2xl font-semibold mb-4">Settings</h1>
      <Card className="shadow-card">
        <CardContent>
          <div className="space-y-4">
            <div>Principal: {identity ? identity.getPrincipal().toString() : 'N/A'}</div>
            <Button onClick={() => { localStorage.clear(); window.location.reload(); }}>Clear Local Demo Data</Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
