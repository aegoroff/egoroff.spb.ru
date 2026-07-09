import vue from '@vitejs/plugin-vue'
import { resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { defineConfig } from 'vite'

const uiRoot = fileURLToPath(new URL('.', import.meta.url))

export default defineConfig(({ mode }) => ({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(uiRoot, 'src'),
    },
  },
  css: {
    preprocessorOptions: {
      scss: {
        api: 'modern-compiler',
      },
    },
  },
  define: {
    'process.env.NODE_ENV': JSON.stringify(mode),
    'process.env': JSON.stringify({ NODE_ENV: mode }),
  },
  build: {
    outDir: resolve(uiRoot, '../static/dist'),
    emptyOutDir: true,
    manifest: true,
    cssCodeSplit: false,
    chunkSizeWarningLimit: 2000,
    sourcemap: mode !== 'production',
    rolldownOptions: {
      input: {
        main: resolve(uiRoot, 'build.html'),
      },
      output: {
        entryFileNames: 'js/[name]-[hash].js',
        chunkFileNames: 'js/[name]-[hash].js',
        assetFileNames: 'css/main-[hash][extname]',
      },
    },
  },
}))
