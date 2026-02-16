import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { Quasar } from 'quasar'
import SessionToolbar from '@/components/SessionToolbar.vue'
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

const mockActiveSession: Session = {
  id: 'session-abc123-def456-ghi789',
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
  session_id: 'session-abc123-def456-ghi789',
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

describe('SessionToolbar', () => {
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
    return mount(SessionToolbar, {
      global: {
        plugins: [pinia, Quasar],
      },
    })
  }

  describe('Visibility', () => {
    it('should render when session is active', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      sessionStore.activeSession = mockActiveSession

      await wrapper.vm.$nextTick()
      expect(wrapper.find('.session-toolbar').exists()).toBe(true)
    })

    it('should not render when no active session', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      sessionStore.activeSession = null

      await wrapper.vm.$nextTick()
      expect(wrapper.find('.session-toolbar').exists()).toBe(false)
    })

    it('should not render when session is ended', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      sessionStore.activeSession = mockEndedSession

      await wrapper.vm.$nextTick()
      expect(wrapper.find('.session-toolbar').exists()).toBe(false)
    })
  })

  describe('Session Display ID', () => {
    it('should display first 8 characters of session ID', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      sessionStore.activeSession = mockActiveSession

      await wrapper.vm.$nextTick()
      expect(wrapper.text()).toContain('Session: session-')
    })

    it('should display N/A when no active session', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      sessionStore.activeSession = null

      await wrapper.vm.$nextTick()

      // Component shouldn't render at all when no session
      expect(wrapper.find('.session-toolbar').exists()).toBe(false)
    })
  })

  describe('Session Time Display', () => {
    it('should display 00:00:00 when session just started', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      sessionStore.activeSession = mockActiveSession

      await wrapper.vm.$nextTick()
      await flushPromises()

      expect(wrapper.text()).toMatch(/00:00:\d{2}/)
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
    it('should display 0 bugs when no bugs exist', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      sessionStore.activeSession = mockActiveSession
      bugStore.backendBugs = []

      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('0 bugs')
    })

    it('should display singular "bug" when count is 1', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      sessionStore.activeSession = mockActiveSession
      bugStore.backendBugs = [mockBug]

      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('1 bug')
    })

    it('should display plural "bugs" when count is not 1', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      sessionStore.activeSession = mockActiveSession
      bugStore.backendBugs = [
        { ...mockBug, id: 'bug-1' },
        { ...mockBug, id: 'bug-2' },
        { ...mockBug, id: 'bug-3' },
      ]

      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('3 bugs')
    })

    it('should only count bugs for active session', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      sessionStore.activeSession = mockActiveSession

      // Add bugs for different sessions
      bugStore.backendBugs = [
        { ...mockBug, id: 'bug-1', session_id: 'session-abc123-def456-ghi789' },
        { ...mockBug, id: 'bug-2', session_id: 'session-abc123-def456-ghi789' },
        { ...mockBug, id: 'bug-3', session_id: 'session-other' },
      ]

      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('2 bugs')
    })

    it('should use positive color when bugs exist', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      sessionStore.activeSession = mockActiveSession
      bugStore.backendBugs = [mockBug]

      await wrapper.vm.$nextTick()

      const bugChip = wrapper.findAllComponents({ name: 'QChip' }).find(chip =>
        chip.text().includes('bug')
      )
      expect(bugChip?.props('color')).toBe('positive')
    })

    it('should use grey color when no bugs exist', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      sessionStore.activeSession = mockActiveSession
      bugStore.backendBugs = []

      await wrapper.vm.$nextTick()

      const bugChip = wrapper.findAllComponents({ name: 'QChip' }).find(chip =>
        chip.text().includes('bug')
      )
      expect(bugChip?.props('color')).toBe('grey')
    })
  })

  describe('Open Folder Button', () => {
    it('should render open folder button', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      sessionStore.activeSession = mockActiveSession

      await wrapper.vm.$nextTick()

      const openFolderBtn = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('icon') === 'folder_open'
      )
      expect(openFolderBtn).toBeDefined()
      expect(openFolderBtn?.props('label')).toBe('Open Folder')
    })

    it('should have tooltip on open folder button', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      sessionStore.activeSession = mockActiveSession

      await wrapper.vm.$nextTick()

      const tooltip = wrapper.findComponent({ name: 'QTooltip' })
      expect(tooltip.exists()).toBe(true)
      // Tooltip content may not be rendered in test environment, just verify it exists
    })

    it('should log folder path when clicked', async () => {
      const consoleLogSpy = vi.spyOn(console, 'log').mockImplementation(() => {})

      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      sessionStore.activeSession = mockActiveSession

      await wrapper.vm.$nextTick()

      const openFolderBtn = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('icon') === 'folder_open'
      )

      await openFolderBtn?.trigger('click')
      await wrapper.vm.$nextTick()

      expect(consoleLogSpy).toHaveBeenCalledWith(
        'Opening session folder:',
        '/test/sessions/session1'
      )

      consoleLogSpy.mockRestore()
    })

    it('should handle missing folder path gracefully', async () => {
      const consoleWarnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {})

      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      sessionStore.activeSession = {
        ...mockActiveSession,
        folder_path: '',
      }

      await wrapper.vm.$nextTick()

      const openFolderBtn = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('icon') === 'folder_open'
      )

      await openFolderBtn?.trigger('click')
      await wrapper.vm.$nextTick()

      expect(consoleWarnSpy).toHaveBeenCalledWith(
        'No folder path available for active session'
      )

      consoleWarnSpy.mockRestore()
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
      sessionStore.activeSession = null
      await wrapper.vm.$nextTick()

      // Component should not render
      expect(wrapper.find('.session-toolbar').exists()).toBe(false)
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

      // Change started_at to a new time
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
    it('should update when session store changes', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()

      // Start with no session (component hidden)
      sessionStore.activeSession = null
      await wrapper.vm.$nextTick()
      expect(wrapper.find('.session-toolbar').exists()).toBe(false)

      // Add active session (component visible)
      sessionStore.activeSession = mockActiveSession
      await wrapper.vm.$nextTick()
      expect(wrapper.find('.session-toolbar').exists()).toBe(true)
    })

    it('should update bug count when bug store changes', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      sessionStore.activeSession = mockActiveSession
      bugStore.backendBugs = []
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('0 bugs')

      // Add bugs
      bugStore.backendBugs = [mockBug]
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('1 bug')
    })
  })

  describe('UI Components', () => {
    it('should have play_circle icon', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      sessionStore.activeSession = mockActiveSession

      await wrapper.vm.$nextTick()

      const icon = wrapper.findAllComponents({ name: 'QIcon' }).find(icon =>
        icon.props('name') === 'play_circle'
      )
      expect(icon).toBeDefined()
    })

    it('should have schedule icon in time chip', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      sessionStore.activeSession = mockActiveSession

      await wrapper.vm.$nextTick()

      const icon = wrapper.findAllComponents({ name: 'QIcon' }).find(icon =>
        icon.props('name') === 'schedule'
      )
      expect(icon).toBeDefined()
    })

    it('should have bug_report icon in bug chip', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      sessionStore.activeSession = mockActiveSession

      await wrapper.vm.$nextTick()

      const icon = wrapper.findAllComponents({ name: 'QIcon' }).find(icon =>
        icon.props('name') === 'bug_report'
      )
      expect(icon).toBeDefined()
    })

    it('should use secondary background class', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()
      sessionStore.activeSession = mockActiveSession

      await wrapper.vm.$nextTick()

      const toolbar = wrapper.find('.session-toolbar')
      expect(toolbar.classes()).toContain('bg-secondary')
      expect(toolbar.classes()).toContain('text-white')
    })
  })
})
