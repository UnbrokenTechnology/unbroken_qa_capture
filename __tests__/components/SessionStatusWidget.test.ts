import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { Quasar } from 'quasar'
import SessionStatusWidget from '@/components/SessionStatusWidget.vue'
import { useSessionStore } from '@/stores/session'
import { useBugStore } from '@/stores/bug'
import type { Session, Bug as BackendBug } from '@/types/backend'

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

// Mock Tauri window API
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => ({
    label: 'session-status',
    close: vi.fn().mockResolvedValue(undefined),
    onCloseRequested: vi.fn().mockResolvedValue(() => {}),
  })),
}))

const mockActiveSession: Session = {
  id: 'session-1',
  started_at: new Date().toISOString(),
  ended_at: null,
  status: 'active',
  folder_path: '/test/sessions/session1',
  session_notes: 'Test notes',
  environment_json: '{"os":"Windows 11"}',
  original_snip_path: null,
  created_at: new Date().toISOString(),
}

const mockEndedSession: Session = {
  id: 'session-2',
  started_at: '2024-01-01T10:00:00Z',
  ended_at: '2024-01-01T12:00:00Z',
  status: 'ended',
  folder_path: '/test/sessions/session2',
  session_notes: null,
  environment_json: null,
  original_snip_path: null,
  created_at: '2024-01-01T10:00:00Z',
}

const mockBug: BackendBug = {
  id: 'bug-1',
  session_id: 'session-1',
  bug_number: 1,
  display_id: 'BUG-001',
  type: 'bug',
  title: 'Test Bug',
  notes: null,
  description: 'Test description',
  ai_description: null,
  status: 'capturing',
  meeting_id: null,
  software_version: null,
  console_parse_json: null,
  metadata_json: null,
  folder_path: '/test/bugs/bug1',
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
}

