import vue from '@vitejs/plugin-vue'
import { readFileSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { defineConfig, type Plugin } from 'vite'

const uiRoot = fileURLToPath(new URL('.', import.meta.url))
const outDir = resolve(uiRoot, '../static/dist')
const askamaIndexPath = resolve(outDir, 'index.html')
const manifestPaths = [
  resolve(outDir, 'manifest.json'),
  resolve(outDir, '.vite/manifest.json'),
]

function injectAskamaAssets(): Plugin {
  return {
    name: 'inject-askama-assets',
    closeBundle() {
      const manifestPath = manifestPaths.find((path) => {
        try {
          readFileSync(path, 'utf8')
          return true
        } catch {
          return false
        }
      })

      if (manifestPath === undefined) {
        throw new Error('Vite manifest was not found after build')
      }

      const manifest = JSON.parse(readFileSync(manifestPath, 'utf8')) as Record<
        string,
        { file: string; css?: string[]; isEntry?: boolean }
      >

      const entry = Object.values(manifest).find((chunk) => chunk.isEntry)
      if (entry === undefined) {
        throw new Error('Vite manifest does not contain an entry chunk')
      }

      const styles = (entry.css ?? [])
        .map((file) => `<link rel="stylesheet" href="/${file}">`)
        .join('\n    ')
      const script = `<script type="module" src="/${entry.file}"></script>`

      const injection = [styles, script].filter((line) => line.length > 0).join('\n    ')
      const marker = '<!-- built files will be auto injected -->'

      let html = readFileSync(askamaIndexPath, 'utf8')
      if (html.includes(marker)) {
        html = html.replace(marker, injection)
      } else {
        html = html.replace('</body>', `    ${injection}\n</body>`)
      }

      writeFileSync(askamaIndexPath, html)
    },
  }
}

export default defineConfig({
  plugins: [vue(), injectAskamaAssets()],
  resolve: {
    alias: {
      '@': resolve(uiRoot, 'src'),
    },
  },
  build: {
    outDir,
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
