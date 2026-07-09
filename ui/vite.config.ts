import vue from '@vitejs/plugin-vue'
import { resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { defineConfig } from 'vite'

const uiRoot = fileURLToPath(new URL('.', import.meta.url))

const scssDeprecationSilence = [
  'import',
  'global-builtin',
  'color-functions',
  'if-function',
] as const

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
        quietDeps: true,
        silenceDeprecations: [...scssDeprecationSilence],
      },
      sass: {
        quietDeps: true,
        silenceDeprecations: [...scssDeprecationSilence],
      },
    },
  },
  define: {
    'process.env.NODE_ENV': JSON.stringify(mode),
    'process.env': JSON.stringify({ NODE_ENV: mode }),
    'import.meta': '{}',
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
      transform: {
        define: {
          'import.meta': '{}',
        },
      },
      output: {
        banner: `var process={env:{NODE_ENV:${JSON.stringify(mode)}}};`,
        entryFileNames: 'js/[name]-[hash].js',
        chunkFileNames: 'js/[name]-[hash].js',
        assetFileNames: 'css/main-[hash][extname]',
        format: 'iife',
        name: 'EgoroffUI',
      },
    },
  },
}))
