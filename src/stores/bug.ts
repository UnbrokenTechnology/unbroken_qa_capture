import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { Bug as BackendBug, BugUpdate, BugStatus } from '../types/backend'
import * as tauri from '../api/tauri'

// Legacy Bug interface for backward compatibility with existing UI components
// TODO: Update BugDetail.vue and Home.vue to use the BackendBug type
export interface Environment {
  os: string
  display_resolution: string
  dpi_scaling: string
  ram: string
  cpu: string
  foreground_app: string
}

export interface BugMetadata {
  meeting_id?: string
  software_version?: string
  environment: Environment
  console_captures: string[]
  custom_fields: Record<string, string>
}

export interface Bug {
  id: string
  title: string
  bug_type: string
  description_steps: string
  description_expected: string
  description_actual: string
  metadata: BugMetadata
  folder_path: string
  captures: string[]
  console_output?: string
  created_at?: string
}

export const useBugStore = defineStore('bug', () => {
  // ============================================================================
  // State
  // ============================================================================

  // Legacy bugs for UI compatibility (will be populated from backend bugs)
  const bugs = ref<Bug[]>([])
  const currentBugId = ref<string | null>(null)

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

  const allBugs = computed(() => bugs.value)

  const currentBug = computed(() => {
    if (!currentBugId.value) return null
    return bugs.value.find(bug => bug.id === currentBugId.value) || null
  })

  const getBugById = (id: string) => {
    return bugs.value.find(bug => bug.id === id)
  }

  const bugCount = computed(() => bugs.value.length)
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
      bugs.value = bugs.value.filter(b => b.id !== id)
      if (activeBug.value?.id === id) {
        activeBug.value = null
      }
      if (currentBugId.value === id) {
        currentBugId.value = null
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
  // Actions - Legacy Bug Operations (for UI compatibility)
  // ============================================================================

  function addBug(bug: Bug) {
    // Check if bug with same ID already exists
    const existingIndex = bugs.value.findIndex(b => b.id === bug.id)
    if (existingIndex >= 0) {
      // Update existing bug
      bugs.value[existingIndex] = bug
    } else {
      // Add new bug
      bugs.value.push(bug)
    }
  }

  function removeBug(id: string) {
    const index = bugs.value.findIndex(bug => bug.id === id)
    if (index >= 0) {
      bugs.value.splice(index, 1)
      if (currentBugId.value === id) {
        currentBugId.value = null
      }
    }
  }

  function updateBug(id: string, updates: Partial<Bug>) {
    const index = bugs.value.findIndex(bug => bug.id === id)
    if (index >= 0) {
      bugs.value[index] = { ...bugs.value[index], ...updates } as Bug
    }
  }

  function setCurrentBug(id: string | null) {
    currentBugId.value = id
  }

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
    bugs.value = []
    backendBugs.value = []
    currentBugId.value = null
    activeBug.value = null
    error.value = null
    lastSessionMeetingId.value = null
    lastSessionSoftwareVersion.value = null
    tagNextScreenshotAsConsole.value = false
  }

  function loadBugs(bugList: Bug[]) {
    bugs.value = bugList
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
  // Development/Testing: Sample Data
  // ============================================================================

  function loadSampleData() {
    const sampleBugs: Bug[] = [
      {
        id: '1',
        title: 'Login button not responding',
        bug_type: 'UI',
        description_steps: '1. Open the application\n2. Navigate to login page\n3. Click the login button\n4. Observe that nothing happens',
        description_expected: 'Login button should be clickable and trigger login process',
        description_actual: 'Login button appears disabled and does not respond to clicks',
        metadata: {
          meeting_id: 'MTG-2024-001',
          software_version: '1.2.3',
          environment: {
            os: 'Windows 11 Pro',
            display_resolution: '1920x1080',
            dpi_scaling: '100%',
            ram: '16GB',
            cpu: 'Intel Core i7-10700K',
            foreground_app: 'QA Capture Test App'
          },
          console_captures: [],
          custom_fields: {}
        },
        folder_path: 'C:\\QA\\Session_2024-01-15\\Bug_001',
        captures: [
          'https://via.placeholder.com/800x600/FF6B6B/FFFFFF?text=Login+Screen',
          'https://via.placeholder.com/800x600/4ECDC4/FFFFFF?text=Button+Disabled'
        ],
        console_output: 'Error: Cannot read property "onClick" of undefined\n    at LoginButton.handleClick (login.js:45)\n    at HTMLButtonElement.<anonymous> (dom.js:123)',
        created_at: '2024-01-15T10:30:00Z'
      },
      {
        id: '2',
        title: 'Data not saving in form',
        bug_type: 'Data',
        description_steps: '1. Fill out the user profile form\n2. Enter name, email, and preferences\n3. Click Save button\n4. Navigate away and return\n5. Observe that data was not saved',
        description_expected: 'Form data should be persisted to the database and retained after page navigation',
        description_actual: 'Form appears to save but data is lost after navigating away',
        metadata: {
          software_version: '1.2.3',
          environment: {
            os: 'Windows 11 Pro',
            display_resolution: '2560x1440',
            dpi_scaling: '125%',
            ram: '32GB',
            cpu: 'AMD Ryzen 9 5900X',
            foreground_app: 'QA Capture Test App'
          },
          console_captures: [],
          custom_fields: {
            severity: 'High',
            priority: 'P1'
          }
        },
        folder_path: 'C:\\QA\\Session_2024-01-15\\Bug_002',
        captures: [
          'https://via.placeholder.com/800x600/95E1D3/FFFFFF?text=Form+Filled',
          'https://via.placeholder.com/800x600/F38181/FFFFFF?text=Data+Lost'
        ],
        console_output: 'POST /api/user/profile 500 Internal Server Error\nError: Database connection timeout',
        created_at: '2024-01-15T11:45:00Z'
      }
    ]

    loadBugs(sampleBugs)
  }

  // ============================================================================
  // Store Return
  // ============================================================================

  return {
    // State
    bugs: allBugs,
    currentBug,
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
    completeBugCapture,
    updateBugStatus,

    // Actions - Legacy (for UI compatibility)
    addBug,
    removeBug,
    updateBug,
    setCurrentBug,
    clearBugs,
    loadBugs,
    getBugById,
    loadSampleData,
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
