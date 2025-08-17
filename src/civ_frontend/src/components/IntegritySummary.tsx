import { useEffect, useState, useCallback } from 'react';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { AlertTriangle, CheckCircle2, RefreshCw } from 'lucide-react';

interface IntegrityDisplay {
  assetCount: number;
  distributionCount: number;
  overAllocated: number[];
  fullyAllocated: number[];
  partiallyAllocated: number[];
  unallocated: number[];
  issues: string[];
}

export function IntegritySummary() {
  const [report, setReport] = useState<IntegrityDisplay | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lastFetched, setLastFetched] = useState<Date | null>(null);

  const fetchReport = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const api = await import('@/lib/api');
      const rep = await api.checkIntegrity();
      setReport(rep);
      setLastFetched(new Date());
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchReport();
  }, [fetchReport]);

  useEffect(() => {
    function handler() {
      fetchReport();
    }
    window.addEventListener('integrity:changed', handler);
    return () => window.removeEventListener('integrity:changed', handler);
  }, [fetchReport]);

  const healthy = !!report && report.overAllocated.length === 0 && report.issues.length === 0;

  return (
    <Card className="shadow-card">
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-medium flex items-center gap-2">
          {healthy ? (
            <CheckCircle2 className="w-4 h-4 text-green-500" />
          ) : (
            <AlertTriangle className="w-4 h-4 text-yellow-500" />
          )}
          Integrity
        </CardTitle>
        <Button variant="ghost" size="icon" onClick={fetchReport} disabled={loading} aria-label="Refresh integrity report">
          <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
        </Button>
      </CardHeader>
      <CardContent className="text-xs space-y-2">
        {error && <div className="text-destructive">Error: {error}</div>}
        {!error && !report && <div>Loading...</div>}
        {report && (
          <div className="space-y-1">
            <div>Assets: {report.assetCount} | Distributions: {report.distributionCount}</div>
            <div className="flex flex-wrap gap-2">
              <span className="rounded bg-muted px-2 py-0.5">Full: {report.fullyAllocated.length}</span>
              <span className="rounded bg-muted px-2 py-0.5">Partial: {report.partiallyAllocated.length}</span>
              <span className="rounded bg-muted px-2 py-0.5">None: {report.unallocated.length}</span>
              <span className="rounded bg-muted px-2 py-0.5">Over: {report.overAllocated.length}</span>
            </div>
            {report.overAllocated.length > 0 && (
              <div className="text-yellow-600">Over-allocated asset IDs: {report.overAllocated.join(', ')}</div>
            )}
            {report.issues.length > 0 && (
              <ul className="list-disc ml-4 text-yellow-700">
                {report.issues.map((i, idx) => (
                  <li key={idx}>{i}</li>
                ))}
              </ul>
            )}
            {healthy && <div className="text-green-600">All invariants healthy.</div>}
            {lastFetched && (
              <div className="text-muted-foreground mt-2">Updated {lastFetched.toLocaleTimeString()}</div>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

export default IntegritySummary;
