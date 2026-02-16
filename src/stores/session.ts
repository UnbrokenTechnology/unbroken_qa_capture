import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { Session, SessionSummary, SessionStatus } from '../types/backend'
import * as tauri from '../api/tauri'

export const useSessionStore = defineStore('session', () => {
  // ============================================================================
  // State
  // ============================================================================

  const activeSession = ref<Session | null>(null)
  const sessions = ref<Session[]>([])
  const sessionSummaries = ref<SessionSummary[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Event listeners cleanup functions
  const eventUnlisteners = ref<UnlistenFn[]>([])

  // ============================================================================
  // Getters
  // ============================================================================

  const isSessionActive = computed(() => activeSession.value?.status === 'active')
  const activeSessionId = computed(() => activeSession.value?.id ?? null)
  const sessionCount = computed(() => sessions.value.length)
  const hasError = computed(() => error.value !== null)

  // ============================================================================
  // Actions - Session CRUD
  // ============================================================================

  async function createSession(sessionData: Partial<Session>): Promise<Session> {
    loading.value = true
    error.value = null
    try {
      const session = await tauri.createSession(sessionData)
      sessions.value.push(session)
      if (session.status === 'active') {
        activeSession.value = session
      }
      return session
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function loadSession(id: string): Promise<Session | null> {
    loading.value = true
    error.value = null
    try {
      const session = await tauri.getSession(id)
      if (session) {
        const index = sessions.value.findIndex(s => s.id === id)
        if (index >= 0) {
          sessions.value[index] = session
        } else {
          sessions.value.push(session)
        }
      }
      return session
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function updateSession(session: Session): Promise<void> {
    loading.value = true
    error.value = null
    try {
      await tauri.updateSession(session)
      const index = sessions.value.findIndex(s => s.id === session.id)
      if (index >= 0) {
        sessions.value[index] = session
      }
      if (activeSession.value?.id === session.id) {
        activeSession.value = session
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function deleteSession(id: string): Promise<void> {
    loading.value = true
    error.value = null
    try {
      await tauri.deleteSession(id)
      sessions.value = sessions.value.filter(s => s.id !== id)
      if (activeSession.value?.id === id) {
        activeSession.value = null
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  // ============================================================================
  // Actions - Session Queries
  // ============================================================================

  async function loadAllSessions(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      sessions.value = await tauri.listSessions()
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function loadActiveSession(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      activeSession.value = await tauri.getActiveSession()
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function loadSessionSummaries(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      sessionSummaries.value = await tauri.getSessionSummaries()
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  // ============================================================================
  // Actions - Session Lifecycle
  // ============================================================================

  async function startSession(sessionData?: Partial<Session>): Promise<Session> {
    // End any active session first
    if (activeSession.value && activeSession.value.status === 'active') {
      await endSession(activeSession.value.id)
    }

    const newSession: Partial<Session> = {
      ...sessionData,
      status: 'active',
      started_at: new Date().toISOString(),
      created_at: new Date().toISOString(),
    }

    return await createSession(newSession)
  }

  async function endSession(id: string): Promise<void> {
    loading.value = true
    error.value = null
    try {
      await tauri.updateSessionStatus(id, 'ended')
      const session = sessions.value.find(s => s.id === id)
      if (session) {
        session.status = 'ended'
        session.ended_at = new Date().toISOString()
      }
      if (activeSession.value?.id === id) {
        activeSession.value = null
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function updateSessionStatus(id: string, status: SessionStatus): Promise<void> {
    loading.value = true
    error.value = null
    try {
      await tauri.updateSessionStatus(id, status)
      const session = sessions.value.find(s => s.id === id)
      if (session) {
        session.status = status
      }
      if (activeSession.value?.id === id) {
        activeSession.value.status = status
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  // ============================================================================
  // Actions - Local State Management
  // ============================================================================

  function setActiveSession(session: Session | null): void {
    activeSession.value = session
  }

  function clearError(): void {
    error.value = null
  }

  function clearAll(): void {
    activeSession.value = null
    sessions.value = []
    sessionSummaries.value = []
    error.value = null
  }

  // ============================================================================
  // Event Listeners
  // ============================================================================

  async function setupEventListeners(): Promise<void> {
    // Listen for session created events
    const unlistenSessionCreated = await listen<Session>('session-created', (event) => {
      const session = event.payload
      const existing = sessions.value.find(s => s.id === session.id)
      if (!existing) {
        sessions.value.push(session)
      }
      if (session.status === 'active') {
        activeSession.value = session
      }
    })
    eventUnlisteners.value.push(unlistenSessionCreated)

    // Listen for session updated events
    const unlistenSessionUpdated = await listen<Session>('session-updated', (event) => {
      const session = event.payload
      const index = sessions.value.findIndex(s => s.id === session.id)
      if (index >= 0) {
        sessions.value[index] = session
      }
      if (activeSession.value?.id === session.id) {
        activeSession.value = session
      }
    })
    eventUnlisteners.value.push(unlistenSessionUpdated)

    // Listen for session deleted events
    const unlistenSessionDeleted = await listen<{ id: string }>('session-deleted', (event) => {
      const { id } = event.payload
      sessions.value = sessions.value.filter(s => s.id !== id)
      if (activeSession.value?.id === id) {
        activeSession.value = null
      }
    })
    eventUnlisteners.value.push(unlistenSessionDeleted)

    // Listen for session status changed events
    const unlistenSessionStatus = await listen<{ id: string; status: SessionStatus }>(
      'session-status-changed',
      (event) => {
        const { id, status } = event.payload
        const session = sessions.value.find(s => s.id === id)
        if (session) {
          session.status = status
        }
        if (activeSession.value?.id === id) {
          activeSession.value.status = status
        }
      }
    )
    eventUnlisteners.value.push(unlistenSessionStatus)
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
    activeSession,
    sessions,
    sessionSummaries,
    loading,
    error,

    // Getters
    isSessionActive,
    activeSessionId,
    sessionCount,
    hasError,

    // Actions - CRUD
    createSession,
    loadSession,
    updateSession,
    deleteSession,

    // Actions - Queries
    loadAllSessions,
    loadActiveSession,
    loadSessionSummaries,

    // Actions - Lifecycle
    startSession,
    endSession,
    updateSessionStatus,

    // Actions - Local State
    setActiveSession,
    clearError,
    clearAll,

    // Actions - Events
    setupEventListeners,
    cleanupEventListeners,
  }
})
