import type { ReactNode } from "react";

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps): React.JSX.Element {
  return (
    <div style={{ minHeight: "100vh", background: "var(--color-bg)", color: "var(--color-text)" }}>
      <header
        style={{
          borderBottom: "1px solid var(--color-border)",
          padding: "1rem 2rem",
          display: "flex",
          alignItems: "center",
          gap: "0.75rem",
        }}
      >
        <span style={{ fontWeight: 700, fontSize: "1.125rem", letterSpacing: "-0.02em" }}>
          FRF Admin
        </span>
        <nav aria-label="Main navigation" style={{ display: "flex", gap: "1.5rem", marginLeft: "2rem" }}>
          <a href="#entities" style={{ fontSize: "0.875rem", textDecoration: "none", color: "inherit" }}>
            Entities
          </a>
        </nav>
      </header>
      {children}
    </div>
  );
}
