<template>
  <q-page class="q-pa-md">
    <div
      class="settings-container"
      style="max-width: 900px; margin: 0 auto;"
    >
      <h1 class="text-h4 q-mb-md">
        Settings
      </h1>

      <!-- Loading state -->
      <q-linear-progress
        v-if="settingsStore.loading"
        indeterminate
        color="primary"
        class="q-mb-md"
      />

      <!-- Error notification -->
      <q-banner
        v-if="settingsStore.hasError"
        class="bg-negative text-white q-mb-md"
        dense
      >
        <template #avatar>
          <q-icon
            name="error"
            color="white"
          />
        </template>
        {{ settingsStore.error }}
        <template #action>
          <q-btn
            flat
            label="Dismiss"
            @click="settingsStore.clearError()"
          />
        </template>
      </q-banner>

      <!-- Hotkeys Section -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            <q-icon
              name="keyboard"
              class="q-mr-sm"
            />
            Hotkeys
          </div>

          <div class="q-gutter-md">
            <q-input
              v-model="localSettings.hotkey_capture"
              label="Capture Bug Hotkey"
              hint="Keyboard shortcut to capture a bug (e.g., Ctrl+Shift+B)"
              outlined
              :error="!isValidHotkey(localSettings.hotkey_capture)"
              error-message="Invalid hotkey format. Use format: Ctrl+Shift+Letter"
              @blur="validateHotkey('hotkey_capture')"
            >
              <template #prepend>
                <q-icon name="camera" />
              </template>
            </q-input>

            <q-input
              v-model="localSettings.hotkey_start_session"
              label="Start Session Hotkey"
              hint="Keyboard shortcut to start a session (e.g., Ctrl+Shift+S)"
              outlined
              :error="!isValidHotkey(localSettings.hotkey_start_session)"
              error-message="Invalid hotkey format. Use format: Ctrl+Shift+Letter"
              @blur="validateHotkey('hotkey_start_session')"
            >
              <template #prepend>
                <q-icon name="play_arrow" />
              </template>
            </q-input>

            <q-input
              v-model="localSettings.hotkey_end_session"
              label="End Session Hotkey"
              hint="Keyboard shortcut to end a session (e.g., Ctrl+Shift+E)"
              outlined
              :error="!isValidHotkey(localSettings.hotkey_end_session)"
              error-message="Invalid hotkey format. Use format: Ctrl+Shift+Letter"
              @blur="validateHotkey('hotkey_end_session')"
            >
              <template #prepend>
                <q-icon name="stop" />
              </template>
            </q-input>
          </div>
        </q-card-section>
      </q-card>

      <!-- Paths Section -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            <q-icon
              name="folder"
              class="q-mr-sm"
            />
            File Paths
          </div>

          <div class="q-gutter-md">
            <q-input
              v-model="localSettings.default_save_path"
              label="Default Save Path"
              hint="Default folder to save QA sessions"
              outlined
              readonly
            >
              <template #prepend>
                <q-icon name="save" />
              </template>
              <template #append>
                <q-btn
                  round
                  dense
                  flat
                  icon="folder_open"
                  @click="selectSavePath"
                >
                  <q-tooltip>Browse</q-tooltip>
                </q-btn>
              </template>
            </q-input>

            <q-input
              v-model="localSettings.custom_template_path"
              label="Custom Template Path"
              hint="Path to custom bug report template (optional)"
              outlined
              readonly
            >
              <template #prepend>
                <q-icon name="description" />
              </template>
              <template #append>
                <q-btn
                  round
                  dense
                  flat
                  icon="folder_open"
                  @click="selectTemplatePath"
                >
                  <q-tooltip>Browse</q-tooltip>
                </q-btn>
                <q-btn
                  v-if="localSettings.custom_template_path"
                  round
                  dense
                  flat
                  icon="clear"
                  @click="localSettings.custom_template_path = ''"
                >
                  <q-tooltip>Clear</q-tooltip>
                </q-btn>
              </template>
            </q-input>
          </div>
        </q-card-section>
      </q-card>

      <!-- Capture Options Section -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            <q-icon
              name="settings"
              class="q-mr-sm"
            />
            Capture Options
          </div>

          <div class="q-gutter-md">
            <q-toggle
              v-model="localSettings.auto_start_recording"
              label="Auto-start recording on session start"
              color="primary"
            />

            <q-toggle
              v-model="localSettings.capture_console"
              label="Capture console output automatically"
              color="primary"
            />

            <q-toggle
              v-model="localSettings.ai_enabled"
              label="Enable AI-powered bug descriptions"
              color="primary"
            />
          </div>
        </q-card-section>
      </q-card>

      <!-- Theme Section -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            <q-icon
              name="palette"
              class="q-mr-sm"
            />
            Appearance
          </div>

          <q-select
            v-model="localSettings.theme"
            :options="themeOptions"
            label="Theme"
            outlined
            emit-value
            map-options
          >
            <template #prepend>
              <q-icon name="brightness_6" />
            </template>
          </q-select>
        </q-card-section>
      </q-card>

      <!-- Action Buttons -->
      <div class="row q-gutter-md justify-end">
        <q-btn
          outline
          color="grey-7"
          label="Reset to Defaults"
          @click="confirmReset"
        />
        <q-btn
          outline
          color="grey-7"
          label="Cancel"
          @click="cancelChanges"
        />
        <q-btn
          color="primary"
          label="Save Settings"
          :disable="!hasValidSettings || settingsStore.loading"
          @click="saveSettings"
        />
      </div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useSettingsStore, SETTINGS_KEYS } from '@/stores/settings'
