import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle
} from "@/components/ui/dialog";
import { AlertTriangle, RefreshCw } from "lucide-react";

interface TimerResetDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: () => void;
}

const TimerResetDialog = ({ open, onOpenChange, onConfirm }: TimerResetDialogProps) => {
  const handleConfirm = () => {
    onConfirm();
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <RefreshCw className="w-5 h-5 text-primary" />
            Reset Dashboard Timer
          </DialogTitle>
          <DialogDescription className="space-y-2">
            <div className="flex items-start gap-2">
              <AlertTriangle className="w-4 h-4 text-warning mt-0.5 flex-shrink-0" />
              <span>
                This action will reset the dashboard timer to the current time.
                This operation cannot be undone and will update the "Last reset" timestamp.
              </span>
            </div>
          </DialogDescription>
        </DialogHeader>
        <DialogFooter className="flex gap-2 sm:gap-0">
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
          >
            Cancel
          </Button>
          <Button
            onClick={handleConfirm}
            className="bg-gradient-primary"
          >
            <RefreshCw className="w-4 h-4 mr-2" />
            Reset Timer
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export { TimerResetDialog };