import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createRouter, createMemoryHistory } from 'vue-router'
import { Quasar } from 'quasar'
import Settings from '@/views/Settings.vue'
import * as tauri from '@/api/tauri'

// Create a mock notify and dialog
const mockNotify = vi.fn()
const mockDialog = vi.fn(() => ({
  onOk: vi.fn()
}))

// Mock useQuasar
vi.mock('quasar', async () => {
  const actual = await vi.importActual('quasar')
  return {
    ...actual,
    useQuasar: () => ({
      notify: mockNotify,
      dialog: mockDialog
    })
  }
})

// Mock Tauri API
vi.mock('@/api/tauri', () => ({
  getSetting: vi.fn(),
  setSetting: vi.fn(),
  getAllSettings: vi.fn(),
  deleteSetting: vi.fn(),
  getClaudeStatus: vi.fn().mockResolvedValue({ status: 'notInstalled', message: 'No credentials' }),
  refreshClaudeStatus: vi.fn().mockResolvedValue({ status: 'notInstalled', message: 'No credentials' }),
  ticketingGetCredentials: vi.fn().mockResolvedValue(null),
  ticketingCheckConnection: vi.fn().mockResolvedValue({ connected: false }),
  ticketingSaveCredentials: vi.fn().mockResolvedValue(undefined),
  ticketingFetchTeams: vi.fn().mockResolvedValue([]),
  getLinearProfileDefaults: vi.fn().mockResolvedValue(null),
  listProfiles: vi.fn().mockResolvedValue([]),
  getActiveProfileId: vi.fn().mockResolvedValue(null),
  hasCompletedSetup: vi.fn().mockResolvedValue(true),
}))

// Mock Tauri dialog plugin
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn()
}))

// Mock Tauri core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue('1.0.0'),
  convertFileSrc: vi.fn((path: string) => `asset://localhost/${path}`),
}))

// Mock Tauri shell
vi.mock('@tauri-apps/plugin-shell', () => ({
  open: vi.fn().mockResolvedValue(undefined)
}))

