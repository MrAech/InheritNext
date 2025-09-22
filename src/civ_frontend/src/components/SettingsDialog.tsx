import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Settings, Palette, DollarSign } from "lucide-react";
import { useSettings, Theme, Currency } from "@/context/SettingsContext";
import { useToast } from "@/hooks/use-toast";
import { useAuth } from "@/context/AuthContext";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";

const SettingsDialog = () => {
  const { theme, currency, setTheme, setCurrency } = useSettings();
  const [isOpen, setIsOpen] = useState(false);
  const [tempTheme, setTempTheme] = useState<Theme>(theme);
  const [tempCurrency, setTempCurrency] = useState<Currency>(currency);
  const { toast } = useToast();
  const { actor } = useAuth();

  const handleRotateSalt = async () => {
    if (!actor) return;
    if (!confirm("Rotating your salt will invalidate previously-derived heir hashes. Are you sure?")) return;
    try {
      const newSalt = await actor.rotate_user_salt();
      toast({ title: "Salt Rotated", description: "Your per-user salt was rotated. You must re-register heirs or update derived hashes." });
      console.log("new salt:", newSalt);
    } catch (e) {
      toast({ title: "Error", description: "Failed to rotate salt.", variant: "destructive" });
    }
  };

  const handleSave = () => {
    setTheme(tempTheme);
    setCurrency(tempCurrency);
    setIsOpen(false);
    toast({
      title: "Settings Updated",
      description: "Your preferences have been saved successfully.",
    });
  };

  const handleCancel = () => {
    setTempTheme(theme);
    setTempCurrency(currency);
    setIsOpen(false);
  };

  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <DialogTrigger asChild>
        <Button variant="outline" size="sm">
          <Settings className="w-4 h-4 mr-2" />
          Settings
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Settings</DialogTitle>
          <DialogDescription>
            Customize your app preferences and display options.
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-6 py-4">
          {/* Theme Setting */}
          <div className="space-y-2">
            <Label className="flex items-center gap-2">
              <Palette className="w-4 h-4" />
              Theme
            </Label>
            <Select
              value={tempTheme}
              onValueChange={(value) => setTempTheme(value as Theme)}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select theme" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="dark">Dark</SelectItem>
                <SelectItem value="light">Light</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Currency Setting */}
          <div className="space-y-2">
            <Label className="flex items-center gap-2">
              <DollarSign className="w-4 h-4" />
              Currency
            </Label>
            <Select
              value={tempCurrency}
              onValueChange={(value) => setTempCurrency(value as Currency)}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select currency" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="INR">INR - Indian Rupee (₹)</SelectItem>
                <SelectItem value="USD">USD - US Dollar ($)</SelectItem>
                <SelectItem value="EUR">EUR - Euro (€)</SelectItem>
                <SelectItem value="GBP">GBP - British Pound (£)</SelectItem>
                <SelectItem value="JPY">JPY - Japanese Yen (¥)</SelectItem>
                <SelectItem value="CAD">CAD - Canadian Dollar (C$)</SelectItem>
                <SelectItem value="AUD">
                  AUD - Australian Dollar (A$)
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label>Security / Salt Rotation</Label>
            <div className="flex items-center gap-2">
              <Button variant="destructive" size="sm" onClick={handleRotateSalt}>
                Rotate My Salt
              </Button>
              <div className="text-sm text-muted-foreground">Rotating salt invalidates previously-registered heirs derived from the old salt.</div>
            </div>
          </div>
        </div>
        <DialogFooter>
          <Button
            type="button"
            variant="outline"
            onClick={handleCancel}
          ></Button>
          <Button
            type="submit"
            onClick={handleSave}
            className="bg-gradient-primary"
          >
            Save Changes
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export { SettingsDialog };
