export function resolveDocsBase(repoName) {
  const explicit = process.env.DOCS_BASE ?? process.env.VITEPRESS_BASE;
  if (explicit) {
    return explicit.endsWith('/') ? explicit : `${explicit}/`;
  }
  if (process.env.PHENOTYPE_CUSTOM_DOMAIN === 'true') return '/';
  if (process.env.GITHUB_PAGES === 'true') return `/${repoName}/`;
  return '/';
}
