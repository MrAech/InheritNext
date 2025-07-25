import React, { createContext, useContext, useState, ReactNode } from "react";

type DemoMode = "normal" | "evaluator";

interface DemoModeContextProps {
  mode: DemoMode;
  setMode: (mode: DemoMode) => void;
}

const DemoModeContext = createContext<DemoModeContextProps | undefined>(undefined);

export const useDemoMode = () => {
  const context = useContext(DemoModeContext);
  if (!context) {
    throw new Error("useDemoMode must be used within a DemoModeProvider");
  }
  return context;
};

export const DemoModeProvider = ({ children }: { children: ReactNode }) => {
  const [mode, setModeState] = useState<DemoMode>(() => {
    const stored = localStorage.getItem("demoMode");
    return stored === "evaluator" ? "evaluator" : "normal";
  });

  const setMode = (newMode: DemoMode) => {
    setModeState(newMode);
    localStorage.setItem("demoMode", newMode);
  };

  // Sync with localStorage changes (e.g., other tabs/windows)
  React.useEffect(() => {
    const handleStorage = (event: StorageEvent) => {
      if (event.key === "demoMode") {
        setModeState(event.newValue === "evaluator" ? "evaluator" : "normal");
      }
    };
    window.addEventListener("storage", handleStorage);
    return () => window.removeEventListener("storage", handleStorage);
  }, []);

  return (
    <DemoModeContext.Provider value={{ mode, setMode }}>
      {children}
    </DemoModeContext.Provider>
  );
};