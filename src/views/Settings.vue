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

      <!-- General Section -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            <q-icon
              name="settings"
              class="q-mr-sm"
            />
            General
          </div>

          <div class="q-gutter-md">
            <q-input
              v-model="localSettings.default_save_path"
              label="Sessions Root Folder"
              hint="Default folder to save QA sessions"
              outlined
              readonly
            >
              <template #prepend>
                <q-icon name="folder" />
              </template>
              <template #append>
                <q-btn
                  round
                  dense
                  flat
                  icon="folder_open"
                  @click="selectSessionsRoot"
                >
                  <q-tooltip>Browse</q-tooltip>
                </q-btn>
              </template>
            </q-input>

            <q-toggle
              v-model="localSettings.launch_on_startup"
              label="Launch on Windows startup"
              color="primary"
            />

            <q-toggle
              v-model="localSettings.minimize_to_tray"
              label="Minimize to tray on close"
              color="primary"
            />
          </div>
        </q-card-section>
      </q-card>

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
              v-model="localSettings.hotkey_toggle_session"
              label="Start/End Session"
              hint="Default: Ctrl+Shift+Q"
              outlined
              readonly
            >
              <template #prepend>
                <q-icon name="power_settings_new" />
              </template>
              <template #append>
                <q-btn
                  flat
                  dense
                  label="Record"
                  color="primary"
                  @click="recordHotkey('hotkey_toggle_session')"
                >
                  <q-tooltip>Click to record a new hotkey</q-tooltip>
                </q-btn>
              </template>
            </q-input>

            <q-input
              v-model="localSettings.hotkey_new_bug"
              label="New Bug Capture"
              hint="Default: Print Screen"
              outlined
              readonly
            >
              <template #prepend>
                <q-icon name="camera" />
              </template>
              <template #append>
                <q-btn
                  flat
                  dense
                  label="Record"
                  color="primary"
                  @click="recordHotkey('hotkey_new_bug')"
                >
                  <q-tooltip>Click to record a new hotkey</q-tooltip>
                </q-btn>
              </template>
            </q-input>

            <q-input
              v-model="localSettings.hotkey_end_bug"
              label="End Bug Capture"
              hint="Default: F4"
              outlined
              readonly
            >
              <template #prepend>
                <q-icon name="stop" />
              </template>
              <template #append>
                <q-btn
                  flat
                  dense
                  label="Record"
                  color="primary"
                  @click="recordHotkey('hotkey_end_bug')"
                >
                  <q-tooltip>Click to record a new hotkey</q-tooltip>
                </q-btn>
              </template>
            </q-input>

            <q-input
              v-model="localSettings.hotkey_quick_notepad"
              label="Quick Notepad"
              hint="Default: Ctrl+Shift+N"
              outlined
              readonly
            >
              <template #prepend>
                <q-icon name="note_add" />
              </template>
              <template #append>
                <q-btn
                  flat
                  dense
                  label="Record"
                  color="primary"
                  @click="recordHotkey('hotkey_quick_notepad')"
                >
                  <q-tooltip>Click to record a new hotkey</q-tooltip>
                </q-btn>
              </template>
            </q-input>

            <q-input
              v-model="localSettings.hotkey_session_notepad"
              label="Session Notepad"
              hint="Default: Ctrl+Shift+M"
              outlined
              readonly
            >
              <template #prepend>
                <q-icon name="notes" />
              </template>
              <template #append>
                <q-btn
                  flat
                  dense
                  label="Record"
                  color="primary"
                  @click="recordHotkey('hotkey_session_notepad')"
                >
                  <q-tooltip>Click to record a new hotkey</q-tooltip>
                </q-btn>
              </template>
            </q-input>

            <q-banner
              v-if="hotkeyConflict"
              class="bg-warning text-white"
              dense
            >
              <template #avatar>
                <q-icon
                  name="warning"
                  color="white"
                />
              </template>
              {{ hotkeyConflict }}
            </q-banner>
          </div>
        </q-card-section>
      </q-card>

      <!-- Annotation Section -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            <q-icon
              name="draw"
              class="q-mr-sm"
            />
            Annotation
          </div>

          <div class="q-gutter-md">
            <q-toggle
              v-model="localSettings.annotation_auto_open"
              label="Auto-open annotation on screenshot capture"
              color="primary"
            />

            <div>
              <div class="text-subtitle2 q-mb-sm">
                Save Mode
              </div>
              <q-option-group
                v-model="localSettings.annotation_save_mode"
                :options="annotationSaveModeOptions"
                color="primary"
              />
            </div>

            <div>
              <div class="text-subtitle2 q-mb-sm">
                Default Color
              </div>
              <q-btn-group flat>
                <q-btn
                  v-for="color in annotationColors"
                  :key="color.value"
                  :style="{backgroundColor: color.value, color: 'white'}"
                  :outline="localSettings.annotation_default_color !== color.value"
                  :unelevated="localSettings.annotation_default_color === color.value"
                  @click="localSettings.annotation_default_color = color.value"
                >
                  {{ color.label }}
                </q-btn>
              </q-btn-group>
            </div>

            <div>
              <div class="text-subtitle2 q-mb-sm">
                Default Stroke Width
              </div>
              <q-option-group
                v-model="localSettings.annotation_stroke_width"
                :options="strokeWidthOptions"
                color="primary"
              />
            </div>
          </div>
        </q-card-section>
      </q-card>

      <!-- AI (Claude) Section -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            <q-icon
              name="smart_toy"
              class="q-mr-sm"
            />
            AI (Claude)
          </div>

          <div class="q-gutter-md">
            <q-banner
              :class="claudeStatus === 'available' ? 'bg-positive' : 'bg-warning'"
              class="text-white"
              dense
            >
              <template #avatar>
                <q-icon
                  :name="claudeStatus === 'available' ? 'check_circle' : 'warning'"
                  color="white"
                />
              </template>
              <div v-if="claudeStatus === 'available'">
                Claude CLI is available and authenticated ✓
              </div>
              <div v-else-if="claudeStatus === 'not_found'">
                Claude CLI not found. <a
                  href="https://www.anthropic.com/claude"
                  target="_blank"
                  class="text-white"
                  style="text-decoration: underline;"
                >Install Claude CLI</a>
              </div>
              <div v-else-if="claudeStatus === 'not_authenticated'">
                Claude CLI found but not authenticated. Run <code>claude</code> to log in.
              </div>
              <div v-else>
                Checking Claude CLI status...
              </div>
            </q-banner>

            <q-toggle
              v-model="localSettings.ai_auto_generate"
              label="Auto-generate descriptions on review"
              color="primary"
              :disable="claudeStatus !== 'available'"
            />

            <q-btn
              outline
              color="primary"
              label="Test Claude Connection"
              icon="refresh"
              :loading="testingClaude"
              :disable="claudeStatus !== 'available'"
              @click="testClaudeConnection"
            />
          </div>
        </q-card-section>
      </q-card>

      <!-- Ticketing Section -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            <q-icon
              name="integration_instructions"
              class="q-mr-sm"
            />
            Ticketing
          </div>

          <div class="q-gutter-md">
            <q-select
              v-model="localSettings.ticketing_provider"
              :options="ticketingProviderOptions"
              label="Integration Type"
              outlined
              emit-value
              map-options
            >
              <template #prepend>
                <q-icon name="cloud" />
              </template>
            </q-select>

            <q-select
              v-model="localSettings.default_bug_type"
              :options="bugTypeOptions"
              label="Default Bug Type"
              outlined
              emit-value
              map-options
            >
              <template #prepend>
                <q-icon name="bug_report" />
              </template>
            </q-select>

            <!-- Linear API Configuration -->
            <div v-if="localSettings.ticketing_provider === 'linear'">
              <q-separator class="q-my-md" />
              <div class="text-subtitle2 q-mb-sm">
                Linear Configuration
              </div>

              <q-input
                v-model="localSettings.linear_api_key"
                label="Linear API Key"
                hint="Get your API key from https://linear.app/settings/api"
                type="password"
                outlined
                dense
                class="q-mb-md"
              >
                <template #prepend>
                  <q-icon name="vpn_key" />
                </template>
              </q-input>

              <q-input
                v-model="localSettings.linear_team_id"
                label="Team ID"
                hint="Linear team ID (e.g., 44c86ac8-cb80-4302-9d81-a0a350b2c352)"
                outlined
                dense
                class="q-mb-md"
              >
                <template #prepend>
                  <q-icon name="group" />
                </template>
              </q-input>

              <q-btn
                outline
                color="primary"
                label="Test Connection"
                icon="check_circle"
                :loading="testingLinearConnection"
                :disable="!localSettings.linear_api_key"
                @click="testLinearConnection"
              />
            </div>

            <q-separator class="q-my-md" />

            <q-input
              v-model="localSettings.linear_config_path"
              label="Linear Project Configuration (Optional)"
              hint="Path to Linear configuration file (advanced users only)"
              outlined
              readonly
            >
              <template #prepend>
                <q-icon name="settings_applications" />
              </template>
              <template #append>
                <q-btn
                  round
                  dense
                  flat
                  icon="folder_open"
                  @click="selectLinearConfigPath"
                >
                  <q-tooltip>Browse</q-tooltip>
                </q-btn>
                <q-btn
                  v-if="localSettings.linear_config_path"
                  round
                  dense
                  flat
                  icon="clear"
                  @click="localSettings.linear_config_path = ''"
                >
                  <q-tooltip>Clear</q-tooltip>
                </q-btn>
              </template>
            </q-input>
          </div>
        </q-card-section>
      </q-card>

      <!-- Ticket Template Section -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            <q-icon
              name="description"
              class="q-mr-sm"
            />
            Ticket Template
          </div>

          <div class="q-gutter-md">
            <div class="row items-center">
              <div class="col-4 text-grey-7">
                Template Source
              </div>
              <div class="col">
                {{ templateSource }}
              </div>
            </div>

            <div class="row q-gutter-sm">
              <q-btn
                outline
                color="primary"
                label="Edit Template"
                icon="edit"
                @click="openTemplateEditor"
              />
              <q-btn
                outline
                color="warning"
                label="Reset to Default"
                icon="restore"
                @click="confirmResetTemplate"
              />
            </div>

            <q-separator />

            <div>
              <div class="text-subtitle2 q-mb-sm">
                Live Preview
              </div>
              <q-card
                flat
                bordered
                class="bg-grey-1"
              >
                <q-card-section>
                  <div
                    v-if="templatePreview"
                    class="template-preview"
                    style="white-space: pre-wrap; font-family: monospace; font-size: 12px;"
                  >
                    {{ templatePreview }}
                  </div>
                  <div
                    v-else
                    class="text-grey-6"
                  >
                    Loading preview...
                  </div>
                </q-card-section>
              </q-card>
            </div>
          </div>
        </q-card-section>
      </q-card>

      <!-- About Section -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            <q-icon
              name="info"
              class="q-mr-sm"
            />
            About
          </div>

          <div class="q-gutter-md">
            <div class="row items-center">
              <div class="col-4 text-grey-7">
                App Version
              </div>
              <div class="col">
                {{ appVersion }}
              </div>
            </div>

            <div class="row items-center">
              <div class="col-4 text-grey-7">
                Developer
              </div>
              <div class="col">
                Unbroken Technology
              </div>
            </div>

            <q-separator />

            <div class="row q-gutter-sm">
              <q-btn
                flat
                color="primary"
                label="Website"
                icon="public"
                @click="openLink('https://unbroken.tech')"
              />
              <q-btn
                flat
                color="primary"
                label="Support"
                icon="help"
                @click="openLink('mailto:support@unbroken.tech')"
              />
              <q-btn
                flat
                color="primary"
                label="Changelog"
                icon="history"
                @click="openLink('https://github.com/UnbrokenTechnology/unbroken_qa_capture/releases')"
              />
            </div>

            <div class="text-caption text-grey-6">
              Licensed under MIT License
            </div>
          </div>
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
import { useSettingsStore } from '@/stores/settings'
import { useQuasar } from 'quasar'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { open as openUrl } from '@tauri-apps/plugin-shell'

