import { convertFileSrc } from '@tauri-apps/api/core'

/**
 * Convert a local filesystem path to a Tauri asset URL.
 *
 * Uses Tauri's built-in `convertFileSrc` which handles platform-specific
 * URL encoding (Windows drive letters, special characters, etc.) correctly.
 *
 * @param path - An absolute filesystem path (Windows or Unix-style)
 * @returns A URL that the Tauri WebView can load
 */
export function toAssetUrl(path: string): string {
  return convertFileSrc(path)
}
