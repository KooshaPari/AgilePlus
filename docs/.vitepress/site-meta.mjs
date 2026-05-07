export function createSiteMeta({ base = '/' } = {}) {
  // For custom domain deployments (e.g., agileplus.phenotype.space), use root base
  // GitHub Pages default URLs include repo name prefix, but custom domains serve from root
  const isCustomDomain = process.env.PHENOTYPE_CUSTOM_DOMAIN === 'true'
  const resolvedBase = isCustomDomain ? '/' : base

  return {
    base: resolvedBase,
    title: 'apps/AgilePlus',
    description: 'Documentation',
    themeConfig: {
      nav: [
        { text: 'Home', link: resolvedBase || '/' },
        { text: 'Guide', link: '/guide/' },
      ],
    },
  }
}
