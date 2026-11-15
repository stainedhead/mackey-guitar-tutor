import js from '@eslint/js';
import jsxA11y from 'eslint-plugin-jsx-a11y';
import reactHooks from 'eslint-plugin-react-hooks';
import globals from 'globals';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  {
    ignores: [
      'dist',
      'dist-mock',
      'node_modules',
      'coverage',
      'test-results',
      'playwright-report',
      'src/bindings',
      'src/mocks/scenarios.json',
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.strictTypeChecked,
  jsxA11y.flatConfigs.recommended,
  {
    files: ['**/*.{ts,tsx}'],
    languageOptions: {
      globals: { ...globals.browser },
      parserOptions: { projectService: true, tsconfigRootDir: import.meta.dirname },
    },
    plugins: { 'react-hooks': reactHooks },
    rules: {
      ...reactHooks.configs.recommended.rules,
      '@typescript-eslint/restrict-template-expressions': ['error', { allowNumber: true }],
    },
  },
  { files: ['**/*.js'], ...tseslint.configs.disableTypeChecked },
  {
    files: ['*.js', '*.ts'],
    languageOptions: { globals: { ...globals.node } },
  },
);