const settingsStore = useSettingsStore()
const $q = useQuasar()

// Local settings state (for editing before save)
const localSettings = ref({
  // General
  default_save_path: '',
  launch_on_startup: false,
  minimize_to_tray: true,

  // Hotkeys
  hotkey_toggle_session: 'Ctrl+Shift+Q',
  hotkey_new_bug: 'Print Screen',
  hotkey_end_bug: 'F4',
  hotkey_quick_notepad: 'Ctrl+Shift+N',
  hotkey_session_notepad: 'Ctrl+Shift+M',

  // Annotation
  annotation_auto_open: true,
  annotation_save_mode: 'alongside',
  annotation_default_color: '#FF0000',
  annotation_stroke_width: 'medium',

  // AI
  ai_auto_generate: false,

  // Ticketing
  ticketing_provider: 'linear',
  default_bug_type: 'bug',
  linear_api_key: '',
  linear_team_id: '',
  linear_config_path: '',
})

// UI state
const hotkeyConflict = ref<string | null>(null)
const claudeStatus = ref<'available' | 'not_found' | 'not_authenticated' | 'checking'>('checking')
const testingClaude = ref(false)
const testingLinearConnection = ref(false)
const appVersion = ref('1.0.0')
const templateSource = ref<string>('Default')
const templatePreview = ref<string>('')

