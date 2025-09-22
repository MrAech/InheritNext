import { useState, useEffect } from "react";
import { useAuth } from "@/context/AuthContext";
import { pbkdf2Hex } from "@/lib/hash";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Progress } from "@/components/ui/progress";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  Edit,
  Trash2,
  Plus,
  User,
  Users,
  Heart,
  Phone,
  Mail,
} from "lucide-react";

import { useToast } from "@/hooks/use-toast";

interface Heir {
  name: string;
  gov_id_hash: string;
  security_question_hash?: string;
}

interface HeirsListProps {
  onHeirsChange?: (heirs: Heir[]) => void;
}

const HeirsList = ({ onHeirsChange }: HeirsListProps = {}) => {
  const { actor } = useAuth();
  const [heirs, setHeirs] = useState<Heir[]>([]);
  const [loading, setLoading] = useState(false);

  const [editingHeir, setEditingHeir] = useState<Heir | null>(null);
  const [isAddingHeir, setIsAddingHeir] = useState(false);
  const { toast } = useToast();

  // Fetch heirs from canister on mount
  useEffect(() => {
    if (!actor) return;
    setLoading(true);
    actor
      .get_user_state()
      .then((userState) => {
        // actor returns opt UserState ([] | [UserState])
        const us = userState.length > 0 ? userState[0] : null;
        const heirListRaw = us ? us.heirs : [];
        const heirList = heirListRaw.map((h) => ({
          name: h.name,
          gov_id_hash: h.gov_id_hash,
          security_question_hash:
            h.security_question_hash && h.security_question_hash.length > 0
              ? h.security_question_hash[0]
              : "",
        }));
        setHeirs(heirList);
        onHeirsChange?.(heirList);
        setLoading(false);
      })
      .catch((error) => {
        console.error("Failed to fetch user state:", error);
        setLoading(false);
      });

  }, [actor, onHeirsChange]);

  const handleUpdateHeir = async (updatedHeir: Heir) => {
    if (!actor) return;
    setLoading(true);
    try {
      // Call canister update
      await actor.update_heir(
        updatedHeir.name,
        updatedHeir.gov_id_hash,
        updatedHeir.security_question_hash
          ? [updatedHeir.security_question_hash]
          : [],
      );
      setEditingHeir(null);
      // Refetch
      const userState = await actor.get_user_state();
      const heirListRaw =
        userState && userState.length > 0 ? userState[0].heirs : [];
      const heirList = heirListRaw.map((h) => ({
        name: h.name,
        gov_id_hash: h.gov_id_hash,
        security_question_hash:
          h.security_question_hash && h.security_question_hash.length > 0
            ? h.security_question_hash[0]
            : "",
      }));
      setHeirs(heirList);
      onHeirsChange?.(heirList);
      toast({
        title: "Heir Updated",
        description: `${updatedHeir.name}'s information has been successfully updated.`,
      });
    } catch {
      toast({
        title: "Error",
        description: "Failed to update heir.",
        variant: "destructive",
      });
    }
    setLoading(false);
  };

  const handleRemoveHeir = async (name: string, gov_id_hash: string) => {
    if (!actor) return;
    setLoading(true);
    try {
      // Call canister remove
      await actor.remove_heir(gov_id_hash);
      // Refetch
      const userState = await actor.get_user_state();
      const heirListRaw =
        userState && userState.length > 0 ? userState[0].heirs : [];
      const heirList = heirListRaw.map((h) => ({
        name: h.name,
        gov_id_hash: h.gov_id_hash,
        security_question_hash:
          h.security_question_hash && h.security_question_hash.length > 0
            ? h.security_question_hash[0]
            : "",
      }));
      setHeirs(heirList);
      onHeirsChange?.(heirList);
      toast({
        title: "Heir Removed",
        description: `Heir has been removed from the beneficiaries.`,
        variant: "destructive",
      });
    } catch {
      toast({
        title: "Error",
        description: "Failed to remove heir.",
        variant: "destructive",
      });
    }
    setLoading(false);
  };

  const handleAddHeir = async (
    newHeir: Omit<Heir, "gov_id_hash"> & { gov_id_hash: string },
  ) => {
    if (!actor) return;
    setLoading(true);
    try {
      // Call canister add_heir
      await actor.add_heir(
        newHeir.name,
        newHeir.gov_id_hash,
        newHeir.security_question_hash ? [newHeir.security_question_hash] : [],
      );
      setIsAddingHeir(false);
      // Refetch
      const userState = await actor.get_user_state();
      const heirListRaw =
        userState && userState.length > 0 ? userState[0].heirs : [];
      const heirList = heirListRaw.map((h) => ({
        name: h.name,
        gov_id_hash: h.gov_id_hash,
        security_question_hash:
          h.security_question_hash && h.security_question_hash.length > 0
            ? h.security_question_hash[0]
            : "",
      }));
      setHeirs(heirList);
      onHeirsChange?.(heirList);
      toast({
        title: "Heir Added",
        description: `${newHeir.name} has been added as a beneficiary.`,
      });
    } catch {
      toast({
        title: "Error",
        description: "Failed to add heir.",
        variant: "destructive",
      });
    }
    setLoading(false);
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

  const getRelationshipColor = (relationship: string) => {
    switch (relationship) {
      case "Spouse":
        return "bg-pink-100 text-pink-800 border-pink-200";
      case "Son":
      case "Daughter":
        return "bg-blue-100 text-blue-800 border-blue-200";
      case "Granddaughter":
      case "Grandson":
        return "bg-purple-100 text-purple-800 border-purple-200";
      case "Charity":
        return "bg-green-100 text-green-800 border-green-200";
      default:
        return "bg-gray-100 text-gray-800 border-gray-200";
    }
  };

  return (
    <div className="space-y-6">
      {/* Controls */}
      <div className="flex justify-between items-center">
        <p className="text-muted-foreground">
          Manage beneficiaries and their inheritance percentages
        </p>
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

      {/* Heirs List */}
      <div className="grid gap-4 md:grid-cols-2">
        {heirs.map((heir) => (
          <Card
            key={heir.gov_id_hash}
            className="shadow-card hover:shadow-elegant transition-shadow"
          >
            <CardHeader className="pb-3">
              <div className="flex items-start justify-between">
                <div className="flex items-center space-x-3">
                  <div className="p-2 bg-primary/10 rounded-lg text-primary">
                    <User className="w-5 h-5" />
                  </div>
                  <div>
                    <CardTitle className="text-lg">{heir.name}</CardTitle>
                    <CardDescription>
                      Gov ID Hash: {heir.gov_id_hash}
                    </CardDescription>
                    {heir.security_question_hash && (
                      <CardDescription>
                        Security Q: {heir.security_question_hash}
                      </CardDescription>
                    )}
                  </div>
                </div>
                <Badge variant="secondary">Heir</Badge>
              </div>
            </CardHeader>
            <CardContent>
              <div className="flex space-x-2">
                <Dialog
                  open={
                    editingHeir?.name === heir.name &&
                    editingHeir?.gov_id_hash === heir.gov_id_hash
                  }
                  onOpenChange={(open) => !open && setEditingHeir(null)}
                >
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
                  onClick={() => handleRemoveHeir(heir.name, heir.gov_id_hash)}
                >
                  <Trash2 className="w-4 h-4 mr-2" />
                  Remove
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
};

interface HeirFormDialogProps {
  heir?: Heir;
  onSubmit: (heir: Heir | Omit<Heir, "id">) => void;
  onCancel: () => void;
  isEditing?: boolean;
}

const HeirFormDialog = ({
  heir,
  onSubmit,
  onCancel,
  isEditing = false,
}: HeirFormDialogProps) => {
  const { identity, actor } = useAuth();
  const [formData, setFormData] = useState({
    name: heir?.name || "",
    // keep gov_id_hash as computed and hidden; prefill when editing
    gov_id_hash: heir?.gov_id_hash || "",
    security_question_hash: heir?.security_question_hash || "",
  });
  const [rawGovId, setRawGovId] = useState("");
  const [maskedGovPreview, setMaskedGovPreview] = useState(
    heir?.gov_id_hash
      ? `${heir?.gov_id_hash.slice(0, 6)}...${heir?.gov_id_hash.slice(-4)}`
      : "",
  );
  const APP_PEPPER = "inheritnext-pepper-v1";

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (isEditing && heir) {
      onSubmit({ ...heir, ...formData });
    } else {
      onSubmit(formData);
    }
  };

  return (
    <DialogContent className="sm:max-w-md max-h-[80vh] overflow-y-auto">
      <DialogHeader>
        <DialogTitle>{isEditing ? "Update Heir" : "Add New Heir"}</DialogTitle>
        <DialogDescription>
          {isEditing
            ? "Modify the heir details below."
            : "Enter the details for the new beneficiary."}
        </DialogDescription>
      </DialogHeader>
      <form onSubmit={handleSubmit} className="space-y-4">
        <div className="space-y-2">
          <Label htmlFor="name">Full Name</Label>
          <Input
            id="name"
            value={formData.name}
            onChange={(e) =>
              setFormData((prev) => ({ ...prev, name: e.target.value }))
            }
            placeholder="Enter full name"
            required
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="gov_id_raw">Government ID</Label>
          <Input
            id="gov_id_raw"
            value={rawGovId}
            onChange={(e) => setRawGovId(e.target.value)}
            placeholder="Enter government ID (kept private)"
            required={!isEditing}
          />
          {maskedGovPreview && (
            <div className="text-sm text-muted-foreground">
              Preview Hash: {maskedGovPreview}
            </div>
          )}
        </div>
        <div className="space-y-2">
          <Label htmlFor="security_question_hash">Security Question Hash</Label>
          <Input
            id="security_question_hash"
            value={formData.security_question_hash}
            onChange={(e) =>
              setFormData((prev) => ({
                ...prev,
                security_question_hash: e.target.value,
              }))
            }
            placeholder="Enter security question hash (optional)"
          />
        </div>
        <DialogFooter>
          <Button type="button" variant="outline" onClick={onCancel}>
            Cancel
          </Button>
          <Button
            type="submit"
            className="bg-gradient-primary"
            onClick={async (ev) => {
              ev.preventDefault();
              // compute gov_id_hash if raw provided
              let computed = formData.gov_id_hash;
              if (rawGovId && rawGovId.length > 0) {
                // obtain per-user salt from canister; fallback to APP_PEPPER
                let salt = APP_PEPPER;
                try {
                  const s = await actor.get_user_salt();
                  if (s) salt = s as unknown as string;
                } catch (e) {
                  // ignore and use APP_PEPPER as fallback
                }
                computed = await pbkdf2Hex(
                  rawGovId.trim().toLowerCase(),
                  salt,
                  100_000,
                  32,
                );
                setFormData((prev) => ({ ...prev, gov_id_hash: computed }));
                setMaskedGovPreview(
                  `${computed.slice(0, 6)}...${computed.slice(-4)}`,
                );
              }
              // submit using updated formData
              if (isEditing && heir) {
                onSubmit({ ...heir, ...formData, gov_id_hash: computed });
              } else {
                onSubmit({ ...formData, gov_id_hash: computed });
              }
            }}
          >
            {isEditing ? "Update Heir" : "Add Heir"}
          </Button>
        </DialogFooter>
      </form>
    </DialogContent>
  );
};

export { HeirsList };
