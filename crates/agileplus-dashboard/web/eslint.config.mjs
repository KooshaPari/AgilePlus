import babelParser from '@babel/eslint-parser';
import jsxA11y from 'eslint-plugin-jsx-a11y';
export default [{ ignores: ['dist/**', 'node_modules/**', 'src/**/*.test.*', 'src/test/**'] }, {
  files: ['src/**/*.{ts,tsx}'],
  languageOptions: { parser: babelParser, parserOptions: { requireConfigFile: false, babelOptions: { parserOpts: { plugins: ["typescript", "jsx"] } } } },
  plugins: { 'jsx-a11y': jsxA11y }, rules: jsxA11y.flatConfigs.recommended.rules,
}];
