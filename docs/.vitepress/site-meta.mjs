export function createSiteMeta({ base = '/' } = {}) {
  // For custom domain deployments (e.g., agileplus.phenotype.space), use root base
  // GitHub Pages default URLs include repo name prefix, but custom domains serve from root
  const isCustomDomain = process.env.PHENOTYPE_CUSTOM_DOMAIN === 'true'
  const resolvedBase = isCustomDomain ? '/' : base

  return {
    base: resolvedBase,
    title: 'AgilePlus',
    description: 'AgilePlus — a lightweight, standalone project-management and PM substrate: requirements, epics, stories, and repo sync, from the CLI or as an embedded library.',
    themeConfig: {
      siteTitle: 'AgilePlus',
      nav: [
        { text: 'Home', link: resolvedBase || '/' },
        { text: 'Guide', link: '/guide/' },
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
