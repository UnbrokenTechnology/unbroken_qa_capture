#!/usr/bin/env node
/**
 * Headless browser smoke test for Unbroken QA Capture frontend.
 *
 * Starts the Vite dev server (if not already running), opens localhost:5173
 * in headless Chromium via Puppeteer, checks that Quasar components render
 * correctly, Material Icons loaded, and the page has visual content.
 *
 * Exit 0 = pass, exit 1 = fail.
 */

import { spawn } from 'node:child_process'
import { createRequire } from 'node:module'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const projectRoot = path.resolve(__dirname, '..')

// ─── Puppeteer import ────────────────────────────────────────────────────────

const require = createRequire(import.meta.url)
let puppeteer
try {
  puppeteer = require(path.join(projectRoot, 'node_modules', 'puppeteer'))
} catch (e) {
  console.error('ERROR: puppeteer not found. Run: npm install -D puppeteer')
  process.exit(1)
}

// ─── Config ──────────────────────────────────────────────────────────────────

const DEV_URL = 'http://localhost:5173'
const TIMEOUT_MS = 60_000
const VITE_STARTUP_TIMEOUT_MS = 30_000
const SCREENSHOT_PATH = '/tmp/smoke-test-home.png'

// ─── Dev server management ────────────────────────────────────────────────────

let devServer = null
let ownedDevServer = false  // true if WE started it (so we should kill it)

async function isPortOpen(url) {
  // Try both IPv4 and IPv6 addresses since Node.js fetch may prefer IPv4
  // but Vite often binds to IPv6 only in Linux containers
  const urlsToTry = [url]
  if (url.includes('localhost')) {
    urlsToTry.push(url.replace('localhost', '[::1]'))
  }
  for (const u of urlsToTry) {
    try {
      const res = await fetch(u, { signal: AbortSignal.timeout(2000) })
      if (res.ok || res.status < 500) return true
    } catch (_) {
      // try next
    }
  }
  return false
}

async function startDevServerIfNeeded() {
  // Check if dev server is already running
  if (await isPortOpen(DEV_URL)) {
    console.log(`Dev server already running at ${DEV_URL} — skipping startup`)
    return
  }

  console.log('Starting Vite dev server...')

  devServer = spawn('npm', ['run', 'dev'], {
    cwd: projectRoot,
    stdio: ['ignore', 'pipe', 'pipe'],
    env: {
      ...process.env,
      BROWSER: 'none',
    },
  })
  ownedDevServer = true

  return new Promise((resolve, reject) => {
    const startTimer = setTimeout(() => {
      reject(new Error('Vite dev server did not become ready within 30 seconds'))
    }, VITE_STARTUP_TIMEOUT_MS)

    function onData(chunk) {
      const text = chunk.toString()
      process.stdout.write(`[vite] ${text}`)
      if (text.includes('Local:') || text.includes('ready in') || text.includes('localhost:5173')) {
        clearTimeout(startTimer)
        setTimeout(resolve, 1000)
      }
    }

    devServer.stdout.on('data', onData)
    devServer.stderr.on('data', (chunk) => {
      const text = chunk.toString()
      process.stdout.write(`[vite err] ${text}`)
      if (text.includes('Local:') || text.includes('ready in') || text.includes('localhost:5173')) {
        clearTimeout(startTimer)
        setTimeout(resolve, 1000)
      }
    })

    devServer.on('error', (err) => {
      clearTimeout(startTimer)
      reject(new Error(`Failed to start dev server: ${err.message}`))
    })

    devServer.on('exit', (code) => {
      clearTimeout(startTimer)
      if (code !== null && code !== 0) {
        reject(new Error(`Dev server exited with code ${code}`))
      }
    })
  })
}

function killDevServer() {
  if (!devServer || !ownedDevServer) return
  try {
    devServer.kill('SIGTERM')
    setTimeout(() => {
      try { devServer.kill('SIGKILL') } catch (_) {}
    }, 2000)
  } catch (_) {}
  devServer = null
}

// ─── Poll until port is open ─────────────────────────────────────────────────

