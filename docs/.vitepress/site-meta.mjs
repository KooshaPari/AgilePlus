import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import { generateSidebar } from '@phenotype/docs/utils'

const __dirname = dirname(fileURLToPath(import.meta.url))
// docs/.vitepress -> docs (the VitePress srcDir)
const docsSrcDir = join(__dirname, '..')

// Top-level docs sections that contain real, navigable content.
// Each becomes both a nav entry and an auto-generated sidebar group,
// scoped so it only shows while browsing under that section's path.
const SECTIONS = [
  { prefix: 'architecture', text: 'Architecture' },
  { prefix: 'guide', text: 'Guide' },
  { prefix: 'sdk', text: 'SDK' },
  { prefix: 'reference', text: 'Reference' },
  { prefix: 'process', text: 'Process' },
  { prefix: 'workflow', text: 'Workflow' },
]

function buildSidebar() {
  const sidebar = {}
  for (const { prefix } of SECTIONS) {
    sidebar[`/${prefix}/`] = generateSidebar({ srcDir: docsSrcDir, prefix })
  }
  return sidebar
}

export function createSiteMeta({ base = '/' } = {}) {
  // For custom domain deployments (e.g., agileplus.phenotype.space), use root base
  // GitHub Pages default URLs include repo name prefix, but custom domains serve from root
  const isCustomDomain = process.env.PHENOTYPE_CUSTOM_DOMAIN === 'true'
  const resolvedBase = isCustomDomain ? '/' : base

  return {
    base: resolvedBase,
    // VitePress resolves `srcDir` relative to the process cwd, which is
    // `docs/` (the working-directory for `bun run docs:build`). The actual
    // markdown content lives directly in this directory, not a nested
    // `docs/docs/`, so srcDir must be '.', not the '@phenotype/docs' default
    // of 'docs'. Without this override VitePress only ever discovers
    // `docs/index.md` (the Home hero) and treats all real content as
    // unreachable, which is the root cause of the missing-sidebar bug.
    srcDir: '.',
    title: 'AgilePlus',
    description: 'AgilePlus — a lightweight, standalone project-management and PM substrate: requirements, epics, stories, and repo sync, from the CLI or as an embedded library.',
    sidebar: buildSidebar(),
    // Only the sections wired into nav/sidebar above are curated, navigable
    // docs. The rest of this directory holds specs, archives, worklogs, etc.
    // that are not part of the public doc site and in some cases (e.g.
    // specs/**) contain frontmatter VitePress can't parse as YAML. Excluding
    // them keeps the build fast and avoids pulling unrelated/broken content
    // into the site. `overrides` is deep-merged over the base VitePress
    // config by createPhenotypeConfig, so this is the correct place for a
    // raw VitePress option like srcExclude that isn't one of its named
    // top-level ConfigOptions fields.
    overrides: {
      srcExclude: [
        '_archive/**', 'adr/**', 'agents/**', 'assets/**', 'audit/**',
        'audits/**', 'boundary/**', 'changes/**', 'checklists/**',
        'concepts/**', 'developers/**', 'docs/**', 'doc-system/**',
        'embeds/**', 'examples/**', 'fa/**', 'fa-Latn/**',
        'frontend-candidates/**', 'guides/**', 'harmonization/**',
        'infra/**', 'intent/**', 'issues/**', 'journeys/**', 'operations/**',
        'pilot/**', 'plans/**', 'remediation/**', 'reports/**',
        'requirements/**', 'research/**', 'roadmap/**', 'security/**',
        'sessions/**', 'sota/**', 'specs/**', 'superpowers/**',
        'templates/**', 'tests/**', 'triage/**', 'vendor/**',
        'worklogs/**', 'zh-CN/**', 'zh-TW/**',
      ],
    },
    themeConfig: {
      siteTitle: 'AgilePlus',
      nav: [
        { text: 'Home', link: resolvedBase || '/' },
        ...SECTIONS.map(({ prefix, text }) => ({ text, link: `/${prefix}/` })),
      ],
      socialLinks: [
        { icon: 'github', link: 'https://github.com/KooshaPari/AgilePlus' },
      ],
    },
    head: [
      ['meta', { name: 'theme-color', content: '#7ebab5' }],
    ],
  }
}
