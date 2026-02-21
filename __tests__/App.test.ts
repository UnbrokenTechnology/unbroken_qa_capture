import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { Quasar } from 'quasar'
import { createRouter, createMemoryHistory } from 'vue-router'
import App from '@/App.vue'
import SessionToolbar from '@/components/SessionToolbar.vue'
import FirstRunWizard from '@/components/FirstRunWizard.vue'
import * as tauri from '@/api/tauri'
import { invoke } from '@tauri-apps/api/core'
import { useSessionStore } from '@/stores/session'

// Mock Tauri API
vi.mock('@/api/tauri', () => ({
  hasCompletedSetup: vi.fn(),
  getClaudeStatus: vi.fn(),
  markSetupComplete: vi.fn(),
  getSetting: vi.fn(),
  setSetting: vi.fn(),
  getAllSettings: vi.fn(),
  deleteSetting: vi.fn(),
  getActiveSession: vi.fn(),
  resumeSession: vi.fn(),
  updateTrayIcon: vi.fn(),
  updateTrayMenu: vi.fn(),
  updateTrayTooltip: vi.fn(),
  openSessionNotesWindow: vi.fn(),
  openSessionStatusWindow: vi.fn(),
  closeSessionStatusWindow: vi.fn(),
}))

// Mock Tauri core invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  convertFileSrc: vi.fn((path: string) => `asset://localhost/${path}`),
}))

// Mock Tauri event API
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}))

// Mock Tauri window API
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => ({
    setAlwaysOnTop: vi.fn(),
  })),
}))

// Mock Tauri dialog API
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}))

// Mock useQuasar
const mockNotify = vi.fn()
const mockDialog = vi.fn()
vi.mock('quasar', async () => {
  const actual = await vi.importActual('quasar')
  return {
    ...actual,
    useQuasar: () => ({
      notify: mockNotify,
      dialog: mockDialog,
      screen: {
        lt: {
          md: false
        }
      }
    })
  }
})

