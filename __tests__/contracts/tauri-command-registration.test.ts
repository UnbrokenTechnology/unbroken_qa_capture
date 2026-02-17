/**
 * Contract test: Tauri command registration
 *
 * Verifies that every command the frontend invokes (src/api/tauri.ts) has a
 * corresponding registered handler in the Rust backend invoke_handler macro
 * (src-tauri/src/lib.rs).
 *
 * Missing backend commands cause silent runtime failures at the Tauri IPC
 * boundary, so we catch them statically here instead of at runtime.
 *
 * Approach:
 *  1. Parse src/api/tauri.ts with a regex to extract all invoke() call strings
 *  2. Parse src-tauri/src/lib.rs with a regex to extract all identifiers inside
 *     the tauri::generate_handler![ … ] macro
 *  3. Assert every frontend command appears in the backend registration list
 *
 * Note: Both files use snake_case for command names at the IPC boundary.
 * The frontend TypeScript wrapper functions may use camelCase names, but the
 * string literals passed to invoke() are always snake_case and must match the
 * Rust function names registered in invoke_handler.
 */

import { describe, it, expect } from 'vitest'
import { readFileSync } from 'fs'
import { resolve } from 'path'

const ROOT = resolve(__dirname, '../../')

// ---------------------------------------------------------------------------
// Parsers
// ---------------------------------------------------------------------------

/**
 * Extract every command string literal passed to invoke() in the given source.
 *
 * Matches both:
 *   invoke('command_name', ...)
 *   invoke<T>('command_name', ...)
 *
 * Returns unique command names in sorted order.
 */
function extractFrontendCommands(source: string): string[] {
  // Match invoke( or invoke<...>( followed by a single-quoted string
  const pattern = /invoke(?:<[^>]*>)?\(\s*'([^']+)'/g
  const commands = new Set<string>()
  let match: RegExpExecArray | null
  while ((match = pattern.exec(source)) !== null) {
    commands.add(match[1]!)
  }
  return [...commands].sort()
}

/**
 * Extract every command identifier listed inside tauri::generate_handler![ … ]
 * in the given Rust source.
 *
 * The macro block is terminated by the first `]` that closes it. Identifiers
 * are separated by commas and optional whitespace/newlines.
 *
 * Returns unique command names in sorted order.
 */
function extractBackendHandlers(source: string): string[] {
  // Find the generate_handler![ ... ] block
  const startMarker = 'tauri::generate_handler!['
  const startIdx = source.indexOf(startMarker)
  if (startIdx === -1) {
    throw new Error('Could not find tauri::generate_handler![ in lib.rs')
  }

  const blockStart = startIdx + startMarker.length
  const blockEnd = source.indexOf(']', blockStart)
  if (blockEnd === -1) {
    throw new Error('Could not find closing ] of tauri::generate_handler! macro')
  }

  const block = source.slice(blockStart, blockEnd)
  const handlers = new Set<string>()

  // Each entry is a Rust identifier (word chars and underscores)
  const identPattern = /\b([a-z][a-z0-9_]*)\b/g
  let match: RegExpExecArray | null
  while ((match = identPattern.exec(block)) !== null) {
    handlers.add(match[1]!)
  }

  return [...handlers].sort()
}

// ---------------------------------------------------------------------------
// Load source files
// ---------------------------------------------------------------------------

const tauriTs = readFileSync(resolve(ROOT, 'src/api/tauri.ts'), 'utf-8')
const libRs = readFileSync(resolve(ROOT, 'src-tauri/src/lib.rs'), 'utf-8')

const frontendCommands = extractFrontendCommands(tauriTs)
const backendHandlers = extractBackendHandlers(libRs)

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('Tauri command registration contract', () => {
  it('frontend calls at least one invoke() command', () => {
    expect(frontendCommands.length).toBeGreaterThan(0)
  })

  it('backend registers at least one handler', () => {
    expect(backendHandlers.length).toBeGreaterThan(0)
  })

  it('every frontend invoke() call has a registered backend handler', () => {
    const backendSet = new Set(backendHandlers)
    const missing = frontendCommands.filter(cmd => !backendSet.has(cmd))

    if (missing.length > 0) {
      // Provide a clear diagnostic showing which commands are missing
      const detail = missing.map(cmd => `  - "${cmd}"`).join('\n')
      throw new Error(
        `The following commands are called from src/api/tauri.ts but are NOT registered in ` +
        `src-tauri/src/lib.rs invoke_handler:\n${detail}\n\n` +
        `Add them to the tauri::generate_handler![] macro in lib.rs, or remove the invoke() call.`
      )
    }

    expect(missing).toHaveLength(0)
  })

  it('parsed frontend commands snapshot (informational)', () => {
    // This snapshot documents what commands are currently called.
    // Update it intentionally when adding or removing commands.
    expect(frontendCommands).toMatchSnapshot()
  })

  it('parsed backend handlers snapshot (informational)', () => {
    // This snapshot documents what commands are currently registered.
    // Update it intentionally when adding or removing backend handlers.
    expect(backendHandlers).toMatchSnapshot()
  })
})