describe('Settings View', () => {
  let pinia: ReturnType<typeof createPinia>
  let router: ReturnType<typeof createRouter>
  let mockOpen: ReturnType<typeof vi.fn>

  beforeEach(async () => {
    pinia = createPinia()
    setActivePinia(pinia)

    // Get the mocked open function
    const dialogModule = await import('@tauri-apps/plugin-dialog')
    mockOpen = vi.mocked(dialogModule.open)

    router = createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: '/settings', name: 'settings', component: Settings }
      ]
    })

    // Reset mocks
    vi.clearAllMocks()

    // Setup default mock implementations
    vi.mocked(tauri.getAllSettings).mockResolvedValue([
      { key: 'default_save_path', value: '/sessions', updated_at: '2024-01-01T00:00:00Z' },
      { key: 'hotkey_toggle_session', value: 'Ctrl+Shift+Q', updated_at: '2024-01-01T00:00:00Z' }
    ])
    vi.mocked(tauri.setSetting).mockResolvedValue()
  })

  it('component is defined and mounts', async () => {
    const wrapper = mount(Settings, {
      global: {
        plugins: [pinia, router, Quasar],
        stubs: {
          QPage: false, // Don't stub QPage warnings are expected in test environment
        }
      }
    })

    await flushPromises()

    // Verify component mounted
    expect(wrapper.vm).toBeDefined()
    // Verify local settings state exists
    expect((wrapper.vm as any).localSettings).toBeDefined()
  })

  it('loads settings on mount', async () => {
    mount(Settings, {
      global: {
        plugins: [pinia, router, Quasar]
      }
    })

    await flushPromises()

    expect(tauri.getAllSettings).toHaveBeenCalled()
  })

  describe('General Section', () => {
    it('displays sessions root folder setting', async () => {
      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any
      expect(vm.localSettings).toHaveProperty('default_save_path')
    })

    it('has launch on startup toggle', async () => {
      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any
      expect(vm.localSettings).toHaveProperty('launch_on_startup')
      expect(typeof vm.localSettings.launch_on_startup).toBe('boolean')
    })

    it('has minimize to tray toggle', async () => {
      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any
      expect(vm.localSettings).toHaveProperty('minimize_to_tray')
      expect(typeof vm.localSettings.minimize_to_tray).toBe('boolean')
    })
  })

  describe('Hotkeys Section', () => {
    it('has all 5 hotkey settings', async () => {
      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any
      expect(vm.localSettings).toHaveProperty('hotkey_toggle_session')
      expect(vm.localSettings).toHaveProperty('hotkey_new_bug')
      expect(vm.localSettings).toHaveProperty('hotkey_end_bug')
      expect(vm.localSettings).toHaveProperty('hotkey_quick_notepad')
      expect(vm.localSettings).toHaveProperty('hotkey_session_notepad')
    })

    it('has recordHotkey function', async () => {
      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any
      expect(typeof vm.recordHotkey).toBe('function')
    })
  })

  describe('Annotation Section', () => {
    it('has annotation settings', async () => {
      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any
      expect(vm.localSettings).toHaveProperty('annotation_auto_open')
      expect(vm.localSettings).toHaveProperty('annotation_save_mode')
      expect(vm.localSettings).toHaveProperty('annotation_default_color')
      expect(vm.localSettings).toHaveProperty('annotation_stroke_width')
    })
  })

  describe('AI Section', () => {
    it('has AI settings', async () => {
      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any
      expect(vm.localSettings).toHaveProperty('ai_auto_generate')
      expect(vm).toHaveProperty('claudeStatus')
    })
  })

  describe('Ticketing Section', () => {
    it('has ticketing settings', async () => {
      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any
      expect(vm.localSettings).toHaveProperty('ticketing_provider')
      expect(vm.localSettings).toHaveProperty('default_bug_type')
      expect(vm.localSettings).toHaveProperty('linear_config_path')
    })
  })

  describe('About Section', () => {
    it('displays app version', async () => {
      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any
      expect(vm).toHaveProperty('appVersion')
    })
  })

  describe('Save Settings', () => {
    it('saves all settings when save button is clicked', async () => {
      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any

      // Modify settings
      vm.localSettings.default_save_path = '/test/sessions'
      vm.localSettings.hotkey_toggle_session = 'Ctrl+Shift+T'

      await wrapper.vm.$nextTick()

      // Call save
      await vm.saveSettings()

      // Verify settings were saved
      expect(tauri.setSetting).toHaveBeenCalled()

      // Verify success notification
      expect(mockNotify).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'positive',
          message: 'Settings saved successfully'
        })
      )
    })

    it('navigates back after successful save', async () => {
      const mockBack = vi.fn()
      const localRouter = createRouter({
        history: createMemoryHistory(),
        routes: [
          { path: '/', component: { template: '<div/>' } },
          { path: '/settings', name: 'settings', component: Settings }
        ]
      })
      vi.spyOn(localRouter, 'back').mockImplementation(mockBack)

      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, localRouter, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any
      await vm.saveSettings()

      expect(mockBack).toHaveBeenCalled()
    })

    it('does not navigate back when save fails', async () => {
      vi.mocked(tauri.setSetting).mockRejectedValueOnce(new Error('Database error'))

      const mockBack = vi.fn()
      const localRouter = createRouter({
        history: createMemoryHistory(),
        routes: [
          { path: '/', component: { template: '<div/>' } },
          { path: '/settings', name: 'settings', component: Settings }
        ]
      })
      vi.spyOn(localRouter, 'back').mockImplementation(mockBack)

      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, localRouter, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any
      await vm.saveSettings()

      expect(mockBack).not.toHaveBeenCalled()
    })

    it('shows error when save fails', async () => {
      vi.mocked(tauri.setSetting).mockRejectedValueOnce(new Error('Database error'))

      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any

      // Try to save
      await vm.saveSettings()

      // Verify error notification
      expect(mockNotify).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'negative',
          message: 'Failed to save settings'
        })
      )
    })
  })

  describe('Navigation', () => {
    it('has goBack function that calls router.back', async () => {
      const mockBack = vi.fn()
      const localRouter = createRouter({
        history: createMemoryHistory(),
        routes: [
          { path: '/', component: { template: '<div/>' } },
          { path: '/settings', name: 'settings', component: Settings }
        ]
      })
      vi.spyOn(localRouter, 'back').mockImplementation(mockBack)

      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, localRouter, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any
      vm.goBack()

      expect(mockBack).toHaveBeenCalled()
    })

    it('navigates back when Escape key is pressed', async () => {
      const mockBack = vi.fn()
      const localRouter = createRouter({
        history: createMemoryHistory(),
        routes: [
          { path: '/', component: { template: '<div/>' } },
          { path: '/settings', name: 'settings', component: Settings }
        ]
      })
      vi.spyOn(localRouter, 'back').mockImplementation(mockBack)

      mount(Settings, {
        global: {
          plugins: [pinia, localRouter, Quasar]
        }
      })

      await flushPromises()

      document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))

      expect(mockBack).toHaveBeenCalled()
    })

    it('back button is accessible via goBack function', async () => {
      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      // Verify goBack function exists and is callable (back button functionality)
      const vm = wrapper.vm as any
      expect(typeof vm.goBack).toBe('function')
    })
  })

  describe('Path Selection', () => {
    it('updates sessions root when folder is selected', async () => {
      mockOpen.mockResolvedValue('/selected/path')

      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any

      // Select sessions root
      await vm.selectSessionsRoot()

      // Verify path was updated
      expect(vm.localSettings.default_save_path).toBe('/selected/path')
    })

    it('handles file selection cancellation', async () => {
      mockOpen.mockResolvedValue(null)

      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any
      const originalPath = vm.localSettings.default_save_path

      // Try to select path (user cancels)
      await vm.selectSessionsRoot()

      // Verify path was not changed
      expect(vm.localSettings.default_save_path).toBe(originalPath)
    })
  })
})
