import { Toaster } from "@/components/ui/toaster";
import { Toaster as Sonner } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { SettingsProvider } from "@/context/SettingsContext";
import SignIn from "@/pages/SignIn";
import Dashboard from "@/pages/Dashboard";
import ProtectedRoute from "@/components/ProtectedRoute";
import DistributionsPage from "@/pages/Distributions";
// Pages (some will reuse existing components or be added)
import AssetsPage from "@/pages/Assets";
import HeirsPage from "@/pages/Heirs";
import DocumentsPage from "@/pages/Documents";
import EscrowPage from "@/pages/Escrow";
import ApprovalsPage from "@/pages/Approvals";
import ClaimPage from "@/pages/Claim";
import SettingsPage from "@/pages/Settings";
import AuditLogPage from "@/pages/AuditLog";
import Navbar from "@/components/Navbar";
import Sidebar from "@/components/Sidebar";
import AuthExpiryBanner from "@/components/AuthExpiryBanner";
import '@/App.css';

const queryClient = new QueryClient();

import { AuthProvider } from "@/context/AuthContext";

const App = () => (
  <QueryClientProvider client={queryClient}>
    <AuthProvider>
      <SettingsProvider>
        <TooltipProvider>
          <Toaster />
          <Sonner />
          <AuthExpiryBanner />
          <BrowserRouter>
            {/* Main application routes */}
            <Routes>
              <Route path="/" element={<SignIn />} />

              {/* Protected routes wrapped with layout */}
              <Route
                path="/dashboard"
                element={
                  <ProtectedRoute>
                    <MainLayout>
                      <Dashboard />
                    </MainLayout>
                  </ProtectedRoute>
                }
              />

              <Route
                path="/assets"
                element={
                  <ProtectedRoute>
                    <MainLayout>
                      <AssetsPage />
                    </MainLayout>
                  </ProtectedRoute>
                }
              />

              <Route
                path="/heirs"
                element={
                  <ProtectedRoute>
                    <MainLayout>
                      <HeirsPage />
                    </MainLayout>
                  </ProtectedRoute>
                }
              />

              <Route
                path="/distributions"
                element={
                  <ProtectedRoute>
                    <MainLayout>
                      <DistributionsPage />
                    </MainLayout>
                  </ProtectedRoute>
                }
              />

              <Route
                path="/documents"
                element={
                  <ProtectedRoute>
                    <MainLayout>
                      <DocumentsPage />
                    </MainLayout>
                  </ProtectedRoute>
                }
              />

              <Route
                path="/escrow"
                element={
                  <ProtectedRoute>
                    <MainLayout>
                      <EscrowPage />
                    </MainLayout>
                  </ProtectedRoute>
                }
              />

              <Route
                path="/approvals"
                element={
                  <ProtectedRoute>
                    <MainLayout>
                      <ApprovalsPage />
                    </MainLayout>
                  </ProtectedRoute>
                }
              />

              <Route
                path="/claim"
                element={
                  <ProtectedRoute>
                    <MainLayout>
                      <ClaimPage />
                    </MainLayout>
                  </ProtectedRoute>
                }
              />

              <Route
                path="/settings"
                element={
                  <ProtectedRoute>
                    <MainLayout>
                      <SettingsPage />
                    </MainLayout>
                  </ProtectedRoute>
                }
              />

              <Route
                path="/audit"
                element={
                  <ProtectedRoute>
                    <MainLayout>
                      <AuditLogPage />
                    </MainLayout>
                  </ProtectedRoute>
                }
              />

              {/* Catch-all route: redirect to sign-in for any invalid path */}
              <Route path="*" element={<SignIn />} />
            </Routes>
          </BrowserRouter>
        </TooltipProvider>
      </SettingsProvider>
    </AuthProvider>
  </QueryClientProvider>
);

export default App;

// Simple MainLayout showing Navbar + Sidebar and content area.
function MainLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="min-h-screen flex bg-background">
      <aside className="w-56 min-w-0 border-r bg-card sticky top-0 h-screen flex-shrink-0">
        <Sidebar />
      </aside>
      <div className="flex-1 flex flex-col">
        <header className="border-b">
          <Navbar />
        </header>
        <main className="flex-1 overflow-auto">{children}</main>
      </div>
    </div>
  );
}
