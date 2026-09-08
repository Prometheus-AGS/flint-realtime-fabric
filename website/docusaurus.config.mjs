import {themes as prismThemes} from 'prism-react-renderer';
import {createRequire} from 'node:module';

const require = createRequire(import.meta.url);

const config = {
  title: 'Flint Realtime Fabric',
  tagline: 'Documentation that belongs to the product.',
  favicon: 'img/favicon.ico',
  url: process.env.SITE_URL ?? 'https://prometheus-ags.github.io',
  baseUrl: process.env.BASE_URL ?? '/flint-realtime-fabric/',
  trailingSlash: false,
  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'throw',
  organizationName: 'prometheus-ags',
  projectName: 'flint-realtime-fabric',
  markdown: {mermaid: true},
  themes: ['@docusaurus/theme-mermaid'],
  presets: [['classic', {docs: {sidebarPath: './sidebars.ts'}, blog: false, theme: {customCss: './src/css/custom.css'}}]],
  plugins: [[require.resolve('@easyops-cn/docusaurus-search-local'), {hashed: true, indexDocs: true}]],
  themeConfig: {
    colorMode: {defaultMode: 'dark', respectPrefersColorScheme: true},
    navbar: {title: 'Flint Realtime Fabric', items: [{type: 'docSidebar', sidebarId: 'docsSidebar', label: 'Documentation', position: 'left'}]},
    prism: {theme: prismThemes.github, darkTheme: prismThemes.dracula},
  },
};

export default config;
