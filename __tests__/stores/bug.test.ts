import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useBugStore } from '@/stores/bug'
import type { Bug as BackendBug } from '@/types/backend'

// Mock Tauri API
vi.mock('@/api/tauri', () => ({
  createBug: vi.fn(),
  getBug: vi.fn(),
  updateBug: vi.fn(),
  deleteBug: vi.fn(),
  listBugs: vi.fn(),
  getBugsBySession: vi.fn(),
}))

// Mock Tauri event listeners
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))

import * as tauri from '@/api/tauri'

describe('Bug Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  const createMockBackendBug = (id: string, title: string): BackendBug => ({
    id,
    session_id: 'session-1',
    bug_number: 1,
    display_id: 'Bug-01',
    type: 'bug',
    title,
    notes: null,
    description: 'Test description',
    ai_description: null,
    status: 'captured',
    meeting_id: null,
    software_version: null,
    console_parse_json: null,
    metadata_json: null,
    folder_path: '/test/path',
    created_at: '2024-01-01T10:00:00Z',
    updated_at: '2024-01-01T10:00:00Z',
  })

  it('should initialize with empty backendBugs array', () => {
    const store = useBugStore()
    expect(store.backendBugs).toEqual([])
    expect(store.bugCount).toBe(0)
  })

  describe('Backend Operations', () => {
    it('should create a backend bug', async () => {
      const store = useBugStore()
      const mockBug = createMockBackendBug('bug-1', 'Test Bug')
      vi.mocked(tauri.createBug).mockResolvedValue(mockBug)

      const result = await store.createBug({ title: 'Test Bug' })

      expect(result).toEqual(mockBug)
      expect(store.backendBugs).toHaveLength(1)
      expect(store.backendBugs[0]).toEqual(mockBug)
      expect(tauri.createBug).toHaveBeenCalledWith({ title: 'Test Bug' })
    })

    it('should load a backend bug', async () => {
      const store = useBugStore()
      const mockBug = createMockBackendBug('bug-1', 'Test Bug')
      vi.mocked(tauri.getBug).mockResolvedValue(mockBug)

      const result = await store.loadBug('bug-1')

      expect(result).toEqual(mockBug)
      expect(store.backendBugs).toHaveLength(1)
      expect(store.backendBugs[0]).toEqual(mockBug)
      expect(tauri.getBug).toHaveBeenCalledWith('bug-1')
    })

    it('should update a backend bug', async () => {
      const store = useBugStore()
      const mockBug = createMockBackendBug('bug-1', 'Original')
      store.backendBugs.push(mockBug)
      vi.mocked(tauri.updateBug).mockResolvedValue()

      await store.updateBackendBug('bug-1', { title: 'Updated' })

      expect(store.backendBugs[0]?.title).toBe('Updated')
      expect(tauri.updateBug).toHaveBeenCalledWith('bug-1', { title: 'Updated' })
    })

    it('should delete a backend bug', async () => {
      const store = useBugStore()
      const mockBug = createMockBackendBug('bug-1', 'Test Bug')
      store.backendBugs.push(mockBug)
      vi.mocked(tauri.deleteBug).mockResolvedValue()

      await store.deleteBug('bug-1')

      expect(store.backendBugs).toHaveLength(0)
      expect(tauri.deleteBug).toHaveBeenCalledWith('bug-1')
    })

    it('should load all bugs', async () => {
      const store = useBugStore()
      const bugs = [
        createMockBackendBug('bug-1', 'Bug 1'),
        createMockBackendBug('bug-2', 'Bug 2'),
      ]
      vi.mocked(tauri.listBugs).mockResolvedValue(bugs)

      await store.loadAllBugs()

      expect(store.backendBugs).toEqual(bugs)
      expect(tauri.listBugs).toHaveBeenCalled()
    })

    it('should load bugs by session', async () => {
      const store = useBugStore()
      const bugs = [createMockBackendBug('bug-1', 'Bug 1')]
      vi.mocked(tauri.getBugsBySession).mockResolvedValue(bugs)

      await store.loadBugsBySession('session-1')

      expect(store.backendBugs).toEqual(bugs)
      expect(tauri.getBugsBySession).toHaveBeenCalledWith('session-1')
    })

    it('should handle errors', async () => {
      const store = useBugStore()
      const error = new Error('Failed to create bug')
      vi.mocked(tauri.createBug).mockRejectedValue(error)

      await expect(store.createBug({})).rejects.toThrow('Failed to create bug')
      expect(store.error).toBe('Failed to create bug')
    })
  })

  describe('Bug Lifecycle', () => {
    it('should start bug capture', async () => {
      const store = useBugStore()
      const mockBug = { ...createMockBackendBug('bug-1', 'Test'), status: 'capturing' as const }
      vi.mocked(tauri.createBug).mockResolvedValue(mockBug)

      const result = await store.startBugCapture({ title: 'Test' })

      expect(result.status).toBe('capturing')
      expect(store.activeBug).toEqual(mockBug)
    })

    it('should complete bug capture', async () => {
      const store = useBugStore()
      const mockBug = createMockBackendBug('bug-1', 'Test')
      store.activeBug = mockBug
      store.backendBugs.push(mockBug)
      vi.mocked(tauri.updateBug).mockResolvedValue()

      await store.completeBugCapture('bug-1')

      expect(store.backendBugs[0]?.status).toBe('captured')
      expect(store.activeBug).toBeNull()
    })

    it('should update bug status', async () => {
      const store = useBugStore()
      const mockBug = createMockBackendBug('bug-1', 'Test')
      store.backendBugs.push(mockBug)
      vi.mocked(tauri.updateBug).mockResolvedValue()

      await store.updateBugStatus('bug-1', 'reviewed')

      expect(store.backendBugs[0]?.status).toBe('reviewed')
    })
  })

  describe('startBugCapture - no active session', () => {
    it('should still create a bug even when no session is set (session_id is caller responsibility)', async () => {
      const store = useBugStore()
      const mockBug = { ...createMockBackendBug('bug-1', 'Test'), status: 'capturing' as const }
      vi.mocked(tauri.createBug).mockResolvedValue(mockBug)

      // startBugCapture does not check for an active session — that's the session store's job
      const result = await store.startBugCapture({})

      expect(result).toEqual(mockBug)
      expect(store.activeBug).toEqual(mockBug)
      expect(store.isCapturing).toBe(true)
    })

    it('should propagate backend error when createBug fails during startBugCapture', async () => {
      const store = useBugStore()
      vi.mocked(tauri.createBug).mockRejectedValue(new Error('No active session'))

      await expect(store.startBugCapture({})).rejects.toThrow('No active session')

      expect(store.activeBug).toBeNull()
      expect(store.isCapturing).toBe(false)
      expect(store.error).toBe('No active session')
    })
  })

  describe('completeBugCapture edge cases', () => {
    it('should handle completeBugCapture when capture was never started (no activeBug)', async () => {
      const store = useBugStore()
      const mockBug = createMockBackendBug('bug-1', 'Orphan bug')
      store.backendBugs.push(mockBug)
      vi.mocked(tauri.updateBug).mockResolvedValue()

      // activeBug is null — this should still succeed for the given id
      await store.completeBugCapture('bug-1')

      expect(store.backendBugs[0]?.status).toBe('captured')
      // activeBug was already null and should remain null
      expect(store.activeBug).toBeNull()
    })

    it('should propagate error when backend fails during completeBugCapture', async () => {
      const store = useBugStore()
      const mockBug = createMockBackendBug('bug-1', 'Test')
      store.activeBug = mockBug
      store.backendBugs.push(mockBug)
      vi.mocked(tauri.updateBug).mockRejectedValue(new Error('Update failed'))

      await expect(store.completeBugCapture('bug-1')).rejects.toThrow('Update failed')

      expect(store.error).toBe('Update failed')
    })
  })

  describe('deleteBug', () => {
    it('should clear activeBug when deleting the currently active bug', async () => {
      const store = useBugStore()
      const mockBug = createMockBackendBug('bug-1', 'Active bug')
      store.backendBugs.push(mockBug)
      store.activeBug = mockBug
      vi.mocked(tauri.deleteBug).mockResolvedValue()

      await store.deleteBug('bug-1')

      expect(store.activeBug).toBeNull()
    })
  })

  describe('getBugsBySession returning empty array', () => {
    it('should handle backend returning empty array for a session with no bugs', async () => {
      const store = useBugStore()
      vi.mocked(tauri.getBugsBySession).mockResolvedValue([])

      await store.loadBugsBySession('session-empty')

      expect(store.backendBugs).toEqual([])
      expect(tauri.getBugsBySession).toHaveBeenCalledWith('session-empty')
    })

    it('should clear existing backendBugs when session has no bugs', async () => {
      const store = useBugStore()
      store.backendBugs.push(createMockBackendBug('old-bug', 'Old'))
      vi.mocked(tauri.getBugsBySession).mockResolvedValue([])

      await store.loadBugsBySession('session-empty')

      expect(store.backendBugs).toHaveLength(0)
    })

    it('should handle error when loading bugs by session', async () => {
      const store = useBugStore()
      vi.mocked(tauri.getBugsBySession).mockRejectedValue(new Error('Session not found'))

      await expect(store.loadBugsBySession('bad-session')).rejects.toThrow('Session not found')

      expect(store.error).toBe('Session not found')
      expect(store.loading).toBe(false)
    })
  })

  describe('Computed Properties', () => {
    it('should compute hasError', () => {
      const store = useBugStore()
      expect(store.hasError).toBe(false)

      store.error = 'Test error'
      expect(store.hasError).toBe(true)
    })

    it('should compute isCapturing', () => {
      const store = useBugStore()
      expect(store.isCapturing).toBe(false)

      const mockBug = { ...createMockBackendBug('bug-1', 'Test'), status: 'capturing' as const }
      store.activeBug = mockBug
      expect(store.isCapturing).toBe(true)
    })

    it('should compute bugCount from backendBugs', () => {
      const store = useBugStore()
      expect(store.bugCount).toBe(0)

      store.backendBugs.push(createMockBackendBug('bug-1', 'Bug 1'))
      store.backendBugs.push(createMockBackendBug('bug-2', 'Bug 2'))
      expect(store.bugCount).toBe(2)
    })
  })

  describe('clearBugs', () => {
    it('should clear all state', () => {
      const store = useBugStore()
      const mockBug = createMockBackendBug('bug-1', 'Bug 1')
      store.backendBugs.push(mockBug)
      store.activeBug = mockBug
      store.setLastSessionMeetingId('meeting-123')
      store.setTagNextScreenshotAsConsole(true)

      store.clearBugs()

      expect(store.backendBugs).toEqual([])
      expect(store.bugCount).toBe(0)
      expect(store.activeBug).toBeNull()
      expect(store.lastSessionMeetingId).toBeNull()
      expect(store.tagNextScreenshotAsConsole).toBe(false)
    })
  })

  describe('Meeting ID pre-population', () => {
    it('initializes lastSessionMeetingId as null', () => {
      const store = useBugStore()
      expect(store.lastSessionMeetingId).toBeNull()
    })

    it('sets and clears lastSessionMeetingId', () => {
      const store = useBugStore()
      store.setLastSessionMeetingId('meeting-123')
      expect(store.lastSessionMeetingId).toBe('meeting-123')

      store.setLastSessionMeetingId(null)
      expect(store.lastSessionMeetingId).toBeNull()
    })

    it('clears lastSessionMeetingId when clearBugs is called', () => {
      const store = useBugStore()
      store.setLastSessionMeetingId('meeting-123')
      store.clearBugs()
      expect(store.lastSessionMeetingId).toBeNull()
    })
  })

  describe('Software version pre-population', () => {
    it('initializes lastSessionSoftwareVersion as null', () => {
      const store = useBugStore()
      expect(store.lastSessionSoftwareVersion).toBeNull()
    })

    it('sets and clears lastSessionSoftwareVersion', () => {
      const store = useBugStore()
      store.setLastSessionSoftwareVersion('2.0.0')
      expect(store.lastSessionSoftwareVersion).toBe('2.0.0')
      store.setLastSessionSoftwareVersion(null)
      expect(store.lastSessionSoftwareVersion).toBeNull()
    })

    it('clears lastSessionSoftwareVersion when clearBugs is called', () => {
      const store = useBugStore()
      store.setLastSessionSoftwareVersion('2.0.0')
      store.clearBugs()
      expect(store.lastSessionSoftwareVersion).toBeNull()
    })
  })

  describe('Console tag toggle', () => {
    it('initializes tagNextScreenshotAsConsole as false', () => {
      const store = useBugStore()
      expect(store.tagNextScreenshotAsConsole).toBe(false)
    })

    it('sets tagNextScreenshotAsConsole', () => {
      const store = useBugStore()
      store.setTagNextScreenshotAsConsole(true)
      expect(store.tagNextScreenshotAsConsole).toBe(true)

      store.setTagNextScreenshotAsConsole(false)
      expect(store.tagNextScreenshotAsConsole).toBe(false)
    })

    it('consumeConsoleTag returns true and clears the flag', () => {
      const store = useBugStore()
      store.setTagNextScreenshotAsConsole(true)

      const result = store.consumeConsoleTag()
      expect(result).toBe(true)
      expect(store.tagNextScreenshotAsConsole).toBe(false)
    })

    it('consumeConsoleTag returns false when flag is not set', () => {
      const store = useBugStore()
      const result = store.consumeConsoleTag()
      expect(result).toBe(false)
    })

    it('clears tagNextScreenshotAsConsole when clearBugs is called', () => {
      const store = useBugStore()
      store.setTagNextScreenshotAsConsole(true)
      store.clearBugs()
      expect(store.tagNextScreenshotAsConsole).toBe(false)
    })
  })
})
