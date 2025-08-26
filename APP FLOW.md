# InheritNext Application Flow Diagrams

This document provides visual flow diagrams for the InheritNext application, showing user journeys and system interactions.

## 🏁 **MAIN APPLICATION FLOW**

```mermaid
graph TD
    A[User Opens App] --> B{User Authenticated?}
    B -->|No| C[Internet Identity Login]
    B -->|Yes| D[Dashboard]
    
    C --> E{Login Successful?}
    E -->|No| F[Show Error]
    E -->|Yes| G[Create/Load User Profile]
    
    F --> C
    G --> H[Initialize Estate Data]
    H --> D
    
    D --> I[Main Navigation]
    I --> J[Assets Page]
    I --> K[Heirs Page]
    I --> L[Distributions Page]
    I --> M[Documents Page]
    I --> N[Escrow Page]
    I --> O[Approvals Page]
    I --> P[Claim Page]
    I --> Q[Settings Page]
    
    J --> J1[Add/Edit Assets]
    K --> K1[Add/Edit Heirs]
    L --> L1[Set Distribution Rules]
    M --> M1[Upload Documents]
    N --> N1[Manage Escrow]
    O --> O1[Set ICRC2 Approvals]
    P --> P1[Heir Claim Process]
    Q --> Q1[Configure Settings]
```

## 💰 **ASSET ADDITION FLOW**

```mermaid
graph TD
    A[User Clicks 'Add Asset'] --> B[Asset Form Modal]
    B --> C[Fill Basic Info]
    C --> D[Select Asset Type]
    
    D --> E{Asset Type?}
    E -->|Fungible Token| F[Enter Token Details]
    E -->|NFT| G[Enter NFT Details]
    E -->|Chain Wrapped| H[Enter Wrapped Token Info]
    E -->|Document| I[Upload Document File]
    
    F --> F1[Token Canister ID]
    F1 --> F2[Decimals]
    F2 --> F3[Current Balance]
    
    G --> G1[NFT Canister ID]
    G1 --> G2[Token ID]
    G2 --> G3[NFT Standard]
    
    H --> H1[Wrapped Type - ckBTC/ckETH]
    H1 --> H2[Current Balance]
    H2 --> H3[Decimals]
    
    I --> I1[File Upload Progress]
    I1 --> I2[Encryption & Chunking]
    
    F3 --> J[Validate Input]
    G3 --> J
    H3 --> J
    I2 --> J
    
    J --> K{Validation Passed?}
    K -->|No| L[Show Validation Errors]
    K -->|Yes| M[Submit to Backend]
    
    L --> C
    
    M --> N[Backend Validation]
    N --> O[Create Asset Record]
    O --> P[Generate Asset ID]
    P --> Q[Add to User's Assets]
    Q --> R[Create Audit Event]
    R --> S[Return Success]
    
    S --> T[Update Frontend State]
    T --> U[Show Success Toast]
    U --> V[Refresh Asset List]
    V --> W[Close Modal]
```

## 🎯 **ASSET INHERITANCE FLOW** (Estate Execution)

```mermaid
graph TD
    A[Timer Expires] --> B[Trigger Estate Execution]
    B --> C[Lock Estate]
    C --> D[Validate Estate State]
    
    D --> E{Estate Valid?}
    E -->|No| F[Log Error & Stop]
    E -->|Yes| G[Get All Assets]
    
    G --> H[Process Each Asset]
    H --> I{Asset Type?}
    
    I -->|Escrow| J[Release from Escrow]
    I -->|Approval| K[Transfer via ICRC2]
    I -->|Custody| L[Release from Custody]
    I -->|Document| M[Grant Access Rights]
    
    J --> J1[Get Asset Distribution Rules]
    K --> K1[Get Asset Distribution Rules]
    L --> L1[Get Asset Distribution Rules]
    M --> M1[Get Document Access Rules]
    
    J1 --> J2[Calculate Heir Portions]
    K1 --> K2[Calculate Heir Portions]
    L1 --> L2[Calculate Heir Portions]
    M1 --> M2[Set Heir Document Access]
    
    J2 --> J3{Heir Payout Preference?}
    K2 --> K3{Heir Payout Preference?}
    L2 --> L3{Heir Payout Preference?}
    
    J3 -->|To Principal| J4[Direct Transfer]
    J3 -->|To Custody| J5[Move to Custody]
    J3 -->|CK Withdraw| J6[Stage for Bridge]
    
    K3 -->|To Principal| K4[Direct Transfer]
    K3 -->|To Custody| K5[Move to Custody]
    K3 -->|CK Withdraw| K6[Stage for Bridge]
    
    L3 -->|To Principal| L4[Direct Release]
    L3 -->|To Custody| L5[Keep in Custody]
    L3 -->|CK Withdraw| L6[Stage for Bridge]
    
    J4 --> N[Create Transfer Record]
    J5 --> N
    J6 --> N
    K4 --> N
    K5 --> N
    K6 --> N
    L4 --> N
    L5 --> N
    L6 --> N
    M2 --> N
    
    N --> O[Handle Transfer Failures]
    O --> P{Retry Needed?}
    P -->|Yes| Q[Add to Retry Queue]
    P -->|No| R[Continue Processing]
    
    Q --> S[Adaptive Retry System]
    S --> T[Schedule Next Attempt]
    T --> U[Exponential Backoff]
    
    R --> V{More Assets?}
    V -->|Yes| H
    V -->|No| W[Generate Execution Summary]
    
    W --> X[Send Notifications]
    X --> Y[Create Final Audit Events]
    Y --> Z[Estate Execution Complete]
```