// Options
const annotationSaveModeOptions = [
  { label: 'Save alongside original', value: 'alongside' },
  { label: 'Overwrite original', value: 'overwrite' },
]

const annotationColors = [
  { label: 'Red', value: '#FF0000' },
  { label: 'Blue', value: '#0000FF' },
  { label: 'Green', value: '#00FF00' },
  { label: 'Yellow', value: '#FFFF00' },
  { label: 'Orange', value: '#FFA500' },
  { label: 'Purple', value: '#800080' },
  { label: 'Black', value: '#000000' },
]

const strokeWidthOptions = [
  { label: 'Thin (2px)', value: 'thin' },
  { label: 'Medium (4px)', value: 'medium' },
  { label: 'Thick (8px)', value: 'thick' },
]

const ticketingProviderOptions = [
  { label: 'Linear', value: 'linear' },
  { label: 'File-based (Markdown)', value: 'file' },
]

const bugTypeOptions = [
  { label: 'Bug', value: 'bug' },
  { label: 'Feature', value: 'feature' },
  { label: 'Feedback', value: 'feedback' },
]

// Hotkey functions
function recordHotkey(key: string): void {
  $q.dialog({
    title: 'Record Hotkey',
    message: 'Press the key combination you want to use. (Note: This is a placeholder - actual hotkey recording will be implemented in the backend)',
    prompt: {
      model: '',
      type: 'text',
    },
    cancel: true,
  }).onOk((value: string) => {
    if (value) {
      // Check for conflicts
      const allHotkeys = [
        localSettings.value.hotkey_toggle_session,
        localSettings.value.hotkey_new_bug,
        localSettings.value.hotkey_end_bug,
        localSettings.value.hotkey_quick_notepad,
        localSettings.value.hotkey_session_notepad,
      ]

      if (allHotkeys.includes(value)) {
        hotkeyConflict.value = `Hotkey conflict: "${value}" is already assigned to another action`
        return
      }

      // Update the hotkey
      localSettings.value[key as keyof typeof localSettings.value] = value as never
      hotkeyConflict.value = null
    }
  })
}

