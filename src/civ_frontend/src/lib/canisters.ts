// Runtime canister ID resolver with safe fallbacks.
// Priority:
// 1) window.__CANISTER_IDS__ (if a canisters.json or inline script populated it at runtime)
// 2) process.env (browser-inlined by the bundler if configured)
// 3) generated declarations (for civ_backend)

// Note: To use runtime IDs, you can host a JSON at /canisters.json like:
// { "civ_backend": "ryjl3-tyaaa-aaaaa-aaaba-cai", "civ_frontend": "qaa6y-5yaaa-aaaaa-aaafa-cai", "internet_identity": "rdmx6-jaaaa-aaaaa-aaadq-cai" }
// and set window.__CANISTER_IDS__ = await (await fetch('/canisters.json')).json() before loading your app.

/* eslint-disable @typescript-eslint/no-explicit-any */
export function getCanisterId(name: "civ_backend" | "civ_frontend" | "internet_identity", generated?: string): string | undefined {
  // 1) Window global map
  try {
    const w = typeof window !== "undefined" ? (window as any) : undefined;
    const map = w && w.__CANISTER_IDS__ as Record<string, string> | undefined;
    if (map && typeof map[name] === "string" && map[name]) return map[name];
  } catch {
    // ignore
  }

  // 2) process.env in browser (if inlined)
  const env: Record<string, string | undefined> | undefined =
    typeof process !== "undefined" && typeof process.env !== "undefined"
      ? (process.env as unknown as Record<string, string | undefined>)
      : undefined;

  if (name === "civ_backend") {
    const id = env?.CANISTER_ID_CIV_BACKEND || env?.CANISTER_ID || generated;
    if (id) return id;
  }
  if (name === "civ_frontend") {
    const id = env?.CANISTER_ID_CIV_FRONTEND || env?.CANISTER_ID;
    if (id) return id;
  }
  if (name === "internet_identity") {
    const id = env?.CANISTER_ID_INTERNET_IDENTITY || env?.VITE_CANISTER_ID_INTERNET_IDENTITY;
    if (id) return id;
  }

  return generated; // last resort
}
