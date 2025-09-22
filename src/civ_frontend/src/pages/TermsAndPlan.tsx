import { useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { useAuth } from "@/context/AuthContext";
import { useToast } from "@/hooks/use-toast";

const TermsAndPlan = () => {
  const { actor } = useAuth();
  const { toast } = useToast();
  const [accepted, setAccepted] = useState(false);
  const [plan, setPlan] = useState<string>("");
  const [loading, setLoading] = useState(false);

  const handleAcceptTerms = async () => {
    if (!actor) return;
    setLoading(true);
    const result = await actor.accept_terms();
    if ("Ok" in result) {
      setAccepted(true);
      toast({
        title: "Terms Accepted",
        description: "You have accepted the terms.",
      });
    } else {
      toast({
        title: "Error",
        description: "Failed to accept terms.",
        variant: "destructive",
      });
    }
    setLoading(false);
  };

  const handleSelectPlan = async (planType: "Basic" | "Tier1" | "Custom") => {
    if (!actor) return;
    setLoading(true);
    let planArg: any = { Basic: null };
    if (planType === "Tier1") planArg = { Tier1: null };
    if (planType === "Custom") planArg = { Custom: null };
    const result = await actor.select_plan(planArg);
    if ("Ok" in result) {
      setPlan(planType);
      toast({
        title: "Plan Selected",
        description: `You have selected the ${planType} plan.`,
      });
    } else {
      toast({
        title: "Error",
        description: "Failed to select plan.",
        variant: "destructive",
      });
    }
    setLoading(false);
  };

  return (
    <div className="flex flex-col items-center justify-center min-h-screen p-4">
      <Card className="max-w-lg w-full">
        <CardHeader>
          <CardTitle>Terms of Service</CardTitle>
          <CardDescription>
            Please read and accept the terms to continue.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="mb-4">
            <p className="text-sm text-muted-foreground">
              By using this service, you agree to the terms and conditions of
              InheritNext. (Full terms text...)
            </p>
          </div>
          <Button
            onClick={handleAcceptTerms}
            disabled={accepted || loading}
            className="w-full mb-4"
          >
            {accepted
              ? "Terms Accepted"
              : loading
                ? "Accepting..."
                : "Accept Terms"}
          </Button>
          {accepted && (
            <>
              <div className="mb-2 font-semibold">Select a Plan:</div>
              <div className="flex gap-2">
                <Button
                  variant={plan === "Basic" ? "default" : "outline"}
                  onClick={() => handleSelectPlan("Basic")}
                  disabled={loading}
                >
                  Basic
                </Button>
                <Button
                  variant={plan === "Tier1" ? "default" : "outline"}
                  onClick={() => handleSelectPlan("Tier1")}
                  disabled={loading}
                >
                  Tier 1
                </Button>
                <Button
                  variant={plan === "Custom" ? "default" : "outline"}
                  onClick={() => handleSelectPlan("Custom")}
                  disabled={loading}
                >
                  Custom
                </Button>
              </div>
              {plan && (
                <div className="mt-4 text-green-600">Selected Plan: {plan}</div>
              )}
            </>
          )}
        </CardContent>
      </Card>
    </div>
  );
};

export default TermsAndPlan;
