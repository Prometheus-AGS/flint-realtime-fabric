import type { SidebarsConfig } from "@docusaurus/plugin-content-docs";

/**
 * One sidebar, ordered as a reading path rather than an alphabetical dump:
 * why the thing exists → how it is shaped → how to build on it → what it looks
 * like in a real product → the decisions and their consequences.
 */
const sidebars: SidebarsConfig = {
  docsSidebar: [
    "index",
    {
      type: "category",
      label: "Theory",
      collapsed: false,
      items: [
        "theory/why",
        "theory/dependency-rule",
        "theory/ports",
        "theory/planes",
        "theory/authorization",
      ],
    },
    {
      type: "category",
      label: "Guides",
      collapsed: false,
      items: [
        "guides/quickstart",
        "guides/writing-an-adapter",
        "guides/authorization",
        "guides/local-first",
      ],
    },
    {
      type: "category",
      label: "Case studies",
      collapsed: false,
      items: [
        "case-studies/overview",
        "case-studies/prior-auth",
        "case-studies/knowme",
      ],
    },
    {
      type: "category",
      label: "Decisions",
      collapsed: true,
      items: ["decisions/overview"],
    },
    "status",
  ],
};

export default sidebars;
