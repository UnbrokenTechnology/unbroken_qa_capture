import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useSettingsStore, SETTINGS_KEYS } from '@/stores/settings'
import type { Setting } from '@/types/backend'

// Mock Tauri API
vi.mock('@/api/tauri', () => ({
  getSetting: vi.fn(),
  setSetting: vi.fn(),
  getAllSettings: vi.fn(),
  deleteSetting: vi.fn(),
}))

import * as tauri from '@/api/tauri'

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {}

  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => {
      store[key] = value
    },
    removeItem: (key: string) => {
      delete store[key]
    },
    clear: () => {
      store = {}
    },
  }
})()

Object.defineProperty(window, 'localStorage', {
  value: localStorageMock,
})

const mockSettings: Setting[] = [
  { key: SETTINGS_KEYS.HOTKEY_CAPTURE, value: 'Ctrl+Shift+C', updated_at: '2024-01-01T10:00:00Z' },
  { key: SETTINGS_KEYS.DEFAULT_SAVE_PATH, value: '/custom/path', updated_at: '2024-01-01T10:00:00Z' },
  { key: SETTINGS_KEYS.THEME, value: 'dark', updated_at: '2024-01-01T10:00:00Z' },
]

describe('Settings Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    localStorageMock.clear()
  })

  afterEach(() => {
    localStorageMock.clear()
  })

  it('should initialize with default settings', () => {
    const store = useSettingsStore()
    expect(store.settings[SETTINGS_KEYS.HOTKEY_CAPTURE]).toBe('Ctrl+Shift+B')
    expect(store.settings[SETTINGS_KEYS.THEME]).toBe('light')
    expect(store.loading).toBe(false)
    expect(store.error).toBeNull()
    expect(store.isDirty).toBe(false)
  })

  it('should have correct computed getters', () => {
    const store = useSettingsStore()
    expect(store.hotkeyCapture).toBe('Ctrl+Shift+B')
    expect(store.theme).toBe('light')
    expect(store.autoStartRecording).toBe(false)
    expect(store.captureConsole).toBe(true)
    expect(store.autoOpenAnnotation).toBe(true)
    expect(store.hasError).toBe(false)
  })

  it('should correctly parse autoOpenAnnotation boolean value', () => {
    const store = useSettingsStore()

    // Test true value
    store.settings[SETTINGS_KEYS.AUTO_OPEN_ANNOTATION] = 'true'
    expect(store.autoOpenAnnotation).toBe(true)

    // Test false value
    store.settings[SETTINGS_KEYS.AUTO_OPEN_ANNOTATION] = 'false'
    expect(store.autoOpenAnnotation).toBe(false)
  })

  describe('loadSetting', () => {
    it('should load a setting from backend', async () => {
      const store = useSettingsStore()
      vi.mocked(tauri.getSetting).mockResolvedValue('Ctrl+Shift+C')

      const value = await store.loadSetting(SETTINGS_KEYS.HOTKEY_CAPTURE)

      expect(value).toBe('Ctrl+Shift+C')
      expect(store.settings[SETTINGS_KEYS.HOTKEY_CAPTURE]).toBe('Ctrl+Shift+C')
      expect(tauri.getSetting).toHaveBeenCalledWith(SETTINGS_KEYS.HOTKEY_CAPTURE)
    })

    it('should handle null value from backend', async () => {
      const store = useSettingsStore()
      vi.mocked(tauri.getSetting).mockResolvedValue(null)

      const value = await store.loadSetting(SETTINGS_KEYS.HOTKEY_CAPTURE)

      expect(value).toBeNull()
      // Should keep default value
      expect(store.settings[SETTINGS_KEYS.HOTKEY_CAPTURE]).toBe('Ctrl+Shift+B')
    })

    it('should handle errors', async () => {
      const store = useSettingsStore()
      const error = new Error('Failed to load setting')
      vi.mocked(tauri.getSetting).mockRejectedValue(error)

      await expect(store.loadSetting(SETTINGS_KEYS.HOTKEY_CAPTURE)).rejects.toThrow(
        'Failed to load setting'
      )
      expect(store.error).toBe('Failed to load setting')
    })
  })

  describe('saveSetting', () => {
    it('should save a setting to backend', async () => {
      const store = useSettingsStore()
      store.isDirty = true
      vi.mocked(tauri.setSetting).mockResolvedValue()

      await store.saveSetting(SETTINGS_KEYS.HOTKEY_CAPTURE, 'Ctrl+Alt+B')

      expect(store.settings[SETTINGS_KEYS.HOTKEY_CAPTURE]).toBe('Ctrl+Alt+B')
      expect(store.isDirty).toBe(false)
      expect(tauri.setSetting).toHaveBeenCalledWith(SETTINGS_KEYS.HOTKEY_CAPTURE, 'Ctrl+Alt+B')
    })

    it('should handle errors', async () => {
      const store = useSettingsStore()
      const error = new Error('Failed to save setting')
      vi.mocked(tauri.setSetting).mockRejectedValue(error)

      await expect(
        store.saveSetting(SETTINGS_KEYS.HOTKEY_CAPTURE, 'Ctrl+Alt+B')
      ).rejects.toThrow('Failed to save setting')
      expect(store.error).toBe('Failed to save setting')
    })
  })

  describe('loadAllSettings', () => {
    it('should load all settings from backend', async () => {
      const store = useSettingsStore()
      vi.mocked(tauri.getAllSettings).mockResolvedValue(mockSettings)

      await store.loadAllSettings()

      expect(store.settings[SETTINGS_KEYS.HOTKEY_CAPTURE]).toBe('Ctrl+Shift+C')
      expect(store.settings[SETTINGS_KEYS.DEFAULT_SAVE_PATH]).toBe('/custom/path')
      expect(store.settings[SETTINGS_KEYS.THEME]).toBe('dark')
      expect(store.isDirty).toBe(false)
      expect(tauri.getAllSettings).toHaveBeenCalled()
    })

    it('should merge with default settings', async () => {
      const store = useSettingsStore()
      const firstSetting = mockSettings[0]
      if (firstSetting) {
        vi.mocked(tauri.getAllSettings).mockResolvedValue([firstSetting])

        await store.loadAllSettings()

        expect(store.settings[SETTINGS_KEYS.HOTKEY_CAPTURE]).toBe('Ctrl+Shift+C')
        // Should have default value for unspecified settings
        expect(store.settings[SETTINGS_KEYS.THEME]).toBe('light')
      }
    })
  })

  describe('deleteSetting', () => {
    it('should delete a setting and reset to default', async () => {
      const store = useSettingsStore()
      store.settings[SETTINGS_KEYS.HOTKEY_CAPTURE] = 'Custom+Hotkey'
      vi.mocked(tauri.deleteSetting).mockResolvedValue()

      await store.deleteSetting(SETTINGS_KEYS.HOTKEY_CAPTURE)

      expect(store.settings[SETTINGS_KEYS.HOTKEY_CAPTURE]).toBe('Ctrl+Shift+B')
      expect(tauri.deleteSetting).toHaveBeenCalledWith(SETTINGS_KEYS.HOTKEY_CAPTURE)
    })

    it('should delete non-default setting completely', async () => {
      const store = useSettingsStore()
      store.settings['custom_key'] = 'custom_value'
      vi.mocked(tauri.deleteSetting).mockResolvedValue()

      await store.deleteSetting('custom_key')

      expect(store.settings['custom_key']).toBeUndefined()
    })
  })

  describe('Local state management', () => {
    it('should set a setting locally', () => {
      const store = useSettingsStore()

      store.setSetting(SETTINGS_KEYS.THEME, 'dark')

      expect(store.settings[SETTINGS_KEYS.THEME]).toBe('dark')
      expect(store.isDirty).toBe(true)
    })

    it('should get a setting with default fallback', () => {
      const store = useSettingsStore()

      const value = store.getSetting(SETTINGS_KEYS.HOTKEY_CAPTURE)
      expect(value).toBe('Ctrl+Shift+B')

      const customValue = store.getSetting('nonexistent', 'fallback')
      expect(customValue).toBe('fallback')
    })

    it('should reset to defaults', () => {
      const store = useSettingsStore()
      store.settings[SETTINGS_KEYS.THEME] = 'dark'
      store.settings[SETTINGS_KEYS.HOTKEY_CAPTURE] = 'Custom'

      store.resetToDefaults()

      expect(store.settings[SETTINGS_KEYS.THEME]).toBe('light')
      expect(store.settings[SETTINGS_KEYS.HOTKEY_CAPTURE]).toBe('Ctrl+Shift+B')
      expect(store.isDirty).toBe(true)
    })

    it('should clear error', () => {
      const store = useSettingsStore()
      store.error = 'Test error'

      store.clearError()

      expect(store.error).toBeNull()
    })
  })

  describe('LocalStorage persistence', () => {
    it('should save to localStorage', () => {
      const store = useSettingsStore()
      store.settings[SETTINGS_KEYS.THEME] = 'dark'

      store.saveToLocalStorage()

      const stored = localStorageMock.getItem('unbroken-qa-settings')
      expect(stored).toBeTruthy()
      const parsed = JSON.parse(stored!)
      expect(parsed[SETTINGS_KEYS.THEME]).toBe('dark')
    })

    it('should load from localStorage', () => {
      const testSettings = {
        [SETTINGS_KEYS.THEME]: 'dark',
        [SETTINGS_KEYS.HOTKEY_CAPTURE]: 'Ctrl+Alt+X',
      }
      localStorageMock.setItem('unbroken-qa-settings', JSON.stringify(testSettings))

      const store = useSettingsStore()
      store.loadFromLocalStorage()

      expect(store.settings[SETTINGS_KEYS.THEME]).toBe('dark')
      expect(store.settings[SETTINGS_KEYS.HOTKEY_CAPTURE]).toBe('Ctrl+Alt+X')
    })

    it('should merge localStorage with defaults', () => {
      const testSettings = {
        [SETTINGS_KEYS.THEME]: 'dark',
      }
      localStorageMock.setItem('unbroken-qa-settings', JSON.stringify(testSettings))

      const store = useSettingsStore()
      store.loadFromLocalStorage()

      expect(store.settings[SETTINGS_KEYS.THEME]).toBe('dark')
      expect(store.settings[SETTINGS_KEYS.HOTKEY_CAPTURE]).toBe('Ctrl+Shift+B')
    })

    it('should clear localStorage', () => {
      localStorageMock.setItem('unbroken-qa-settings', '{"test":"value"}')

      const store = useSettingsStore()
      store.clearLocalStorage()

      const stored = localStorageMock.getItem('unbroken-qa-settings')
      expect(stored).toBeNull()
    })

    it('should handle invalid JSON in localStorage', () => {
      localStorageMock.setItem('unbroken-qa-settings', 'invalid json')

      const store = useSettingsStore()
      // Should not throw
      expect(() => store.loadFromLocalStorage()).not.toThrow()
      // Should still have defaults
      expect(store.settings[SETTINGS_KEYS.THEME]).toBe('light')
    })
  })

  describe('getAllSettings with malformed backend response', () => {
    it('should handle backend returning an entry with a null value', async () => {
      const store = useSettingsStore()
      // Simulate a response where value field is null (type mismatch from backend)
      const malformedSettings = [
        { key: SETTINGS_KEYS.THEME, value: null as unknown as string, updated_at: '2024-01-01T10:00:00Z' },
      ]
      vi.mocked(tauri.getAllSettings).mockResolvedValue(malformedSettings)

      await store.loadAllSettings()

      // null overrides the default — the store stores whatever backend sends
      expect(store.settings[SETTINGS_KEYS.THEME]).toBeNull()
    })

    it('should handle backend returning an empty array (no settings stored)', async () => {
      const store = useSettingsStore()
      vi.mocked(tauri.getAllSettings).mockResolvedValue([])

      await store.loadAllSettings()

      // All settings should fall back to defaults
      expect(store.settings[SETTINGS_KEYS.THEME]).toBe('light')
      expect(store.settings[SETTINGS_KEYS.HOTKEY_CAPTURE]).toBe('Ctrl+Shift+B')
      expect(store.isDirty).toBe(false)
    })

    it('should handle backend returning duplicate keys (last one wins)', async () => {
      const store = useSettingsStore()
      const duplicateSettings = [
        { key: SETTINGS_KEYS.THEME, value: 'dark', updated_at: '2024-01-01T09:00:00Z' },
        { key: SETTINGS_KEYS.THEME, value: 'solarized', updated_at: '2024-01-01T10:00:00Z' },
      ]
      vi.mocked(tauri.getAllSettings).mockResolvedValue(duplicateSettings)

      await store.loadAllSettings()

      expect(store.settings[SETTINGS_KEYS.THEME]).toBe('solarized')
    })

    it('should handle backend returning unknown keys without crashing', async () => {
      const store = useSettingsStore()
      const settingsWithUnknown = [
        { key: 'unknown_future_setting', value: 'some_value', updated_at: '2024-01-01T10:00:00Z' },
        { key: SETTINGS_KEYS.THEME, value: 'dark', updated_at: '2024-01-01T10:00:00Z' },
      ]
      vi.mocked(tauri.getAllSettings).mockResolvedValue(settingsWithUnknown)

      await expect(store.loadAllSettings()).resolves.toBeUndefined()

      expect(store.settings[SETTINGS_KEYS.THEME]).toBe('dark')
      expect(store.settings['unknown_future_setting']).toBe('some_value')
    })
  })

  describe('setSetting with network failure', () => {
    it('should set error and rethrow on network failure', async () => {
      const store = useSettingsStore()
      vi.mocked(tauri.setSetting).mockRejectedValue(new Error('Network timeout'))

      await expect(
        store.saveSetting(SETTINGS_KEYS.THEME, 'dark')
      ).rejects.toThrow('Network timeout')

      expect(store.error).toBe('Network timeout')
    })

    it('should not update local settings value when backend save fails', async () => {
      const store = useSettingsStore()
      vi.mocked(tauri.setSetting).mockRejectedValue(new Error('Save failed'))

      const originalValue = store.settings[SETTINGS_KEYS.THEME]
      await expect(
        store.saveSetting(SETTINGS_KEYS.THEME, 'dark')
      ).rejects.toThrow()

      // The store only updates the local value AFTER backend succeeds, so it should remain unchanged
      // Note: looking at the implementation, it updates local state AFTER await, so it won't be updated
      expect(store.settings[SETTINGS_KEYS.THEME]).toBe(originalValue)
    })

    it('should reset loading state after network failure', async () => {
      const store = useSettingsStore()
      vi.mocked(tauri.setSetting).mockRejectedValue(new Error('Connection refused'))

      await expect(
        store.saveSetting(SETTINGS_KEYS.HOTKEY_CAPTURE, 'Ctrl+Alt+Z')
      ).rejects.toThrow()

      expect(store.loading).toBe(false)
    })
  })

  describe('settings defaults when backend is unavailable', () => {
    it('should keep default settings when backend returns error and no localStorage', async () => {
      const store = useSettingsStore()
      vi.mocked(tauri.getAllSettings).mockRejectedValue(new Error('Backend offline'))

      await expect(store.loadAllSettings()).rejects.toThrow('Backend offline')

      // All settings remain at defaults
      expect(store.settings[SETTINGS_KEYS.THEME]).toBe('light')
      expect(store.settings[SETTINGS_KEYS.HOTKEY_CAPTURE]).toBe('Ctrl+Shift+B')
      expect(store.settings[SETTINGS_KEYS.AUTO_OPEN_ANNOTATION]).toBe('true')
    })

    it('should use localStorage values when backend is unavailable', async () => {
      const savedSettings = {
        [SETTINGS_KEYS.THEME]: 'dark',
        [SETTINGS_KEYS.HOTKEY_CAPTURE]: 'Ctrl+Alt+X',
      }
      localStorageMock.setItem('unbroken-qa-settings', JSON.stringify(savedSettings))

      const store = useSettingsStore()
      store.loadFromLocalStorage()

      vi.mocked(tauri.getAllSettings).mockRejectedValue(new Error('Backend offline'))

      await expect(store.loadAllSettings()).rejects.toThrow('Backend offline')

      // Should still have the localStorage values loaded earlier
      expect(store.settings[SETTINGS_KEYS.THEME]).toBe('dark')
      expect(store.settings[SETTINGS_KEYS.HOTKEY_CAPTURE]).toBe('Ctrl+Alt+X')
    })

    it('should return defaults from getSetting when backend has never responded', () => {
      const store = useSettingsStore()

      expect(store.getSetting(SETTINGS_KEYS.THEME)).toBe('light')
      expect(store.getSetting(SETTINGS_KEYS.HOTKEY_CAPTURE)).toBe('Ctrl+Shift+B')
      expect(store.getSetting('nonexistent_key', 'my-fallback')).toBe('my-fallback')
    })

  })

  describe('initialize', () => {
    it('should load from localStorage and then backend', async () => {
      const localSettings = { [SETTINGS_KEYS.THEME]: 'dark' }
      localStorageMock.setItem('unbroken-qa-settings', JSON.stringify(localSettings))

      vi.mocked(tauri.getAllSettings).mockResolvedValue([
        {
          key: SETTINGS_KEYS.THEME,
          value: 'light',
          updated_at: '2024-01-01T10:00:00Z',
        },
      ])

      const store = useSettingsStore()
      store.initialize()

      // Should have localStorage value initially
      expect(store.settings[SETTINGS_KEYS.THEME]).toBe('dark')

      // Wait for backend load
      await vi.waitFor(() => {
        expect(tauri.getAllSettings).toHaveBeenCalled()
      })
    })

    it('should fallback to localStorage if backend fails', async () => {
      const localSettings = { [SETTINGS_KEYS.THEME]: 'dark' }
      localStorageMock.setItem('unbroken-qa-settings', JSON.stringify(localSettings))

      vi.mocked(tauri.getAllSettings).mockRejectedValue(new Error('Backend error'))

      const store = useSettingsStore()
      store.initialize()

      // Should still have localStorage value
      expect(store.settings[SETTINGS_KEYS.THEME]).toBe('dark')

      await vi.waitFor(() => {
        expect(tauri.getAllSettings).toHaveBeenCalled()
      })
    })
  })
})
