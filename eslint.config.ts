// ESLint = finds bugs and bad patterns in your code (it doesn't format).
// Prettier = formats code. "skipFormatting" turns off ESLint's style rules
// so the two tools never fight each other.
import pluginVue from 'eslint-plugin-vue'
import { defineConfigWithVueTs, vueTsConfigs } from '@vue/eslint-config-typescript'
import skipFormatting from '@vue/eslint-config-prettier/skip-formatting'

export default defineConfigWithVueTs(
  { name: 'app/files-to-lint', files: ['**/*.{ts,mts,vue}'] },
  {
    name: 'app/files-to-ignore',
    ignores: [
      'ui/dist/**',
      'target/**',
      'src-tauri/gen/**',
      'node_modules/**',
      'test-results/**',
      'playwright-report/**',
      '.superpowers/**'
    ]
  },

  pluginVue.configs['flat/recommended'], // Vue best-practice rules
  vueTsConfigs.recommended, // TypeScript rules
  skipFormatting
)
