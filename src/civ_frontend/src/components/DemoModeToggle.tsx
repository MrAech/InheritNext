import React from "react";
import { useDemoMode } from "@/context/DemoModeContext";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

const DemoModeToggle: React.FC = () => {
  const { mode, setMode } = useDemoMode();

  const toggleMode = () => {
    setMode(mode === "normal" ? "evaluator" : "normal");
  };

  return (
    <div className="fixed top-4 right-4 z-50 flex items-center gap-2">
      <Badge variant={mode === "evaluator" ? "default" : "secondary"}>
        {mode === "evaluator" ? "Evaluator Mode" : "Normal Mode"}
      </Badge>
      <Button size="sm" variant="outline" onClick={toggleMode}>
        Switch Mode
      </Button>
    </div>
  );
};

export default DemoModeToggle;