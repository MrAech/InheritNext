import React, { createContext, useContext, useState, ReactNode } from "react";

type OverlayType = "none" | "onboarding" | "walkthrough" | "simulation";

interface OverlayManagerContextProps {
  overlay: OverlayType;
  setOverlay: (type: OverlayType) => void;
  overlayProps?: any;
  setOverlayProps: (props: any) => void;
}

const OverlayManagerContext = createContext<OverlayManagerContextProps | undefined>(undefined);

export const useOverlayManager = () => {
  const context = useContext(OverlayManagerContext);
  if (!context) {
    throw new Error("useOverlayManager must be used within an OverlayManagerProvider");
  }
  return context;
};

export const OverlayManagerProvider = ({ children }: { children: ReactNode }) => {
  const [overlay, setOverlay] = useState<OverlayType>("none");
  const [overlayProps, setOverlayProps] = useState<any>(undefined);

  return (
    <OverlayManagerContext.Provider value={{ overlay, setOverlay, overlayProps, setOverlayProps }}>
      {children}
    </OverlayManagerContext.Provider>
  );
};