import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { Quasar } from 'quasar'
import { createRouter, createMemoryHistory } from 'vue-router'
import App from '@/App.vue'
import SessionToolbar from '@/components/SessionToolbar.vue'
import FirstRunWizard from '@/components/FirstRunWizard.vue'
import * as tauri from '@/api/tauri'

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
vi.mock('quasar', async () => {
  const actual = await vi.importActual('quasar')
  return {
    ...actual,
    useQuasar: () => ({
      notify: mockNotify,
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
      ],
    })

    vi.clearAllMocks()

    // Setup default mocks
    vi.mocked(tauri.hasCompletedSetup).mockResolvedValue(true)
    vi.mocked(tauri.getSetting).mockResolvedValue(null)
    vi.mocked(tauri.setSetting).mockResolvedValue(undefined)
    vi.mocked(tauri.getAllSettings).mockResolvedValue([])
    vi.mocked(tauri.deleteSetting).mockResolvedValue(undefined)
    vi.mocked(tauri.getActiveSession).mockResolvedValue(null)
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
})
