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

interface Heir {
  id: string;
  name: string;
  relationship: string;
  percentage: number;
  email: string;
  phone: string;
  address: string;
}

interface HeirsListProps {
  onHeirsChange?: (heirs: Heir[]) => void;
}

const HeirsList = ({ onHeirsChange }: HeirsListProps = {}) => {
  const [heirs, setHeirs] = useState<Heir[]>([
    {
      id: "1",
      name: "Sarah Johnson",
      relationship: "Daughter",
      percentage: 40,
      email: "sarah.johnson@email.com",
      phone: "+1 (555) 123-4567",
      address: "123 Oak Street, Beverly Hills, CA"
    },
    {
      id: "2",
      name: "Michael Johnson",
      relationship: "Son",
      percentage: 30,
      email: "michael.johnson@email.com",
      phone: "+1 (555) 234-5678",
      address: "456 Pine Avenue, Los Angeles, CA"
    },
    {
      id: "3",
      name: "Emily Davis",
      relationship: "Granddaughter",
      percentage: 20,
      email: "emily.davis@email.com",
      phone: "+1 (555) 345-6789",
      address: "789 Maple Drive, Santa Monica, CA"
    },
    {
      id: "4",
      name: "Children's Hospital Foundation",
      relationship: "Charity",
      percentage: 10,
      email: "donations@childrenshospital.org",
      phone: "+1 (555) 456-7890",
      address: "321 Charity Lane, Los Angeles, CA"
    }
  ]);

  const [editingHeir, setEditingHeir] = useState<Heir | null>(null);
  const [isAddingHeir, setIsAddingHeir] = useState(false);
  const { toast } = useToast();

  // Initialize the parent component with heirs data
  useEffect(() => {
    onHeirsChange?.(heirs);
  }, []);

  const getTotalPercentage = () => {
    return heirs.reduce((sum, heir) => sum + heir.percentage, 0);
  };

  const handleUpdateHeir = (updatedHeir: Heir) => {
    const newHeirs = heirs.map(heir =>
      heir.id === updatedHeir.id ? updatedHeir : heir
    );
    setHeirs(newHeirs);
    onHeirsChange?.(newHeirs);
    setEditingHeir(null);
    toast({
      title: "Heir Updated",
      description: `${updatedHeir.name}'s information has been successfully updated.`,
    });
  };

  const handleRemoveHeir = (heirId: string) => {
    const heirToRemove = heirs.find(h => h.id === heirId);
    const newHeirs = heirs.filter(heir => heir.id !== heirId);
    setHeirs(newHeirs);
    onHeirsChange?.(newHeirs);
    toast({
      title: "Heir Removed",
      description: `${heirToRemove?.name} has been removed from the beneficiaries.`,
      variant: "destructive",
    });
  };

  const handleAddHeir = (newHeir: Omit<Heir, 'id'>) => {
    const heir: Heir = {
      ...newHeir,
      id: Date.now().toString()
    };
    const newHeirs = [...heirs, heir];
    setHeirs(newHeirs);
    onHeirsChange?.(newHeirs);
    setIsAddingHeir(false);
    toast({
      title: "Heir Added",
      description: `${heir.name} has been added as a beneficiary.`,
    });
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

  const totalPercentage = getTotalPercentage();
  const isPercentageValid = totalPercentage === 100;

  return (
    <div className="space-y-6">
      {/* Percentage Summary */}
      <Card className="shadow-card">
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            <span>Inheritance Distribution</span>
            <Badge
              variant={isPercentageValid ? "secondary" : "destructive"}
              className="text-sm"
            >
              {totalPercentage}% Total
            </Badge>
          </CardTitle>
          <CardDescription>
            {isPercentageValid
              ? "Distribution is complete and balanced."
              : `Distribution needs adjustment. ${totalPercentage > 100 ? 'Over' : 'Under'}-allocated by ${Math.abs(100 - totalPercentage)}%.`
            }
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Progress value={totalPercentage} className="h-3" />
        </CardContent>
      </Card>

      {/* Controls */}
      <div className="flex justify-between items-center">
        <p className="text-muted-foreground">Manage beneficiaries and their inheritance percentages</p>
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
                        className={getRelationshipColor(heir.relationship)}
                      >
                        {heir.relationship}
                      </Badge>
                    </CardDescription>
                  </div>
                </div>
                <div className="text-right">
                  <div className="text-2xl font-bold text-primary">{heir.percentage}%</div>
                  <div className="text-sm text-muted-foreground">inheritance</div>
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                <div className="space-y-2">
                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <Mail className="w-4 h-4" />
                    {heir.email}
                  </div>
                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <Phone className="w-4 h-4" />
                    {heir.phone}
                  </div>
                </div>
                <div className="flex space-x-2">
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
  onSubmit: (heir: Heir | Omit<Heir, 'id'>) => void;
  onCancel: () => void;
  isEditing?: boolean;
}

const HeirFormDialog = ({ heir, onSubmit, onCancel, isEditing = false }: HeirFormDialogProps) => {
  const [formData, setFormData] = useState({
    name: heir?.name || "",
    relationship: heir?.relationship || "",
    percentage: heir?.percentage || 0,
    email: heir?.email || "",
    phone: heir?.phone || "",
    address: heir?.address || ""
  });

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
          <Label htmlFor="percentage">Inheritance Percentage (%)</Label>
          <Input
            id="percentage"
            type="number"
            min="0"
            max="100"
            value={formData.percentage}
            onChange={(e) => setFormData(prev => ({ ...prev, percentage: Number(e.target.value) }))}
            placeholder="Enter percentage"
            required
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="email">Email Address</Label>
          <Input
            id="email"
            type="email"
            value={formData.email}
            onChange={(e) => setFormData(prev => ({ ...prev, email: e.target.value }))}
            placeholder="Enter email address"
            required
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="phone">Phone Number</Label>
          <Input
            id="phone"
            value={formData.phone}
            onChange={(e) => setFormData(prev => ({ ...prev, phone: e.target.value }))}
            placeholder="Enter phone number"
            required
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="address">Address</Label>
          <Input
            id="address"
            value={formData.address}
            onChange={(e) => setFormData(prev => ({ ...prev, address: e.target.value }))}
            placeholder="Enter full address"
            required
          />
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