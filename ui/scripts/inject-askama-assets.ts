import { readFileSync, writeFileSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const uiRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const outDir = resolve(uiRoot, '../static/dist')
const indexPath = resolve(outDir, 'index.html')
const manifestPath = resolve(outDir, '.vite/manifest.json')
const marker = '<!-- built files will be auto injected -->'

type ManifestChunk = {
  file: string
  css?: string[]
  isEntry?: boolean
}

function loadManifest(): Record<string, ManifestChunk> {
  try {
    return JSON.parse(readFileSync(manifestPath, 'utf8')) as Record<string, ManifestChunk>
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    throw new Error(`Failed to read Vite manifest at ${manifestPath}: ${message}`)
  }
}

function buildInjection(entry: ManifestChunk): string {
  const styles = (entry.css ?? [])
    .map((file) => `<link rel="stylesheet" href="/${file}">`)
    .join('\n    ')
  const script = `<script type="module" src="/${entry.file}"></script>`

  return [styles, script].filter((line) => line.length > 0).join('\n    ')
}

function injectAssets(html: string, injection: string): string {
  if (html.includes(marker)) {
    return html.replace(marker, injection)
  }

  const previousInjection =
    /<link rel="stylesheet" href="\/css\/[^"]+">\s*\n\s*<script type="module" src="\/js\/[^"]+"><\/script>/

  if (previousInjection.test(html)) {
    return html.replace(previousInjection, injection)
  }

  return html.replace('</body>', `    ${injection}\n</body>`)
}

const manifest = loadManifest()
const entry = Object.values(manifest).find((chunk) => chunk.isEntry)

if (entry === undefined) {
  throw new Error('Vite manifest does not contain an entry chunk')
}

const injection = buildInjection(entry)
const html = readFileSync(indexPath, 'utf8')
const updated = injectAssets(html, injection)

writeFileSync(indexPath, updated)

console.log(`Injected assets into ${indexPath}`)
console.log(`  css: ${(entry.css ?? []).join(', ') || '(none)'}`)
console.log(`  js:  ${entry.file}`)
