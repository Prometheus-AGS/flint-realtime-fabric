import type * as Preset from "@docusaurus/preset-classic";
import type { Config } from "@docusaurus/types";
import { themes as prismThemes } from "prism-react-renderer";

const config: Config = {
  title: "Flint Realtime Fabric",
  tagline: "One event spine. Every plane. Authorization first.",

  favicon: "img/favicon.svg",

  url: "https://prometheus-ags.github.io",
  baseUrl: "/flint-realtime-fabric/",
  organizationName: "Prometheus-AGS",
  projectName: "flint-realtime-fabric",

  // Mirrors the prometheus-entity-management docs site, which builds green on
  // this same toolchain. See website/README.md for the open build blocker.
  future: {
    faster: {
      swcJsLoader: true,
      swcJsMinimizer: true,
      swcHtmlMinimizer: true,
      lightningCssMinimizer: false,
      mdxCrossCompilerCache: true,
      rspackBundler: true,
      rspackPersistentCache: true,
      ssgWorkerThreads: false,
      gitEagerVcs: true,
    },
  },

  trailingSlash: true,

  // A broken link is a documentation defect, not a warning. The Pages build
  // fails rather than shipping a dead reference.
  onBrokenLinks: "throw",
  onDuplicateRoutes: "throw",

  markdown: {
    mermaid: true,
    hooks: {
      onBrokenMarkdownLinks: "throw",
    },
  },

  themes: ["@docusaurus/theme-mermaid"],

  i18n: {
    defaultLocale: "en",
    locales: ["en"],
  },

  presets: [
    [
      "classic",
      {
        docs: {
          sidebarPath: "./sidebars.ts",
          editUrl:
            "https://github.com/Prometheus-AGS/flint-realtime-fabric/tree/main/website/",
          showLastUpdateTime: true,
        },
        blog: false,
        theme: {
          customCss: "./src/css/custom.css",
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: "img/social-card.svg",
    colorMode: {
      defaultMode: "dark",
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: "Flint Realtime Fabric",
      logo: {
        alt: "Flint Realtime Fabric",
        src: "img/logo.svg",
      },
      items: [
        {
          type: "docSidebar",
          sidebarId: "docsSidebar",
          position: "left",
          label: "Documentation",
        },
        {
          to: "/docs/case-studies/overview",
          label: "Case studies",
          position: "left",
        },
        {
          href: "https://github.com/Prometheus-AGS/flint-realtime-fabric",
          label: "GitHub",
          position: "right",
        },
      ],
    },
    footer: {
      style: "dark",
      links: [
        {
          title: "Understand",
          items: [
            { label: "Why this exists", to: "/docs/theory/why" },
            { label: "The dependency rule", to: "/docs/theory/dependency-rule" },
            { label: "Ports and adapters", to: "/docs/theory/ports" },
          ],
        },
        {
          title: "Build",
          items: [
            { label: "Quickstart", to: "/docs/guides/quickstart" },
            { label: "Writing an adapter", to: "/docs/guides/writing-an-adapter" },
            { label: "Authorization", to: "/docs/guides/authorization" },
          ],
        },
        {
          title: "Learn from",
          items: [
            { label: "Prior authorization (ASO)", to: "/docs/case-studies/prior-auth" },
            { label: "KnowMe", to: "/docs/case-studies/knowme" },
            { label: "Decision records", to: "/docs/decisions/overview" },
          ],
        },
      ],
      copyright: `Prometheus AGS · Flint Realtime Fabric · ${new Date().getFullYear()}`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ["rust", "toml", "bash", "json", "yaml", "sql"],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
