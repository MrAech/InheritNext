# InheritNext Architecture & Integration Overview

## Project Structure

- **Backend (Rust/IC Canister)**
  - Location: `src/civ_backend/`
  - Models: Asset, Heir, Distribution
  - API: Candid interface, per-asset inheritance, CRUD endpoints

- **Frontend (TypeScript/React)**
  - Location: `src/civ_frontend/`
  - Components: Asset/Heir CRUD, Distribution, Dashboard
  - API Layer: `src/civ_frontend/src/lib/api.ts`
  - Types: `src/civ_frontend/src/types/backend.ts`
  - Auth: Internet Identity via `@dfinity/auth-client`

## Integration Flow

1. **Authentication**
   - User logs in via Internet Identity.
   - Auth context provides identity and actor for secure backend calls.

2. **API Service Layer**
   - All frontend CRUD operations use the API layer for backend communication.
   - Error handling, loading states, and success feedback are implemented.

3. **Frontend Components**
   - Assets and Heirs managed via React components.
   - UI/UX feedback for all operations.
   - Types are aligned with backend models, with frontend extensions for UX.

## Architecture Diagram

```
[User]
   |
[React Frontend]
   |--[AuthContext]---[Internet Identity]
   |--[API Layer]-----[DFINITY Agent/Candid]
   |
[IC Canister Backend]
   |--[Rust Models]
   |--[Candid API]
```

## Key Files

- [`src/civ_backend/civ_backend.did`](src/civ_backend/civ_backend.did:1): Backend API and types
- [`src/civ_frontend/src/types/backend.ts`](src/civ_frontend/src/types/backend.ts:1): TypeScript interfaces
- [`src/civ_frontend/src/lib/api.ts`](src/civ_frontend/src/lib/api.ts:1): API service layer
- [`src/civ_frontend/src/context/AuthContext.tsx`](src/civ_frontend/src/context/AuthContext.tsx:1): Authentication/session management
- [`src/civ_frontend/src/components/AssetsList.tsx`](src/civ_frontend/src/components/AssetsList.tsx:1): Asset CRUD UI
- [`src/civ_frontend/src/components/HeirsList.tsx`](src/civ_frontend/src/components/HeirsList.tsx:1): Heir CRUD UI

## Notes

- All major flows are integrated and production-ready.
- UI/UX, error handling, and session management are implemented.
- Further improvements: accessibility, performance, security, i18n, analytics, rollback/recovery.
