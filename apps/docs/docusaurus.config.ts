import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const siteUrl = process.env.DOCS_SITE_URL ?? 'https://paddles.spoke.sh';
const baseUrl = process.env.DOCS_BASE_URL ?? '/';
const repoUrl = 'https://github.com/spoke-sh/paddles';

const config: Config = {
  title: 'Paddles',
  tagline: 'Recursive in-context planning harness for local-first coding agents',
  favicon: 'img/favicon.svg',
  future: {
    v4: true,
  },
  url: siteUrl,
  baseUrl,
  organizationName: 'spoke-sh',
  projectName: 'paddles',
  onBrokenLinks: 'throw',
  markdown: {
    hooks: {
      onBrokenMarkdownLinks: 'throw',
    },
  },
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },
  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          routeBasePath: 'docs',
          editUrl: `${repoUrl}/tree/main/apps/docs/`,
          showLastUpdateAuthor: false,
          showLastUpdateTime: true,
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],
  themeConfig: {
    colorMode: {
      defaultMode: 'light',
      disableSwitch: true,
      respectPrefersColorScheme: false,
    },
    navbar: {
      title: 'Paddles',
      items: [
        {
          type: 'doc',
          docId: 'intro',
          label: 'Docs',
          position: 'left',
        },
        {
          to: '/docs/start-here/installation',
          label: 'Start Here',
          position: 'left',
        },
        {
          to: '/docs/concepts/turn-loop',
          label: 'Concepts',
          position: 'left',
        },
        {
          href: 'https://www.spoke.sh',
          label: 'Spoke',
          position: 'right',
        },
        {
          href: repoUrl,
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Get Started',
          items: [
            {
              label: 'Intro',
              to: '/docs/intro',
            },
            {
              label: 'Installation',
              to: '/docs/start-here/installation',
            },
            {
              label: 'First Turn',
              to: '/docs/start-here/first-turn',
            },
          ],
        },
        {
          title: 'Concepts',
          items: [
            {
              label: 'Turn Loop',
              to: '/docs/concepts/turn-loop',
            },
            {
              label: 'Search and Retrieval',
              to: '/docs/concepts/search-retrieval',
            },
            {
              label: 'Context Tiers',
              to: '/docs/concepts/context-tiers',
            },
            {
              label: 'Recursive Planning',
              to: '/docs/concepts/recursive-planning',
            },
          ],
        },
        {
          title: 'Project',
          items: [
            {
              label: 'GitHub',
              href: repoUrl,
            },
            {
              label: 'Foundational Docs',
              to: '/docs/reference/foundational-docs',
            },
          ],
        },
      ],
      copyright: `Copyright \u00a9 ${new Date().getFullYear()} Paddles contributors.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
