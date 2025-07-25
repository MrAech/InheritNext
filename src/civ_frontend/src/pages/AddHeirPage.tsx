import React, { useState, useEffect } from "react";
import { useDemoMode } from "@/context/DemoModeContext";
import { Card, CardHeader, CardTitle, CardContent, CardDescription } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipTrigger, TooltipContent } from "@/components/ui/tooltip";
import { Badge } from "@/components/ui/badge";

import { useOverlayManager } from "@/context/OverlayManagerContext";
import { useNavigate } from "react-router-dom";
import { addHeir as addHeirAPI } from "../lib/api";

const AddHeirPage: React.FC = () => {
  const { mode } = useDemoMode();
  const { setOverlay, setOverlayProps } = useOverlayManager();
  const navigate = useNavigate();
  const [formData, setFormData] = useState({
    name: mode === "evaluator" ? "Simulated Heir" : "",
    relationship: mode === "evaluator" ? "Son" : "",
    email: mode === "evaluator" ? "simulated.heir@example.com" : "",
    phone: mode === "evaluator" ? "1234567890" : "",
    address: mode === "evaluator" ? "Simulated Address" : "",
    kyc: "Simulated",
    iiConnection: "Simulated",
  });
  const [showConfirm, setShowConfirm] = useState(false);

  // Reset formData when mode changes
  useEffect(() => {
    if (mode === "evaluator") {
      setFormData({
        name: "Simulated Heir",
        relationship: "Son",
        email: "simulated.heir@example.com",
        phone: "1234567890",
        address: "Simulated Address",
        kyc: "Simulated",
        iiConnection: "Simulated",
      });
    } else {
      setFormData({
        name: "",
        relationship: "",
        email: "",
        phone: "",
        address: "",
        kyc: "Simulated",
        iiConnection: "Simulated",
      });
    }
  }, [mode]);

  const handleChange = (field: string, value: string) => {
    setFormData(prev => ({ ...prev, [field]: value }));
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setShowConfirm(true);
  };

  const handleConfirm = async () => {
    const { name, relationship, email, phone, address } = formData;
    await addHeirAPI({
      name,
      relationship,
      email,
      phone,
      address,
    });
    setShowConfirm(false);
    navigate("/dashboard");
  };

  return (
    <div className="flex justify-center items-center min-h-screen bg-background">
      <Card className="max-w-xl w-full p-6 shadow-elegant">
        <CardHeader>
          <CardTitle>Add New Heir</CardTitle>
          <CardDescription>
            Enter heir details. KYC and II connection are simulated for evaluator mode.
          </CardDescription>
        </CardHeader>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <Tooltip>
              <TooltipTrigger asChild>
                <Label htmlFor="name">Full Name</Label>
              </TooltipTrigger>
              <TooltipContent>Enter the full name of the heir.</TooltipContent>
            </Tooltip>
            <Input
              id="name"
              value={formData.name}
              onChange={e => handleChange("name", e.target.value)}
              placeholder="Enter full name"
              required
            />
          </div>
          <div className="space-y-2">
            <Tooltip>
              <TooltipTrigger asChild>
                <Label htmlFor="relationship">Relationship</Label>
              </TooltipTrigger>
              <TooltipContent>Specify the relationship (e.g., Son, Daughter, Charity).</TooltipContent>
            </Tooltip>
            <Input
              id="relationship"
              value={formData.relationship}
              onChange={e => handleChange("relationship", e.target.value)}
              placeholder="e.g., Son, Daughter, Spouse, Charity"
              required
            />
          </div>
          <div className="space-y-2">
            <Tooltip>
              <TooltipTrigger asChild>
                <Label htmlFor="email">Email</Label>
              </TooltipTrigger>
              <TooltipContent>Enter the heir's email address.</TooltipContent>
            </Tooltip>
            <Input
              id="email"
              value={formData.email}
              onChange={e => handleChange("email", e.target.value)}
              placeholder="Enter email"
              required
            />
          </div>
          <div className="space-y-2">
            <Tooltip>
              <TooltipTrigger asChild>
                <Label htmlFor="phone">Phone</Label>
              </TooltipTrigger>
              <TooltipContent>Enter the heir's phone number.</TooltipContent>
            </Tooltip>
            <Input
              id="phone"
              value={formData.phone}
              onChange={e => handleChange("phone", e.target.value)}
              placeholder="Enter phone"
              required
            />
          </div>
          <div className="space-y-2">
            <Tooltip>
              <TooltipTrigger asChild>
                <Label htmlFor="address">Address</Label>
              </TooltipTrigger>
              <TooltipContent>Enter the heir's address.</TooltipContent>
            </Tooltip>
            <Input
              id="address"
              value={formData.address}
              onChange={e => handleChange("address", e.target.value)}
              placeholder="Enter address"
              required
            />
          </div>
          <div className="space-y-2">
            <Label>KYC</Label>
            <Badge variant="outline">{formData.kyc}</Badge>
            <div className="flex gap-2 mt-2">
              <Button
                size="sm"
                variant={formData.kyc === "Success" ? "default" : "outline"}
                onClick={() => setFormData(prev => ({ ...prev, kyc: "Success" }))}
              >
                Simulate Success
              </Button>
              <Button
                size="sm"
                variant={formData.kyc === "Failure" ? "destructive" : "outline"}
                onClick={() => setFormData(prev => ({ ...prev, kyc: "Failure" }))}
              >
                Simulate Failure
              </Button>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Badge
                    variant="secondary"
                    onClick={() => {
                      setOverlayProps({
                        title: "Simulated KYC",
                        description:
                          "KYC is simulated. In a real flow, this would verify the heir's identity."
                      });
                      setOverlay("simulation");
                    }}
                  >
                    Simulated
                  </Badge>
                </TooltipTrigger>
                <TooltipContent>
                  Click for explanation of simulated KYC.
                </TooltipContent>
              </Tooltip>
            </div>
          </div>
          <div className="space-y-2">
            <Label>II Connection</Label>
            <Badge variant="outline">{formData.iiConnection}</Badge>
            <div className="flex gap-2 mt-2">
              <Button
                size="sm"
                variant={formData.iiConnection === "Success" ? "default" : "outline"}
                onClick={() => setFormData(prev => ({ ...prev, iiConnection: "Success" }))}
              >
                Simulate Success
              </Button>
              <Button
                size="sm"
                variant={formData.iiConnection === "Failure" ? "destructive" : "outline"}
                onClick={() => setFormData(prev => ({ ...prev, iiConnection: "Failure" }))}
              >
                Simulate Failure
              </Button>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Badge
                    variant="secondary"
                    onClick={() => {
                      setOverlayProps({
                        title: "Simulated II Connection",
                        description:
                          "II connection is simulated. In a real flow, this would connect the heir to Internet Identity."
                      });
                      setOverlay("simulation");
                    }}
                  >
                    Simulated
                  </Badge>
                </TooltipTrigger>
                <TooltipContent>
                  Click for explanation of simulated II connection.
                </TooltipContent>
              </Tooltip>
            </div>
          </div>
          {mode === "evaluator" ? (
            <>
              <Button
                type="submit"
                className="bg-gradient-primary w-full"
                disabled={
                  formData.kyc !== "Success" ||
                  formData.iiConnection !== "Success"
                }
              >
                Add Heir (Simulated)
              </Button>
              {(formData.kyc !== "Success" || formData.iiConnection !== "Success") && (
                <div className="text-red-500 text-sm mt-2">
                  Please set both KYC and II Connection to "Success" to add a simulated heir.
                </div>
              )}
            </>
          ) : (
            <Button
              type="submit"
              className="bg-green-600 w-full"
              disabled={
                formData.kyc === "Simulated" ||
                formData.iiConnection === "Simulated"
              }
            >
              Add Heir
            </Button>
          )}
        </form>
      </Card>
      {/* Confirmation Modal */}
      {showConfirm && (
        <div className="fixed inset-0 z-[110] flex items-center justify-center bg-black/40">
          <Card className="max-w-md p-6 shadow-elegant">
            <h2 className="text-xl font-bold mb-4">Confirm Heir Addition</h2>
            <div className="mb-4 space-y-2">
              <div><strong>Name:</strong> {formData.name}</div>
              <div><strong>Relationship:</strong> {formData.relationship}</div>
              <div><strong>Email:</strong> {formData.email}</div>
              <div><strong>Phone:</strong> {formData.phone}</div>
              <div><strong>Address:</strong> {formData.address}</div>
              <div><strong>KYC:</strong> {formData.kyc}</div>
              <div><strong>II Connection:</strong> {formData.iiConnection}</div>
            </div>
            <div className="flex gap-2 justify-end">
              <Button size="sm" variant="outline" onClick={() => setShowConfirm(false)}>
                Cancel
              </Button>
              <Button size="sm" variant="default" onClick={handleConfirm}>
                Confirm & Add
              </Button>
            </div>
          </Card>
        </div>
      )}
    </div>
  );
};

export default AddHeirPage;