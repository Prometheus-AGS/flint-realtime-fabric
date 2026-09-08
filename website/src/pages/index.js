import Link from '@docusaurus/Link';
import Layout from '@theme/Layout';

// Flat 2.0: regions are distinguished by filled backgrounds and spacing only —
// no borders, separator lines, gradients, or decorative shadows.
const PILLARS = [
  {
    index: '01',
    title: 'The compiler enforces the architecture',
    body: 'Domain and application crates do not list adapter crates as dependencies. A layering violation is not a review comment — it fails to build.',
  },
  {
    index: '02',
    title: 'One port, one adapter',
    body: 'Each adapter crate implements exactly one port trait. Swapping Iggy for another log, or Keto for another authorizer, touches one crate and the composition root.',
  },
  {
    index: '03',
    title: 'Authorization is not application code',
    body: 'Tenant isolation lives in Zanzibar relation tuples, action policy in Cedar, identity at the gateway boundary. Business logic never re-implements a permission check.',
  },
  {
    index: '04',
    title: 'Planes compile in or out',
    body: 'Entity, agent, media, and federation are Cargo features. A deployment that does not run media does not link it.',
  },
  {
    index: '05',
    title: 'Proven is a separate word from built',
    body: 'This documentation distinguishes code that exists from behaviour demonstrated by an executed test, and says which is which on every page.',
  },
  {
    index: '06',
    title: 'Local-first is a first-class consumer',
    body: 'An authorized shape facade lets an on-device replica sync only the rows a caller may see — the boundary is enforced server-side, not by the client.',
  },
];

export default function Home() {
  return (
    <Layout
      title="Flint Realtime Fabric"
      description="A ports-and-adapters realtime fabric: one event spine, an entity plane, an agent plane, and a media plane, with authorization enforced outside application code."
    >
      <header className="frfHero">
        <div className="container">
          <p className="frfHero__eyebrow">Prometheus AGS</p>
          <h1 className="frfHero__title">One event spine. Every plane.</h1>
          <p className="frfHero__lede">
            Flint Realtime Fabric is a Rust realtime backend built as ports and
            adapters, where the dependency rule is enforced by the compiler and
            authorization lives outside application code. This site explains the
            reasoning, not just the API.
          </p>
          <div className="frfHero__actions">
            <Link className="button button--primary button--lg" to="/docs/theory/why">
              Why this exists
            </Link>
            <Link className="button button--secondary button--lg" to="/docs/case-studies/prior-auth">
              See it in a real product
            </Link>
          </div>
        </div>
      </header>

      <main className="frfPillars">
        <div className="container">
          <div className="frfGrid">
            {PILLARS.map((pillar) => (
              <article className="frfGrid__cell" key={pillar.index}>
                <span className="frfGrid__index">{pillar.index}</span>
                <h2 className="frfGrid__title">{pillar.title}</h2>
                <p className="frfGrid__body">{pillar.body}</p>
              </article>
            ))}
          </div>
        </div>
      </main>
    </Layout>
  );
}
