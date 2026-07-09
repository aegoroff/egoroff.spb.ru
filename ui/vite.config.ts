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
  define: {
    'process.env.NODE_ENV': JSON.stringify(mode),
    'process.env': JSON.stringify({ NODE_ENV: mode }),
  },
  build: {
    outDir: resolve(uiRoot, '../static/dist'),
    emptyOutDir: true,
    manifest: true,
    cssCodeSplit: false,
    sourcemap: mode !== 'production',
    rollupOptions: {
      input: {
        main: resolve(uiRoot, 'build.html'),
      },
      output: {
        banner: `var process={env:{NODE_ENV:${JSON.stringify(mode)}}};`,
        entryFileNames: 'js/[name]-[hash].js',
        chunkFileNames: 'js/[name]-[hash].js',
        assetFileNames: 'css/main-[hash][extname]',
        format: 'iife',
        name: 'EgoroffUI',
        inlineDynamicImports: true,
      },
    },
  },
}))
