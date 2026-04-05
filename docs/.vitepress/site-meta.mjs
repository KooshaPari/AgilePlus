export function createSiteMeta({ base = '/' } = {}) {
  return {
    base,
    title: 'apps/AgilePlus',
    description: 'Documentation',
    themeConfig: {
      nav: [
        { text: 'Home', link: base || '/' },
        { text: 'Guide', link: '/guide/' },
        { text: 'Dashboard', link: '/dashboard/' },
        { text: 'CLI', link: '/cli/' },
        { text: 'API', link: '/api/' },
        { text: 'Contributing', link: '/contributing/documentation' },
      ],
      sidebar: {
        '/dashboard/': [
          {
            text: 'Dashboard',
            items: [
              { text: 'Overview', link: '/dashboard/' },
              { text: 'Service Controls', link: '/dashboard/service-controls' },
            ]
          }
        ],
        '/cli/': [
          {
            text: 'CLI',
            items: [
              { text: 'Overview', link: '/cli/' },
            ]
          }
        ],
        '/api/': [
          {
            text: 'API',
            items: [
              { text: 'Overview', link: '/api/' },
            ]
          }
        ],
        '/guide/': [
          {
            text: 'Guide',
            items: [
              { text: 'Getting Started', link: '/guide/' },
            ]
          }
        ],
        '/contributing/': [
          {
            text: 'Contributing',
            items: [
              { text: 'Overview', link: '/contributing/' },
              { text: 'Documentation', link: '/contributing/documentation' },
              { text: 'Recording Visuals', link: '/contributing/recording-visuals' },
              { text: 'pvalidate Tool', link: '/contributing/pvalidate' },
            ]
          }
        ],
      },
    },
  }
}