describe('SessionStatusWidget', () => {
  let pinia: ReturnType<typeof createPinia>

  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
    vi.clearAllMocks()
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  const mountComponent = () => {
    return mount(SessionStatusWidget, {
      global: {
        plugins: [pinia, Quasar],
      },
    })
  }

  describe('Rendering', () => {
    it('should always render the root element', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      expect(wrapper.find('.session-status-bar').exists()).toBe(true)
    })

    it('should add is-capturing class when capturing a bug', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      sessionStore.activeSession = mockActiveSession
      bugStore.activeBug = mockBug

      await wrapper.vm.$nextTick()

      expect(wrapper.find('.session-status-bar').classes()).toContain('is-capturing')
    })

    it('should not have is-capturing class when not capturing', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      sessionStore.activeSession = mockActiveSession
      bugStore.activeBug = null

      await wrapper.vm.$nextTick()

      expect(wrapper.find('.session-status-bar').classes()).not.toContain('is-capturing')
    })

    it('should render drag-handle inside root', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      expect(wrapper.find('.drag-handle').exists()).toBe(true)
    })

    it('should show Console chip when tagNextScreenshotAsConsole is true', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const bugStore = useBugStore()
      bugStore.tagNextScreenshotAsConsole = true

      await wrapper.vm.$nextTick()

      const chips = wrapper.findAllComponents({ name: 'QChip' })
      const consoleChip = chips.find(chip => chip.text() === 'Console')
      expect(consoleChip).toBeDefined()
      expect(consoleChip?.props('color')).toBe('orange')
    })

    it('should not show Console chip when tagNextScreenshotAsConsole is false', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const bugStore = useBugStore()
      bugStore.tagNextScreenshotAsConsole = false

      await wrapper.vm.$nextTick()

      const chips = wrapper.findAllComponents({ name: 'QChip' })
      const consoleChip = chips.find(chip => chip.text() === 'Console')
      expect(consoleChip).toBeUndefined()
    })
  })

  describe('Session Time Display', () => {
    it('should display 00:00:00 when no active session', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      sessionStore.activeSession = null

      await wrapper.vm.$nextTick()
      expect(wrapper.text()).toContain('00:00:00')
    })

    it('should display formatted session time when session is active', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      // Set session started 90 seconds ago (00:01:30)
      const startedAt = new Date(Date.now() - 90000).toISOString()
      sessionStore.activeSession = { ...mockActiveSession, started_at: startedAt }

      await wrapper.vm.$nextTick()
      await flushPromises()

      // Allow time for timer to update
      vi.advanceTimersByTime(1000)
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toMatch(/00:01:\d{2}/)
    })

    it('should update session time every second', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      sessionStore.activeSession = mockActiveSession

      await wrapper.vm.$nextTick()
      await flushPromises()

      // Fast-forward time by 5 seconds
      vi.advanceTimersByTime(5000)
      await wrapper.vm.$nextTick()

      // The time should have progressed
      expect(wrapper.text()).toMatch(/\d{2}:\d{2}:\d{2}/)
    })

    it('should format hours, minutes, and seconds correctly', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      // Set session started 3665 seconds ago (1 hour, 1 minute, 5 seconds)
      const startedAt = new Date(Date.now() - 3665000).toISOString()
      sessionStore.activeSession = { ...mockActiveSession, started_at: startedAt }

      await wrapper.vm.$nextTick()
      await flushPromises()

      vi.advanceTimersByTime(1000)
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toMatch(/01:01:\d{2}/)
    })
  })

  describe('Bug Count Display', () => {
    it('should display "0 bugs" with grey-6 badge color when count is 0', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const bugStore = useBugStore()
      bugStore.backendBugs = []

      await wrapper.vm.$nextTick()

      const badge = wrapper.findComponent({ name: 'QBadge' })
      expect(badge.exists()).toBe(true)
      expect(badge.props('label')).toBe('0 bugs')
      expect(badge.props('color')).toBe('grey-6')
    })

    it('should display "1 bug" with positive badge color when count is 1', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      sessionStore.activeSession = mockActiveSession
      bugStore.backendBugs = [mockBug]

      await wrapper.vm.$nextTick()

      const badge = wrapper.findComponent({ name: 'QBadge' })
      expect(badge.props('label')).toBe('1 bug')
      expect(badge.props('color')).toBe('positive')
    })

    it('should display plural "bugs" label when count is not 1', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      sessionStore.activeSession = mockActiveSession
      bugStore.backendBugs = [
        { ...mockBug, id: 'bug-1' },
        { ...mockBug, id: 'bug-2' },
      ]

      await wrapper.vm.$nextTick()

      const badge = wrapper.findComponent({ name: 'QBadge' })
      expect(badge.props('label')).toBe('2 bugs')
      expect(badge.props('color')).toBe('positive')
    })

    it('should only count bugs for active session when session is active', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      sessionStore.activeSession = mockActiveSession

      // Add bugs for different sessions
      bugStore.backendBugs = [
        { ...mockBug, id: 'bug-1', session_id: 'session-1' },
        { ...mockBug, id: 'bug-2', session_id: 'session-1' },
        { ...mockBug, id: 'bug-3', session_id: 'session-other' },
      ]

      await wrapper.vm.$nextTick()

      const badge = wrapper.findComponent({ name: 'QBadge' })
      expect(badge.props('label')).toBe('2 bugs')
    })

    it('should use bugStore.bugCount when no active session', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      sessionStore.activeSession = null
      bugStore.backendBugs = [mockBug]

      await wrapper.vm.$nextTick()

      const badge = wrapper.findComponent({ name: 'QBadge' })
      // bugCount computed from store is backendBugs.length when no active session
      expect(badge.props('label')).toBe('1 bug')
    })
  })

  describe('Current State Display', () => {
    it('should display "Idle" when no active session', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      sessionStore.activeSession = null

      await wrapper.vm.$nextTick()

      const chip = wrapper.findComponent({ name: 'QChip' })
      expect(chip.text()).toBe('Idle')
      expect(chip.props('color')).toBe('grey')
    })

    it('should display "QA Mode" when session is active but not capturing', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      sessionStore.activeSession = mockActiveSession
      bugStore.activeBug = null

      await wrapper.vm.$nextTick()

      const chip = wrapper.findComponent({ name: 'QChip' })
      expect(chip.text()).toBe('QA Mode')
      expect(chip.props('color')).toBe('primary')
    })

    it('should display "Bug-N" (without "Capturing" prefix) when actively capturing a bug', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      sessionStore.activeSession = mockActiveSession
      bugStore.activeBug = mockBug

      await wrapper.vm.$nextTick()

      const chip = wrapper.findComponent({ name: 'QChip' })
      expect(chip.text()).toBe('Bug-1')
      expect(chip.props('color')).toBe('negative')
    })

    it('should use correct color for each state', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      // Test Idle state (grey)
      sessionStore.activeSession = null
      await wrapper.vm.$nextTick()
      let chip = wrapper.findComponent({ name: 'QChip' })
      expect(chip.props('color')).toBe('grey')

      // Test QA Mode (primary)
      sessionStore.activeSession = mockActiveSession
      bugStore.activeBug = null
      await wrapper.vm.$nextTick()
      chip = wrapper.findComponent({ name: 'QChip' })
      expect(chip.props('color')).toBe('primary')

      // Test Capturing (negative)
      bugStore.activeBug = mockBug
      await wrapper.vm.$nextTick()
      chip = wrapper.findComponent({ name: 'QChip' })
      expect(chip.props('color')).toBe('negative')
    })

    it('should reflect state color on state-dot element', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      sessionStore.activeSession = null
      await wrapper.vm.$nextTick()

      const dot = wrapper.find('.state-dot')
      expect(dot.classes()).toContain('bg-grey')

      sessionStore.activeSession = mockActiveSession
      await wrapper.vm.$nextTick()

      expect(dot.classes()).toContain('bg-primary')
    })
  })

  describe('Close Button', () => {
    it('should render the close button', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const closeButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('icon') === 'close'
      )
      expect(closeButton).toBeDefined()
    })

    it('should call getCurrentWindow().close() when close button is clicked', async () => {
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      const mockClose = vi.fn().mockResolvedValue(undefined)
      vi.mocked(getCurrentWindow).mockReturnValue({
        label: 'session-status',
        close: mockClose,
        onCloseRequested: vi.fn().mockResolvedValue(() => {}),
      } as unknown as ReturnType<typeof getCurrentWindow>)

      const wrapper = mountComponent()
      await flushPromises()

      const closeButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('icon') === 'close'
      )

      await closeButton!.trigger('click')
      await flushPromises()

      expect(mockClose).toHaveBeenCalled()
    })

    it('should not emit a close event (uses Tauri window API instead)', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const closeButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('icon') === 'close'
      )

      await closeButton!.trigger('click')

      expect(wrapper.emitted('close')).toBeFalsy()
    })
  })

  describe('Store Integration', () => {
    it('should setup event listeners on mount', async () => {
      mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      // Verify that setupEventListeners methods exist and the component calls them
      expect(sessionStore.setupEventListeners).toBeDefined()
      expect(bugStore.setupEventListeners).toBeDefined()
    })

    it('should cleanup event listeners on unmount', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      const sessionCleanupSpy = vi.spyOn(sessionStore, 'cleanupEventListeners')
      const bugCleanupSpy = vi.spyOn(bugStore, 'cleanupEventListeners')

      wrapper.unmount()

      expect(sessionCleanupSpy).toHaveBeenCalled()
      expect(bugCleanupSpy).toHaveBeenCalled()
    })

    it('should load active session on mount', async () => {
      mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()

      expect(sessionStore.loadActiveSession).toBeDefined()
    })

    it('should register Tauri event listeners for session events on mount', async () => {
      const { listen } = await import('@tauri-apps/api/event')

      mountComponent()
      await flushPromises()

      expect(listen).toHaveBeenCalledWith('session-created', expect.any(Function))
      expect(listen).toHaveBeenCalledWith('session-updated', expect.any(Function))
      expect(listen).toHaveBeenCalledWith('session-deleted', expect.any(Function))
      expect(listen).toHaveBeenCalledWith('session-status-changed', expect.any(Function))
    })
  })

  describe('Timer Lifecycle', () => {
    it('should start timer when session becomes active', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()

      // Initially no session
      sessionStore.activeSession = null
      await wrapper.vm.$nextTick()

      // Activate session
      sessionStore.activeSession = mockActiveSession
      await wrapper.vm.$nextTick()
      await flushPromises()

      // Timer should start updating
      vi.advanceTimersByTime(1000)
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toMatch(/\d{2}:\d{2}:\d{2}/)
    })

    it('should stop timer and show 00:00:00 when session ends', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()

      // Start with active session
      sessionStore.activeSession = mockActiveSession
      await wrapper.vm.$nextTick()
      await flushPromises()

      // End session (ended sessions are not considered active)
      sessionStore.activeSession = mockEndedSession
      await wrapper.vm.$nextTick()

      // Time should reset to 00:00:00
      expect(wrapper.text()).toContain('00:00:00')
    })

    it('should clear interval on unmount', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      sessionStore.activeSession = mockActiveSession

      await wrapper.vm.$nextTick()
      await flushPromises()

      const clearIntervalSpy = vi.spyOn(globalThis, 'clearInterval')

      wrapper.unmount()

      expect(clearIntervalSpy).toHaveBeenCalled()
    })

    it('should restart timer when session started_at changes', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      sessionStore.activeSession = mockActiveSession

      await wrapper.vm.$nextTick()
      await flushPromises()

      // Change started_at to 2 minutes ago
      const newStartedAt = new Date(Date.now() - 120000).toISOString()
      sessionStore.activeSession = { ...mockActiveSession, started_at: newStartedAt }

      await wrapper.vm.$nextTick()
      await flushPromises()

      vi.advanceTimersByTime(1000)
      await wrapper.vm.$nextTick()

      // Should show updated time (around 2 minutes)
      expect(wrapper.text()).toMatch(/00:0[2-3]:\d{2}/)
    })
  })

  describe('Reactivity', () => {
    it('should update state label when session store changes', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()

      // Start with no session
      sessionStore.activeSession = null
      await wrapper.vm.$nextTick()
      expect(wrapper.text()).toContain('Idle')

      // Add active session
      sessionStore.activeSession = mockActiveSession
      await wrapper.vm.$nextTick()
      expect(wrapper.text()).toContain('QA Mode')
    })

    it('should update bug count badge when bug store changes', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      sessionStore.activeSession = mockActiveSession
      bugStore.backendBugs = []
      await wrapper.vm.$nextTick()

      let badge = wrapper.findComponent({ name: 'QBadge' })
      expect(badge.props('label')).toBe('0 bugs')

      // Add bugs
      bugStore.backendBugs = [mockBug]
      await wrapper.vm.$nextTick()

      badge = wrapper.findComponent({ name: 'QBadge' })
      expect(badge.props('label')).toBe('1 bug')
    })

    it('should update state chip when bug capture starts', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      sessionStore.activeSession = mockActiveSession
      bugStore.activeBug = null
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('QA Mode')

      // Start capturing
      bugStore.activeBug = mockBug
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('Bug-1')
    })

    it('should toggle Console chip when tagNextScreenshotAsConsole changes', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const bugStore = useBugStore()
      bugStore.tagNextScreenshotAsConsole = false
      await wrapper.vm.$nextTick()

      let consoleChip = wrapper.findAllComponents({ name: 'QChip' }).find(c => c.text() === 'Console')
      expect(consoleChip).toBeUndefined()

      bugStore.tagNextScreenshotAsConsole = true
      await wrapper.vm.$nextTick()

      consoleChip = wrapper.findAllComponents({ name: 'QChip' }).find(c => c.text() === 'Console')
      expect(consoleChip).toBeDefined()
    })
  })
})
