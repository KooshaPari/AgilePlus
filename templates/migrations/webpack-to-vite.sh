#!/bin/bash
# Webpack to Vite Migration Template

echo "Migrating Webpack to Vite..."

rm -f webpack.config.js webpack.config.ts
rm -rf webpack/

if [ -f package.json ]; then
    bun remove webpack webpack-cli 2>/dev/null || npm uninstall webpack webpack-cli 2>/dev/null || true
    bun add -D vite 2>/dev/null || npm add -D vite 2>/dev/null || true
fi

cat > vite.config.ts << 'VITE'
import { defineConfig } from 'vite'

export default defineConfig({
  server: { port: 3000 },
  build: { outDir: 'dist' },
})
VITE

echo "Webpack to Vite migration complete"
