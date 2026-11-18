import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'

// Tauri expects a fixed port and fails if it is taken.
export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  build: { outDir: 'dist', emptyOutDir: true, target: 'es2022' },
  test: { environment: 'node', include: ['src/**/*.test.ts'] }
})
