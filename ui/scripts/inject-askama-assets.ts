import { mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const uiRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const outDir = resolve(uiRoot, '../static/dist')
const indexPath = resolve(outDir, 'index.html')
const manifestPath = resolve(outDir, '.vite/manifest.json')
const stylesMarker = '<!-- built styles will be auto injected -->'
const scriptsMarker = '<!-- built scripts will be auto injected -->'
const legacyMarker = '<!-- built files will be auto injected -->'
const injectedStyleLine =
  /^\s*<link rel="stylesheet" href="\/css\/[^"]+">\s*$/
const injectedScriptLine =
  /^\s*<script(?: defer)?(?: type="module")? src="\/js\/[^"]+"><\/script>\s*$/
const legacyInjectionPattern =
  /<link rel="stylesheet" href="\/css\/[^"]+">\s*\n\s*<script(?: defer)?(?: type="module")? src="\/js\/[^"]+"><\/script>/g

type ManifestChunk = {
  file?: string
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

function collectStyles(manifest: Record<string, ManifestChunk>): string[] {
  const styles = new Set<string>()

  for (const chunk of Object.values(manifest)) {
    for (const file of chunk.css ?? []) {
      styles.add(file)
    }

    if (chunk.file?.endsWith('.css')) {
      styles.add(chunk.file)
    }
  }

  return [...styles]
}

function buildStyles(files: string[]): string {
  return files
    .map((file) => `<link rel="stylesheet" href="/${file}">`)
    .join('\n    ')
}

function buildScript(file: string): string {
  return `<script type="module" src="/${file}"></script>`
}

function replaceMarker(html: string, marker: string, injection: string): string {
  if (html.includes(marker)) {
    return html.replace(marker, injection)
  }

  return html
}

function replaceUntilStable(
  html: string,
  pattern: RegExp,
  replacement: string,
): string {
  let updated = html

  for (;;) {
    const next = updated.replace(pattern, replacement)
    if (next === updated) {
      return next
    }
    updated = next
  }
}

function removeInjectedAssetLines(html: string): string {
  return html
    .split('\n')
    .filter((line) => !injectedStyleLine.test(line) && !injectedScriptLine.test(line))
    .join('\n')
}

function stripLegacyInjection(html: string): string {
  const withoutLegacyPair = replaceUntilStable(
    html,
    legacyInjectionPattern,
    legacyMarker,
  )

  return removeInjectedAssetLines(withoutLegacyPair)
}

function injectAssets(html: string, styles: string, script: string): string {
  let updated = stripLegacyInjection(html)

  if (styles.length > 0) {
    updated = replaceMarker(updated, stylesMarker, styles)
    if (!updated.includes(styles)) {
      updated = updated.replace('</head>', `    ${styles}\n</head>`)
    }
  } else if (updated.includes(stylesMarker)) {
    updated = updated.replace(stylesMarker, '')
  }

  updated = replaceMarker(updated, scriptsMarker, script)
  if (!updated.includes(script)) {
    updated = updated.replace('</body>', `    ${script}\n</body>`)
  }

  if (updated.includes(legacyMarker)) {
    updated = updated.replace(legacyMarker, script)
  }

  return updated
}

const manifest = loadManifest()
const entry = Object.values(manifest).find((chunk) => chunk.isEntry)

if (entry?.file === undefined) {
  throw new Error('Vite manifest does not contain an entry chunk')
}

const styleFiles = collectStyles(manifest)
const styles = buildStyles(styleFiles)
const script = buildScript(entry.file)
const html = readFileSync(indexPath, 'utf8')
const updated = injectAssets(html, styles, script)

writeFileSync(indexPath, updated)

if (styleFiles.length === 0) {
  const cssDir = resolve(outDir, 'css')
  mkdirSync(cssDir, { recursive: true })
  writeFileSync(resolve(cssDir, '.keep'), '')
}

console.log(`Injected assets into ${indexPath}`)
console.log(`  css: ${styleFiles.join(', ') || '(none)'}`)
console.log(`  js:  ${entry.file}`)
