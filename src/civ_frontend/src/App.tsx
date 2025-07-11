
// import React, { useState, useEffect } from 'react';
// import { AuthClient } from '@dfinity/auth-client';
// import { createActor } from 'declarations/civ_backend';
// import { canisterId } from 'declarations/civ_backend';
// import './App.css';

// const network = process.env.DFX_NETWORK;
// const identityProvider =
//   network === 'ic'
//     ? 'https://identity.ic0.app'
//     : `http://${process.env.CANISTER_ID_INTERNET_IDENTITY}.localhost:4943/`;

// function App() {
//   const [state, setState] = useState({
//     actor: undefined,
//     authClient: undefined,
//     isAuthenticated: false
//   });

//   const updateActor = async () => {
//     const authClient = await AuthClient.create();
//     const identity = authClient.getIdentity();
//     const actor = createActor(canisterId, { agentOptions: { identity } });
//     const isAuthenticated = await authClient.isAuthenticated();
//     setState({ authClient, actor, isAuthenticated });
//   };

//   useEffect(() => {
//     updateActor();
//   }, []);

//   const login = async () => {
//     await state.authClient.login({
//       identityProvider,
//       onSuccess: updateActor
//     });
//   };

//   const logout = async () => {
//     await state.authClient.logout();
//     updateActor();
//   };

//   return (
//     <div className="wrapper">
//       {!state.isAuthenticated ? (
//         <>
//           <div className="header">
//             <div className="icon">🛡️</div>
//             <h1 className="title">InheritNext</h1>
//             <p className="subtitle">Secure estate management platform</p>
//           </div>

//           <div className="login-box">
//             <h2 className="login-heading">🔐 Sign In</h2>
//             <p className="info">Use Internet Identity to access your dashboard</p>
//             <button className="login-button" onClick={login}>
//               Sign In with Internet Identity
//             </button>
//           </div>
//         </>
//       ) : (
//         <div className="dashboard">
//           <h1>Welcome to CIV</h1>
//           <button className="logout-button" onClick={logout}>
//             Logout
//           </button>
//         </div>
//       )}
//     </div>
//   );
// }

// export default App;


import { Toaster } from "@/components/ui/toaster";
import { Toaster as Sonner } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import SignIn from "./pages/SignIn";
import Dashboard from "./pages/Dashboard";
import ProtectedRoute from "@/components/ProtectedRoute";
import './App.css';

const queryClient = new QueryClient();

const App = () => (
  <QueryClientProvider client={queryClient}>
    <TooltipProvider>
      <Toaster />
      <Sonner />
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<SignIn />} />
          <Route 
            path="/dashboard" 
            element={
              <ProtectedRoute>
                <Dashboard />
              </ProtectedRoute>
            } 
          />
          {/* ADD ALL CUSTOM ROUTES ABOVE THE CATCH-ALL "*" ROUTE */}
          <Route path="*" element={<SignIn />} />
        </Routes>
      </BrowserRouter>
    </TooltipProvider>
  </QueryClientProvider>
);

export default App;