import { useQuasar } from 'quasar'
import { open } from '@tauri-apps/plugin-dialog'

const settingsStore = useSettingsStore()
const $q = useQuasar()

// Local settings state (for editing before save)
const localSettings = ref({
  hotkey_capture: '',
  hotkey_start_session: '',
  hotkey_end_session: '',
  default_save_path: '',
  custom_template_path: '',
  auto_start_recording: false,
  capture_console: false,
  ai_enabled: false,
  theme: 'light',
})

// Theme options
const themeOptions = [
  { label: 'Light', value: 'light' },
  { label: 'Dark', value: 'dark' },
]

// Hotkey validation
function isValidHotkey(hotkey: string): boolean {
  if (!hotkey || hotkey.trim() === '') return true // Allow empty for now

  // Basic validation: should contain at least one modifier and a key
  const parts = hotkey.split('+')
  if (parts.length < 2) return false

  // Check for common modifiers
  const modifiers = ['Ctrl', 'Alt', 'Shift', 'Meta', 'Cmd', 'Command', 'Super']
  const hasModifier = parts.some(part => modifiers.includes(part))

  return hasModifier
}

function validateHotkey(key: string): void {
  const value = localSettings.value[key as keyof typeof localSettings.value] as string
  if (value && !isValidHotkey(value)) {
    $q.notify({
      type: 'warning',
      message: 'Invalid hotkey format',
      caption: 'Use format like: Ctrl+Shift+B',
    })
  }
}

// Check if all settings are valid
const hasValidSettings = computed(() => {
  return (
    isValidHotkey(localSettings.value.hotkey_capture) &&
    isValidHotkey(localSettings.value.hotkey_start_session) &&
    isValidHotkey(localSettings.value.hotkey_end_session)
  )
})

// File path selection
async function selectSavePath(): Promise<void> {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select Default Save Folder',
    })
    if (selected) {
      localSettings.value.default_save_path = selected as string
    }
  } catch (err) {
    console.error('Failed to select save path:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to select folder',
    })
  }
}

async function selectTemplatePath(): Promise<void> {
  try {
    const selected = await open({
      multiple: false,
      title: 'Select Custom Template File',
      filters: [
        {
          name: 'Markdown',
          extensions: ['md', 'markdown'],
        },
      ],
    })
    if (selected) {
      localSettings.value.custom_template_path = selected as string
    }
  } catch (err) {
    console.error('Failed to select template path:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to select template file',
    })
  }
}

// Load settings from store
function loadSettings(): void {
  localSettings.value = {
    hotkey_capture: settingsStore.hotkeyCapture || '',
    hotkey_start_session: settingsStore.hotkeyStartSession || '',
    hotkey_end_session: settingsStore.hotkeyEndSession || '',
    default_save_path: settingsStore.defaultSavePath || '',
    custom_template_path: settingsStore.customTemplatePath || '',
    auto_start_recording: settingsStore.autoStartRecording,
    capture_console: settingsStore.captureConsole,
    ai_enabled: settingsStore.aiEnabled,
    theme: settingsStore.theme || 'light',
  }
}

// Save settings
async function saveSettings(): Promise<void> {
  try {
    // Save all settings to backend
    await settingsStore.saveSetting(SETTINGS_KEYS.HOTKEY_CAPTURE, localSettings.value.hotkey_capture)
    await settingsStore.saveSetting(SETTINGS_KEYS.HOTKEY_START_SESSION, localSettings.value.hotkey_start_session)
    await settingsStore.saveSetting(SETTINGS_KEYS.HOTKEY_END_SESSION, localSettings.value.hotkey_end_session)
    await settingsStore.saveSetting(SETTINGS_KEYS.DEFAULT_SAVE_PATH, localSettings.value.default_save_path)
    await settingsStore.saveSetting(SETTINGS_KEYS.CUSTOM_TEMPLATE_PATH, localSettings.value.custom_template_path)
    await settingsStore.saveSetting(SETTINGS_KEYS.AUTO_START_RECORDING, localSettings.value.auto_start_recording.toString())
    await settingsStore.saveSetting(SETTINGS_KEYS.CAPTURE_CONSOLE, localSettings.value.capture_console.toString())
    await settingsStore.saveSetting(SETTINGS_KEYS.AI_ENABLED, localSettings.value.ai_enabled.toString())
    await settingsStore.saveSetting(SETTINGS_KEYS.THEME, localSettings.value.theme)

    $q.notify({
      type: 'positive',
      message: 'Settings saved successfully',
      position: 'top',
    })
  } catch (err) {
    console.error('Failed to save settings:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to save settings',
      caption: err instanceof Error ? err.message : String(err),
    })
  }
}

// Cancel changes
function cancelChanges(): void {
  loadSettings()
  $q.notify({
    type: 'info',
    message: 'Changes cancelled',
  })
}

// Reset to defaults
function confirmReset(): void {
  $q.dialog({
    title: 'Reset Settings',
    message: 'Are you sure you want to reset all settings to their default values?',
    cancel: true,
    persistent: true,
  }).onOk(() => {
    settingsStore.resetToDefaults()
    loadSettings()
    $q.notify({
      type: 'info',
      message: 'Settings reset to defaults',
    })
  })
}

// Initialize
onMounted(async () => {
  await settingsStore.initialize()
  loadSettings()
})
</script>

<style scoped>
.settings-container {
  padding-bottom: 2rem;
}
</style>
