// AgilePlus VitePress config.
// The deploy workflow runs with `working-directory: docs`, so this config is
// loaded as docs/.vitepress/config.mts. srcDir: '.' means VitePress treats the
// docs/ directory itself as the content root — existing docs/requirements/*.md,
// docs/tooling.md etc. are served directly without moving files.
//
// Base-path: served at kooshapari.github.io/AgilePlus/ → base='/AgilePlus/' on
// GitHub Pages. No custom domain is configured. If a custom domain is added,
// set PHENOTYPE_CUSTOM_DOMAIN=true in the deploy workflow and add a CNAME file
// under docs/public/.
import { createPhenotypeConfig } from '@phenotype/docs/config'

const isPagesBuild =
  process.env.GITHUB_ACTIONS === 'true' || process.env.GITHUB_PAGES === 'true'
const repoName = process.env.GITHUB_REPOSITORY?.split('/')[1] ?? 'AgilePlus'
// Honor custom-domain override: PHENOTYPE_CUSTOM_DOMAIN=true → serve from /
const customDomain = process.env.PHENOTYPE_CUSTOM_DOMAIN === 'true'
const docsBase = customDomain ? '/' : isPagesBuild ? `/${repoName}/` : '/'

export default createPhenotypeConfig({
  title: 'AgilePlus',
  description: 'Phenotype-org spec-driven development platform',
  srcDir: '.',
  base: docsBase,
  githubOrg: 'KooshaPari',
  githubRepo: repoName,

  nav: [
    { text: 'Requirements', link: '/requirements/agileplus-frnfr' },
    { text: 'Tooling', link: '/tooling' },
  ],

  sidebar: {
    '/requirements/': [
      {
        text: 'Requirements',
        items: [
          { text: 'AgilePlus FR/NFR', link: '/requirements/agileplus-frnfr' },
          { text: 'AuthVault FR/NFR', link: '/requirements/authvault-frnfr' },
          { text: 'PhenoMCP FR/NFR', link: '/requirements/phenomcp-frnfr' },
          {
            text: 'PhenoObservability FR/NFR',
            link: '/requirements/phenoobservability-frnfr',
          },
          {
            text: 'Phenotype Voxel FR/NFR',
            link: '/requirements/phenotype-voxel-frnfr',
          },
          { text: 'Tracera FR/NFR', link: '/requirements/tracera-frnfr' },
        ],
      },
    ],
  },
})
