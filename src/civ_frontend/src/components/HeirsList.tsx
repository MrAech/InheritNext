import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Progress } from "@/components/ui/progress";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter, DialogHeader, DialogTitle, DialogTrigger
} from "@/components/ui/dialog";
import {
  Edit,
  Trash2,
  Plus,
  User,
  Users,
  Heart,
  Phone,
  Mail
} from "lucide-react";
import { useToast } from "@/hooks/use-toast";
import {
  listHeirs,
  addHeir,
  updateHeir,
  removeHeir
} from "@/lib/api";
import type { Heir, HeirInput } from "@/types/backend";

interface HeirsListProps {
  onHeirsChange?: (heirs: Heir[]) => void;
}

const HeirsList = ({ onHeirsChange }: HeirsListProps = {}) => {
  const [heirs, setHeirs] = useState<Heir[]>([]);
  const [loading, setLoading] = useState(true);
  const [editingHeir, setEditingHeir] = useState<Heir | null>(null);
  const [isAddingHeir, setIsAddingHeir] = useState(false);
  const { toast } = useToast();

  useEffect(() => {
    async function fetchHeirs() {
      setLoading(true);
      try {
        const data = await listHeirs();
        setHeirs(data);
        onHeirsChange?.(data);
      } catch (err) {
        toast({
          title: "Error loading heirs",
          description: String(err),
          variant: "destructive",
        });
      } finally {
        setLoading(false);
      }
    }
    fetchHeirs();
    // eslint-disable-next-line react-hooks/exhaustive-deps 
  }, []);

  const handleUpdateHeir = async (updatedHeir: Heir) => {
    setLoading(true);
    try {
      const req: HeirInput = {
        name: updatedHeir.name,
        relationship: updatedHeir.relationship,
        email: updatedHeir.email,
        phone: updatedHeir.phone,
        address: updatedHeir.address,
      };
      const ok = await updateHeir(updatedHeir.id, req);
      if (ok) {
        const data = await listHeirs();
        setHeirs(data);
        onHeirsChange?.(data);
        setEditingHeir(null);
        toast({
          title: "Heir Updated",
          description: `${updatedHeir.name}'s information has been successfully updated.`,
        });
      } else {
        toast({
          title: "Update Failed",
          description: "Could not update heir.",
          variant: "destructive",
        });
      }
    } catch (err) {
      toast({
        title: "Error updating heir",
        description: String(err),
        variant: "destructive",
      });
    } finally {
      setLoading(false);
    }
  };

  const handleRemoveHeir = async (heirId: number) => {
    setLoading(true);
    try {
      const ok = await removeHeir(heirId);
      if (ok) {
        const data = await listHeirs();
        setHeirs(data);
        onHeirsChange?.(data);
        toast({
          title: "Heir Removed",
          description: `Heir has been removed from the beneficiaries.`,
          variant: "destructive",
        });
      } else {
        toast({
          title: "Remove Failed",
          description: "Could not remove heir.",
          variant: "destructive",
        });
      }
    } catch (err) {
      toast({
        title: "Error removing heir",
        description: String(err),
        variant: "destructive",
      });
    } finally {
      setLoading(false);
    }
  };

  const handleAddHeir = async (newHeir: HeirInput) => {
    setLoading(true);
    try {
      const ok = await addHeir(newHeir);
      if (ok) {
        const data = await listHeirs();
        setHeirs(data);
        onHeirsChange?.(data);
        setIsAddingHeir(false);
        toast({
          title: "Heir Added",
          description: `${newHeir.name} has been added as a beneficiary.`,
        });
      } else {
        toast({
          title: "Add Failed",
          description: "Could not add heir.",
          variant: "destructive",
        });
      }
    } catch (err) {
      toast({
        title: "Error adding heir",
        description: String(err),
        variant: "destructive",
      });
    } finally {
      setLoading(false);
    }
  };

  const getRelationshipIcon = (relationship: string) => {
    switch (relationship) {
      case "Charity":
        return <Heart className="w-5 h-5" />;
      case "Spouse":
        return <Users className="w-5 h-5" />;
      default:
        return <User className="w-5 h-5" />;
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <p className="text-muted-foreground">Manage beneficiaries and their inheritance relationships</p>
        <Dialog open={isAddingHeir} onOpenChange={setIsAddingHeir}>
          <DialogTrigger asChild>
            <Button size="sm" className="bg-gradient-success">
              <Plus className="w-4 h-4 mr-2" />
              Add Heir
            </Button>
          </DialogTrigger>
          <HeirFormDialog
            onSubmit={handleAddHeir}
            onCancel={() => setIsAddingHeir(false)}
          />
        </Dialog>
      </div>

      {loading ? (
        <div className="text-center text-muted-foreground py-8">Loading heirs...</div>
      ) : (
        <div className="grid gap-4 md:grid-cols-2">
          {heirs.map((heir) => (
            <Card key={heir.id} className="shadow-card hover:shadow-elegant transition-shadow">
              <CardHeader className="pb-3">
                <div className="flex items-start justify-between">
                  <div className="flex items-center space-x-3">
                    <div className="p-2 bg-primary/10 rounded-lg text-primary">
                      {getRelationshipIcon(heir.relationship)}
                    </div>
                    <div>
                      <CardTitle className="text-lg">{heir.name}</CardTitle>
                      <CardDescription>
                        <Badge
                          variant="outline"
                        >
                          {heir.relationship}
                        </Badge>
                      </CardDescription>
                    </div>
                  </div>
                  <div className="text-right">
                    <div className="text-sm text-muted-foreground">{heir.email}</div>
                    <div className="text-sm text-muted-foreground">{heir.phone}</div>
                    <div className="text-sm text-muted-foreground">{heir.address}</div>
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {/* Additional fields for future */}
                </div>
              </CardContent>
              <div className="flex space-x-2 px-4 pb-4">
                <Dialog open={editingHeir?.id === heir.id} onOpenChange={(open) => !open && setEditingHeir(null)}>
                  <DialogTrigger asChild>
                    <Button
                      variant="outline"
                      size="sm"
                      className="flex-1"
                      onClick={() => setEditingHeir(heir)}
                    >
                      <Edit className="w-4 h-4 mr-2" />
                      Update
                    </Button>
                  </DialogTrigger>
                  {editingHeir && (
                    <HeirFormDialog
                      heir={editingHeir}
                      onSubmit={handleUpdateHeir}
                      onCancel={() => setEditingHeir(null)}
                      isEditing
                    />
                  )}
                </Dialog>
                <Button
                  variant="outline"
                  size="sm"
                  className="flex-1 text-destructive hover:bg-destructive hover:text-destructive-foreground"
                  onClick={() => handleRemoveHeir(heir.id)}
                >
                  <Trash2 className="w-4 h-4 mr-2" />
                  Remove
                </Button>
              </div>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
};

interface HeirFormDialogProps {
  heir?: Heir;
  onSubmit: (heir: Heir | HeirInput) => void;
  onCancel: () => void;
  isEditing?: boolean;
}

const HeirFormDialog = ({ heir, onSubmit, onCancel, isEditing = false }: HeirFormDialogProps) => {
  type HeirFormState = {
    name: string;
    relationship: string;
    email: string;
    phone: string;
    address: string;
    aadhaar: string;
  };

  const [formData, setFormData] = useState<HeirFormState>({
    name: heir?.name || "",
    relationship: heir?.relationship || "",
    email: heir?.email || "",
    phone: heir?.phone || "",
    address: heir?.address || "",
    aadhaar: "", // raw input local-only; will only send hashed form
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    (async () => {
      if (isEditing && heir) {
        onSubmit({ ...heir, ...formData });
        return;
      }
      // For new heirs: if aadhaar provided, generate salt and hash
  const rawAadhaar = formData.aadhaar;
      let salt: string | null = null;
      let hashed: string | null = null;
      if (rawAadhaar && typeof rawAadhaar === 'string' && rawAadhaar.trim().length > 0) {
        // generate random 16-byte salt, hex encoded
        const saltBytes = crypto.getRandomValues(new Uint8Array(16));
        salt = Array.from(saltBytes).map(b => b.toString(16).padStart(2, '0')).join('');
        // compute sha256(salt + aadhaar)
        const encoder = new TextEncoder();
        const data = encoder.encode(salt + rawAadhaar.trim());
        const digest = await crypto.subtle.digest('SHA-256', data);
        hashed = Array.from(new Uint8Array(digest)).map(b => b.toString(16).padStart(2, '0')).join('');
      }
      const payload: HeirInput = {
        name: formData.name,
        relationship: formData.relationship,
        email: formData.email,
        phone: formData.phone,
        address: formData.address,
        salt: salt ?? undefined,
        adhaarnum: hashed ?? undefined,
      };
      onSubmit(payload);
    })();
  };

  return (
    <DialogContent className="sm:max-w-md max-h-[80vh] overflow-y-auto">
      <DialogHeader>
        <DialogTitle>
          {isEditing ? "Update Heir" : "Add New Heir"}
        </DialogTitle>
        <DialogDescription>
          {isEditing ? "Modify the heir details below." : "Enter the details for the new beneficiary."}
        </DialogDescription>
      </DialogHeader>
      <form onSubmit={handleSubmit} className="space-y-4">
        <div className="space-y-2">
          <Label htmlFor="name">Full Name</Label>
          <Input
            id="name"
            value={formData.name}
            onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
            placeholder="Enter full name"
            required
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="email">Email</Label>
          <Input
            id="email"
            value={formData.email}
            onChange={(e) => setFormData(prev => ({ ...prev, email: e.target.value }))}
            placeholder="Enter email"
            required
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="phone">Phone</Label>
          <Input
            id="phone"
            value={formData.phone}
            onChange={(e) => setFormData(prev => ({ ...prev, phone: e.target.value }))}
            placeholder="Enter phone"
            required
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="address">Address</Label>
          <Input
            id="address"
            value={formData.address}
            onChange={(e) => setFormData(prev => ({ ...prev, address: e.target.value }))}
            placeholder="Enter address"
            required
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="relationship">Relationship</Label>
          <Input
            id="relationship"
            value={formData.relationship}
            onChange={(e) => setFormData(prev => ({ ...prev, relationship: e.target.value }))}
            placeholder="e.g., Son, Daughter, Spouse, Charity"
            required
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="aadhaar">Aadhaar Number (optional)</Label>
          <Input
            id="aadhaar"
            value={formData.aadhaar}
            onChange={(e) => setFormData(prev => ({ ...prev, aadhaar: e.target.value }))}
            placeholder="Enter Aadhaar number (will be hashed)"
          />
          <div className="text-xs text-muted-foreground">If provided, the Aadhaar will be salted & hashed in your browser; raw value is not sent to the server.</div>
        </div>
        <DialogFooter>
          <Button type="button" variant="outline" onClick={onCancel}>
            Cancel
          </Button>
          <Button type="submit" className="bg-gradient-primary">
            {isEditing ? "Update Heir" : "Add Heir"}
          </Button>
        </DialogFooter>
      </form>
    </DialogContent>
  );
};

export { HeirsList };
