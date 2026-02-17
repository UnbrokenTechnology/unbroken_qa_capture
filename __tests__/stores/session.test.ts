import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useSessionStore } from '@/stores/session'
import type { Session, SessionSummary } from '@/types/backend'

// Mock Tauri API
vi.mock('@/api/tauri', () => ({
  createSession: vi.fn(),
  getSession: vi.fn(),
  updateSession: vi.fn(),
  deleteSession: vi.fn(),
  listSessions: vi.fn(),
  getActiveSession: vi.fn(),
  getSessionSummaries: vi.fn(),
  updateSessionStatus: vi.fn(),
}))

// Mock Tauri event listeners
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))

// Mock Tauri core invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}))

import * as tauri from '@/api/tauri'
import { invoke } from '@tauri-apps/api/core'

const mockSession: Session = {
  id: 'session-1',
  started_at: '2024-01-01T10:00:00Z',
  ended_at: null,
  status: 'active',
  folder_path: '/test/sessions/session1',
  session_notes: 'Test notes',
  environment_json: '{"os":"Windows 11"}',
  original_snip_path: null,
  created_at: '2024-01-01T10:00:00Z',
}

const mockSessionSummary: SessionSummary = {
  id: 'session-1',
  started_at: '2024-01-01T10:00:00Z',
  ended_at: null,
  status: 'active',
  bug_count: 3,
}

