import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';

export type Theme = 'light' | 'dark';
export type Currency = 'USD' | 'EUR' | 'GBP' | 'JPY' | 'CAD' | 'AUD' | 'INR';

interface SettingsContextType {
  theme: Theme;
  currency: Currency;
  setTheme: (theme: Theme) => void;
  setCurrency: (currency: Currency) => void;
  formatCurrency: (amount: number) => string;
}


const SettingsContext = createContext<SettingsContextType | undefined>(undefined);

export const useSettings = () => {
  const context = useContext(SettingsContext);
  if (context === undefined) {
    throw new Error('useSettings must be used within a SettingsProvider');
  }
  return context;
};

interface SettingsProviderProps {
  children: ReactNode;
}

export const SettingsProvider = ({ children }: SettingsProviderProps) => {
  const [theme, setTheme] = useState<Theme>(() => {
    const stored = localStorage.getItem('theme') as Theme;
    return stored || 'dark';
  });

  const [currency, setCurrency] = useState<Currency>(() => {
    const stored = localStorage.getItem('currency') as Currency;
    return stored || 'INR';
  });
  // Apply theme to document
  useEffect(() => {
    localStorage.setItem('theme', theme);
    if (theme === 'light') {
      document.documentElement.classList.remove('dark');
      document.documentElement.classList.add('light');
    } else {
      document.documentElement.classList.remove('light');
      document.documentElement.classList.add('dark');
    }
  }, [theme]);

  // Store currency preference
  useEffect(() => {
    localStorage.setItem('currency', currency);
  }, [currency]);
  const formatCurrency = (amount: number): string => {
    const currencyMap: Record<Currency, { code: string; symbol: string }> = {
      USD: { code: 'USD', symbol: '$' },
      EUR: { code: 'EUR', symbol: '€' },
      GBP: { code: 'GBP', symbol: '£' },
      JPY: { code: 'JPY', symbol: '¥' },
      CAD: { code: 'CAD', symbol: 'C$' },
      AUD: { code: 'AUD', symbol: 'A$' },
      INR: { code: 'INR', symbol: '₹' },
    };

    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: currencyMap[currency].code,
      minimumFractionDigits: 0,
      maximumFractionDigits: 0,
    }).format(amount);
  };

  return (
    <SettingsContext.Provider value={{
      theme,
      currency,
      setTheme,
      setCurrency,
      formatCurrency
    }}>
      {children}
    </SettingsContext.Provider>
  );
};
