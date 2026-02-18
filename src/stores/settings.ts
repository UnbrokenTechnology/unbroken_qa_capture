import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Setting } from '../types/backend'
import * as tauri from '../api/tauri'

// Settings keys
export const SETTINGS_KEYS = {
  // Legacy hotkey keys (deprecated, kept for migration)
  HOTKEY_CAPTURE: 'hotkey_capture',
  HOTKEY_START_SESSION: 'hotkey_start_session',
  HOTKEY_END_SESSION: 'hotkey_end_session',
  // New hotkey keys (matching hotkey.rs HotkeyAction enum)
  HOTKEY_TOGGLE_SESSION: 'hotkey_toggle_session',
  HOTKEY_START_BUG_CAPTURE: 'hotkey_start_bug_capture',
  HOTKEY_END_BUG_CAPTURE: 'hotkey_end_bug_capture',
  HOTKEY_OPEN_QUICK_NOTEPAD: 'hotkey_open_quick_notepad',
  HOTKEY_OPEN_SESSION_NOTEPAD: 'hotkey_open_session_notepad',
  DEFAULT_SAVE_PATH: 'default_save_path',
  CUSTOM_TEMPLATE_PATH: 'custom_template_path',
  AUTO_START_RECORDING: 'auto_start_recording',
  CAPTURE_CONSOLE: 'capture_console',
  AI_ENABLED: 'ai_enabled',
  THEME: 'theme',
  ANNOTATION_SAVE_MODE: 'annotation_save_mode',
  AUTO_OPEN_ANNOTATION: 'auto_open_annotation',
  SHOW_STATUS_WIDGET: 'show_status_widget',
} as const

export type SettingsKey = typeof SETTINGS_KEYS[keyof typeof SETTINGS_KEYS]

// Default settings values
const DEFAULT_SETTINGS: Record<SettingsKey, string> = {
  // Legacy hotkey defaults (deprecated)
  [SETTINGS_KEYS.HOTKEY_CAPTURE]: 'Ctrl+Shift+B',
  [SETTINGS_KEYS.HOTKEY_START_SESSION]: 'Ctrl+Shift+S',
  [SETTINGS_KEYS.HOTKEY_END_SESSION]: 'Ctrl+Shift+E',
  // New hotkey defaults (matching hotkey.rs HotkeyConfig::default())
  [SETTINGS_KEYS.HOTKEY_TOGGLE_SESSION]: 'F5',
  [SETTINGS_KEYS.HOTKEY_START_BUG_CAPTURE]: 'F7',
  [SETTINGS_KEYS.HOTKEY_END_BUG_CAPTURE]: 'F9',
  [SETTINGS_KEYS.HOTKEY_OPEN_QUICK_NOTEPAD]: 'Ctrl+Shift+N',
  [SETTINGS_KEYS.HOTKEY_OPEN_SESSION_NOTEPAD]: 'Ctrl+Shift+M',
  [SETTINGS_KEYS.DEFAULT_SAVE_PATH]: '',
  [SETTINGS_KEYS.CUSTOM_TEMPLATE_PATH]: '',
  [SETTINGS_KEYS.AUTO_START_RECORDING]: 'false',
  [SETTINGS_KEYS.CAPTURE_CONSOLE]: 'true',
  [SETTINGS_KEYS.AI_ENABLED]: 'false',
  [SETTINGS_KEYS.THEME]: 'light',
  [SETTINGS_KEYS.ANNOTATION_SAVE_MODE]: 'alongside',
  [SETTINGS_KEYS.AUTO_OPEN_ANNOTATION]: 'true',
  [SETTINGS_KEYS.SHOW_STATUS_WIDGET]: 'false',
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
  // Legacy hotkey getters (deprecated)
  const hotkeyCapture = computed(() => settings.value[SETTINGS_KEYS.HOTKEY_CAPTURE])
  const hotkeyStartSession = computed(() => settings.value[SETTINGS_KEYS.HOTKEY_START_SESSION])
  const hotkeyEndSession = computed(() => settings.value[SETTINGS_KEYS.HOTKEY_END_SESSION])
  // New hotkey getters
  const hotkeyToggleSession = computed(() => settings.value[SETTINGS_KEYS.HOTKEY_TOGGLE_SESSION])
  const hotkeyStartBugCapture = computed(() => settings.value[SETTINGS_KEYS.HOTKEY_START_BUG_CAPTURE])
  const hotkeyEndBugCapture = computed(() => settings.value[SETTINGS_KEYS.HOTKEY_END_BUG_CAPTURE])
  const hotkeyOpenQuickNotepad = computed(() => settings.value[SETTINGS_KEYS.HOTKEY_OPEN_QUICK_NOTEPAD])
  const hotkeyOpenSessionNotepad = computed(() => settings.value[SETTINGS_KEYS.HOTKEY_OPEN_SESSION_NOTEPAD])
  const defaultSavePath = computed(() => settings.value[SETTINGS_KEYS.DEFAULT_SAVE_PATH])
  const customTemplatePath = computed(() => settings.value[SETTINGS_KEYS.CUSTOM_TEMPLATE_PATH])
  const autoStartRecording = computed(() => settings.value[SETTINGS_KEYS.AUTO_START_RECORDING] === 'true')
  const captureConsole = computed(() => settings.value[SETTINGS_KEYS.CAPTURE_CONSOLE] === 'true')
  const aiEnabled = computed(() => settings.value[SETTINGS_KEYS.AI_ENABLED] === 'true')
  const theme = computed(() => settings.value[SETTINGS_KEYS.THEME])
  const annotationSaveMode = computed(() => settings.value[SETTINGS_KEYS.ANNOTATION_SAVE_MODE] as 'alongside' | 'overwrite')
  const autoOpenAnnotation = computed(() => settings.value[SETTINGS_KEYS.AUTO_OPEN_ANNOTATION] === 'true')
  const showStatusWidget = computed(() => settings.value[SETTINGS_KEYS.SHOW_STATUS_WIDGET] === 'true')

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
  }

  function getSetting(key: string, defaultValue?: string): string {
    return settings.value[key] ?? defaultValue ?? DEFAULT_SETTINGS[key as SettingsKey] ?? ''
  }

  function resetToDefaults(): void {
    settings.value = { ...DEFAULT_SETTINGS }
    isDirty.value = true
  }

  function clearError(): void {
    error.value = null
  }

  // ============================================================================
  // Initialization
  // ============================================================================

  function initialize(): void {
    loadAllSettings().catch((err) => {
      console.warn('Failed to load settings from backend, using defaults:', err)
    })
  }

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
    hotkeyToggleSession,
    hotkeyStartBugCapture,
    hotkeyEndBugCapture,
    hotkeyOpenQuickNotepad,
    hotkeyOpenSessionNotepad,
    defaultSavePath,
    customTemplatePath,
    autoStartRecording,
    captureConsole,
    aiEnabled,
    theme,
    annotationSaveMode,
    autoOpenAnnotation,
    showStatusWidget,

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

    // Actions - Lifecycle
    initialize,
  }
})
