import React, { useState } from "react";
import { useDemoMode } from "@/context/DemoModeContext";
import { useOverlayManager } from "@/context/OverlayManagerContext";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";

const steps = [
  {
    title: "Welcome to Evaluator Mode",
    description: "This walkthrough will guide you through the main features and simulated flows of the app.",
  },
  {
    title: "Add Asset",
    description: "Use the 'Add Asset (Simulated)' button on the Dashboard to open the asset addition page. Here you can enter asset details and simulate tokenization/KYC.",
  },
  {
    title: "Add Heir",
    description: "Use the 'Add Heir (Simulated)' button on the Dashboard to open the heir addition page. Here you can enter heir details and simulate KYC/II connection.",
  },
  {
    title: "Simulation Controls",
    description: "On asset/heir pages, use simulation controls to toggle success/failure for tokenization, KYC, and II connection. Click badges for explanations.",
  },
  {
    title: "Feedback",
    description: "Leave feedback via the GitHub link in the footer.",
  },
];

const GuidedWalkthrough: React.FC = () => {
  const { mode } = useDemoMode();
  const { overlay, setOverlay } = useOverlayManager();
  const [step, setStep] = useState(0);

  if (mode !== "evaluator" || overlay !== "walkthrough") return null;

  const nextStep = () => setStep((prev) => Math.min(prev + 1, steps.length - 1));
  const prevStep = () => setStep((prev) => Math.max(prev - 1, 0));
  const close = () => setOverlay("none");

  return (
    <div className="fixed inset-0 z-[101] flex items-center justify-center bg-black/40">
      <Card className="max-w-lg p-8 shadow-elegant">
        <h2 className="text-2xl font-bold mb-4">{steps[step].title}</h2>
        <p className="mb-6 text-base">{steps[step].description}</p>
        <div className="flex justify-between">
          <Button size="sm" variant="outline" onClick={prevStep} disabled={step === 0}>
            Previous
          </Button>
          <Button size="sm" variant="default" onClick={nextStep} disabled={step === steps.length - 1}>
            Next
          </Button>
          <Button size="sm" variant="destructive" onClick={close}>
            Close
          </Button>
        </div>
      </Card>
    </div>
  );
};

export default GuidedWalkthrough;