import { NavLink } from "react-router-dom";

const links = [
  { to: "/dashboard", label: "Dashboard" },
  { to: "/assets", label: "Assets" },
  { to: "/heirs", label: "Heirs" },
  { to: "/distributions", label: "Distributions" },
  { to: "/audit", label: "Audit Log" },
  { to: "/documents", label: "Documents" },
  { to: "/escrow", label: "Escrow" },
  { to: "/approvals", label: "Approvals" },
  { to: "/claim", label: "Claim" },
  { to: "/settings", label: "Settings" },
];

export default function Sidebar() {
  return (
    <nav className="h-full p-3 overflow-x-hidden">
      <ul className="space-y-2">
        {links.map(l => (
          <li key={l.to}>
            <NavLink
              to={l.to}
              className={({ isActive }) => `block px-2 py-2 rounded ${isActive ? 'bg-primary/10 font-semibold' : 'text-muted-foreground'}`}
            >
              {l.label}
            </NavLink>
          </li>
        ))}
      </ul>
    </nav>
  );
}