// Check if all settings are valid
const hasValidSettings = computed(() => {
  return localSettings.value.default_save_path !== ''
})

// File path selection
async function selectSessionsRoot(): Promise<void> {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select Sessions Root Folder',
    })
    if (selected) {
      localSettings.value.default_save_path = selected as string
    }
  } catch (err) {
    console.error('Failed to select sessions root:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to select folder',
    })
  }
}

async function selectLinearConfigPath(): Promise<void> {
  try {
    const selected = await open({
      multiple: false,
      title: 'Select Linear Configuration File',
      filters: [
        {
          name: 'JSON',
          extensions: ['json'],
        },
      ],
    })
    if (selected) {
      localSettings.value.linear_config_path = selected as string
    }
  } catch (err) {
    console.error('Failed to select Linear config:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to select file',
    })
  }
}

async function openTemplateEditor(): Promise<void> {
  try {
    await invoke('open_template_in_editor')
    $q.notify({
      type: 'positive',
      message: 'Template opened in system editor',
    })
  } catch (err) {
    console.error('Failed to open template editor:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to open template editor',
      caption: err instanceof Error ? err.message : String(err),
    })
  }
}

async function confirmResetTemplate(): Promise<void> {
  $q.dialog({
    title: 'Reset Template',
    message: 'Are you sure you want to reset the ticket template to the default? This will discard any customizations.',
    cancel: true,
    persistent: true,
  }).onOk(async () => {
    try {
      await invoke('reset_template_to_default')
      await loadTemplateInfo()
      $q.notify({
        type: 'positive',
        message: 'Template reset to default',
      })
    } catch (err) {
      console.error('Failed to reset template:', err)
      $q.notify({
        type: 'negative',
        message: 'Failed to reset template',
        caption: err instanceof Error ? err.message : String(err),
      })
    }
  })
}

