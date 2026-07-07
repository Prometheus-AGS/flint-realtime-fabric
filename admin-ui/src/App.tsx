import { lazy, Suspense, useState, useSyncExternalStore } from "react";
import { Layout } from "./core/Layout.js";
import { EntitiesPage } from "./features/entities/pages/EntitiesPage.js";
import { AgentActivityPanel } from "./features/agents/components/AgentActivityPanel.js";
import { LoginGate } from "./features/auth/components/LoginGate.js";
import { restoreToken } from "./features/auth/services/authService.js";

const SignalingDemoPage = lazy(
  () => import("./features/signaling/pages/SignalingDemoPage.js").then((m) => ({ default: m.SignalingDemoPage })),
);

function useHash(): string {
  return useSyncExternalStore(
    (cb) => {
      window.addEventListener("hashchange", cb);
      return () => window.removeEventListener("hashchange", cb);
    },
    () => window.location.hash,
    () => "",
  );
}

function Router(): React.JSX.Element {
  const hash = useHash();
  if (hash === "#demo/signaling") {
    return (
      <Suspense fallback={<p style={{ padding: "2rem" }}>Loading…</p>}>
        <SignalingDemoPage />
      </Suspense>
    );
  }
  if (hash === "#agents") {
    return <AgentActivityPanel />;
  }
  return <EntitiesPage />;
}

export function App(): React.JSX.Element {
  // Restore a persisted token once, before first render of the gated tree, so a
  // returning operator is not asked to log in again on reload.
  useState(restoreToken);

  return (
    <LoginGate>
      <Layout>
        <Router />
      </Layout>
    </LoginGate>
  );
}
