import { useEffect, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { useToast } from "@/hooks/use-toast";
import { listAuditLog } from "@/lib/api";

export default function AuditLogPage() {
  const [entries, setEntries] = useState<{ id: number; ts: number; msg: string }[]>([]);
  const [loading, setLoading] = useState(false);
  const { toast } = useToast();

  const load = async () => {
    setLoading(true);
    try {
      const res = await listAuditLog(100);
      setEntries(res);
    } catch (e) {
      console.error("Failed to load audit log", e);
      toast({ title: "Load error", description: String(e), variant: "destructive" });
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, []);

  return (
    <div className="container mx-auto px-4 py-8">
      <h1 className="text-2xl font-semibold mb-4">Audit Log</h1>
      <Card className="shadow-card">
        <CardHeader className="flex items-center justify-between p-4">
          <CardTitle className="text-sm font-medium">Recent events</CardTitle>
          <div className="flex items-center gap-2">
            <Button onClick={load} disabled={loading}>Refresh</Button>
            <Button variant="ghost" onClick={() => { console.log('Audit entries:', entries); }}>Dump to Console</Button>
          </div>
        </CardHeader>
        <CardContent>
          <div className="overflow-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="text-left">
                  <th className="py-2 pr-4">ID</th>
                  <th className="py-2 pr-4">Time</th>
                  <th className="py-2">Message</th>
                </tr>
              </thead>
              <tbody>
                {entries.length === 0 ? (
                  <tr><td colSpan={3} className="py-6 text-center text-muted-foreground">No audit events</td></tr>
                ) : (
                  entries.slice().reverse().map(e => (
                    <tr key={e.id} className="border-t">
                      <td className="py-2 pr-4 align-top">{e.id}</td>
                      <td className="py-2 pr-4 align-top">{new Date(e.ts).toLocaleString()}</td>
                      <td className="py-2 align-top">{e.msg}</td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
