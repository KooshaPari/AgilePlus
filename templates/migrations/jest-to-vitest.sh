#!/bin/bash
# Jest to Vitest Migration Template

echo "Migrating Jest to Vitest..."

# Remove jest config
rm -f jest.config.js jest.config.ts jest.setup.js jest.setup.ts

# Update package.json
if [ -f package.json ]; then
    sed -i '' '/"jest"/d' package.json 2>/dev/null || sed -i '/"jest"/d' package.json
    sed -i '' '/"@types\/jest"/d' package.json 2>/dev/null || sed -i '/"@types\/jest"/d' package.json
    bun add -D vitest @vitest/ui || npm add -D vitest @vitest/ui
fi

# Create vitest.config.ts
cat > vitest.config.ts << 'VITEST'
import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
    include: ['src/**/*.{test,spec}.{js,ts}'],
  },
})
VITEST

echo "Jest to Vitest migration complete"
