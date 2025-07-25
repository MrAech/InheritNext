import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Shield, Lock } from "lucide-react";
import { useToast } from "@/hooks/use-toast";
import { useAuth } from "@/context/AuthContext";

import { useDemoMode } from "@/context/DemoModeContext";
import { useOverlayManager } from "@/context/OverlayManagerContext";

const SignIn = () => {
  const { login, isAuthenticated } = useAuth();
  const navigate = useNavigate();
  const { toast } = useToast();
  const { mode } = useDemoMode();
  const { setOverlay } = useOverlayManager();

  useEffect(() => {
    if (isAuthenticated) {
      navigate("/dashboard", { replace: true });
    }
  }, [isAuthenticated, navigate]);

  useEffect(() => {
    if (mode === "evaluator") {
      setOverlay("onboarding");
    }
  }, [mode, setOverlay]);

  const handleLogin = async () => {
    await login();
    toast({
      title: "Welcome to InheritNext!",
      description: "You have successfully signed in.",
    });
  };

  return (
    <div className="min-h-screen bg-gradient-hero flex items-center justify-center p-4">
      <div className="w-full max-w-md animate-fade-in">
        <div className="text-center mb-8">
          <div className="mx-auto w-16 h-16 bg-primary/10 rounded-2xl flex items-center justify-center mb-4">
            <Shield className="w-8 h-8 text-primary" />
          </div>
          <h1 className="text-3xl font-bold dark:text-white light:text-black mb-2">
            InheritNext
          </h1>
          <p className="dark:text-white/80 light:text-black/80">
            Secure Inheritance management platform
            Dont forget about Evaluator Mode top right 
          </p>
        </div>

        <Card className="shadow-elegant border-0">
          <CardHeader className="text-center">
            <CardTitle className="flex items-center justify-center gap-2">
              <Lock className="w-5 h-5" />
              Sign In
            </CardTitle>
            <CardDescription>
              Use Internet Identity to access your dashboard
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Button 
              onClick={handleLogin}
              className="w-full bg-gradient-primary hover:scale-105 transition-transform"
            >
              Sign In with Internet Identity
            </Button>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};

export default SignIn;