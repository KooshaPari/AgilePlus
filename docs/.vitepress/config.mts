import { createPhenotypeConfig } from '@phenotype/docs/config'
import { createSiteMeta } from './site-meta.mjs'
import { resolveDocsBase } from './resolve-base.mjs'

const repoName = process.env.GITHUB_REPOSITORY?.split('/')[1] || 'AgilePlus'
const docsBase = resolveDocsBase(repoName)
const siteMeta = createSiteMeta({ base: docsBase, repoName })

export default createPhenotypeConfig(siteMeta)
