import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createRouter, createMemoryHistory } from 'vue-router'
import { Quasar, Notify } from 'quasar'
import ActiveSessionView from '@/views/ActiveSessionView.vue'

// Mock useQuasar
const mockNotify = vi.fn()
vi.mock('quasar', async () => {
  const actual = await vi.importActual('quasar')
  return {
    ...actual,
    useQuasar: () => ({
      notify: mockNotify,
    }),
  }
})

// Mock Tauri API
vi.mock('@/api/tauri', () => ({
  createSession: vi.fn(),
  getActiveSession: vi.fn().mockResolvedValue(null),
  getSessionSummaries: vi.fn().mockResolvedValue([]),
  updateSession: vi.fn(),
  deleteSession: vi.fn(),
  listSessions: vi.fn().mockResolvedValue([]),
  updateSessionStatus: vi.fn(),
  createBug: vi.fn(),
  getBug: vi.fn(),
  updateBug: vi.fn(),
  deleteBug: vi.fn(),
  listBugs: vi.fn().mockResolvedValue([]),
  getBugsBySession: vi.fn().mockResolvedValue([]),
  openAnnotationWindow: vi.fn(),
}))

// Mock Tauri event listeners
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))

// Mock Tauri core invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}))

// Mock Tauri window
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn().mockReturnValue({
    close: vi.fn().mockResolvedValue(undefined),
  }),
}))

const mockSession = {
  id: 'session-1',
  started_at: new Date().toISOString(),
  ended_at: null,
  status: 'active' as const,
  folder_path: '/test/sessions/session1',
  session_notes: null,
  environment_json: null,
  original_snip_path: null,
  created_at: new Date().toISOString(),
}

const mockBug = {
  id: 'bug-1',
  session_id: 'session-1',
  bug_number: 1,
  display_id: 'BUG-001',
  type: 'bug' as const,
  title: 'Test Bug',
  notes: 'Some notes',
  description: null,
  ai_description: null,
  status: 'capturing' as const,
  meeting_id: null,
  software_version: null,
  console_parse_json: null,
  metadata_json: null,
  folder_path: '/test/sessions/session1/bug-1',
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
}

