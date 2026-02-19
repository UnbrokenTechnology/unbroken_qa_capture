import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { Bug as BackendBug, BugUpdate, BugStatus } from '../types/backend'
import * as tauri from '../api/tauri'

export const useBugStore = defineStore('bug', () => {
  // ============================================================================
  // State
  // ============================================================================

  // Backend bugs (source of truth)
  const backendBugs = ref<BackendBug[]>([])
  const activeBug = ref<BackendBug | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Event listeners cleanup functions
  const eventUnlisteners = ref<UnlistenFn[]>([])

  // Tracks the most recently used meeting ID within a session, for pre-populating new bugs
  const lastSessionMeetingId = ref<string | null>(null)

  // Tracks the most recently used software version within a session, for pre-populating new bugs
  const lastSessionSoftwareVersion = ref<string | null>(null)

  // When true, the next screenshot captured for the active bug will be tagged as a console capture
  const tagNextScreenshotAsConsole = ref(false)

  // ============================================================================
  // Getters
  // ============================================================================

  const bugCount = computed(() => backendBugs.value.length)
  const hasError = computed(() => error.value !== null)
  const isCapturing = computed(() => activeBug.value?.status === 'capturing')

  // ============================================================================
  // Actions - Backend Bug Operations
  // ============================================================================

  async function createBug(bugData: Partial<BackendBug>): Promise<BackendBug> {
    loading.value = true
    error.value = null
    try {
      const bug = await tauri.createBug(bugData)
      backendBugs.value.push(bug)
      return bug
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function loadBug(id: string): Promise<BackendBug | null> {
    loading.value = true
    error.value = null
    try {
      const bug = await tauri.getBug(id)
      if (bug) {
        const index = backendBugs.value.findIndex(b => b.id === id)
        if (index >= 0) {
          backendBugs.value[index] = bug
        } else {
          backendBugs.value.push(bug)
        }
      }
      return bug
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function updateBackendBug(id: string, update: BugUpdate): Promise<void> {
    loading.value = true
    error.value = null
    try {
      await tauri.updateBug(id, update)
      if (update.type !== undefined) {
        await tauri.updateBugType(id, update.type)
      }
      const bug = backendBugs.value.find(b => b.id === id)
      if (bug) {
        Object.assign(bug, update)
      }
      if (activeBug.value?.id === id) {
        Object.assign(activeBug.value, update)
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function deleteBug(id: string): Promise<void> {
    loading.value = true
    error.value = null
    try {
      await tauri.deleteBug(id)
      backendBugs.value = backendBugs.value.filter(b => b.id !== id)
      if (activeBug.value?.id === id) {
        activeBug.value = null
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function loadAllBugs(sessionId?: string): Promise<void> {
    loading.value = true
    error.value = null
    try {
      backendBugs.value = await tauri.listBugs(sessionId)
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function loadBugsBySession(sessionId: string): Promise<void> {
    loading.value = true
    error.value = null
    try {
      backendBugs.value = await tauri.getBugsBySession(sessionId)
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  // ============================================================================
  // Actions - Bug Lifecycle
  // ============================================================================

  async function startBugCapture(bugData?: Partial<BackendBug>): Promise<BackendBug> {
    const newBug: Partial<BackendBug> = {
      ...bugData,
      status: 'capturing',
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }
    const bug = await createBug(newBug)
    activeBug.value = bug
    return bug
  }

  /**
   * Resume capturing for an existing bug (sets it as activeBug without creating a new one).
   * Useful when the user navigates to a bug detail and wants to add more screenshots.
   */
  async function resumeBugCapture(bugToResume: BackendBug): Promise<void> {
    const updated = await tauri.resumeBugCapture(bugToResume.id)
    const index = backendBugs.value.findIndex(b => b.id === bugToResume.id)
    if (index >= 0) {
      backendBugs.value[index] = updated
    }
    activeBug.value = updated
  }

  async function completeBugCapture(id: string): Promise<void> {
    await updateBackendBug(id, { status: 'captured' })
    if (activeBug.value?.id === id) {
      activeBug.value = null
    }
  }

  async function updateBugStatus(id: string, status: BugStatus): Promise<void> {
    await updateBackendBug(id, { status })
  }

  // ============================================================================
  // Actions - State Management
  // ============================================================================

  function setLastSessionMeetingId(meetingId: string | null) {
    lastSessionMeetingId.value = meetingId
  }

  function setLastSessionSoftwareVersion(softwareVersion: string | null) {
    lastSessionSoftwareVersion.value = softwareVersion
  }

  function setTagNextScreenshotAsConsole(value: boolean) {
    tagNextScreenshotAsConsole.value = value
  }

  /** Consume the console tag flag: returns true if set, then clears it. */
  function consumeConsoleTag(): boolean {
    const wasSet = tagNextScreenshotAsConsole.value
    tagNextScreenshotAsConsole.value = false
    return wasSet
  }

  function clearBugs() {
    backendBugs.value = []
    activeBug.value = null
    error.value = null
    lastSessionMeetingId.value = null
    lastSessionSoftwareVersion.value = null
    tagNextScreenshotAsConsole.value = false
  }

  function clearError(): void {
    error.value = null
  }

  // ============================================================================
  // Event Listeners
  // ============================================================================

  async function setupEventListeners(): Promise<void> {
    // Listen for bug created events
    const unlistenBugCreated = await listen<BackendBug>('bug-created', (event) => {
      const bug = event.payload
      const existing = backendBugs.value.find(b => b.id === bug.id)
      if (!existing) {
        backendBugs.value.push(bug)
      }
    })
    eventUnlisteners.value.push(unlistenBugCreated)

    // Listen for bug updated events
    const unlistenBugUpdated = await listen<BackendBug>('bug-updated', (event) => {
      const bug = event.payload
      const index = backendBugs.value.findIndex(b => b.id === bug.id)
      if (index >= 0) {
        backendBugs.value[index] = bug
      }
      if (activeBug.value?.id === bug.id) {
        activeBug.value = bug
      }
    })
    eventUnlisteners.value.push(unlistenBugUpdated)

    // Listen for bug deleted events
    const unlistenBugDeleted = await listen<{ id: string }>('bug-deleted', (event) => {
      const { id } = event.payload
      backendBugs.value = backendBugs.value.filter(b => b.id !== id)
      if (activeBug.value?.id === id) {
        activeBug.value = null
      }
    })
    eventUnlisteners.value.push(unlistenBugDeleted)

    // Listen for bug status changed events
    const unlistenBugStatus = await listen<{ id: string; status: BugStatus }>(
      'bug-status-changed',
      (event) => {
        const { id, status } = event.payload
        const bug = backendBugs.value.find(b => b.id === id)
        if (bug) {
          bug.status = status
        }
        if (activeBug.value?.id === id) {
          activeBug.value.status = status
        }
      }
    )
    eventUnlisteners.value.push(unlistenBugStatus)
  }

  function cleanupEventListeners(): void {
    eventUnlisteners.value.forEach(unlisten => unlisten())
    eventUnlisteners.value = []
  }

  // ============================================================================
  // Store Return
  // ============================================================================

  return {
    // State
    bugCount,
    backendBugs,
    activeBug,
    loading,
    error,
    lastSessionMeetingId,
    lastSessionSoftwareVersion,
    tagNextScreenshotAsConsole,

    // Getters
    hasError,
    isCapturing,

    // Actions - Backend Operations
    createBug,
    loadBug,
    updateBackendBug,
    deleteBug,
    loadAllBugs,
    loadBugsBySession,

    // Actions - Lifecycle
    startBugCapture,
    resumeBugCapture,
    completeBugCapture,
    updateBugStatus,

    // Actions - State Management
    clearBugs,
    clearError,
    setLastSessionMeetingId,
    setLastSessionSoftwareVersion,
    setTagNextScreenshotAsConsole,
    consumeConsoleTag,

    // Actions - Events
    setupEventListeners,
    cleanupEventListeners,
  }
})
