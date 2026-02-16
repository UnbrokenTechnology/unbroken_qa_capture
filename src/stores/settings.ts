import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import type { Setting } from '../types/backend'
import * as tauri from '../api/tauri'

// Settings keys
export const SETTINGS_KEYS = {
  HOTKEY_CAPTURE: 'hotkey_capture',
  HOTKEY_START_SESSION: 'hotkey_start_session',
  HOTKEY_END_SESSION: 'hotkey_end_session',
  DEFAULT_SAVE_PATH: 'default_save_path',
  CUSTOM_TEMPLATE_PATH: 'custom_template_path',
  AUTO_START_RECORDING: 'auto_start_recording',
  CAPTURE_CONSOLE: 'capture_console',
  AI_ENABLED: 'ai_enabled',
  THEME: 'theme',
} as const

export type SettingsKey = typeof SETTINGS_KEYS[keyof typeof SETTINGS_KEYS]

// Default settings values
const DEFAULT_SETTINGS: Record<SettingsKey, string> = {
  [SETTINGS_KEYS.HOTKEY_CAPTURE]: 'Ctrl+Shift+B',
  [SETTINGS_KEYS.HOTKEY_START_SESSION]: 'Ctrl+Shift+S',
  [SETTINGS_KEYS.HOTKEY_END_SESSION]: 'Ctrl+Shift+E',
  [SETTINGS_KEYS.DEFAULT_SAVE_PATH]: '',
  [SETTINGS_KEYS.CUSTOM_TEMPLATE_PATH]: '',
  [SETTINGS_KEYS.AUTO_START_RECORDING]: 'false',
  [SETTINGS_KEYS.CAPTURE_CONSOLE]: 'true',
  [SETTINGS_KEYS.AI_ENABLED]: 'false',
  [SETTINGS_KEYS.THEME]: 'light',
}

export const useSettingsStore = defineStore('settings', () => {
  // ============================================================================
  // State
  // ============================================================================

  const settings = ref<Record<string, string>>({ ...DEFAULT_SETTINGS })
  const loading = ref(false)
  const error = ref<string | null>(null)
  const isDirty = ref(false)

  // ============================================================================
  // Getters
  // ============================================================================

  const hasError = computed(() => error.value !== null)

  // Typed getters for specific settings
  const hotkeyCapture = computed(() => settings.value[SETTINGS_KEYS.HOTKEY_CAPTURE])
  const hotkeyStartSession = computed(() => settings.value[SETTINGS_KEYS.HOTKEY_START_SESSION])
  const hotkeyEndSession = computed(() => settings.value[SETTINGS_KEYS.HOTKEY_END_SESSION])
  const defaultSavePath = computed(() => settings.value[SETTINGS_KEYS.DEFAULT_SAVE_PATH])
  const customTemplatePath = computed(() => settings.value[SETTINGS_KEYS.CUSTOM_TEMPLATE_PATH])
  const autoStartRecording = computed(() => settings.value[SETTINGS_KEYS.AUTO_START_RECORDING] === 'true')
  const captureConsole = computed(() => settings.value[SETTINGS_KEYS.CAPTURE_CONSOLE] === 'true')
  const aiEnabled = computed(() => settings.value[SETTINGS_KEYS.AI_ENABLED] === 'true')
  const theme = computed(() => settings.value[SETTINGS_KEYS.THEME])

  // ============================================================================
  // Actions - Backend Operations
  // ============================================================================

  async function loadSetting(key: string): Promise<string | null> {
    loading.value = true
    error.value = null
    try {
      const value = await tauri.getSetting(key)
      if (value !== null) {
        settings.value[key] = value
      }
      return value
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function saveSetting(key: string, value: string): Promise<void> {
    loading.value = true
    error.value = null
    try {
      await tauri.setSetting(key, value)
      settings.value[key] = value
      isDirty.value = false
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function loadAllSettings(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      const backendSettings = await tauri.getAllSettings()
      const settingsMap: Record<string, string> = { ...DEFAULT_SETTINGS }
      backendSettings.forEach((setting: Setting) => {
        settingsMap[setting.key] = setting.value
      })
      settings.value = settingsMap
      isDirty.value = false
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function deleteSetting(key: string): Promise<void> {
    loading.value = true
    error.value = null
    try {
      await tauri.deleteSetting(key)
      // Reset to default value
      if (key in DEFAULT_SETTINGS) {
        settings.value[key] = DEFAULT_SETTINGS[key as SettingsKey]
      } else {
        delete settings.value[key]
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  // ============================================================================
  // Actions - Local State Management
  // ============================================================================

  function setSetting(key: string, value: string): void {
    settings.value[key] = value
    isDirty.value = true
    // Persist to localStorage
    saveToLocalStorage()
  }

  function getSetting(key: string, defaultValue?: string): string {
    return settings.value[key] ?? defaultValue ?? DEFAULT_SETTINGS[key as SettingsKey] ?? ''
  }

  function resetToDefaults(): void {
    settings.value = { ...DEFAULT_SETTINGS }
    isDirty.value = true
    saveToLocalStorage()
  }

  function clearError(): void {
    error.value = null
  }

  // ============================================================================
  // Local Storage Persistence
  // ============================================================================

  const STORAGE_KEY = 'unbroken-qa-settings'

  function saveToLocalStorage(): void {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(settings.value))
    } catch (err) {
      console.error('Failed to save settings to localStorage:', err)
    }
  }

  function loadFromLocalStorage(): void {
    try {
      const stored = localStorage.getItem(STORAGE_KEY)
      if (stored) {
        const parsed = JSON.parse(stored)
        settings.value = { ...DEFAULT_SETTINGS, ...parsed }
      }
    } catch (err) {
      console.error('Failed to load settings from localStorage:', err)
    }
  }

  function clearLocalStorage(): void {
    try {
      localStorage.removeItem(STORAGE_KEY)
    } catch (err) {
      console.error('Failed to clear settings from localStorage:', err)
    }
  }

  // ============================================================================
  // Initialization
  // ============================================================================

  function initialize(): void {
    // Load from localStorage first (for offline/fallback)
    loadFromLocalStorage()

    // Then try to load from backend (will override localStorage if available)
    loadAllSettings().catch((err) => {
      console.warn('Failed to load settings from backend, using localStorage:', err)
    })
  }

  // Watch for changes and auto-save to localStorage
  watch(
    () => settings.value,
    () => {
      saveToLocalStorage()
    },
    { deep: true }
  )

  // ============================================================================
  // Store Return
  // ============================================================================

  return {
    // State
    settings,
    loading,
    error,
    isDirty,

    // Getters
    hasError,
    hotkeyCapture,
    hotkeyStartSession,
    hotkeyEndSession,
    defaultSavePath,
    customTemplatePath,
    autoStartRecording,
    captureConsole,
    aiEnabled,
    theme,

    // Actions - Backend
    loadSetting,
    saveSetting,
    loadAllSettings,
    deleteSetting,

    // Actions - Local
    setSetting,
    getSetting,
    resetToDefaults,
    clearError,

    // Actions - Storage
    saveToLocalStorage,
    loadFromLocalStorage,
    clearLocalStorage,

    // Actions - Lifecycle
    initialize,
  }
})
