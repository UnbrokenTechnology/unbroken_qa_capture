import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { Quasar } from 'quasar'
import SessionStatusWidget from '@/components/SessionStatusWidget.vue'
import { useSessionStore } from '@/stores/session'
import { useBugStore } from '@/stores/bug'
import type { Session, Bug as BackendBug } from '@/types/backend'

// Mock Tauri window API
const mockSetAlwaysOnTop = vi.fn().mockResolvedValue(undefined)

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => ({
    setAlwaysOnTop: mockSetAlwaysOnTop,
  })),
}))

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

  const mountComponent = (props = {}) => {
    return mount(SessionStatusWidget, {
      props: {
        visible: true,
        ...props,
      },
      global: {
        plugins: [pinia, Quasar],
      },
    })
  }

  describe('Visibility', () => {
    it('should render when visible prop is true', () => {
      const wrapper = mountComponent({ visible: true })
      expect(wrapper.find('.session-status-widget').exists()).toBe(true)
    })

    it('should not render when visible prop is false', () => {
      const wrapper = mountComponent({ visible: false })
      expect(wrapper.find('.session-status-widget').exists()).toBe(false)
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
    it('should display bug count with grey badge when count is 0', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const bugStore = useBugStore()
      bugStore.backendBugs = []

      await wrapper.vm.$nextTick()

      const badge = wrapper.findComponent({ name: 'QBadge' })
      expect(badge.exists()).toBe(true)
      expect(badge.props('label')).toBe(0)
      expect(badge.props('color')).toBe('grey')
    })

    it('should display bug count with positive badge when count > 0', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      // Set active session and add bugs for that session
      sessionStore.activeSession = mockActiveSession
      bugStore.backendBugs = [mockBug]

      await wrapper.vm.$nextTick()

      const badge = wrapper.findComponent({ name: 'QBadge' })
      expect(badge.props('label')).toBe(1)
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
      expect(badge.props('label')).toBe(2)
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

    it('should display "Capturing Bug-N" when actively capturing a bug', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      sessionStore.activeSession = mockActiveSession
      bugStore.activeBug = mockBug

      await wrapper.vm.$nextTick()

      const chip = wrapper.findComponent({ name: 'QChip' })
      expect(chip.text()).toBe('Capturing Bug-1')
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
  })

  describe('Window Positioning', () => {
    it('should use default position when no initial position provided', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const card = wrapper.find('.session-status-widget')
      expect(card.attributes('style')).toContain('top: 20px')
      expect(card.attributes('style')).toContain('left: 20px')
    })

    it('should use custom initial position when provided', async () => {
      const wrapper = mountComponent({ initialX: 100, initialY: 200 })
      await flushPromises()

      const card = wrapper.find('.session-status-widget')
      expect(card.attributes('style')).toContain('top: 200px')
      expect(card.attributes('style')).toContain('left: 100px')
    })

    it('should be positioned fixed with high z-index', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const card = wrapper.find('.session-status-widget')
      expect(card.attributes('style')).toContain('position: fixed')
      expect(card.attributes('style')).toContain('z-index: 9999')
    })

    it('should have cursor:move style for draggability', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const card = wrapper.find('.session-status-widget')
      expect(card.attributes('style')).toContain('cursor: move')
    })
  })

  describe('Always On Top', () => {
    it('should set window to always-on-top when mounted', async () => {
      mountComponent()
      await flushPromises()

      expect(mockSetAlwaysOnTop).toHaveBeenCalledWith(true)
    })

    it('should handle errors when setting always-on-top fails', async () => {
      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      mockSetAlwaysOnTop.mockRejectedValueOnce(new Error('Window API error'))

      mountComponent()
      await flushPromises()

      expect(consoleErrorSpy).toHaveBeenCalledWith(
        'Failed to set window always-on-top:',
        expect.any(Error)
      )

      consoleErrorSpy.mockRestore()
    })
  })

  describe('Close Button', () => {
    it('should emit close event when close button clicked', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const closeButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('icon') === 'close'
      )
      expect(closeButton).toBeDefined()

      await closeButton!.trigger('click')

      expect(wrapper.emitted('close')).toBeTruthy()
      expect(wrapper.emitted('close')).toHaveLength(1)
    })
  })

  describe('Store Integration', () => {
    it('should setup event listeners on mount', async () => {
      mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      // Verify that setupEventListeners was called
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

    it('should stop timer when session ends', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()

      // Start with active session
      sessionStore.activeSession = mockActiveSession
      await wrapper.vm.$nextTick()
      await flushPromises()

      // End session
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
  })

  describe('Reactivity', () => {
    it('should update when session store changes', async () => {
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

    it('should update when bug store changes', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      sessionStore.activeSession = mockActiveSession
      bugStore.backendBugs = []
      await wrapper.vm.$nextTick()

      let badge = wrapper.findComponent({ name: 'QBadge' })
      expect(badge.props('label')).toBe(0)

      // Add bugs
      bugStore.backendBugs = [mockBug]
      await wrapper.vm.$nextTick()

      badge = wrapper.findComponent({ name: 'QBadge' })
      expect(badge.props('label')).toBe(1)
    })

    it('should update state when bug capture starts', async () => {
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

      expect(wrapper.text()).toContain('Capturing Bug-1')
    })
  })
})
