import { Layout } from "./core/Layout.js";
import { EntitiesPage } from "./features/entities/pages/EntitiesPage.js";

export function App(): React.JSX.Element {
  return (
    <Layout>
      <EntitiesPage />
    </Layout>
  );
}
