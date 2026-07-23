import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'phench',
  description: 'phench',
  outDir: '../docs-dist',
  themeConfig: {
    nav: [{ text: 'Home', link: '/' }],
    sidebar: [{ text: 'Overview', link: '/' }]
  }
})
