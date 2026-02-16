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

import * as tauri from '@/api/tauri'

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
    expect(store.error).toBeNull()
  })

  it('should have correct computed getters', () => {
    const store = useSessionStore()
    expect(store.isSessionActive).toBe(false)
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
      vi.mocked(tauri.updateSessionStatus).mockResolvedValue()
      vi.mocked(tauri.createSession).mockResolvedValue({
        ...mockSession,
        id: 'session-2',
      })

      await store.startSession()

      expect(tauri.updateSessionStatus).toHaveBeenCalledWith('session-1', 'ended')
    })
  })

  describe('endSession', () => {
    it('should end a session', async () => {
      const store = useSessionStore()
      store.sessions.push({ ...mockSession })
      store.activeSession = { ...mockSession }
      vi.mocked(tauri.updateSessionStatus).mockResolvedValue()

      await store.endSession('session-1')

      expect(store.sessions[0]?.status).toBe('ended')
      expect(store.activeSession).toBeNull()
      expect(tauri.updateSessionStatus).toHaveBeenCalledWith('session-1', 'ended')
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
})
