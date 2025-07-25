import React from "react";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { useOverlayManager } from "@/context/OverlayManagerContext";

const SimulationModal: React.FC<{ title: string; description: string }> = ({ title, description }) => {
  const { overlay, setOverlay } = useOverlayManager();

  if (overlay !== "simulation") return null;

  return (
    <div className="fixed inset-0 z-[102] flex items-center justify-center bg-black/40">
      <Card className="max-w-md p-6 shadow-elegant">
        <h2 className="text-xl font-bold mb-4">{title}</h2>
        <p className="mb-6 text-base">{description}</p>
        <Button size="sm" variant="default" onClick={() => setOverlay("none")}>
          Close
        </Button>
      </Card>
    </div>
  );
};

export default SimulationModal;