describe('Session Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('should initialize with empty state', () => {
    const store = useSessionStore()
    expect(store.activeSession).toBeNull()
    expect(store.sessions).toEqual([])
    expect(store.sessionSummaries).toEqual([])
    expect(store.loading).toBe(false)
    expect(store.starting).toBe(false)
    expect(store.initializing).toBe(false)
    expect(store.error).toBeNull()
  })

  it('should have correct computed getters', () => {
    const store = useSessionStore()
    expect(store.isSessionActive).toBe(false)
    expect(store.isStartingSession).toBe(false)
    expect(store.activeSessionId).toBeNull()
    expect(store.sessionCount).toBe(0)
    expect(store.hasError).toBe(false)
  })

  describe('createSession', () => {
    it('should create a new session', async () => {
      const store = useSessionStore()
      vi.mocked(tauri.createSession).mockResolvedValue(mockSession)

      const result = await store.createSession({ folder_path: '/test' })

      expect(result).toEqual(mockSession)
      expect(store.sessions).toHaveLength(1)
      expect(store.sessions[0]).toEqual(mockSession)
      expect(store.activeSession).toEqual(mockSession)
      expect(tauri.createSession).toHaveBeenCalledWith({ folder_path: '/test' })
    })

    it('should handle errors when creating session', async () => {
      const store = useSessionStore()
      const error = new Error('Failed to create session')
      vi.mocked(tauri.createSession).mockRejectedValue(error)

      await expect(store.createSession({})).rejects.toThrow('Failed to create session')
      expect(store.error).toBe('Failed to create session')
    })
  })

  describe('loadSession', () => {
    it('should load an existing session', async () => {
      const store = useSessionStore()
      vi.mocked(tauri.getSession).mockResolvedValue(mockSession)

      const result = await store.loadSession('session-1')

      expect(result).toEqual(mockSession)
      expect(store.sessions).toHaveLength(1)
      expect(store.sessions[0]).toEqual(mockSession)
      expect(tauri.getSession).toHaveBeenCalledWith('session-1')
    })

    it('should return null if session not found', async () => {
      const store = useSessionStore()
      vi.mocked(tauri.getSession).mockResolvedValue(null)

      const result = await store.loadSession('nonexistent')

      expect(result).toBeNull()
    })
  })

  describe('updateSession', () => {
    it('should update an existing session', async () => {
      const store = useSessionStore()
      store.sessions.push({ ...mockSession })
      vi.mocked(tauri.updateSession).mockResolvedValue()

      const updated = { ...mockSession, session_notes: 'Updated notes' }
      await store.updateSession(updated)

      expect(store.sessions[0]?.session_notes).toBe('Updated notes')
      expect(tauri.updateSession).toHaveBeenCalledWith(updated)
    })

    it('should update active session if it matches', async () => {
      const store = useSessionStore()
      store.activeSession = { ...mockSession }
      store.sessions.push({ ...mockSession })
      vi.mocked(tauri.updateSession).mockResolvedValue()

      const updated = { ...mockSession, session_notes: 'Updated notes' }
      await store.updateSession(updated)

      expect(store.activeSession?.session_notes).toBe('Updated notes')
    })
  })

  describe('deleteSession', () => {
    it('should delete a session', async () => {
      const store = useSessionStore()
      store.sessions.push({ ...mockSession })
      vi.mocked(tauri.deleteSession).mockResolvedValue()

      await store.deleteSession('session-1')

      expect(store.sessions).toHaveLength(0)
      expect(tauri.deleteSession).toHaveBeenCalledWith('session-1')
    })

    it('should clear active session if it matches deleted session', async () => {
      const store = useSessionStore()
      store.activeSession = { ...mockSession }
      store.sessions.push({ ...mockSession })
      vi.mocked(tauri.deleteSession).mockResolvedValue()

      await store.deleteSession('session-1')

      expect(store.activeSession).toBeNull()
    })
  })

  describe('loadAllSessions', () => {
    it('should load all sessions', async () => {
      const store = useSessionStore()
      const sessions = [mockSession, { ...mockSession, id: 'session-2' }]
      vi.mocked(tauri.listSessions).mockResolvedValue(sessions)

      await store.loadAllSessions()

      expect(store.sessions).toEqual(sessions)
      expect(tauri.listSessions).toHaveBeenCalled()
    })
  })

  describe('loadActiveSession', () => {
    it('should load the active session', async () => {
      const store = useSessionStore()
      vi.mocked(tauri.getActiveSession).mockResolvedValue(mockSession)

      await store.loadActiveSession()

      expect(store.activeSession).toEqual(mockSession)
      expect(tauri.getActiveSession).toHaveBeenCalled()
    })

    it('should handle no active session', async () => {
      const store = useSessionStore()
      vi.mocked(tauri.getActiveSession).mockResolvedValue(null)

      await store.loadActiveSession()

      expect(store.activeSession).toBeNull()
    })

    it('should set initializing=true while loading and false after', async () => {
      const store = useSessionStore()
      let initializingDuringCall = false

      vi.mocked(tauri.getActiveSession).mockImplementation(async () => {
        initializingDuringCall = store.initializing
        return mockSession
      })

      expect(store.initializing).toBe(false)

      await store.loadActiveSession()

      expect(initializingDuringCall).toBe(true)
      expect(store.initializing).toBe(false)
    })

    it('should clear initializing state even on error', async () => {
      const store = useSessionStore()
      vi.mocked(tauri.getActiveSession).mockRejectedValue(new Error('Failed'))

      await expect(store.loadActiveSession()).rejects.toThrow('Failed')

      expect(store.initializing).toBe(false)
    })
  })

  describe('loadSessionSummaries', () => {
    it('should load session summaries', async () => {
      const store = useSessionStore()
      const summaries = [mockSessionSummary]
      vi.mocked(tauri.getSessionSummaries).mockResolvedValue(summaries)

      await store.loadSessionSummaries()

      expect(store.sessionSummaries).toEqual(summaries)
      expect(tauri.getSessionSummaries).toHaveBeenCalled()
    })
  })

  describe('startSession', () => {
    it('should start a new session', async () => {
      const store = useSessionStore()
      vi.mocked(tauri.createSession).mockResolvedValue(mockSession)

      const result = await store.startSession({ folder_path: '/test' })

      expect(result.status).toBe('active')
      expect(tauri.createSession).toHaveBeenCalled()
    })

    it('should end active session before starting new one', async () => {
      const store = useSessionStore()
      store.activeSession = { ...mockSession }
      vi.mocked(invoke).mockResolvedValue(undefined)
      vi.mocked(tauri.createSession).mockResolvedValue({
        ...mockSession,
        id: 'session-2',
      })

      await store.startSession()

      expect(invoke).toHaveBeenCalledWith('end_session', { sessionId: 'session-1' })
    })

    it('should set starting=true while session is being created and false after', async () => {
      const store = useSessionStore()
      let startingDuringCall = false

      vi.mocked(tauri.createSession).mockImplementation(async () => {
        startingDuringCall = store.starting
        return mockSession
      })

      expect(store.starting).toBe(false)
      expect(store.isStartingSession).toBe(false)

      await store.startSession()

      expect(startingDuringCall).toBe(true)
      expect(store.starting).toBe(false)
      expect(store.isStartingSession).toBe(false)
    })

    it('should clear starting state even on error', async () => {
      const store = useSessionStore()
      vi.mocked(tauri.createSession).mockRejectedValue(new Error('Failed'))

      await expect(store.startSession()).rejects.toThrow('Failed')

      expect(store.starting).toBe(false)
      expect(store.isStartingSession).toBe(false)
    })
  })

  describe('endSession', () => {
    it('should end a session', async () => {
      const store = useSessionStore()
      store.sessions.push({ ...mockSession })
      store.activeSession = { ...mockSession }
      vi.mocked(invoke).mockResolvedValue(undefined)

      await store.endSession('session-1')

      expect(store.sessions[0]?.status).toBe('ended')
      expect(store.activeSession).toBeNull()
      expect(invoke).toHaveBeenCalledWith('end_session', { sessionId: 'session-1' })
    })
  })

  describe('updateSessionStatus', () => {
    it('should update session status', async () => {
      const store = useSessionStore()
      store.sessions.push({ ...mockSession })
      vi.mocked(tauri.updateSessionStatus).mockResolvedValue()

      await store.updateSessionStatus('session-1', 'reviewed')

      expect(store.sessions[0]?.status).toBe('reviewed')
      expect(tauri.updateSessionStatus).toHaveBeenCalledWith('session-1', 'reviewed')
    })
  })

  describe('Local state management', () => {
    it('should set active session', () => {
      const store = useSessionStore()
      store.setActiveSession(mockSession)

      expect(store.activeSession).toEqual(mockSession)
    })

    it('should clear error', () => {
      const store = useSessionStore()
      store.error = 'Test error'

      store.clearError()

      expect(store.error).toBeNull()
    })

    it('should clear all data', () => {
      const store = useSessionStore()
      store.activeSession = { ...mockSession }
      store.sessions = [mockSession]
      store.sessionSummaries = [mockSessionSummary]
      store.error = 'Test error'

      store.clearAll()

      expect(store.activeSession).toBeNull()
      expect(store.sessions).toEqual([])
      expect(store.sessionSummaries).toEqual([])
      expect(store.error).toBeNull()
    })
  })

  describe('createSession error paths', () => {
    it('should reset loading state after backend failure', async () => {
      const store = useSessionStore()
      vi.mocked(tauri.createSession).mockRejectedValue(new Error('Network error'))

      await expect(store.createSession({})).rejects.toThrow('Network error')

      expect(store.loading).toBe(false)
      expect(store.error).toBe('Network error')
      expect(store.sessions).toHaveLength(0)
    })

    it('should not add session to list when backend call fails', async () => {
      const store = useSessionStore()
      vi.mocked(tauri.createSession).mockRejectedValue(new Error('Server error'))

      await expect(store.createSession({ folder_path: '/test' })).rejects.toThrow()

      expect(store.sessions).toHaveLength(0)
      expect(store.activeSession).toBeNull()
    })

    it('should handle concurrent session creation - second call waits independently', async () => {
      const store = useSessionStore()
      const session1 = { ...mockSession, id: 'session-A' }
      const session2 = { ...mockSession, id: 'session-B' }

      let resolve1!: (v: typeof session1) => void
      let resolve2!: (v: typeof session2) => void

      vi.mocked(tauri.createSession)
        .mockImplementationOnce(() => new Promise(r => { resolve1 = r }))
        .mockImplementationOnce(() => new Promise(r => { resolve2 = r }))

      const p1 = store.createSession({})
      const p2 = store.createSession({})

      resolve2(session2)
      resolve1(session1)

      const [r1, r2] = await Promise.all([p1, p2])

      expect(r1.id).toBe('session-A')
      expect(r2.id).toBe('session-B')
      expect(store.sessions).toHaveLength(2)
    })
  })

  describe('getActiveSession edge cases', () => {
    it('should return null when no session is active (backend returns null)', async () => {
      const store = useSessionStore()
      vi.mocked(tauri.getActiveSession).mockResolvedValue(null)

      await store.loadActiveSession()

      expect(store.activeSession).toBeNull()
      expect(store.activeSessionId).toBeNull()
      expect(store.isSessionActive).toBe(false)
    })

    it('should handle error from getActiveSession gracefully', async () => {
      const store = useSessionStore()
      vi.mocked(tauri.getActiveSession).mockRejectedValue(new Error('Backend unavailable'))

      await expect(store.loadActiveSession()).rejects.toThrow('Backend unavailable')

      expect(store.error).toBe('Backend unavailable')
      expect(store.activeSession).toBeNull()
      expect(store.initializing).toBe(false)
      expect(store.loading).toBe(false)
    })

    it('should reflect no active session in computed properties when activeSession is null', () => {
      const store = useSessionStore()
      store.activeSession = null

      expect(store.isSessionActive).toBe(false)
      expect(store.activeSessionId).toBeNull()
      expect(store.hasError).toBe(false)
    })
  })

  describe('resumeSession - session no longer exists in backend', () => {
    it('should return null when loading a session that no longer exists', async () => {
      const store = useSessionStore()
      vi.mocked(tauri.getSession).mockResolvedValue(null)

      const result = await store.loadSession('deleted-session-id')

      expect(result).toBeNull()
      expect(store.sessions).toHaveLength(0)
    })

    it('should keep loading=false after failed session load', async () => {
      const store = useSessionStore()
      vi.mocked(tauri.getSession).mockResolvedValue(null)

      await store.loadSession('nonexistent')

      expect(store.loading).toBe(false)
    })
  })

  describe('startSession error paths', () => {
    it('should reset starting state when backend call fails', async () => {
      const store = useSessionStore()
      vi.mocked(tauri.createSession).mockRejectedValue(new Error('Failed to create'))

      await expect(store.startSession()).rejects.toThrow('Failed to create')

      expect(store.starting).toBe(false)
      expect(store.isStartingSession).toBe(false)
    })

    it('should set error when startSession fails', async () => {
      const store = useSessionStore()
      vi.mocked(tauri.createSession).mockRejectedValue(new Error('Session creation failed'))

      await expect(store.startSession()).rejects.toThrow()

      expect(store.error).toBe('Session creation failed')
    })
  })

})
