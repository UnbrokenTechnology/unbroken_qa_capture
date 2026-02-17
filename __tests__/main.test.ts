/**
 * Contract test: main.ts Quasar icon set and CSS configuration
 *
 * Verifies that src/main.ts:
 *  1. Imports and registers the Quasar material-icons icon set
 *  2. Imports the required @quasar/extras icon CSS
 *  3. Imports the Quasar base CSS
 *  4. Installs the Pinia plugin
 *  5. Installs the Vue Router plugin
 *  6. Installs the Quasar plugin with the correct icon set
 *
 * Approach: static analysis of src/main.ts via readFileSync.
 * This catches missing imports / configuration before they cause
 * invisible runtime failures (blank icons, unstyled components, etc.).
 */

import { describe, it, expect } from 'vitest'
import { readFileSync } from 'fs'
import { resolve } from 'path'

const ROOT = resolve(__dirname, '../')
const mainTs = readFileSync(resolve(ROOT, 'src/main.ts'), 'utf-8')

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Return true if any import statement in the file matches the given module path. */
function hasImport(source: string, modulePath: string): boolean {
  // Matches: import ... from 'modulePath'  (exact match, single or double quotes)
  const escaped = modulePath.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const pattern = new RegExp(`import[^'"]*['"]${escaped}['"]`)
  // Also match bare side-effect imports: import 'modulePath'
  const sideEffect = new RegExp(`import\\s*['"]${escaped}['"]`)
  return pattern.test(source) || sideEffect.test(source)
}

/** Return true if the file contains an iconSet option passed to app.use(Quasar, …). */
function hasIconSetOption(source: string): boolean {
  return /iconSet\s*:/.test(source)
}

/** Return true if the file imports from the given quasar icon-set path. */
function hasIconSetImport(source: string): boolean {
  return /import[^'"]*['"]quasar\/icon-set\/[^'"]+['"]/.test(source)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('main.ts – Quasar icon set configuration', () => {
  it('imports an icon set from quasar/icon-set/*', () => {
    expect(hasIconSetImport(mainTs)).toBe(true)
  })

  it('imports the material-icons icon set specifically', () => {
    expect(hasImport(mainTs, 'quasar/icon-set/material-icons')).toBe(true)
  })

  it('passes iconSet to the Quasar plugin options', () => {
    expect(hasIconSetOption(mainTs)).toBe(true)
  })
})

describe('main.ts – required CSS imports', () => {
  it('imports @quasar/extras material-icons CSS', () => {
    expect(hasImport(mainTs, '@quasar/extras/material-icons/material-icons.css')).toBe(true)
  })

  it('imports Quasar base CSS (quasar/src/css/index.sass or quasar/dist/quasar.css)', () => {
    const hasIndexSass = hasImport(mainTs, 'quasar/src/css/index.sass')
    const hasDistCss = hasImport(mainTs, 'quasar/dist/quasar.css')
    expect(hasIndexSass || hasDistCss).toBe(true)
  })
})

describe('main.ts – Vue app plugin installation', () => {
  it('installs Pinia via app.use(createPinia())', () => {
    // Must both import createPinia and call it
    expect(hasImport(mainTs, 'pinia')).toBe(true)
    expect(/createPinia/.test(mainTs)).toBe(true)
    expect(/app\.use\(\s*createPinia\(\)/.test(mainTs)).toBe(true)
  })

  it('installs Vue Router via app.use(router)', () => {
    // Must import router and pass it to app.use
    expect(/import\s+router/.test(mainTs)).toBe(true)
    expect(/app\.use\(\s*router\s*\)/.test(mainTs)).toBe(true)
  })

  it('installs Quasar plugin via app.use(Quasar, …)', () => {
    expect(/app\.use\(\s*Quasar/.test(mainTs)).toBe(true)
  })

  it('mounts the app to #app', () => {
    expect(/app\.mount\(\s*['"]#app['"]\s*\)/.test(mainTs)).toBe(true)
  })
})

describe('main.ts – full source snapshot (informational)', () => {
  it('matches the expected structure snapshot', () => {
    // This snapshot documents the current state of main.ts configuration.
    // Update it intentionally when changing bootstrap configuration.
    expect(mainTs).toMatchSnapshot()
  })
})
