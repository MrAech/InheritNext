import React, { useEffect, useMemo, useState } from 'react';
import { Card, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { assignDistributions, listDistributions } from '@/lib/api';
import { AssetDistributionChart } from './AssetDistributionChart';
import type { AssetDistribution as BackendDistribution } from '@/types/backend';
import { useToast } from '@/hooks/use-toast';

type Asset = { id: number; name: string; value?: number };
type Heir = { id: number; name: string };

function Pie({ data }: { data: { label: string; value: number; color?: string }[] }) {
  const total = data.reduce((s, d) => s + Math.max(0, d.value), 0) || 1;
  let angle = 0;
  const slices = data.map((d, i) => {
    const start = angle;
    const portion = (d.value / total) * 360;
    const end = start + portion;
    angle = end;
    const large = portion > 180 ? 1 : 0;
    // polar to cartesian
    const r = 80;
    const cx = 100;
    const cy = 100;
    const startRad = (start - 90) * (Math.PI / 180);
    const endRad = (end - 90) * (Math.PI / 180);
    const x1 = cx + r * Math.cos(startRad);
    const y1 = cy + r * Math.sin(startRad);
    const x2 = cx + r * Math.cos(endRad);
    const y2 = cy + r * Math.sin(endRad);
    const dPath = `M ${cx} ${cy} L ${x1} ${y1} A ${r} ${r} 0 ${large} 1 ${x2} ${y2} Z`;
    return { d: dPath, color: d.color ?? (`hsl(${(i * 73) % 360} 70% 50%)`), label: d.label, value: d.value };
  });
  return (
    <svg width={200} height={200} viewBox="0 0 200 200" aria-hidden>
      {slices.map((s, i) => (
        <path key={i} d={s.d} fill={s.color} stroke="#fff" strokeWidth={1} />
      ))}
      <circle cx={100} cy={100} r={36} fill="#fff" />
    </svg>
  );
}

type AssetDistribution = { assetId: number; heirId: number; percentage: number };
type DistributionItem = { assetId: number; heirId: number; percentage: number };

export default function DistributionsManager({ assets, heirs }: { assets: Asset[]; heirs: Heir[] }) {
  const [selectedAsset, setSelectedAsset] = useState<number | null>(assets[0]?.id ?? null);
  const [entries, setEntries] = useState<Record<number, number>>(() => ({}));
  const [saving, setSaving] = useState(false);
  const { toast } = useToast();
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    setSelectedAsset(assets[0]?.id ?? null);
  }, [assets]);

  // Load distributions from backend/mock store and prefill entries for selected asset
  useEffect(() => {
    let mounted = true;
    async function load() {
      if (!selectedAsset) return;
      setLoading(true);
      try {
        const all = await listDistributions();
        // Filter for selected asset
        const forAsset = all.filter(d => Number(d.asset_id) === selectedAsset);
        const map: Record<number, number> = {};
        for (const h of heirs) map[h.id] = 0;
        for (const d of forAsset) {
          map[Number(d.heir_id)] = Number(d.percentage);
        }
        if (mounted) setEntries(map);
      } catch (e) {
        console.warn('Failed to load distributions', e);
      } finally {
        if (mounted) setLoading(false);
      }
    }
    void load();
    return () => { mounted = false; };
  }, [selectedAsset, heirs]);

  const total = useMemo(() => Object.values(entries).reduce((s, v) => s + Number(v || 0), 0), [entries]);

  const [allDistributions, setAllDistributions] = useState<DistributionItem[]>([]);
  const pieData = heirs.map((h, idx) => ({ label: h.name, value: entries[h.id] ?? 0 }));

  useEffect(() => {
    let mounted = true;
    async function loadAll() {
      try {
        const all = await listDistributions();
        if (mounted) setAllDistributions(all.map(d => ({ assetId: Number(d.asset_id), heirId: Number(d.heir_id), percentage: Number(d.percentage) })));
      } catch (e) {
        console.warn('Failed to load all distributions', e);
      }
    }
    void loadAll();
    return () => { mounted = false; };
  }, []);

  const save = async () => {
    if (!selectedAsset) return;
    if (total !== 100) {
      toast({ title: 'Invalid', description: 'Percentages must total 100%', variant: 'destructive' });
      return;
    }
    setSaving(true);
    try {
      // Only send distributions for the selected asset
      const dists: BackendDistribution[] = heirs.map(h => ({ asset_id: selectedAsset!, heir_id: h.id, percentage: Number(entries[h.id] || 0) }));
      const ok = await assignDistributions(dists);
      if (ok) {
        toast({ title: 'Saved', description: 'Distributions saved' });
        try {
          // In demo/mock mode, ensure the inheritance timer starts when distributions are assigned.
          // Use dynamic import to avoid top-level coupling and keep behavior demo-only.
          const api = await import('@/lib/api');
          if (typeof (api as any).resetTimer === 'function') {
            const started = await (api as any).resetTimer();
          if (started) {
            toast({ title: 'Timer Started', description: 'Inheritance timer started (demo).' });
            try {
              // notify dashboard to refresh timer display
              // eslint-disable-next-line @typescript-eslint/no-explicit-any
              (window as any).dispatchEvent(new CustomEvent('inheritnext:timer-started'));
            } catch (e) {
              console.warn('[DistributionsManager] dispatch timer-started event failed', e);
            }
          }
          }
        } catch (e) {
          console.warn('[DistributionsManager] resetTimer failed', e);
        }
      } else {
        toast({ title: 'Failed', description: 'Assign failed', variant: 'destructive' });
      }
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
      <Card>
        <CardContent>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-1">Select asset</label>
              <select
                value={selectedAsset ?? ''}
                onChange={(e) => setSelectedAsset(Number(e.target.value))}
                className="w-full border rounded p-2 bg-black text-white"
                style={{ appearance: 'none' }}
              >
                {assets.map(a => (
                  <option key={a.id} value={a.id} className="bg-black text-white">{a.name}</option>
                ))}
              </select>
            </div>
            <div>
              <h3 className="font-medium mb-2">Assign percentages to heirs</h3>
              <div className="space-y-2">
                {heirs.map(h => (
                  <div key={h.id} className="flex items-center gap-2">
                    <div className="w-32">{h.name}</div>
                    <Input className="w-24" type="number" min={0} max={100} value={entries[h.id] ?? ''} onChange={(e) => setEntries(prev => ({ ...prev, [h.id]: Number(e.target.value) }))} />
                    <div className="text-sm text-muted-foreground">%</div>
                    <div className="ml-2 text-xs text-muted-foreground">{entries[h.id] ?? 0}%</div>
                  </div>
                ))}
                <div className="text-sm">Total: <strong>{total}%</strong></div>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <Button onClick={() => void save()} disabled={saving}>{saving ? 'Saving...' : 'Save distributions'}</Button>
              <Button variant="ghost" onClick={() => setEntries({})}>Reset</Button>
            </div>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardContent>
          <div className="flex flex-col items-stretch gap-4">
            <div>
              <h3 className="font-medium mb-2">Preview</h3>
              <div className="text-sm text-muted-foreground mb-2">Per-heir breakdown for the selected asset</div>
            </div>
            <div className="md:flex md:items-start md:gap-8">
              <div className="md:flex-1 pr-6">
                <div className="space-y-3">
                  {heirs.map((h, i) => (
                    <div key={h.id} className="flex justify-between items-center text-sm py-2">
                      <div className="flex items-center gap-3"><span className="inline-block w-3 h-3 rounded-full" style={{ background: `hsl(${(i * 73) % 360} 70% 50%)` }} />{h.name}</div>
                      <div className="font-medium">{entries[h.id] ?? 0}%</div>
                    </div>
                  ))}
                </div>
              </div>
              {/* vertical divider */}
              <div className="hidden md:block w-px bg-border mx-2 h-auto" style={{ alignSelf: 'stretch' }} />
              <div className="md:w-64 md:flex-shrink-0 mt-4 md:mt-0 flex justify-center">
                <div className="w-full max-w-[280px] overflow-visible flex justify-center items-center">
                  {selectedAsset && (() => {
                    const found = assets.find(a => a.id === selectedAsset);
                    const selected = found ? { id: found.id, name: found.name, value: found.value ?? 0 } : { id: selectedAsset, name: 'Asset', value: 0 };
                    return <div className="-mx-4">{/* negative margin to ensure pie isn't clipped */}
                      <AssetDistributionChart asset={selected} heirs={heirs} distributions={allDistributions} />
                    </div>;
                  })()}
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
