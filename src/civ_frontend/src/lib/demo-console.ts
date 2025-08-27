/*
Demo console helpers for InheritNext frontend (dev only)

Usage (in browser devtools while vite dev server running):
1) Load helpers:
   import('/src/lib/demo-console.ts').then(() => console.log('Demo console loaded'));
2) Use exposed helpers from console via `window.__InheritNextDemo`, for example:
   await window.__InheritNextDemo.executeEstateNow();
   await window.__InheritNextDemo.showAudit(20);
   await window.__InheritNextDemo.listAssets();
   await window.__InheritNextDemo.listHeirs();
   await window.__InheritNextDemo.listDistributions();
   await window.__InheritNextDemo.timerStatus();
   await window.__InheritNextDemo.resetTimer();
   await window.__InheritNextDemo.redeemClaim('CLAIM-ABC-123', 'optionalSecret');

Notes:
- These call the existing front-end mock API functions (no backend changes required).
- Intended for local dev / demos only. Do NOT ship to production.

*/
import * as api from './api';

type Demo = {
  executeEstateNow: () => Promise<void>;
  showAudit: (limit?: number) => Promise<void>;
  listAssets: () => Promise<void>;
  listHeirs: () => Promise<void>;
  listDistributions: () => Promise<void>;
  timerStatus: () => Promise<void>;
  resetTimer: () => Promise<void>;
  assignDistributions: (dists: { asset_id: number; heir_id: number; percentage: number }[]) => Promise<void>;
  createClaim: (heirId: number, assets: number[]) => Promise<string | null>;
  redeemClaim: (code: string, secret?: string) => Promise<void>;
  verifyClaimSecret: (code: string, secret: string) => Promise<void>;
};

const demo: Demo = {
  async executeEstateNow() {
    try {
      console.log('[demo] executeEstateNow() -> calling api.executeEstateNow() ...');
      const ok = await (api as any).executeEstateNow();
      console.log('[demo] executeEstateNow result:', ok);
    } catch (e) {
      console.error('[demo] executeEstateNow error:', e);
    }
  },

  async showAudit(limit = 20) {
    try {
      console.log(`[demo] listAuditLog(${limit}) ...`);
      const out = await (api as any).listAuditLog(limit);
      console.table(out.slice(-limit));
      return out;
    } catch (e) {
      console.error('[demo] showAudit error:', e);
    }
  },

  async listAssets() {
    try {
      const assets = await (api as any).listAssets();
      console.table(assets);
      return assets;
    } catch (e) {
      console.error('[demo] listAssets error:', e);
    }
  },

  async listHeirs() {
    try {
      const heirs = await (api as any).listHeirs();
      console.table(heirs);
      return heirs;
    } catch (e) {
      console.error('[demo] listHeirs error:', e);
    }
  },

  async listDistributions() {
    try {
      const d = await (api as any).listDistributions();
      console.table(d);
      return d;
    } catch (e) {
      console.error('[demo] listDistributions error:', e);
    }
  },

  async timerStatus() {
    try {
      const t = await (api as any).timerStatus();
      console.log('[demo] timerStatus:', t);
      return t;
    } catch (e) {
      console.error('[demo] timerStatus error:', e);
    }
  },

  async resetTimer() {
    try {
      const ok = await (api as any).resetTimer();
      console.log('[demo] resetTimer result:', ok);
      return ok;
    } catch (e) {
      console.error('[demo] resetTimer error:', e);
    }
  },

  async assignDistributions(dists) {
    try {
      console.log('[demo] assignDistributions payload:', dists);
      const ok = await (api as any).assignDistributions(dists);
      console.log('[demo] assignDistributions result:', ok);
      return ok;
    } catch (e) {
      console.error('[demo] assignDistributions error:', e);
    }
  },

  async createClaim(heirId: number, assets: number[]) {
    try {
      console.log(`[demo] createClaim heir=${heirId} assets=[${assets.join(',')}]`);
      if (typeof (api as any).createDemoClaim === 'function') {
        const code = (api as any).createDemoClaim(heirId, assets);
        console.log('[demo] createClaim generated code:', code);
        return code;
      } else {
        console.warn('[demo] createDemoClaim not available on api');
        return null;
      }
    } catch (e) {
      console.error('[demo] createClaim error:', e);
      return null;
    }
  },

  async redeemClaim(code: string, secret?: string) {
    try {
      console.log(`[demo] redeemClaim(${code}, ${secret ? '***' : 'no-secret'})`);
      const res = await (api as any).redeemClaim(code, secret);
      console.log('[demo] redeemClaim result:', res);
      return res;
    } catch (e) {
      console.error('[demo] redeemClaim error:', e);
    }
  },

  async verifyClaimSecret(code: string, secret: string) {
    try {
      console.log(`[demo] verifyClaimSecret(${code}, ***)`);
      const res = await (api as any).verifyClaimSecret(code, secret);
      console.log('[demo] verifyClaimSecret result:', res);
      return res;
    } catch (e) {
      console.error('[demo] verifyClaimSecret error:', e);
    }
  }
};

// attach to window for easy access in browser console
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
if (typeof window !== 'undefined') {
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  window.__InheritNextDemo = demo;
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  console.info('[InheritNext Demo] window.__InheritNextDemo available — call import("/src/lib/demo-console.ts") to load.');
}

export default demo;
