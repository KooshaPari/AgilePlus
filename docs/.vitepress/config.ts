import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'AgilePlus Documentation',
  description: 'Phenotype AgilePlus Ecosystem Documentation',
  base: '/AgilePlus/',
  themeConfig: {
    nav: [
      { text: 'Home', link: '/' },
      { text: 'FR Specs', link: '/specs/' },
      { text: 'Governance', link: '/GOVERNANCE' },
    ],
    sidebar: [
      {
        text: 'Documentation',
        items: [
          { text: 'Introduction', link: '/README' },
          { text: 'FR Specifications', link: '/specs/' },
          { text: 'Governance', link: '/GOVERNANCE' },
        ]
      }
    ],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/phenotype/AgilePlus' }
    ]
  }
})
