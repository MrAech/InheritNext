import React from "react";
import { useDemoMode } from "@/context/DemoModeContext";
import { Button } from "@/components/ui/button";

const ResetDemoStateButton: React.FC = () => {
  const { mode } = useDemoMode();

  if (mode !== "evaluator") return null;

  const handleReset = () => {
    window.location.reload();
  };

  return (
    <div className="fixed top-16 right-4 z-50">
      <Button size="sm" variant="destructive" onClick={handleReset}>
        Reset Demo State
      </Button>
    </div>
  );
};

export default ResetDemoStateButton;