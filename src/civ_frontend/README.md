# Civ Frontend (dev)

This frontend provides minimal UI pages to record token approvals and register vaulted NFTs and demonstrates calling the backend canister in both local (dfx) and on‑IC environments.

Prerequisites
- Node 18+ and npm
- DFX installed and configured (https://smartcontracts.org/docs/current/developer-docs/quickstart/)

Local development (dfx)

1) Start the local replica and deploy canisters:

```bash
# start the local replica (in another terminal)
dfx start --background
# create & deploy canisters
dfx deploy --no-wallet
```

2) Inject the backend canister id into the frontend environment. The frontend expects a variable named `VITE_BACKEND_CANISTER_ID`.

You can copy the local canister id from `.dfx/local/canister_ids.json` into a `.env` file at the frontend root (or the workspace root) like:

```bash
# example: .env in src/civ_frontend
VITE_BACKEND_CANISTER_ID="ryjl3-tyaaa-aaaaa-aaaba-cai"
```

dfx's build step can also populate canister ids for production builds; for local dev the `.env` approach is simplest.

3) Install and run the frontend dev server:

```bash
cd src/civ_frontend
npm install
npm run start
```

4) Open the app in your browser. Use the app's Sign In flow (Internet Identity) to authenticate. The frontend uses `@dfinity/auth-client` and will create an authenticated agent for actor calls.

Notes and behavior
- The frontend actor (`src/civ_frontend/src/lib/actor.ts`) resolves the backend canister id in this order:
	1. `process.env.VITE_BACKEND_CANISTER_ID` (recommended for dfx builds)
	2. `window.__CIV_BACKEND_CANISTER_ID` (optional override for hosting)
- The actor uses `@dfinity/auth-client` to log in with Internet Identity and recreates the HttpAgent with the user's identity so calls from the UI are authenticated.
- For local dev (non-production) the agent fetches the root key to validate candid responses from the local replica. This is skipped in production builds.
- The backend uses canonical ICRC ledger methods (e.g., `icrc1_transfer`, `icrc2_transfer_from`) and when running locally the `.dfx` service DID contains these signatures so calls will match the local ledger canister.

Troubleshooting
- If you see an error about missing `VITE_BACKEND_CANISTER_ID`, confirm you copied the canister id into `.env` or exposed it via the hosting page by setting `window.__CIV_BACKEND_CANISTER_ID`.
- If actor calls fail with candid decoding errors when running local, ensure `dfx start` is running and you deployed the canisters with `dfx deploy`.

Extending for production
- When building for production, arrange your build pipeline to inject the real backend canister id into the frontend build (the variable name `VITE_BACKEND_CANISTER_ID` is used by the Vite build step). dfx's `dfx deploy` and `dfx generate` steps can help automate this.

This README focuses on developer workflows; update and extend these instructions for your CI/CD and production hosting environment.
