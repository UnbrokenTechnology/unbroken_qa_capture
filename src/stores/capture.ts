import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Bug, Capture, CaptureAssignmentSuggestion } from '../types/backend'
import * as tauri from '../api/tauri'
import { toAssetUrl } from '../utils/paths'

// ============================================================================
// Helpers
// ============================================================================

const VIDEO_EXTENSIONS = ['.mp4', '.webm', '.mkv', '.mov', '.avi']

function isVideoPath(path: string): boolean {
  const lower = path.toLowerCase()
  return VIDEO_EXTENSIONS.some(ext => lower.endsWith(ext))
}

// ============================================================================
// Store
// ============================================================================

export const useCaptureStore = defineStore('capture', () => {
  // ============================================================================
  // State
  // ============================================================================

  const unsortedCaptures = ref<Capture[]>([])
  const bugCaptureCounts = ref<Map<string, { screenshots: number; videos: number; thumbnail: string | null }>>(new Map())
  const loading = ref(false)
  const error = ref<string | null>(null)

  // AI suggestion state: maps captureId -> suggestion result
  const aiSuggestions = ref<Map<string, CaptureAssignmentSuggestion>>(new Map())
  // Set of captureIds currently being processed by AI
  const aiSuggestionsPending = ref<Set<string>>(new Set())

  // ============================================================================
  // Getters
  // ============================================================================

  const unsortedCount = computed(() => unsortedCaptures.value.length)
  const hasError = computed(() => error.value !== null)

  // ============================================================================
  // Actions
  // ============================================================================

  /**
   * Load all unsorted captures (captures not yet assigned to a bug) for a session.
   */
  async function loadUnsortedCaptures(sessionId: string): Promise<void> {
    loading.value = true
    error.value = null
    try {
      unsortedCaptures.value = await tauri.getUnsortedCaptures(sessionId)
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * Load capture counts (screenshots, videos, thumbnail) for a list of bugs.
   * Updates bugCaptureCounts map in place.
   *
   * Moved from ActiveSessionView.vue — centralised here so all components share
   * the same cached counts without separate fetches.
   */
  async function loadBugCaptureCounts(bugs: Bug[]): Promise<void> {
    for (const bug of bugs) {
      try {
        const captures = await tauri.getBugCaptures(bug.id)
        const screenshots = captures.filter(c => !isVideoPath(c.file_path))
        const videos = captures.filter(c => isVideoPath(c.file_path))
        const rawThumb = screenshots[0]?.file_path ?? null
        const thumbnail = rawThumb ? toAssetUrl(rawThumb) : null
        bugCaptureCounts.value.set(bug.id, {
          screenshots: screenshots.length,
          videos: videos.length,
          thumbnail,
        })
      } catch {
        // Non-fatal: show zero counts if load fails
        bugCaptureCounts.value.set(bug.id, { screenshots: 0, videos: 0, thumbnail: null })
      }
    }
  }

  /**
   * Assign a capture to a bug.
   * Optimistically removes the capture from unsortedCaptures immediately,
   * then calls the backend to persist the association.
   */
  async function assignCaptureToBug(captureId: string, bugId: string): Promise<void> {
    // Optimistic update: remove from unsorted list right away
    unsortedCaptures.value = unsortedCaptures.value.filter(c => c.id !== captureId)

    try {
      await tauri.assignCaptureToBug(captureId, bugId)
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    }
  }

  /**
   * Refresh all capture state for a session: unsorted captures and counts for
   * every provided bug. Convenience method for use on session load.
   */
  async function refreshAll(sessionId: string, bugs: Bug[]): Promise<void> {
    await Promise.all([
      loadUnsortedCaptures(sessionId),
      loadBugCaptureCounts(bugs),
    ])
  }

  /**
   * Request an AI suggestion for which bug a capture belongs to.
   * No-ops if a suggestion is already pending or available for this capture.
   */
  async function requestAiSuggestion(captureId: string, sessionId: string): Promise<void> {
    if (aiSuggestionsPending.value.has(captureId) || aiSuggestions.value.has(captureId)) return

    aiSuggestionsPending.value = new Set(aiSuggestionsPending.value).add(captureId)
    try {
      const suggestion = await tauri.suggestCaptureAssignment(captureId, sessionId)
      const next = new Map(aiSuggestions.value)
      next.set(captureId, suggestion)
      aiSuggestions.value = next
    } catch (e) {
      console.error('AI suggestion failed for capture', captureId, e)
    } finally {
      const next = new Set(aiSuggestionsPending.value)
      next.delete(captureId)
      aiSuggestionsPending.value = next
    }
  }

  /**
   * Get the AI suggestion for a capture (if available).
   */
  function getAiSuggestion(captureId: string): CaptureAssignmentSuggestion | undefined {
    return aiSuggestions.value.get(captureId)
  }

  /**
   * Returns true if an AI suggestion request is in-flight for the given capture.
   */
  function isAiPending(captureId: string): boolean {
    return aiSuggestionsPending.value.has(captureId)
  }

  /**
   * Accept the AI suggestion for a capture: assign it to the suggested bug.
   * Clears the suggestion after accepting.
   */
  async function acceptAiSuggestion(captureId: string): Promise<void> {
    const suggestion = aiSuggestions.value.get(captureId)
    if (suggestion?.suggestedBugId) {
      await assignCaptureToBug(captureId, suggestion.suggestedBugId)
      const next = new Map(aiSuggestions.value)
      next.delete(captureId)
      aiSuggestions.value = next
    }
  }

  /**
   * Reset all capture state (e.g. when a session ends or changes).
   */
  function $reset(): void {
    unsortedCaptures.value = []
    bugCaptureCounts.value = new Map()
    loading.value = false
    error.value = null
    aiSuggestions.value = new Map()
    aiSuggestionsPending.value = new Set()
  }

  function clearError(): void {
    error.value = null
  }

  // ============================================================================
  // Store Return
  // ============================================================================

  return {
    // State
    unsortedCaptures,
    bugCaptureCounts,
    loading,
    error,
    aiSuggestions,
    aiSuggestionsPending,

    // Getters
    unsortedCount,
    hasError,

    // Actions
    loadUnsortedCaptures,
    loadBugCaptureCounts,
    assignCaptureToBug,
    refreshAll,
    requestAiSuggestion,
    getAiSuggestion,
    isAiPending,
    acceptAiSuggestion,
    $reset,
    clearError,
  }
})