describe('App.vue', () => {
  let pinia: ReturnType<typeof createPinia>
  let router: ReturnType<typeof createRouter>

  beforeEach(async () => {
    pinia = createPinia()
    setActivePinia(pinia)

    // Create a minimal router
    router = createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: '/', name: 'home', component: { template: '<div>Home</div>' } },
        { path: '/active-session', name: 'active-session', component: { template: '<div>Active Session</div>' } },
        { path: '/settings', name: 'settings', component: { template: '<div>Settings</div>' } },
        { path: '/bug/:id', name: 'bug-detail', component: { template: '<div>Bug Detail</div>' }, props: true },
        { path: '/session-review', name: 'session-review', component: { template: '<div>Session Review</div>' } },
      ],
    })

    vi.clearAllMocks()

    // Default invoke mock (for tray and end_session calls)
    vi.mocked(invoke).mockResolvedValue(undefined)

    // Setup default mocks
    vi.mocked(tauri.hasCompletedSetup).mockResolvedValue(true)
    vi.mocked(tauri.getSetting).mockResolvedValue(null)
    vi.mocked(tauri.setSetting).mockResolvedValue(undefined)
    vi.mocked(tauri.getAllSettings).mockResolvedValue([])
    vi.mocked(tauri.deleteSetting).mockResolvedValue(undefined)
    vi.mocked(tauri.getActiveSession).mockResolvedValue(null)
    vi.mocked(tauri.resumeSession).mockResolvedValue({
      id: 'session-1',
      status: 'active',
      started_at: new Date().toISOString(),
      ended_at: null,
      folder_path: '/tmp/session-1',
      display_id: 'S-1',
    } as any)
    vi.mocked(tauri.updateTrayIcon).mockResolvedValue(undefined)
    vi.mocked(tauri.updateTrayMenu).mockResolvedValue(undefined)
    vi.mocked(tauri.updateTrayTooltip).mockResolvedValue(undefined)
    // Default: dialog does not open (no active session case)
    mockDialog.mockReturnValue({ onOk: vi.fn().mockReturnThis(), onCancel: vi.fn().mockReturnThis() })
  })

  const mountComponent = () => {
    return mount(App, {
      global: {
        plugins: [
          pinia,
          router,
          [Quasar, {
            plugins: {}
          }]
        ],
        stubs: {
          // Don't stub the components we want to test
          SessionToolbar: false,
          FirstRunWizard: false,
        }
      },
      attachTo: document.body,
    })
  }

  describe('SessionToolbar Visibility', () => {
    it('should show SessionToolbar when setup is complete', async () => {
      vi.mocked(tauri.hasCompletedSetup).mockResolvedValue(true)

      const wrapper = mountComponent()
      await flushPromises()

      // SessionToolbar component should be present in the component tree
      const toolbar = wrapper.findComponent(SessionToolbar)
      expect(toolbar.exists()).toBe(true)
    })

    it('should hide SessionToolbar during first-run wizard', async () => {
      vi.mocked(tauri.hasCompletedSetup).mockResolvedValue(false)

      const wrapper = mountComponent()
      await flushPromises()

      // Wait for the wizard to potentially show
      await new Promise(resolve => setTimeout(resolve, 100))
      await flushPromises()

      // FirstRunWizard should be shown
      const wizard = wrapper.findComponent(FirstRunWizard)
      expect(wizard.exists()).toBe(true)
      expect(wizard.props('modelValue')).toBe(true)

      // SessionToolbar should NOT be rendered in the DOM
      // even though the component tree includes it
      // Check the actual rendered HTML
      const html = wrapper.html()
      expect(html).not.toContain('session-toolbar')
    })

    it('should show SessionToolbar after wizard completes', async () => {
      vi.mocked(tauri.hasCompletedSetup).mockResolvedValue(false)

      const wrapper = mountComponent()
      await flushPromises()

      // Wait for the wizard to show
      await new Promise(resolve => setTimeout(resolve, 100))
      await flushPromises()

      // Verify wizard is shown
      let wizard = wrapper.findComponent(FirstRunWizard)
      expect(wizard.props('modelValue')).toBe(true)

      // Simulate wizard completion by updating the modelValue to false
      await wizard.vm.$emit('update:modelValue', false)
      await wrapper.vm.$nextTick()
      await flushPromises()

      // The showFirstRunWizard ref should be false now
      // So SessionToolbar v-if should pass
      // Note: SessionToolbar component should exist in the tree now
      const toolbar = wrapper.findComponent(SessionToolbar)
      expect(toolbar.exists()).toBe(true)
    })
  })

  describe('First-Run Setup Flow', () => {
    it('should not show wizard when setup is already complete', async () => {
      vi.mocked(tauri.hasCompletedSetup).mockResolvedValue(true)

      const wrapper = mountComponent()
      await flushPromises()

      const wizard = wrapper.findComponent(FirstRunWizard)
      expect(wizard.exists()).toBe(true)
      expect(wizard.props('modelValue')).toBe(false)
    })

    it('should show wizard when setup is not complete', async () => {
      vi.mocked(tauri.hasCompletedSetup).mockResolvedValue(false)

      const wrapper = mountComponent()
      await flushPromises()

      // Wait a bit for the wizard to show
      await new Promise(resolve => setTimeout(resolve, 100))
      await flushPromises()

      const wizard = wrapper.findComponent(FirstRunWizard)
      expect(wizard.exists()).toBe(true)
      expect(wizard.props('modelValue')).toBe(true)
    })
  })

  describe('Crash Recovery Dialog', () => {
    const activeSessionFixture = {
      id: 'session-crash-1',
      status: 'active' as const,
      started_at: '2026-02-17T10:00:00.000Z',
      ended_at: null,
      folder_path: '/tmp/session-crash-1',
      display_id: 'S-CRASH-1',
    }

    it('should not show recovery dialog when no active session exists', async () => {
      vi.mocked(tauri.getActiveSession).mockResolvedValue(null)

      mountComponent()
      await flushPromises()

      expect(mockDialog).not.toHaveBeenCalled()
    })

    it('should show recovery dialog when an active session is found on startup', async () => {
      vi.mocked(tauri.getActiveSession).mockResolvedValue(activeSessionFixture as any)

      mockDialog.mockReturnValue({
        onOk: vi.fn().mockReturnThis(),
        onCancel: vi.fn().mockReturnThis(),
      })

      mountComponent()
      await flushPromises()

      expect(mockDialog).toHaveBeenCalledWith(
        expect.objectContaining({
          title: 'Session Recovery',
          persistent: true,
        })
      )
    })

    it('should call resumeSession and navigate to active-session when user chooses Resume', async () => {
      vi.mocked(tauri.getActiveSession).mockResolvedValue(activeSessionFixture as any)

      let capturedOnOk: (() => Promise<void>) | undefined
      mockDialog.mockReturnValue({
        onOk: vi.fn().mockImplementation((cb: () => Promise<void>) => {
          capturedOnOk = cb
          return { onCancel: vi.fn().mockReturnThis() }
        }),
        onCancel: vi.fn().mockReturnThis(),
      })

      mountComponent()
      await flushPromises()

      // Trigger the onOk callback (user clicked "Resume Session")
      expect(capturedOnOk).toBeDefined()
      await capturedOnOk!()
      await flushPromises()

      expect(tauri.resumeSession).toHaveBeenCalledWith('session-crash-1')
      expect(router.currentRoute.value.name).toBe('active-session')
    })

    it('should call endSession and stay on home when user chooses End Session', async () => {
      vi.mocked(tauri.getActiveSession).mockResolvedValue(activeSessionFixture as any)

      let capturedOnCancel: (() => Promise<void>) | undefined
      mockDialog.mockReturnValue({
        onOk: vi.fn().mockReturnValue({ onCancel: vi.fn().mockImplementation((cb: () => Promise<void>) => { capturedOnCancel = cb; return {} }) }),
        onCancel: vi.fn().mockImplementation((cb: () => Promise<void>) => { capturedOnCancel = cb; return {} }),
      })

      mountComponent()
      await flushPromises()

      expect(capturedOnCancel).toBeDefined()
      await capturedOnCancel!()
      await flushPromises()

      // Should remain on home route (not navigate to active-session)
      expect(router.currentRoute.value.name).toBe('home')
    })
  })

  describe('Home Navigation Button', () => {
    it('should render a home button in the toolbar', async () => {
      vi.mocked(tauri.hasCompletedSetup).mockResolvedValue(true)

      const wrapper = mountComponent()
      await flushPromises()

      // Find the home button by its icon
      const homeBtn = wrapper.find('.q-toolbar .q-btn[aria-label="Sessions List"]')
        || wrapper.findAll('.q-toolbar .q-btn').find(btn => btn.html().includes('home'))
      expect(homeBtn).toBeTruthy()
    })

    it('should navigate to home when home button is clicked from settings', async () => {
      vi.mocked(tauri.hasCompletedSetup).mockResolvedValue(true)

      const wrapper = mountComponent()
      await flushPromises()

      // Navigate to settings first
      await router.push({ name: 'settings' })
      await flushPromises()

      expect(router.currentRoute.value.name).toBe('settings')

      // Find the home button (first button in toolbar with 'home' icon)
      const toolbarBtns = wrapper.findAll('.q-toolbar .q-btn')
      const homeBtn = toolbarBtns.find(btn => btn.html().includes('home'))
      expect(homeBtn).toBeTruthy()

      await homeBtn!.trigger('click')
      await flushPromises()

      expect(router.currentRoute.value.name).toBe('home')
    })

    it('should be disabled when already on home route', async () => {
      vi.mocked(tauri.hasCompletedSetup).mockResolvedValue(true)

      const wrapper = mountComponent()
      await flushPromises()

      // Already on home route
      expect(router.currentRoute.value.name).toBe('home')

      // Find the home button
      const toolbarBtns = wrapper.findAll('.q-toolbar .q-btn')
      const homeBtn = toolbarBtns.find(btn => btn.html().includes('home'))
      expect(homeBtn).toBeTruthy()

      // Should be disabled
      expect(homeBtn!.attributes('disabled')).toBeDefined()
    })
  })

  describe('Settings Navigation Button', () => {
    it('should render a settings button in the toolbar', async () => {
      vi.mocked(tauri.hasCompletedSetup).mockResolvedValue(true)

      const wrapper = mountComponent()
      await flushPromises()

      const toolbarBtns = wrapper.findAll('.q-toolbar .q-btn')
      const settingsBtn = toolbarBtns.find(btn => btn.html().includes('settings'))
      expect(settingsBtn).toBeTruthy()
    })

    it('should navigate to settings when settings button is clicked', async () => {
      vi.mocked(tauri.hasCompletedSetup).mockResolvedValue(true)

      const wrapper = mountComponent()
      await flushPromises()

      // Start on home
      expect(router.currentRoute.value.name).toBe('home')

      const toolbarBtns = wrapper.findAll('.q-toolbar .q-btn')
      const settingsBtn = toolbarBtns.find(btn => btn.html().includes('settings'))
      expect(settingsBtn).toBeTruthy()

      await settingsBtn!.trigger('click')
      await flushPromises()

      expect(router.currentRoute.value.name).toBe('settings')
    })

    it('should be disabled when already on settings route', async () => {
      vi.mocked(tauri.hasCompletedSetup).mockResolvedValue(true)

      const wrapper = mountComponent()
      await flushPromises()

      // Navigate to settings first
      await router.push({ name: 'settings' })
      await flushPromises()

      expect(router.currentRoute.value.name).toBe('settings')

      const toolbarBtns = wrapper.findAll('.q-toolbar .q-btn')
      const settingsBtn = toolbarBtns.find(btn => btn.html().includes('settings'))
      expect(settingsBtn).toBeTruthy()

      expect(settingsBtn!.attributes('disabled')).toBeDefined()
    })
  })

  describe('Back Navigation', () => {
    const makeSession = (id: string): import('@/types/backend').Session => ({
      id,
      status: 'active' as const,
      started_at: new Date().toISOString(),
      ended_at: null,
      folder_path: `/tmp/${id}`,
      session_notes: null,
      environment_json: null,
      original_snip_path: null,
      created_at: new Date().toISOString(),
    })

    it('should not redirect from bug-detail when session is updated (not started)', async () => {
      vi.mocked(tauri.getActiveSession).mockResolvedValue(null)

      mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()

      // Set initial session so watcher has oldId = 'session-nav-1'
      sessionStore.activeSession = makeSession('session-nav-1')
      await flushPromises()

      // Navigate to bug-detail (simulating user clicking a bug)
      await router.push({ name: 'bug-detail', params: { id: 'bug-123' } })
      await flushPromises()

      expect(router.currentRoute.value.name).toBe('bug-detail')

      // Simulate a session-updated backend event (same session id, just updated metadata)
      // Should NOT redirect because the session id hasn't changed (null→active transition already happened)
      sessionStore.activeSession = { ...makeSession('session-nav-1'), session_notes: 'updated' }
      await flushPromises()

      // User should still be on bug-detail, not redirected to active-session
      expect(router.currentRoute.value.name).toBe('bug-detail')
    })

    it('should navigate to active-session when a new session starts from home', async () => {
      vi.mocked(tauri.getActiveSession).mockResolvedValue(null)

      mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()

      // Start on home
      expect(router.currentRoute.value.name).toBe('home')

      // Simulate session starting (null → active)
      sessionStore.activeSession = makeSession('session-new-1')
      await flushPromises()

      // Should navigate to active-session
      expect(router.currentRoute.value.name).toBe('active-session')
    })

    it('should navigate home when session ends while on active-session', async () => {
      vi.mocked(tauri.getActiveSession).mockResolvedValue(null)

      mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()

      // Set up active session first (null → active)
      sessionStore.activeSession = makeSession('session-end-1')
      await flushPromises()

      // User is now on active-session
      await router.push({ name: 'active-session' })
      await flushPromises()

      // Session ends (active → null)
      sessionStore.activeSession = null
      await flushPromises()

      // Should navigate home
      expect(router.currentRoute.value.name).toBe('home')
    })

    it('should not navigate away from session-review when session ends', async () => {
      vi.mocked(tauri.getActiveSession).mockResolvedValue(null)

      mountComponent()
      await flushPromises()

      const sessionStore = useSessionStore()

      // Set up active session
      sessionStore.activeSession = makeSession('session-review-1')
      await flushPromises()

      // Navigate to session-review
      await router.push({ name: 'session-review' })
      await flushPromises()

      // Session ends
      sessionStore.activeSession = null
      await flushPromises()

      // Should stay on session-review (explicitly exempt from home redirect)
      expect(router.currentRoute.value.name).toBe('session-review')
    })
  })
})
