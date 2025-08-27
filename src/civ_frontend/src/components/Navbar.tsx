import { useAuth } from "@/context/useAuth";
import { Button } from "@/components/ui/button";
import { Link } from "react-router-dom";

export default function Navbar() {
  const { identity, logout } = useAuth();
  return (
    <div className="container mx-auto px-4 py-3 flex items-center justify-between">
      <div className="flex items-center space-x-4">
        <Link to="/dashboard" className="font-bold">InheritNext</Link>
      </div>
      <div className="flex items-center space-x-4">
        <div className="text-sm text-muted-foreground mr-4">{identity ? (identity.getPrincipal().toString?.() ?? 'Principal') : 'Not signed in'}</div>
        <Button variant="outline" size="sm" onClick={() => void logout()}>
          Sign Out
        </Button>
      </div>
    </div>
  );
}
