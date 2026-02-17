import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useSessionStore } from '../stores/session'
import { useBugStore } from '../stores/bug'
import { useSettingsStore } from '../stores/settings'
import { getBugCaptures, updateCaptureConsoleFlag, openAnnotationWindow } from '../api/tauri'

/**
 * Composable that wires up the screenshot:captured event handler.
 *
 * Extracted from session store to eliminate circular dynamic imports:
 * session.ts previously imported bug.ts and settings.ts dynamically inside
 * the event handler to avoid circular deps. This composable sits outside the
 * store layer and can freely import all stores.
 *
 * Wire this up in App.vue alongside other event listeners.
 */
export function useCaptureEventHandler() {
  async function setup(): Promise<UnlistenFn> {
    const sessionStore = useSessionStore()
    const bugStore = useBugStore()
    const settingsStore = useSettingsStore()

    return listen<{ filePath: string; timestamp: number }>(
      'screenshot:captured',
      async (event) => {
        const { filePath, timestamp } = event.payload

        // Only process if we have an active session
        if (sessionStore.activeSession?.status !== 'active') return

        // Notify watchers about the screenshot, including which bug it belongs to
        const activeBugDisplayId = bugStore.activeBug?.display_id ?? null
        sessionStore.setLastScreenshotEvent({ filePath, bugDisplayId: activeBugDisplayId, timestamp })

        // If "tag next screenshot as console" is active, find and tag the new capture
        if (bugStore.consumeConsoleTag() && bugStore.activeBug) {
          try {
            const captures = await getBugCaptures(bugStore.activeBug.id)
            const fileName = filePath.replace(/\\/g, '/').split('/').pop() ?? ''
            const matched = captures.find(c =>
              c.file_path === filePath ||
              (fileName !== '' && c.file_path.endsWith(fileName))
            )
            if (matched) {
              await updateCaptureConsoleFlag(matched.id, true)
            }
          } catch (err) {
            console.error('Failed to tag screenshot as console capture:', err)
          }
        }

        if (settingsStore.autoOpenAnnotation) {
          try {
            let captureId: string | undefined
            if (bugStore.activeBug) {
              try {
                const captures = await getBugCaptures(bugStore.activeBug.id)
                const fileName = filePath.replace(/\\/g, '/').split('/').pop() ?? ''
                const matched = captures.find(c =>
                  c.file_path === filePath ||
                  (fileName !== '' && c.file_path.endsWith(fileName))
                )
                captureId = matched?.id
              } catch {
                // Non-fatal: annotation will still save the file, just won't update DB
              }
            }
            await openAnnotationWindow(filePath, captureId)
          } catch (err) {
            console.error('Failed to auto-open annotation window:', err)
          }
        }
      }
    )
  }

  return { setup }
}
