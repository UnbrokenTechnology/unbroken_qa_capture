import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createRouter, createMemoryHistory } from 'vue-router'
import { Quasar } from 'quasar'
import Settings from '@/views/Settings.vue'
import { useSettingsStore } from '@/stores/settings'
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
  deleteSetting: vi.fn()
}))

// Mock Tauri dialog plugin
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn()
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
      { key: 'hotkey_capture', value: 'Ctrl+Shift+B', updated_at: '2024-01-01T00:00:00Z' },
      { key: 'hotkey_start_session', value: 'Ctrl+Shift+S', updated_at: '2024-01-01T00:00:00Z' },
      { key: 'hotkey_end_session', value: 'Ctrl+Shift+E', updated_at: '2024-01-01T00:00:00Z' },
      { key: 'theme', value: 'light', updated_at: '2024-01-01T00:00:00Z' }
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

  describe('Hotkey Validation', () => {
    it('accepts valid hotkey with modifiers', async () => {
      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any

      // Test valid hotkeys
      expect(vm.isValidHotkey('Ctrl+Shift+B')).toBe(true)
      expect(vm.isValidHotkey('Alt+Shift+C')).toBe(true)
      expect(vm.isValidHotkey('Ctrl+Alt+Delete')).toBe(true)
      expect(vm.isValidHotkey('Meta+A')).toBe(true)
      expect(vm.isValidHotkey('Command+Shift+S')).toBe(true)
    })

    it('rejects invalid hotkey without modifiers', async () => {
      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any

      // Test invalid hotkeys (no modifiers)
      expect(vm.isValidHotkey('B')).toBe(false)
      expect(vm.isValidHotkey('Enter')).toBe(false)
    })

    it('rejects hotkey with only one part', async () => {
      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any

      expect(vm.isValidHotkey('InvalidKey')).toBe(false)
    })

    it('allows empty hotkey', async () => {
      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any

      // Empty hotkeys should be allowed (not yet set)
      expect(vm.isValidHotkey('')).toBe(true)
      expect(vm.isValidHotkey('   ')).toBe(true)
    })

    it('shows error for invalid hotkey format', async () => {
      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any

      // Set invalid hotkey
      vm.localSettings.hotkey_capture = 'InvalidKey'
      await wrapper.vm.$nextTick()

      // Validate
      vm.validateHotkey('hotkey_capture')

      // Check that notification was shown
      expect(mockNotify).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'warning',
          message: 'Invalid hotkey format'
        })
      )
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
      vm.localSettings.hotkey_capture = 'Ctrl+Alt+B'
      vm.localSettings.theme = 'dark'
      vm.localSettings.ai_enabled = true

      await wrapper.vm.$nextTick()

      // Call save
      await vm.saveSettings()

      // Verify all settings were saved
      expect(tauri.setSetting).toHaveBeenCalledWith('hotkey_capture', 'Ctrl+Alt+B')
      expect(tauri.setSetting).toHaveBeenCalledWith('theme', 'dark')
      expect(tauri.setSetting).toHaveBeenCalledWith('ai_enabled', 'true')

      // Verify success notification
      expect(mockNotify).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'positive',
          message: 'Settings saved successfully'
        })
      )
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

    it('disables save button when settings are invalid', async () => {
      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any

      // Set invalid hotkey
      vm.localSettings.hotkey_capture = 'InvalidKey'
      await wrapper.vm.$nextTick()

      // Check that hasValidSettings is false
      expect(vm.hasValidSettings).toBe(false)
    })
  })

  describe('Reset to Defaults', () => {
    it('resets settings to defaults when confirmed', async () => {
      const mockOnOk = vi.fn((callback) => {
        callback()
        return { onOk: mockOnOk }
      })

      mockDialog.mockReturnValue({
        onOk: mockOnOk
      })

      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any
      const settingsStore = useSettingsStore()

      // Spy on resetToDefaults
      const resetSpy = vi.spyOn(settingsStore, 'resetToDefaults')

      // Call confirmReset
      vm.confirmReset()

      // Verify dialog was shown
      expect(mockDialog).toHaveBeenCalledWith(
        expect.objectContaining({
          title: 'Reset Settings',
          message: expect.stringContaining('reset all settings')
        })
      )

      // Verify reset was called
      expect(resetSpy).toHaveBeenCalled()
    })
  })

  describe('Cancel Changes', () => {
    it('reloads settings from store when cancelled', async () => {
      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any

      // Modify local settings
      const originalValue = vm.localSettings.hotkey_capture
      vm.localSettings.hotkey_capture = 'Ctrl+Alt+X'

      // Cancel changes
      vm.cancelChanges()

      // Verify settings were reloaded
      expect(vm.localSettings.hotkey_capture).toBe(originalValue)

      // Verify notification
      expect(mockNotify).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'info',
          message: 'Changes cancelled'
        })
      )
    })
  })

  describe('Path Selection', () => {
    it('updates save path when folder is selected', async () => {
      mockOpen.mockResolvedValue('/selected/path')

      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any

      // Select save path
      await vm.selectSavePath()

      // Verify path was updated
      expect(vm.localSettings.default_save_path).toBe('/selected/path')
    })

    it('updates template path when file is selected', async () => {
      mockOpen.mockResolvedValue('/selected/template.md')

      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any

      // Select template path
      await vm.selectTemplatePath()

      // Verify path was updated
      expect(vm.localSettings.custom_template_path).toBe('/selected/template.md')
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
      await vm.selectSavePath()

      // Verify path was not changed
      expect(vm.localSettings.default_save_path).toBe(originalPath)
    })
  })

  describe('Theme Selection', () => {
    it('shows theme options', async () => {
      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any

      expect(vm.themeOptions).toEqual([
        { label: 'Light', value: 'light' },
        { label: 'Dark', value: 'dark' }
      ])
    })

    it('updates theme when changed', async () => {
      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any

      vm.localSettings.theme = 'dark'

      await vm.saveSettings()

      expect(tauri.setSetting).toHaveBeenCalledWith('theme', 'dark')
    })
  })

  describe('Toggle Options', () => {
    it('converts boolean toggles to string when saving', async () => {
      const wrapper = mount(Settings, {
        global: {
          plugins: [pinia, router, Quasar]
        }
      })

      await flushPromises()

      const vm = wrapper.vm as any

      vm.localSettings.auto_start_recording = true
      vm.localSettings.capture_console = false
      vm.localSettings.ai_enabled = true

      await vm.saveSettings()

      expect(tauri.setSetting).toHaveBeenCalledWith('auto_start_recording', 'true')
      expect(tauri.setSetting).toHaveBeenCalledWith('capture_console', 'false')
      expect(tauri.setSetting).toHaveBeenCalledWith('ai_enabled', 'true')
    })
  })
})