async function waitForPort(url, maxMs = 15_000) {
  const start = Date.now()
  while (Date.now() - start < maxMs) {
    if (await isPortOpen(url)) return
    await new Promise(r => setTimeout(r, 500))
  }
  throw new Error(`Server at ${url} did not respond within ${maxMs}ms`)
}

// ─── Checks ───────────────────────────────────────────────────────────────────

const PASS = '✓'
const FAIL = '✗'
const failures = []

function check(name, passed, detail = '') {
  if (passed) {
    console.log(`  ${PASS} ${name}${detail ? ' (' + detail + ')' : ''}`)
  } else {
    console.log(`  ${FAIL} ${name}${detail ? ': ' + detail : ''}`)
    failures.push(`${name}${detail ? ': ' + detail : ''}`)
  }
}

// ─── Main ────────────────────────────────────────────────────────────────────

let browser = null

async function cleanup() {
  if (browser) {
    try { await browser.close() } catch (_) {}
    browser = null
  }
  killDevServer()
}

process.on('exit', () => { killDevServer() })
process.on('SIGINT', async () => { await cleanup(); process.exit(1) })
process.on('SIGTERM', async () => { await cleanup(); process.exit(1) })

async function main() {
  const overallTimeout = setTimeout(async () => {
    console.error('\nERROR: Smoke test timed out after 60 seconds')
    await cleanup()
    process.exit(1)
  }, TIMEOUT_MS)

  try {
    // 1. Start dev server (or use existing)
    await startDevServerIfNeeded()
    console.log('Waiting for dev server to be reachable...')
    await waitForPort(DEV_URL, 15_000)
    console.log(`Dev server is up at ${DEV_URL}`)

    // 2. Launch headless browser
    console.log('\nLaunching headless Chromium...')
    browser = await puppeteer.launch({
      headless: true,
      args: [
        '--no-sandbox',
        '--disable-setuid-sandbox',
        '--disable-dev-shm-usage',
        '--disable-gpu',
      ],
    })

    const page = await browser.newPage()
    page.setDefaultNavigationTimeout(20_000)

    // 3. Capture console errors
    const consoleErrors = []
    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        const text = msg.text()
        // Ignore expected Tauri invoke failures and network errors for Tauri protocol.
        // These always occur when running via Vite without the Tauri runtime.
        if (
          text.includes('invoke') ||
          text.includes('__TAURI__') ||
          text.includes('tauri') ||
          text.includes('IPC') ||
          text.includes('net::ERR') ||
          text.includes('ERR_FAILED') ||
          // Tauri window/plugin API calls that fail without the Tauri runtime
          text.includes('window always-on-top') ||
          // Static resource 404s are expected (Tauri-specific assets, favicon, etc.)
          text.includes('Failed to load resource')
        ) {
          return
        }
        consoleErrors.push(text)
      }
    })

    // 4. Navigate to home page
    console.log(`\nNavigating to ${DEV_URL}...`)
    try {
      await page.goto(DEV_URL, { waitUntil: 'networkidle2', timeout: 20_000 })
    } catch (_) {
      // networkidle2 can timeout on apps that keep long-polling; fall back
      console.log('  (networkidle2 timed out, falling back to domcontentloaded)')
      await page.goto(DEV_URL, { waitUntil: 'domcontentloaded', timeout: 20_000 })
    }

    // Wait for Vue to mount
    await page.waitForFunction(() => {
      const app = document.querySelector('#app')
      return app !== null && app.children.length > 0
    }, { timeout: 10_000 }).catch(() => {
      console.log('  (warning: #app did not render children in time)')
    })

    // Extra settle time for fonts and async CSS
    await new Promise(r => setTimeout(r, 2000))

    // 5. Run checks
    console.log('\nRunning checks:')

    // (a) Quasar components render — not plain text
    const quasarCheck = await page.evaluate(() => {
      const bodyText = document.body.innerText || ''
      const rawTagPattern = /\bq-(btn|card|toolbar|icon|page|layout)\b/
      const hasRawTags = rawTagPattern.test(bodyText)
      const btnCount = document.querySelectorAll('button.q-btn').length
      return { hasRawTags, btnCount }
    })
    check(
      'Quasar components render (not plain text)',
      !quasarCheck.hasRawTags,
      quasarCheck.hasRawTags ? 'raw Quasar tag names found in body text' : ''
    )
    check(
      'At least one q-btn renders as a real button element',
      quasarCheck.btnCount > 0,
      quasarCheck.btnCount === 0 ? 'no button.q-btn found' : `${quasarCheck.btnCount} found`
    )

    // (b) Material Icons font loaded
    const fontLoaded = await page.evaluate(async () => {
      await document.fonts.ready
      return document.fonts.check('1em "Material Icons"')
    })
    check(
      'Material Icons font loaded',
      fontLoaded,
      fontLoaded ? '' : '"Material Icons" not in document.fonts'
    )

    // (c) Icons render as icons, not as raw text
    // Material Icons uses font ligatures: the DOM text is the icon name (e.g. "bug_report")
    // but the font renders it as the glyph. If the font fails to load, the raw text is visible.
    // Since we already verify font load in check (b), we verify here that:
    // - Icon elements exist in the DOM (Quasar rendered them)
    // - No icon is rendering an aria-hidden span with just the raw ligature text AND the font failed
    // The combined signal: font loaded + icons present = icons are rendering correctly
    const iconCheck = await page.evaluate(() => {
      const iconEls = Array.from(document.querySelectorAll('.q-icon, .material-icons'))
      const ligatureIcons = iconEls
        .map(el => el.textContent?.trim() ?? '')
        .filter(text => /^[a-z_]{3,}$/.test(text))
      return { total: iconEls.length, ligatureCount: ligatureIcons.length }
    })
    // Ligature icons are expected and correct — it means Quasar rendered them.
    // The font-loaded check (b) confirms the font renders them as glyphs, not raw text.
    check(
      'Icon elements are present in DOM (Quasar rendered icons)',
      iconCheck.total > 0,
      `${iconCheck.total} icon element(s) found`
    )

    // (d) No critical console errors
    check(
      'No unexpected console errors',
      consoleErrors.length === 0,
      consoleErrors.length > 0 ? consoleErrors.slice(0, 3).join('; ') : ''
    )

    // (e) Page has visual content
    const contentCheck = await page.evaluate(() => {
      const scrollHeight = document.body.scrollHeight
      const qPage = document.querySelector('.q-page, main, #app')
      const rect = qPage ? qPage.getBoundingClientRect() : null
      return {
        scrollHeight,
        hasQPage: qPage !== null,
        contentHeight: rect ? Math.round(rect.height) : 0,
        contentWidth: rect ? Math.round(rect.width) : 0,
      }
    })
    check(
      'Page has visual content (scrollHeight > 100)',
      contentCheck.scrollHeight > 100,
      `scrollHeight = ${contentCheck.scrollHeight}px`
    )
    check(
      'Main content area exists with dimensions',
      contentCheck.hasQPage && contentCheck.contentHeight > 0,
      contentCheck.hasQPage
        ? `${contentCheck.contentWidth}×${contentCheck.contentHeight}px`
        : 'no .q-page/main/#app with positive dimensions'
    )

    // (f) Screenshot
    await page.screenshot({ path: SCREENSHOT_PATH, fullPage: true })
    console.log(`\n  Screenshot saved: ${SCREENSHOT_PATH}`)

    // 6. Results
    clearTimeout(overallTimeout)
    console.log('')
    if (failures.length === 0) {
      console.log('SMOKE TEST PASSED')
    } else {
      console.log(`SMOKE TEST FAILED (${failures.length} check(s) failed):`)
      failures.forEach(f => console.log(`  - ${f}`))
    }

    await cleanup()
    process.exit(failures.length === 0 ? 0 : 1)

  } catch (err) {
    clearTimeout(overallTimeout)
    console.error(`\nSMOKE TEST ERROR: ${err.message}`)
    if (err.stack) console.error(err.stack)
    await cleanup()
    process.exit(1)
  }
}

main()