## 👥 **HEIR CLAIM PROCESS FLOW**

```mermaid
graph TD
    A[Heir Receives Claim Link] --> B[Opens Claim Link]
    B --> C[Enter Claim Code]
    C --> D[Backend Validates Code]
    
    D --> E{Valid Code?}
    E -->|No| F[Show Error]
    E -->|Yes| G[Create Heir Session]
    
    F --> C
    G --> H[Enter Identity Secret]
    H --> I[Backend Validates Secret]
    
    I --> J{Secret Valid?}
    J -->|No| K[Increment Attempt Counter]
    J -->|Yes| L[Secret Verified]
    
    K --> M{Max Attempts?}
    M -->|Yes| N[Rate Limit Triggered]
    M -->|No| H
    
    N --> O[Exponential Backoff]
    O --> P[Wait Period]
    P --> H
    
    L --> Q[Optional: Bind Principal]
    Q --> R[Set Payout Preferences]
    R --> S[View Available Assets]
    
    S --> T{Asset in Custody?}
    T -->|Yes| U[Withdraw from Custody]
    T -->|No| V{Need CK Bridge?}
    
    V -->|Yes| W[Request CK Withdraw]
    V -->|No| X[Direct Transfer]
    
    U --> Y[Create Transfer Record]
    W --> W1[Submit to Bridge]
    W1 --> W2[Poll Bridge Status]
    W2 --> Y
    X --> Y
    
    Y --> Z[Update Audit Log]
    Z --> AA[Send Completion Notification]
    AA --> BB[Claim Process Complete]
```

## 📄 **DOCUMENT UPLOAD FLOW**

```mermaid
graph TD
    A[User Selects File] --> B[Validate File Size]
    B --> C{Size Valid?}
    C -->|No| D[Show Size Error]
    C -->|Yes| E[Start Upload Session]
    
    D --> A
    E --> F[Generate Upload ID]
    F --> G[Initialize Progress Bar]
    G --> H[Split File into Chunks]
    
    H --> I[Encrypt Each Chunk]
    I --> J[Upload Chunk]
    J --> K[Update Progress]
    K --> L{More Chunks?}
    
    L -->|Yes| I
    L -->|No| M[Finalize Upload]
    
    M --> N[Backend Validates Checksum]
    N --> O{Checksum Valid?}
    O -->|No| P[Upload Failed]
    O -->|Yes| Q[Create Document Record]
    
    P --> R[Show Error Message]
    R --> S[Cleanup Session]
    
    Q --> T[Generate Document ID]
    T --> U[Add to User Documents]
    U --> V[Create Audit Event]
    V --> W[Show Success Message]
    W --> X[Refresh Document List]
```

## ⚙️ **SYSTEM BACKGROUND PROCESSES**

