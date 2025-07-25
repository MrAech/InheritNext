import { Toaster } from "@/components/ui/toaster";
import { Toaster as Sonner } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { SettingsProvider } from "@/context/SettingsContext";
import SignIn from "@/pages/SignIn";
import Dashboard from "@/pages/Dashboard";
import ProtectedRoute from "@/components/ProtectedRoute";
import '@/App.css';

const queryClient = new QueryClient();

import { AuthProvider } from "@/context/AuthContext";

import { DemoModeProvider } from "@/context/DemoModeContext";
import DemoModeToggle from "@/components/DemoModeToggle";
import OnboardingOverlay from "@/components/OnboardingOverlay";
import ResetDemoStateButton from "@/components/ResetDemoStateButton";
import AddAssetPage from "@/pages/AddAssetPage";
import AddHeirPage from "@/pages/AddHeirPage";
import GuidedWalkthrough from "@/components/GuidedWalkthrough";
import FeedbackFooter from "@/components/FeedbackFooter";
import { OverlayManagerProvider } from "@/context/OverlayManagerContext";
import SimulationModal from "@/components/SimulationModal";
import { SimulatedDataProvider } from "@/context/SimulatedDataContext";

const App = () => (
  <QueryClientProvider client={queryClient}>
    <AuthProvider>
      <SettingsProvider>
        <DemoModeProvider>
          <SimulatedDataProvider>
            <OverlayManagerProvider>
              <TooltipProvider>
                <Toaster />
                <Sonner />
                <DemoModeToggle />
                <OnboardingOverlay />
                <GuidedWalkthrough />
                <SimulationModal title="Simulation Explanation" description="This is a simulated flow. In production, this would perform real validation and actions." />
                <ResetDemoStateButton />
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
                    <Route
                      path="/add-asset"
                      element={
                        <ProtectedRoute>
                          <AddAssetPage />
                        </ProtectedRoute>
                      }
                    />
                    <Route
                      path="/add-heir"
                      element={
                        <ProtectedRoute>
                          <AddHeirPage />
                        </ProtectedRoute>
                      }
                    />
                    {/* Catch-all route: redirect to sign-in for any invalid path */}
                    <Route path="*" element={<SignIn />} />
                  </Routes>
                </BrowserRouter>
                <FeedbackFooter />
              </TooltipProvider>
            </OverlayManagerProvider>
          </SimulatedDataProvider>
        </DemoModeProvider>
      </SettingsProvider>
    </AuthProvider>
  </QueryClientProvider>
);

export default App;