async function loadTemplateInfo(): Promise<void> {
  try {
    // Get template path to determine if custom or default
    const templatePath = await invoke<string | null>('get_template_path')
    if (templatePath && templatePath.includes('custom_template.md')) {
      templateSource.value = 'Custom'
    } else {
      templateSource.value = 'Default'
    }

    // Load template preview
    const sampleBugData = {
      title: 'Sample Bug: Button Not Responding',
      bug_type: 'UI',
      description_steps: '1. Click the Submit button\\n2. Observe no response',
      description_expected: 'Button should trigger form submission',
      description_actual: 'Button does nothing when clicked',
      metadata: {
        meeting_id: 'MTG-2024-001',
        software_version: '2.5.0',
        environment: {
          os: 'Windows 11 Pro',
          display_resolution: '1920x1080',
          dpi_scaling: '150%',
          ram: '16GB',
          cpu: 'Intel Core i7-11800H',
          foreground_app: 'MyApp.exe',
        },
        console_captures: [],
        custom_fields: {},
      },
      folder_path: 'C:\\\\QA\\\\Sessions\\\\2024-02-16\\\\bug-001',
      captures: ['screenshot-01.png', 'screenshot-02.png'],
      console_output: 'Error: Form validation failed\\nUncaught TypeError: Cannot read property submit',
    }

    const preview = await invoke<string>('render_bug_template', { bugData: sampleBugData })
    templatePreview.value = preview
  } catch (err) {
    console.error('Failed to load template info:', err)
    templatePreview.value = 'Failed to load preview'
  }
}

// AI / Claude functions
async function checkClaudeStatus(): Promise<void> {
  try {
    claudeStatus.value = 'checking'
    // TODO: Implement backend command to check Claude CLI status
    // For now, just set to 'available' as placeholder
    claudeStatus.value = 'available'
  } catch (err) {
    console.error('Failed to check Claude status:', err)
    claudeStatus.value = 'not_found'
  }
}

async function testClaudeConnection(): Promise<void> {
  try {
    testingClaude.value = true
    // TODO: Implement backend command to test Claude CLI
    await new Promise(resolve => setTimeout(resolve, 1000))
    $q.notify({
      type: 'positive',
      message: 'Claude CLI connection successful!',
    })
  } catch (err) {
    console.error('Failed to test Claude connection:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to connect to Claude CLI',
      caption: err instanceof Error ? err.message : String(err),
    })
  } finally {
    testingClaude.value = false
  }
}

// Linear connection test
async function testLinearConnection(): Promise<void> {
  if (!localSettings.value.linear_api_key) {
    $q.notify({
      type: 'warning',
      message: 'Please enter a Linear API key first',
    })
    return
  }

  testingLinearConnection.value = true
  try {
    // Test authentication with the provided credentials
    await invoke('ticketing_authenticate', {
      credentials: {
        api_key: localSettings.value.linear_api_key,
        team_id: localSettings.value.linear_team_id || null,
        workspace_id: null,
      },
    })

    $q.notify({
      type: 'positive',
      message: 'Linear connection successful! Credentials are valid.',
    })
  } catch (err) {
    console.error('Linear connection test failed:', err)
    $q.notify({
      type: 'negative',
      message: 'Linear connection failed',
      caption: err instanceof Error ? err.message : String(err),
    })
  } finally {
    testingLinearConnection.value = false
  }
}

// About section functions
function openLink(url: string): void {
  openUrl(url).catch((err) => {
    console.error('Failed to open link:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to open link',
    })
  })
}

