import { createPhenotypeConfig } from "@phenotype/docs/config";

const isPagesBuild =
  process.env.GITHUB_ACTIONS === "true" || process.env.GITHUB_PAGES === "true";
const isCustomDomain = process.env.PHENOTYPE_CUSTOM_DOMAIN === "true";
const repoName = process.env.GITHUB_REPOSITORY?.split("/")[1] || "AgilePlus";
const docsBase = isCustomDomain ? "/" : isPagesBuild ? `/${repoName}/` : "/";

import { createSiteMeta } from "./site-meta.mjs";

const siteMeta = createSiteMeta({ base: docsBase, repoName });

const baseConfig = createPhenotypeConfig(siteMeta);

export default {
  ...baseConfig,
  themeConfig: {
    ...baseConfig.themeConfig,
    nav: [
      ...(baseConfig.themeConfig?.nav ?? []),
      { text: "Compete", link: "/compete/" },
    ],
  },
};
