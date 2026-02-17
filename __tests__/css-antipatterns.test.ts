/**
 * CSS anti-pattern test: wildcard font-family overrides
 *
 * Scans all .vue files for `font-family` declarations on wildcard/universal
 * selectors (*, :deep(*), ::v-deep(*)).  Such overrides clobber the
 * `font-family: 'Material Icons'` rule that Quasar sets on .material-icons
 * elements, causing icon glyphs to render as plain text (e.g. "bug_report"
 * instead of the bug icon).
 *
 * The correct way to configure the app's typography font is via
 * `$typography-font-family` in src/quasar-variables.sass — Quasar applies it
 * without touching icon-font elements.
 *
 * Allowed: font-family on specific scoped selectors (e.g. .console-line)
 * Forbidden: font-family on *, :deep(*), ::v-deep(*), or :deep(*, …)
 *
 * Approach: static analysis of every src/**\/*.vue file.
 */

import { describe, it, expect } from 'vitest'
import { readFileSync, readdirSync, statSync } from 'fs'
import { resolve, join } from 'path'

const ROOT = resolve(__dirname, '../')

// ---------------------------------------------------------------------------
// File discovery
// ---------------------------------------------------------------------------

/** Recursively collect all .vue files under the given directory. */
function collectVueFiles(dir: string): string[] {
  const results: string[] = []
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry)
    const stat = statSync(full)
    if (stat.isDirectory()) {
      results.push(...collectVueFiles(full))
    } else if (entry.endsWith('.vue')) {
      results.push(full)
    }
  }
  return results
}

const vueFiles = collectVueFiles(resolve(ROOT, 'src'))

// ---------------------------------------------------------------------------
// Detection helpers
// ---------------------------------------------------------------------------

/**
 * Extract all <style> block contents from a .vue SFC source string.
 * Returns an array of style block content strings (without the tags).
 */
function extractStyleBlocks(source: string): string[] {
  const blocks: string[] = []
  // Match <style ...>...</style> (including scoped/lang attributes)
  const styleTagRe = /<style[^>]*>([\s\S]*?)<\/style>/gi
  let match: RegExpExecArray | null
  while ((match = styleTagRe.exec(source)) !== null) {
    if (match[1] !== undefined) {
      blocks.push(match[1])
    }
  }
  return blocks
}

/**
 * Returns true if the CSS text contains a `font-family` property on a
 * wildcard selector: *, :deep(*), ::v-deep(*), or :deep(*, …).
 *
 * We look for the following patterns anywhere in the style block:
 *   *   { font-family: ... }        — bare universal selector
 *   :deep(*) { font-family: ... }   — Vue deep combinator with wildcard
 *   ::v-deep(*) { font-family: ... }
 *
 * We do NOT flag:
 *   .console-line { font-family: monospace }   — scoped class selector
 *   :deep(.q-header) { ... }                   — deep with specific selector
 *   * + * { ... }                              — no font-family property
 */