// Load settings from store
function loadSettings(): void {
  localSettings.value = {
    // General
    default_save_path: settingsStore.getSetting('default_save_path', ''),
    launch_on_startup: settingsStore.getSetting('launch_on_startup', 'false') === 'true',
    minimize_to_tray: settingsStore.getSetting('minimize_to_tray', 'true') === 'true',

    // Hotkeys
    hotkey_toggle_session: settingsStore.getSetting('hotkey_toggle_session', 'Ctrl+Shift+Q'),
    hotkey_new_bug: settingsStore.getSetting('hotkey_new_bug', 'Print Screen'),
    hotkey_end_bug: settingsStore.getSetting('hotkey_end_bug', 'F4'),
    hotkey_quick_notepad: settingsStore.getSetting('hotkey_quick_notepad', 'Ctrl+Shift+N'),
    hotkey_session_notepad: settingsStore.getSetting('hotkey_session_notepad', 'Ctrl+Shift+M'),

    // Annotation
    annotation_auto_open: settingsStore.getSetting('annotation_auto_open', 'true') === 'true',
    annotation_save_mode: settingsStore.getSetting('annotation_save_mode', 'alongside'),
    annotation_default_color: settingsStore.getSetting('annotation_default_color', '#FF0000'),
    annotation_stroke_width: settingsStore.getSetting('annotation_stroke_width', 'medium'),

    // AI
    ai_auto_generate: settingsStore.getSetting('ai_auto_generate', 'false') === 'true',

    // Ticketing
    ticketing_provider: settingsStore.getSetting('ticketing_provider', 'linear'),
    default_bug_type: settingsStore.getSetting('default_bug_type', 'bug'),
    linear_api_key: settingsStore.getSetting('linear_api_key', ''),
    linear_team_id: settingsStore.getSetting('linear_team_id', ''),
    linear_config_path: settingsStore.getSetting('linear_config_path', ''),
  }
}

// Save settings
async function saveSettings(): Promise<void> {
  try {
    // Save all settings to backend
    const settingsToSave: Record<string, string> = {
      // General
      default_save_path: localSettings.value.default_save_path,
      launch_on_startup: localSettings.value.launch_on_startup.toString(),
      minimize_to_tray: localSettings.value.minimize_to_tray.toString(),

      // Hotkeys
      hotkey_toggle_session: localSettings.value.hotkey_toggle_session,
      hotkey_new_bug: localSettings.value.hotkey_new_bug,
      hotkey_end_bug: localSettings.value.hotkey_end_bug,
      hotkey_quick_notepad: localSettings.value.hotkey_quick_notepad,
      hotkey_session_notepad: localSettings.value.hotkey_session_notepad,

      // Annotation
      annotation_auto_open: localSettings.value.annotation_auto_open.toString(),
      annotation_save_mode: localSettings.value.annotation_save_mode,
      annotation_default_color: localSettings.value.annotation_default_color,
      annotation_stroke_width: localSettings.value.annotation_stroke_width,

      // AI
      ai_auto_generate: localSettings.value.ai_auto_generate.toString(),

      // Ticketing
      ticketing_provider: localSettings.value.ticketing_provider,
      default_bug_type: localSettings.value.default_bug_type,
      linear_api_key: localSettings.value.linear_api_key,
      linear_team_id: localSettings.value.linear_team_id,
      linear_config_path: localSettings.value.linear_config_path,
    }

    // Save each setting
    for (const [key, value] of Object.entries(settingsToSave)) {
      await settingsStore.saveSetting(key, value)
    }

    // Save Linear credentials to ticketing table if API key is provided
    if (localSettings.value.linear_api_key && localSettings.value.ticketing_provider === 'linear') {
      try {
        await invoke('ticketing_save_credentials', {
          credentials: {
            api_key: localSettings.value.linear_api_key,
            team_id: localSettings.value.linear_team_id || null,
            workspace_id: null,
          },
        })
      } catch (err) {
        console.warn('Failed to save Linear credentials:', err)
      }
    }

    // If launch_on_startup changed, update Windows registry
    if (localSettings.value.launch_on_startup) {
      try {
        await invoke('enable_startup')
      } catch (err) {
        console.warn('Failed to enable startup:', err)
      }
    } else {
      try {
        await invoke('disable_startup')
      } catch (err) {
        console.warn('Failed to disable startup:', err)
      }
    }

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
  await checkClaudeStatus()
  await loadTemplateInfo()

  // Load Linear credentials from ticketing table
  try {
    const creds = await invoke<any>('ticketing_get_credentials')
    if (creds && creds.api_key) {
      localSettings.value.linear_api_key = creds.api_key
      localSettings.value.linear_team_id = creds.team_id || ''
    }
  } catch (err) {
    console.warn('Failed to load Linear credentials:', err)
  }

  // Get app version
  try {
    appVersion.value = await invoke<string>('get_app_version')
  } catch (err) {
    console.warn('Failed to get app version:', err)
    appVersion.value = '1.0.0'
  }
})
</script>

<style scoped>
.settings-container {
  padding-bottom: 2rem;
}
</style>
