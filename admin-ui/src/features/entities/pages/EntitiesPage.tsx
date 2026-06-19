import { EntityGraph } from "../components/EntityGraph.js";

const CHANNEL_ID = import.meta.env["VITE_CHANNEL_ID"] ?? "00000000-0000-0000-0000-000000000001";
const CONSUMER_ID = "admin-ui-" + Math.random().toString(36).slice(2, 10);

export function EntitiesPage(): React.JSX.Element {
  return (
    <main style={{ padding: "2rem", maxWidth: "960px", margin: "0 auto" }}>
      <h1 style={{ fontSize: "1.5rem", fontWeight: 700, marginBottom: "1.5rem" }}>
        Entity Stream
      </h1>
      <EntityGraph channelId={CHANNEL_ID} consumerId={CONSUMER_ID} />
    </main>
  );
}