describe('ActiveSessionView', () => {
  let pinia: ReturnType<typeof createPinia>
  let router: ReturnType<typeof createRouter>

  beforeEach(async () => {
    pinia = createPinia()
    setActivePinia(pinia)

    router = createRouter({
      history: createMemoryHistory(),
      routes: [
        {
          path: '/active',
          name: 'active-session',
          component: ActiveSessionView,
        },
        {
          path: '/session-review/:id',
          name: 'session-review',
          component: { template: '<div>review</div>' },
        },
        {
          path: '/bug/:id',
          name: 'bug-detail',
          component: { template: '<div>bug detail</div>' },
        },
      ],
    })

    vi.clearAllMocks()

    // Re-establish default mock implementations after clearAllMocks
    const tauriApi = await import('@/api/tauri')
    vi.mocked(tauriApi.getBugsBySession).mockResolvedValue([])
    vi.mocked(tauriApi.getActiveSession).mockResolvedValue(null)
    vi.mocked(tauriApi.getSessionSummaries).mockResolvedValue([])
  })

  function mountActiveSessionView(provide?: Record<string, unknown>) {
    return mount(ActiveSessionView, {
      global: {
        plugins: [pinia, router, [Quasar, { plugins: { Notify } }]],
        provide: provide ?? {},
        stubs: {
          QPage: { template: '<div><slot /></div>' },
          QCard: { template: '<div class="q-card"><slot /></div>' },
          QCardSection: { template: '<div><slot /></div>' },
          QBtn: {
            template:
              '<button class="q-btn" :disabled="$attrs.disable" @click="!$attrs.disable && $attrs.onClick && $attrs.onClick()">{{ $attrs.label }}<slot /></button>',
          },
          QIcon: { template: '<span class="q-icon"><slot /></span>' },
          QList: { template: '<ul><slot /></ul>' },
          QItem: {
            template:
              '<li class="q-item" @click="$attrs.onClick && $attrs.onClick()"><slot /></li>',
          },
          QItemSection: { template: '<div><slot /></div>' },
          QItemLabel: { template: '<span><slot /></span>' },
          QBadge: { template: '<span class="q-badge">{{ $attrs.label }}</span>' },
          QSpinner: { template: '<div class="q-spinner" />' },
          QExpansionItem: { template: '<div class="q-expansion-item"><slot /></div>' },
          QSpace: { template: '<span />' },
          SessionNotepad: { template: '<div class="session-notepad" />' },
        },
      },
    })
  }

  it('mounts without errors', async () => {
    const wrapper = mountActiveSessionView()
    await flushPromises()
    expect(wrapper.vm).toBeDefined()
  })

  describe('status bar', () => {
    it('shows session duration when active session is present', async () => {
      const { useSessionStore } = await import('@/stores/session')
      const sessionStore = useSessionStore()
      sessionStore.setActiveSession(mockSession)

      const wrapper = mountActiveSessionView()
      await flushPromises()

      // Status bar should be visible (no first run wizard)
      expect(wrapper.find('.status-bar').exists()).toBe(true)
    })

    it('shows the schedule icon in status bar', async () => {
      const { useSessionStore } = await import('@/stores/session')
      const sessionStore = useSessionStore()
      sessionStore.setActiveSession(mockSession)

      const wrapper = mountActiveSessionView()
      await flushPromises()

      // Should display time-related content in status bar
      expect(wrapper.find('.q-card').exists()).toBe(true)
    })

    it('hides status bar when first-run wizard is shown', async () => {
      const { ref } = await import('vue')
      const showFirstRunWizard = ref(true)

      const wrapper = mountActiveSessionView({
        showFirstRunWizard,
      })
      await flushPromises()

      expect(wrapper.find('.status-bar').exists()).toBe(false)
    })

    it('shows QA Mode badge when not capturing', async () => {
      const { useSessionStore } = await import('@/stores/session')
      const sessionStore = useSessionStore()
      sessionStore.setActiveSession(mockSession)

      const wrapper = mountActiveSessionView()
      await flushPromises()

      expect(wrapper.text()).toContain('QA Mode')
    })

    it('shows capturing badge text when bug capture is active', async () => {
      const { useSessionStore } = await import('@/stores/session')
      const { useBugStore } = await import('@/stores/bug')
      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      sessionStore.setActiveSession(mockSession)
      bugStore.activeBug = mockBug

      const wrapper = mountActiveSessionView()
      await flushPromises()

      // Badge should reflect the capture state with bug ID
      expect(wrapper.text()).toContain('BUG-001')
    })
  })

  describe('bug list section', () => {
    it('shows "Captured Bugs" heading', async () => {
      const wrapper = mountActiveSessionView()
      await flushPromises()
      expect(wrapper.text()).toContain('Captured Bugs')
    })

    it('shows empty state when no bugs exist', async () => {
      const wrapper = mountActiveSessionView()
      await flushPromises()
      expect(wrapper.text()).toContain('No bugs captured yet')
    })

    it('shows empty state hint text', async () => {
      const wrapper = mountActiveSessionView()
      await flushPromises()
      expect(wrapper.text()).toContain('Press Print Screen to start capturing a bug')
    })

    it('shows loading spinner while bugs are loading', async () => {
      const { useBugStore } = await import('@/stores/bug')
      const bugStore = useBugStore()
      bugStore.loading = true

      const wrapper = mountActiveSessionView()
      await flushPromises()

      expect(wrapper.find('.q-spinner').exists()).toBe(true)
    })

    it('shows bug list when bugs are available', async () => {
      const { useSessionStore } = await import('@/stores/session')
      const tauriApi = await import('@/api/tauri')
      const sessionStore = useSessionStore()

      sessionStore.setActiveSession(mockSession)
      vi.mocked(tauriApi.getBugsBySession).mockResolvedValue([mockBug])

      const wrapper = mountActiveSessionView()
      await flushPromises()

      expect(wrapper.find('ul').exists()).toBe(true)
    })

    it('renders bug display_id in the list', async () => {
      const { useSessionStore } = await import('@/stores/session')
      const tauriApi = await import('@/api/tauri')
      const sessionStore = useSessionStore()

      sessionStore.setActiveSession(mockSession)
      vi.mocked(tauriApi.getBugsBySession).mockResolvedValue([mockBug])

      const wrapper = mountActiveSessionView()
      await flushPromises()

      expect(wrapper.text()).toContain('BUG-001')
    })

    it('renders notes preview when bug has notes', async () => {
      const { useSessionStore } = await import('@/stores/session')
      const tauriApi = await import('@/api/tauri')
      const sessionStore = useSessionStore()

      sessionStore.setActiveSession(mockSession)
      vi.mocked(tauriApi.getBugsBySession).mockResolvedValue([mockBug])

      const wrapper = mountActiveSessionView()
      await flushPromises()

      expect(wrapper.text()).toContain('Some notes')
    })

    it('navigates to bug-detail on bug click', async () => {
      const { useSessionStore } = await import('@/stores/session')
      const tauriApi = await import('@/api/tauri')
      const sessionStore = useSessionStore()

      sessionStore.setActiveSession(mockSession)
      vi.mocked(tauriApi.getBugsBySession).mockResolvedValue([mockBug])

      const wrapper = mountActiveSessionView()
      await flushPromises()

      const pushSpy = vi.spyOn(router, 'push')
      const items = wrapper.findAll('li.q-item')
      expect(items.length).toBeGreaterThan(0)
      await items[0]!.trigger('click')

      expect(pushSpy).toHaveBeenCalledWith(
        expect.objectContaining({ name: 'bug-detail', params: { id: 'bug-1' } })
      )
    })
  })

  describe('session controls', () => {
    it('renders New Bug Capture button', async () => {
      const wrapper = mountActiveSessionView()
      await flushPromises()
      expect(wrapper.text()).toContain('New Bug Capture')
    })

    it('renders End Bug Capture button', async () => {
      const wrapper = mountActiveSessionView()
      await flushPromises()
      expect(wrapper.text()).toContain('End Bug Capture')
    })

    it('renders End Session button', async () => {
      const wrapper = mountActiveSessionView()
      await flushPromises()
      expect(wrapper.text()).toContain('End Session')
    })

    it('End Bug Capture button is disabled when not capturing', async () => {
      const wrapper = mountActiveSessionView()
      await flushPromises()

      const buttons = wrapper.findAll('button.q-btn')
      const endCaptureBtn = buttons.find(b => b.text().includes('End Bug Capture'))
      expect(endCaptureBtn).toBeDefined()
      expect(endCaptureBtn!.attributes('disabled')).toBeDefined()
    })

    it('End Bug Capture button is enabled when capturing', async () => {
      const { useBugStore } = await import('@/stores/bug')
      const bugStore = useBugStore()
      bugStore.activeBug = mockBug

      const wrapper = mountActiveSessionView()
      await flushPromises()

      const buttons = wrapper.findAll('button.q-btn')
      const endCaptureBtn = buttons.find(b => b.text().includes('End Bug Capture'))
      expect(endCaptureBtn).toBeDefined()
      expect(endCaptureBtn!.attributes('disabled')).toBeUndefined()
    })

    it('shows warning notification when New Bug Capture clicked without active session', async () => {
      const wrapper = mountActiveSessionView()
      await flushPromises()

      const buttons = wrapper.findAll('button.q-btn')
      const newBugBtn = buttons.find(b => b.text().includes('New Bug Capture'))
      await newBugBtn!.trigger('click')
      await flushPromises()

      expect(mockNotify).toHaveBeenCalledWith(
        expect.objectContaining({ type: 'warning' })
      )
    })

    it('shows warning notification when End Session clicked without active session', async () => {
      const wrapper = mountActiveSessionView()
      await flushPromises()

      const buttons = wrapper.findAll('button.q-btn')
      const endSessionBtn = buttons.find(b => b.text().includes('End Session'))
      await endSessionBtn!.trigger('click')
      await flushPromises()

      expect(mockNotify).toHaveBeenCalledWith(
        expect.objectContaining({ type: 'warning' })
      )
    })

    it('calls startBugCapture when New Bug Capture clicked with active session', async () => {
      const { useSessionStore } = await import('@/stores/session')
      const { useBugStore } = await import('@/stores/bug')
      const sessionStore = useSessionStore()
      const bugStore = useBugStore()

      sessionStore.setActiveSession(mockSession)
      const startCaptureSpy = vi
        .spyOn(bugStore, 'startBugCapture')
        .mockResolvedValue(mockBug)

      const wrapper = mountActiveSessionView()
      await flushPromises()

      const buttons = wrapper.findAll('button.q-btn')
      const newBugBtn = buttons.find(b => b.text().includes('New Bug Capture'))
      await newBugBtn!.trigger('click')
      await flushPromises()

      expect(startCaptureSpy).toHaveBeenCalledWith(
        expect.objectContaining({ session_id: 'session-1' })
      )
    })

    it('navigates to session-review when End Session succeeds', async () => {
      const { useSessionStore } = await import('@/stores/session')
      const sessionStore = useSessionStore()
      sessionStore.setActiveSession(mockSession)

      const endSessionSpy = vi
        .spyOn(sessionStore, 'endSession')
        .mockResolvedValue(undefined)
      const pushSpy = vi.spyOn(router, 'push')

      const wrapper = mountActiveSessionView()
      await flushPromises()

      const buttons = wrapper.findAll('button.q-btn')
      const endSessionBtn = buttons.find(b => b.text().includes('End Session'))
      await endSessionBtn!.trigger('click')
      await flushPromises()

      expect(endSessionSpy).toHaveBeenCalledWith('session-1')
      expect(pushSpy).toHaveBeenCalledWith(
        expect.objectContaining({ name: 'session-review', params: { id: 'session-1' } })
      )
    })

    it('shows negative notification when End Session fails', async () => {
      const { useSessionStore } = await import('@/stores/session')
      const sessionStore = useSessionStore()
      sessionStore.setActiveSession(mockSession)

      vi.spyOn(sessionStore, 'endSession').mockRejectedValue(new Error('Backend error'))

      const wrapper = mountActiveSessionView()
      await flushPromises()

      const buttons = wrapper.findAll('button.q-btn')
      const endSessionBtn = buttons.find(b => b.text().includes('End Session'))
      await endSessionBtn!.trigger('click')
      await flushPromises()

      expect(mockNotify).toHaveBeenCalledWith(
        expect.objectContaining({ type: 'negative', message: 'Failed to end session' })
      )
    })
  })

  describe('session notes section', () => {
    it('renders the session notes expansion item', async () => {
      const wrapper = mountActiveSessionView()
      await flushPromises()
      expect(wrapper.find('.q-expansion-item').exists()).toBe(true)
    })

    it('renders SessionNotepad component inside expansion', async () => {
      const wrapper = mountActiveSessionView()
      await flushPromises()
      expect(wrapper.find('.session-notepad').exists()).toBe(true)
    })
  })

  describe('bug count display', () => {
    it('shows 0 bugs when no bugs captured', async () => {
      const { useSessionStore } = await import('@/stores/session')
      const sessionStore = useSessionStore()
      sessionStore.setActiveSession(mockSession)

      const wrapper = mountActiveSessionView()
      await flushPromises()

      expect(wrapper.text()).toContain('0 bugs')
    })

    it('shows correct bug count when bugs exist', async () => {
      const { useSessionStore } = await import('@/stores/session')
      const tauriApi = await import('@/api/tauri')
      const sessionStore = useSessionStore()

      sessionStore.setActiveSession(mockSession)
      const twoBugs = [mockBug, { ...mockBug, id: 'bug-2', display_id: 'BUG-002' }]
      vi.mocked(tauriApi.getBugsBySession).mockResolvedValue(twoBugs)

      const wrapper = mountActiveSessionView()
      await flushPromises()

      expect(wrapper.text()).toContain('2 bugs')
    })

    it('uses singular "bug" for exactly 1 bug', async () => {
      const { useSessionStore } = await import('@/stores/session')
      const tauriApi = await import('@/api/tauri')
      const sessionStore = useSessionStore()

      sessionStore.setActiveSession(mockSession)
      vi.mocked(tauriApi.getBugsBySession).mockResolvedValue([mockBug])

      const wrapper = mountActiveSessionView()
      await flushPromises()

      expect(wrapper.text()).toContain('1 bug')
      expect(wrapper.text()).not.toContain('1 bugs')
    })
  })
})