function hasWildcardFontFamily(css: string): boolean {
  // Strip CSS comments first so they don't confuse the scan
  const stripped = css.replace(/\/\*[\s\S]*?\*\//g, '')

  // Split into "rule blocks" by finding selector { ... } patterns.
  // We look for each opening brace to find the rule selector text.
  const lines = stripped.split('\n')

  // State machine: track current selector context
  let braceDepth = 0
  let currentSelector = ''
  let selectorBuffer = ''

  for (const line of lines) {
    const trimmed = line.trim()

    for (let i = 0; i < trimmed.length; i++) {
      const ch = trimmed[i]

      if (ch === '{') {
        if (braceDepth === 0) {
          // Starting a new top-level rule block
          currentSelector = selectorBuffer.trim()
          selectorBuffer = ''
        }
        braceDepth++
      } else if (ch === '}') {
        braceDepth--
        if (braceDepth === 0) {
          currentSelector = ''
        }
      } else if (braceDepth === 0) {
        selectorBuffer += ch
      }
    }

    // After consuming the line, if we're inside a rule block,
    // check if this line declares font-family and the selector is a wildcard.
    if (braceDepth > 0 && /font-family\s*:/i.test(trimmed)) {
      if (isWildcardSelector(currentSelector)) {
        return true
      }
    }
  }

  return false
}

/**
 * Returns true if the selector string is a universal/wildcard selector that
 * would override font-family on icon elements.
 *
 * Wildcard selectors:
 *   *
 *   :deep(*)
 *   ::v-deep(*)
 *   :deep(*,  ...)   — wildcard inside :deep group
 */
function isWildcardSelector(selector: string): boolean {
  const s = selector.trim()

  // Bare * selector (optionally with whitespace or combinators before it)
  // e.g. "  *  " or "* >"
  if (/^\s*\*\s*$/.test(s)) return true

  // :deep(*) or ::v-deep(*) — possibly with whitespace inside the parens
  if (/::?(?:v-deep|deep)\s*\(\s*\*\s*\)/.test(s)) return true

  // :deep(*,  ...) — wildcard is the first item in a selector group inside :deep
  if (/::?(?:v-deep|deep)\s*\(\s*\*/.test(s)) return true

  return false
}

// ---------------------------------------------------------------------------
// Violation reporter
// ---------------------------------------------------------------------------

interface Violation {
  file: string
  styleBlock: number
  selector: string
}

/**
 * Scan a .vue file source for wildcard font-family violations.
 * Returns an array of violation objects (empty if none found).
 */
function findViolations(filePath: string): Violation[] {
  const source = readFileSync(filePath, 'utf-8')
  const styleBlocks = extractStyleBlocks(source)
  const violations: Violation[] = []

  styleBlocks.forEach((block, blockIndex) => {
    // Use the more targeted line-by-line approach
    const stripped = block.replace(/\/\*[\s\S]*?\*\//g, '')
    const lines = stripped.split('\n')

    let braceDepth = 0
    let currentSelector = ''
    let selectorBuffer = ''

    for (const line of lines) {
      const trimmed = line.trim()

      for (let i = 0; i < trimmed.length; i++) {
        const ch = trimmed[i]

        if (ch === '{') {
          if (braceDepth === 0) {
            currentSelector = selectorBuffer.trim()
            selectorBuffer = ''
          }
          braceDepth++
        } else if (ch === '}') {
          braceDepth--
          if (braceDepth === 0) {
            currentSelector = ''
          }
        } else if (braceDepth === 0) {
          selectorBuffer += ch
        }
      }

      if (braceDepth > 0 && /font-family\s*:/i.test(trimmed)) {
        if (isWildcardSelector(currentSelector)) {
          violations.push({
            file: filePath.replace(ROOT + '/', ''),
            styleBlock: blockIndex + 1,
            selector: currentSelector,
          })
        }
      }
    }
  })

  return violations
}

// ---------------------------------------------------------------------------
// Unit tests for the detection logic
// ---------------------------------------------------------------------------

describe('CSS anti-pattern detector – unit tests', () => {
  describe('isWildcardSelector()', () => {
    it('detects bare * selector', () => {
      expect(isWildcardSelector('*')).toBe(true)
    })

    it('detects * with surrounding whitespace', () => {
      expect(isWildcardSelector('  *  ')).toBe(true)
    })

    it('detects :deep(*)', () => {
      expect(isWildcardSelector(':deep(*)')).toBe(true)
    })

    it('detects ::v-deep(*)', () => {
      expect(isWildcardSelector('::v-deep(*)')).toBe(true)
    })

    it('detects :deep(*) with internal whitespace', () => {
      expect(isWildcardSelector(':deep( * )')).toBe(true)
    })

    it('does NOT flag .console-line (scoped class selector)', () => {
      expect(isWildcardSelector('.console-line')).toBe(false)
    })

    it('does NOT flag :deep(.q-header) (deep with specific selector)', () => {
      expect(isWildcardSelector(':deep(.q-header)')).toBe(false)
    })

    it('does NOT flag h1 (element selector)', () => {
      expect(isWildcardSelector('h1')).toBe(false)
    })

    it('does NOT flag .app-wrapper (class selector)', () => {
      expect(isWildcardSelector('.app-wrapper')).toBe(false)
    })
  })

  describe('hasWildcardFontFamily()', () => {
    it('detects font-family on * selector', () => {
      const css = `
        * {
          font-family: 'Inter', sans-serif;
        }
      `
      expect(hasWildcardFontFamily(css)).toBe(true)
    })

    it('detects font-family on :deep(*)', () => {
      const css = `
        :deep(*) {
          font-family: 'Inter', sans-serif;
        }
      `
      expect(hasWildcardFontFamily(css)).toBe(true)
    })

    it('detects font-family on ::v-deep(*)', () => {
      const css = `
        ::v-deep(*) {
          font-family: monospace;
        }
      `
      expect(hasWildcardFontFamily(css)).toBe(true)
    })

    it('does NOT flag font-family on .console-line', () => {
      const css = `
        .console-line {
          font-family: monospace;
        }
      `
      expect(hasWildcardFontFamily(css)).toBe(false)
    })

    it('does NOT flag :deep(*) { -webkit-font-smoothing: antialiased } (no font-family)', () => {
      const css = `
        :deep(*) {
          -webkit-font-smoothing: antialiased;
        }
      `
      expect(hasWildcardFontFamily(css)).toBe(false)
    })

    it('does NOT flag font-family in CSS comments', () => {
      const css = `
        /* * { font-family: 'Inter'; } — removed because it broke icons */
        .app {
          color: red;
        }
      `
      expect(hasWildcardFontFamily(css)).toBe(false)
    })

    it('does NOT flag :deep(.q-header) with font-family', () => {
      const css = `
        :deep(.q-header) {
          font-family: 'Inter', sans-serif;
        }
      `
      expect(hasWildcardFontFamily(css)).toBe(false)
    })
  })
})

// ---------------------------------------------------------------------------
// Integration: scan all Vue SFCs in the project
// ---------------------------------------------------------------------------

describe('CSS anti-pattern scan – all Vue SFCs', () => {
  it('found .vue files to scan', () => {
    expect(vueFiles.length).toBeGreaterThan(0)
  })

  it('no .vue file uses font-family on a wildcard selector', () => {
    const allViolations: Violation[] = []

    for (const file of vueFiles) {
      allViolations.push(...findViolations(file))
    }

    if (allViolations.length > 0) {
      const report = allViolations
        .map((v) => `  ${v.file} (style block ${v.styleBlock}): selector "${v.selector}"`)
        .join('\n')
      // Fail with a clear message listing every violating file
      expect.fail(
        `Found font-family on wildcard selector(s) — this clobbers Material Icons rendering:\n${report}\n\n` +
          `Fix: configure the app font via $typography-font-family in src/quasar-variables.sass instead.`,
      )
    }

    expect(allViolations).toHaveLength(0)
  })
})