```mermaid
graph TD
    A[System Timer Tick] --> B[Check Pending Tasks]
    B --> C[Process Retry Queue]
    B --> D[Cleanup Upload Sessions]
    B --> E[Audit Log Maintenance]
    B --> F[Performance Monitoring]
    
    C --> C1[Get Due Retries]
    C1 --> C2[Execute Retry Logic]
    C2 --> C3{Success?}
    C3 -->|Yes| C4[Remove from Queue]
    C3 -->|No| C5[Update Attempt Count]
    C5 --> C6{Max Attempts?}
    C6 -->|Yes| C7[Mark Terminal]
    C6 -->|No| C8[Calculate Next Backoff]
    
    D --> D1[Find Expired Sessions]
    D1 --> D2[Cleanup Session Data]
    D2 --> D3[Create GC Audit Event]
    
    E --> E1[Check Audit Log Size]
    E1 --> E2{Too Large?}
    E2 -->|Yes| E3[Prune Old Events]
    E2 -->|No| E4[Continue]
    E3 --> E5[Create Prune Audit Event]
    
    F --> F1[Collect Metrics]
    F1 --> F2[Analyze Performance]
    F2 --> F3{Alerts Needed?}
    F3 -->|Yes| F4[Generate Alerts]
    F3 -->|No| F5[Update Metrics]
```

## 🔐 **SECURITY & AUTH FLOW**

```mermaid
graph TD
    A[User Action] --> B[Check Authentication]
    B --> C{Authenticated?}
    C -->|No| D[Redirect to Login]
    C -->|Yes| E[Get User Principal]
    
    E --> F[Load User Data]
    F --> G{User Exists?}
    G -->|No| H[Create New User]
    G -->|Yes| I[Validate Permissions]
    
    H --> J[Initialize Estate]
    J --> K[Create Audit Event]
    K --> I
    
    I --> L{Authorized?}
    L -->|No| M[Return Error]
    L -->|Yes| N[Execute Action]
    
    N --> O[Create Audit Event]
    O --> P[Update State]
    P --> Q[Return Success]
```

## 📊 **DATA FLOW ARCHITECTURE**

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│                 │    │                 │    │                 │
│   Frontend      │    │   IC Canister   │    │   External      │
│   (React)       │    │   (Rust)        │    │   Services      │
│                 │    │                 │    │                 │
├─────────────────┤    ├─────────────────┤    ├─────────────────┤
│ • React Router  │    │ • API Modules   │    │ • ICRC1/2 Tokens│
│ • State Mgmt    │───▶│ • Storage Layer │───▶│ • NFT Canisters │
│ • UI Components │    │ • Crypto Utils  │    │ • Bridge Services│
│ • Hooks/API     │    │ • Audit System  │    │ • Email/SMS     │
│ • Type Defs     │    │ • Retry System  │    │ • Price Feeds   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ Internet        │    │ Stable Memory   │    │ External APIs   │
│ Identity        │    │ Storage         │    │ & Integrations  │
│ Authentication  │    │ User Data       │    │ Price Data      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 🎛️ **STATE MANAGEMENT FLOW**

```mermaid
graph LR
    A[User Action] --> B[Component Handler]
    B --> C[API Call]
    C --> D[Backend Processing]
    D --> E[State Update]
    E --> F[Audit Event]
    F --> G[Response]
    G --> H[Frontend State Update]
    H --> I[UI Re-render]
    I --> J[User Feedback]
```

## 📝 **KEY INTEGRATION POINTS**

### **Frontend ↔ Backend**
- **Authentication**: Internet Identity integration
- **API Calls**: Candid interface with type safety
- **Error Handling**: Centralized error normalization
- **State Sync**: Real-time updates via polling/webhooks

### **Backend ↔ External Services**
- **Token Operations**: ICRC1/ICRC2 standard compliance
- **Bridge Services**: ckBTC/ckETH integration
- **Notifications**: Email/SMS delivery (when implemented)
- **Price Feeds**: Asset valuation services

### **Security Boundaries**
- **Principal Isolation**: Each user's data segregated
- **Crypto Operations**: Secure random generation & encryption
- **Audit Trail**: Immutable event logging
- **Rate Limiting**: Protection against abuse

---

## 🔄 **FLOW SUMMARY**

1. **User Entry**: Authentication → Profile Creation → Dashboard
2. **Asset Management**: Add Assets → Set Distributions → Configure Escrow/Approvals
3. **Heir Setup**: Add Heirs → Set Secrets → Create Claim Links
4. **Document Management**: Upload → Encrypt → Store → Index
5. **Estate Execution**: Timer Expiry → Asset Distribution → Notifications
6. **Heir Claims**: Link Access → Secret Verification → Asset Withdrawal
7. **Background**: Retry Processing → Cleanup → Monitoring → Auditing

Each flow includes comprehensive error handling, audit logging, and security validation at every step.

---

_Flow diagrams use Mermaid syntax for visualization_