import React, { useState } from "react";
import { useDemoMode } from "@/context/DemoModeContext";
import { useOverlayManager } from "@/context/OverlayManagerContext";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";

const OnboardingOverlay: React.FC = () => {
  const { mode } = useDemoMode();
  const { overlay, setOverlay } = useOverlayManager();

  if (mode !== "evaluator" || overlay !== "onboarding") return null;

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/40">
      <Card className="max-w-lg p-8 shadow-elegant">
        <h2 className="text-2xl font-bold mb-4">Welcome to Evaluator Mode</h2>
        <ul className="list-disc pl-6 mb-4 text-base">
          <li>Switch between demo and normal mode using the toggle in the top-right.</li>
          <li>Hover over features for quick descriptions.</li>
          <li>Click features/pages for detailed explanations and simulated flows.</li>
          <li>Use the "Reset Demo State" button to restart the demo experience.</li>
          <li>Leave feedback via the GitHub link in the footer.</li>
        </ul>
        <Button onClick={() => setOverlay("none")} size="sm" variant="default">
          Got it!
        </Button>
      </Card>
    </div>
  );
};

export default OnboardingOverlay;