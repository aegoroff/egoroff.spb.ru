import vue from '@vitejs/plugin-vue'
import { resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { defineConfig } from 'vite'

const uiRoot = fileURLToPath(new URL('.', import.meta.url))

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(uiRoot, 'src'),
    },
  },
  build: {
    outDir: resolve(uiRoot, '../static/dist'),
    emptyOutDir: true,
    manifest: true,
    sourcemap: process.env.NODE_ENV !== 'production',
    rollupOptions: {
      input: resolve(uiRoot, 'src/main.ts'),
      output: {
        entryFileNames: 'js/[name]-[hash].js',
        chunkFileNames: 'js/[name]-[hash].js',
        assetFileNames: (assetInfo) => {
          if (assetInfo.names.some((name) => name.endsWith('.css'))) {
            return 'css/[name]-[hash][extname]'
          }

          return 'assets/[name]-[hash][extname]'
        },
      },
    },
  },
})
