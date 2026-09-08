/**
 * Post-build inspection. `docusaurus build` already throws on broken links and
 * duplicate routes; this asserts the artifact a deploy would actually upload is
 * present and non-trivial, so an empty or partial build cannot reach Pages.
 */
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";

const buildDir = new URL("../build/", import.meta.url).pathname;

function fail(message) {
  console.error(`check-build: ${message}`);
  process.exit(1);
}

let entries;
try {
  entries = readdirSync(buildDir);
} catch {
  fail("build/ does not exist — run `pnpm run build` first");
}

if (!entries.includes("index.html")) fail("build/index.html is missing");

const html = readFileSync(join(buildDir, "index.html"), "utf8");
if (!html.includes("Flint Realtime Fabric")) {
  fail("build/index.html does not carry the site title");
}

const REQUIRED_ROUTES = [
  "docs/theory/why",
  "docs/theory/dependency-rule",
  "docs/guides/quickstart",
  "docs/case-studies/prior-auth",
  "docs/case-studies/knowme",
];
for (const route of REQUIRED_ROUTES) {
  const page = join(buildDir, route, "index.html");
  try {
    if (statSync(page).size < 1024) fail(`${route} built but is suspiciously small`);
  } catch {
    fail(`${route}/index.html is missing from the build`);
  }
}

console.log(`check-build: OK — ${entries.length} top-level entries, ${REQUIRED_ROUTES.length} key routes present.`);
