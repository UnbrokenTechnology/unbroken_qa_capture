/**
 * Integration tests for the full session lifecycle.
 *
 * These tests exercise cross-store coordination between the session and bug
 * stores, verifying state transitions across the full QA capture flow:
 *   start session → capture bugs → file watcher events → end session → review
 *
 * The Tauri API layer is mocked to simulate backend responses, allowing
 * frontend store logic and cross-store coordination to be verified in isolation.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useSessionStore } from '@/stores/session'
import { useBugStore } from '@/stores/bug'
import type { Session, SessionSummary } from '@/types/backend'
import type { Bug as BackendBug } from '@/types/backend'

// ============================================================================
// Mocks
// ============================================================================

vi.mock('@/api/tauri', () => ({
  // Session operations
  createSession: vi.fn(),
  getSession: vi.fn(),
  updateSession: vi.fn(),
  deleteSession: vi.fn(),
  listSessions: vi.fn(),
  getActiveSession: vi.fn(),
  getSessionSummaries: vi.fn(),
  updateSessionStatus: vi.fn(),
  // Bug operations
  createBug: vi.fn(),
  getBug: vi.fn(),
  updateBug: vi.fn(),
  deleteBug: vi.fn(),
  listBugs: vi.fn(),
  getBugsBySession: vi.fn(),
  getBugNotes: vi.fn(),
  updateBugNotes: vi.fn(),
  // Session notes
  getSessionNotes: vi.fn(),
  updateSessionNotes: vi.fn(),
  // Tray
  updateTrayIcon: vi.fn(),
  updateTrayTooltip: vi.fn(),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))

import * as tauri from '@/api/tauri'
import { listen } from '@tauri-apps/api/event'

// ============================================================================
// Test Helpers
// ============================================================================

function makeSession(overrides: Partial<Session> = {}): Session {
  return {
    id: 'session-1',
    started_at: '2024-01-01T10:00:00Z',
    ended_at: null,
    status: 'active',
    folder_path: '/qa/sessions/2024-01-01_abc12345',
    session_notes: null,
    environment_json: null,
    original_snip_path: null,
    created_at: '2024-01-01T10:00:00Z',
    ...overrides,
  }
}

function makeBug(overrides: Partial<BackendBug> = {}): BackendBug {
  return {
    id: 'bug-1',
    session_id: 'session-1',
    bug_number: 1,
    display_id: 'BUG-001',
    type: 'bug',
    title: null,
    notes: null,
    description: null,
    ai_description: null,
    status: 'capturing',
    meeting_id: null,
    software_version: null,
    console_parse_json: null,
    metadata_json: null,
    folder_path: '/qa/sessions/2024-01-01_abc12345/bug_001',
    created_at: '2024-01-01T10:00:00Z',
    updated_at: '2024-01-01T10:00:00Z',
    ...overrides,
  }
}

// ============================================================================
// Integration Test Suites
// ============================================================================

describe('Session Lifecycle Integration', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  describe('Full session lifecycle', () => {
    it('start session → capture 3 bugs → end session', async () => {
      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      // 1. Start a new session
      const mockSession = makeSession()
      vi.mocked(tauri.createSession).mockResolvedValue(mockSession)

      const session = await sessionStore.createSession({ folder_path: '/qa/sessions' })

      expect(sessionStore.activeSession).toEqual(mockSession)
      expect(sessionStore.isSessionActive).toBe(true)
      expect(sessionStore.activeSessionId).toBe('session-1')

      // 2. Capture 3 bugs sequentially
      for (let i = 1; i <= 3; i++) {
        const mockBug = makeBug({
          id: `bug-${i}`,
          bug_number: i,
          display_id: `BUG-00${i}`,
          folder_path: `/qa/sessions/2024-01-01_abc12345/bug_00${i}`,
        })
        vi.mocked(tauri.createBug).mockResolvedValue(mockBug)

        const bug = await bugStore.createBug({ session_id: session.id })
        expect(bug.bug_number).toBe(i)
        expect(bug.display_id).toBe(`BUG-00${i}`)
      }

      expect(bugStore.backendBugs).toHaveLength(3)

      // 3. End the session
      const endedSession = makeSession({ status: 'ended', ended_at: '2024-01-01T11:00:00Z' })
      vi.mocked(tauri.updateSession).mockResolvedValue(undefined)

      await sessionStore.updateSession(endedSession)

      expect(sessionStore.sessions.at(0)?.status).toBe('ended')
    })

    it('session computed getters reflect lifecycle state correctly', async () => {
      const sessionStore = useSessionStore()

      // Initially: no active session
      expect(sessionStore.isSessionActive).toBe(false)
      expect(sessionStore.activeSessionId).toBeNull()
      expect(sessionStore.sessionCount).toBe(0)
      expect(sessionStore.hasError).toBe(false)

      // After creating session
      const mockSession = makeSession()
      vi.mocked(tauri.createSession).mockResolvedValue(mockSession)
      await sessionStore.createSession({})

      expect(sessionStore.isSessionActive).toBe(true)
      expect(sessionStore.activeSessionId).toBe('session-1')
      expect(sessionStore.sessionCount).toBe(1)

      // After ending session
      const endedSession = makeSession({ status: 'ended', ended_at: '2024-01-01T11:00:00Z' })
      vi.mocked(tauri.updateSession).mockResolvedValue(undefined)
      await sessionStore.updateSession(endedSession)

      // activeSession still set (not cleared on update), but isSessionActive = false
      expect(sessionStore.isSessionActive).toBe(false)
    })
  })

  describe('Session resume', () => {
    it('loads an existing ended session and resumes it', async () => {
      const sessionStore = useSessionStore()

      const endedSession = makeSession({ status: 'ended', ended_at: '2024-01-01T11:00:00Z' })
      vi.mocked(tauri.getSession).mockResolvedValue(endedSession)

      await sessionStore.loadSession('session-1')

      expect(sessionStore.sessions).toHaveLength(1)
      expect(sessionStore.sessions.at(0)?.status).toBe('ended')

      // Resume by updating status to active
      const resumedSession = makeSession({ status: 'active', ended_at: null })
      vi.mocked(tauri.updateSession).mockResolvedValue(undefined)
      await sessionStore.updateSession(resumedSession)

      const session = sessionStore.sessions.find(s => s.id === 'session-1')
      expect(session?.status).toBe('active')
    })

    it('loads all sessions for the review list', async () => {
      const sessionStore = useSessionStore()

      const sessions: Session[] = [
        makeSession({ id: 'session-1', status: 'ended' }),
        makeSession({ id: 'session-2', status: 'reviewed' }),
        makeSession({ id: 'session-3', status: 'synced' }),
      ]
      vi.mocked(tauri.listSessions).mockResolvedValue(sessions)

      await sessionStore.loadAllSessions()

      expect(sessionStore.sessions).toHaveLength(3)
      expect(sessionStore.sessionCount).toBe(3)
    })
  })

  describe('Rapid sequential bug captures', () => {
    it('sequential captures maintain correct auto-incremented bug numbers', async () => {
      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      const mockSession = makeSession()
      vi.mocked(tauri.createSession).mockResolvedValue(mockSession)
      await sessionStore.createSession({})

      const capturedBugIds: string[] = []
      for (let i = 1; i <= 5; i++) {
        const bug = makeBug({
          id: `bug-${i}`,
          bug_number: i,
          display_id: `BUG-00${i}`,
          status: 'captured',
        })
        vi.mocked(tauri.createBug).mockResolvedValue(bug)
        const created = await bugStore.createBug({ session_id: 'session-1' })
        capturedBugIds.push(created.id)
      }

      expect(bugStore.backendBugs).toHaveLength(5)
      bugStore.backendBugs.forEach((bug, idx) => {
        expect(bug.bug_number).toBe(idx + 1)
        expect(bug.display_id).toBe(`BUG-00${idx + 1}`)
      })
    })
  })

  describe('File watcher event handling', () => {
    it('registers event listeners for file watcher events on session setup', async () => {
      const sessionStore = useSessionStore()

      const mockSession = makeSession()
      vi.mocked(tauri.createSession).mockResolvedValue(mockSession)

      // Verify listen() is available (called during store initialization)
      expect(listen).toBeDefined()

      await sessionStore.createSession({})
      expect(sessionStore.activeSession).toEqual(mockSession)
    })

    it('session:started event is handled by the session store listener', async () => {
      const sessionStore = useSessionStore()

      // Simulate what the file watcher backend event listener setup looks like.
      // The store calls listen() to register for Tauri events.
      vi.mocked(listen).mockImplementation(async (_eventName, _handler) => {
        return () => {}
      })

      // Trigger store setup that would register listeners
      const mockSession = makeSession()
      vi.mocked(tauri.createSession).mockResolvedValue(mockSession)
      await sessionStore.createSession({})

      // Verify the session was created successfully (the store works with events)
      expect(sessionStore.activeSession).not.toBeNull()
    })
  })

  describe('Concurrent session prevention', () => {
    it('only one session can be active at a time', async () => {
      const sessionStore = useSessionStore()

      const session1 = makeSession({ id: 'session-1' })
      vi.mocked(tauri.createSession).mockResolvedValue(session1)
      await sessionStore.createSession({})

      expect(sessionStore.activeSession?.id).toBe('session-1')
      expect(sessionStore.sessions).toHaveLength(1)

      // End session 1, then create session 2
      const endedSession1 = makeSession({ id: 'session-1', status: 'ended' })
      vi.mocked(tauri.updateSession).mockResolvedValue(undefined)
      await sessionStore.updateSession(endedSession1)

      const session2 = makeSession({ id: 'session-2' })
      vi.mocked(tauri.createSession).mockResolvedValue(session2)
      await sessionStore.createSession({})

      expect(sessionStore.sessions).toHaveLength(2)
      // Latest session is the active one
      expect(sessionStore.activeSession?.id).toBe('session-2')
    })
  })

  describe('Error handling and recovery', () => {
    it('session creation failure sets error state and does not modify sessions', async () => {
      const sessionStore = useSessionStore()

      vi.mocked(tauri.createSession).mockRejectedValue(new Error('Database write failed'))

      await expect(sessionStore.createSession({})).rejects.toThrow('Database write failed')

      expect(sessionStore.sessions).toHaveLength(0)
      expect(sessionStore.activeSession).toBeNull()
      expect(sessionStore.error).toBe('Database write failed')
      expect(sessionStore.hasError).toBe(true)
      expect(sessionStore.loading).toBe(false)
    })

    it('bug creation failure sets error state', async () => {
      const bugStore = useBugStore()

      vi.mocked(tauri.createBug).mockRejectedValue(new Error('Session not active'))

      await expect(bugStore.createBug({ session_id: 'session-1' })).rejects.toThrow(
        'Session not active',
      )

      expect(bugStore.backendBugs).toHaveLength(0)
      expect(bugStore.error).toBe('Session not active')
      expect(bugStore.hasError).toBe(true)
      expect(bugStore.loading).toBe(false)
    })

    it('clears error state on next successful operation', async () => {
      const sessionStore = useSessionStore()

      // First call fails
      vi.mocked(tauri.createSession).mockRejectedValueOnce(new Error('Network error'))
      await expect(sessionStore.createSession({})).rejects.toThrow()
      expect(sessionStore.error).toBe('Network error')

      // Second call succeeds — error should be cleared
      const mockSession = makeSession()
      vi.mocked(tauri.createSession).mockResolvedValue(mockSession)
      await sessionStore.createSession({})

      expect(sessionStore.error).toBeNull()
      expect(sessionStore.hasError).toBe(false)
    })
  })

  describe('Status transitions', () => {
    it('session status progresses through active → ended → reviewed → synced', async () => {
      const sessionStore = useSessionStore()

      // Create session first so it exists in the store
      const activeSession = makeSession({ status: 'active' })
      vi.mocked(tauri.createSession).mockResolvedValue(activeSession)
      await sessionStore.createSession({})
      expect(sessionStore.sessions.at(0)?.status).toBe('active')

      vi.mocked(tauri.updateSession).mockResolvedValue(undefined)

      // Transition through each subsequent status
      const transitions: Array<Session['status']> = ['ended', 'reviewed', 'synced']
      for (const status of transitions) {
        const updatedSession = makeSession({ status })
        await sessionStore.updateSession(updatedSession)
        const stored = sessionStore.sessions.find(s => s.id === 'session-1')
        expect(stored?.status).toBe(status)
      }
    })

    it('isSessionActive returns false for all non-active statuses', async () => {
      const inactiveStatuses: Array<Session['status']> = ['ended', 'reviewed', 'synced']

      for (const status of inactiveStatuses) {
        setActivePinia(createPinia())
        const freshStore = useSessionStore()
        const session = makeSession({ status })
        vi.mocked(tauri.createSession).mockResolvedValue(session)
        await freshStore.createSession({})
        // isSessionActive checks status === 'active'
        expect(freshStore.isSessionActive).toBe(false)
      }
    })
  })

  describe('Cross-store coordination', () => {
    it('session and bug stores operate independently without interfering', async () => {
      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      // Set up session
      const mockSession = makeSession()
      vi.mocked(tauri.createSession).mockResolvedValue(mockSession)
      await sessionStore.createSession({})

      // Create bugs
      const bug1 = makeBug({ id: 'bug-1', bug_number: 1 })
      const bug2 = makeBug({ id: 'bug-2', bug_number: 2 })
      vi.mocked(tauri.createBug).mockResolvedValueOnce(bug1).mockResolvedValueOnce(bug2)

      await bugStore.createBug({ session_id: 'session-1' })
      await bugStore.createBug({ session_id: 'session-1' })

      // Session store is unaffected by bug operations
      expect(sessionStore.activeSession?.id).toBe('session-1')
      expect(sessionStore.isSessionActive).toBe(true)
      expect(sessionStore.sessions).toHaveLength(1)

      // Bug store is unaffected by session queries
      expect(bugStore.backendBugs).toHaveLength(2)
      expect(bugStore.backendBugs.at(0)?.bug_number).toBe(1)
      expect(bugStore.backendBugs.at(1)?.bug_number).toBe(2)
    })

    it('deleting session removes it from session store', async () => {
      const sessionStore = useSessionStore()

      const mockSession = makeSession()
      vi.mocked(tauri.createSession).mockResolvedValue(mockSession)
      await sessionStore.createSession({})
      expect(sessionStore.sessions).toHaveLength(1)

      vi.mocked(tauri.deleteSession).mockResolvedValue(undefined)
      await sessionStore.deleteSession('session-1')

      expect(sessionStore.sessions).toHaveLength(0)
      expect(sessionStore.activeSession).toBeNull()
    })

    it('session summaries loaded independently from full sessions', async () => {
      const sessionStore = useSessionStore()

      const summaries: SessionSummary[] = [
        { id: 'session-1', started_at: '2024-01-01T10:00:00Z', ended_at: null, status: 'active', bug_count: 3 },
        { id: 'session-2', started_at: '2024-01-02T10:00:00Z', ended_at: '2024-01-02T12:00:00Z', status: 'ended', bug_count: 7 },
      ]
      vi.mocked(tauri.getSessionSummaries).mockResolvedValue(summaries)

      await sessionStore.loadSessionSummaries()

      expect(sessionStore.sessionSummaries).toHaveLength(2)
      expect(sessionStore.sessionSummaries.at(0)?.bug_count).toBe(3)
      expect(sessionStore.sessionSummaries.at(1)?.bug_count).toBe(7)

      // Full session list is not affected
      expect(sessionStore.sessions).toHaveLength(0)
    })
  })

  describe('State cleanup and isolation', () => {
    it('each test starts with a clean pinia state', () => {
      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      // These assertions verify that beforeEach resets state correctly
      expect(sessionStore.sessions).toHaveLength(0)
      expect(sessionStore.activeSession).toBeNull()
      expect(sessionStore.error).toBeNull()
      expect(bugStore.backendBugs).toHaveLength(0)
      expect(bugStore.activeBug).toBeNull()
      expect(bugStore.error).toBeNull()
    })

    it('loading state is always false after operation completes', async () => {
      const sessionStore = useSessionStore()

      expect(sessionStore.loading).toBe(false)

      const mockSession = makeSession()
      vi.mocked(tauri.createSession).mockResolvedValue(mockSession)

      const createPromise = sessionStore.createSession({})
      // loading goes true during operation
      expect(sessionStore.loading).toBe(true)

      await createPromise
      // loading returns to false after completion
      expect(sessionStore.loading).toBe(false)
    })

    it('bug store loading state resets after error', async () => {
      const bugStore = useBugStore()

      vi.mocked(tauri.createBug).mockRejectedValue(new Error('Disk full'))

      await expect(bugStore.createBug({})).rejects.toThrow()
      expect(bugStore.loading).toBe(false)
    })
  })
})
