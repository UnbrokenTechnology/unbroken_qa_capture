<template>
  <q-dialog
    v-model="show"
    persistent
    maximized
    transition-show="slide-up"
    transition-hide="slide-down"
  >
    <q-card class="wizard-card">
      <q-card-section class="wizard-header bg-primary text-white">
        <div class="text-h4">
          Welcome to Unbroken QA Capture
        </div>
        <div class="text-subtitle1">
          Let's get you set up
        </div>
      </q-card-section>

      <q-card-section class="wizard-content">
        <q-stepper
          ref="stepper"
          v-model="step"
          color="primary"
          animated
          header-nav
          :contracted="$q.screen.lt.md"
        >
          <!-- Step 1: Welcome -->
          <q-step
            :name="1"
            title="Welcome"
            icon="waving_hand"
            :done="maxStepReached > 1 && step1Complete"
            :error="maxStepReached > 1 && !step1Complete"
          >
            <div class="q-pa-md text-center">
              <q-icon
                name="badge"
                size="100px"
                color="primary"
                class="q-mb-md"
              />
              <h5 class="q-mt-none q-mb-md">
                Welcome to Unbroken QA Capture!
              </h5>
              <p class="text-body1">
                Unbroken QA Capture helps you capture, organize, and report bugs with ease.
              </p>
              <p class="text-body1">
                This wizard will guide you through the initial setup in just a few steps.
              </p>
              <ul
                class="text-left q-mt-lg"
                style="max-width: 500px; margin-left: auto; margin-right: auto;"
              >
                <li class="q-mb-sm">
                  📁 Choose where to save your QA sessions
                </li>
                <li class="q-mb-sm">
                  ⌨️ Configure keyboard shortcuts
                </li>
                <li class="q-mb-sm">
                  🔗 Optional: Connect to Linear for issue tracking
                </li>
                <li class="q-mb-sm">
                  🤖 Optional: Enable AI-powered descriptions
                </li>
              </ul>
            </div>
          </q-step>

          <!-- Step 2: Sessions Root Folder -->
          <q-step
            :name="2"
            title="Sessions Folder"
            icon="folder"
            :done="maxStepReached > 2 && step2Complete"
            :error="maxStepReached > 2 && !step2Complete"
          >
            <div class="q-pa-md">
              <h6 class="q-mt-none q-mb-md">
                Choose Sessions Root Folder
              </h6>
              <p class="text-body1 q-mb-md">
                Select a folder where all your QA sessions will be saved.
                Each session will create a subfolder with bug reports, screenshots, and recordings.
              </p>

              <q-input
                v-model="sessionsFolderPath"
                label="Sessions Folder Path"
                outlined
                readonly
                class="q-mb-md"
                :error="!sessionsFolderPath"
                error-message="Please select a folder"
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
                    @click="selectSessionsFolder"
                  >
                    <q-tooltip>Browse</q-tooltip>
                  </q-btn>
                </template>
              </q-input>

              <q-banner
                v-if="sessionsFolderPath"
                class="bg-positive text-white"
                dense
              >
                <template #avatar>
                  <q-icon
                    name="check_circle"
                    color="white"
                  />
                </template>
                Selected: {{ sessionsFolderPath }}
              </q-banner>
            </div>
          </q-step>

          <!-- Step 3: Configure Hotkeys -->
          <q-step
            :name="3"
            title="Hotkeys"
            icon="keyboard"
            :done="maxStepReached > 3 && step3Complete"
            :error="maxStepReached > 3 && !step3Complete"
          >
            <div class="q-pa-md">
              <h6 class="q-mt-none q-mb-md">
                Current Keyboard Shortcuts
              </h6>
              <p class="text-body1 q-mb-md">
                These are the default global hotkeys for quickly capturing bugs and managing sessions.
                You can customize these in Settings after setup is complete.
              </p>

              <div class="q-gutter-md">
                <q-input
                  v-model="hotkeys.toggleSession"
                  label="Toggle Session (Start/Stop)"
                  hint="Default: Ctrl+Shift+Q"
                  outlined
                  readonly
                >
                  <template #prepend>
                    <q-icon name="toggle_on" />
                  </template>
                </q-input>

                <q-input
                  v-model="hotkeys.startBugCapture"
                  label="Start Bug Capture"
                  hint="Default: PrintScreen"
                  outlined
                  readonly
                >
                  <template #prepend>
                    <q-icon name="camera" />
                  </template>
                </q-input>

                <q-input
                  v-model="hotkeys.endBugCapture"
                  label="End Bug Capture"
                  hint="Default: F4"
                  outlined
                  readonly
                >
                  <template #prepend>
                    <q-icon name="stop" />
                  </template>
                </q-input>

                <q-input
                  v-model="hotkeys.openQuickNotepad"
                  label="Open Quick Notepad"
                  hint="Default: Ctrl+Shift+N"
                  outlined
                  readonly
                >
                  <template #prepend>
                    <q-icon name="note_add" />
                  </template>
                </q-input>

                <q-input
                  v-model="hotkeys.openSessionNotepad"
                  label="Open Session Notepad"
                  hint="Default: Ctrl+Shift+M"
                  outlined
                  readonly
                >
                  <template #prepend>
                    <q-icon name="description" />
                  </template>
                </q-input>
              </div>

              <q-banner class="q-mt-md bg-info text-white">
                <template #avatar>
                  <q-icon
                    name="info"
                    color="white"
                  />
                </template>
                You can customize these hotkeys later in the Settings page.
              </q-banner>
            </div>
          </q-step>

          <!-- Step 4: Linear Setup (Optional) -->
          <q-step
            :name="4"
            title="Linear Setup"
            icon="bug_report"
            :done="maxStepReached > 4 && step4Complete"
            :error="maxStepReached > 4 && !step4Complete"
          >
            <div class="q-pa-md">
              <h6 class="q-mt-none q-mb-md">
                Connect to Linear (Optional)
              </h6>
              <p class="text-body1 q-mb-md">
                Optionally connect to Linear for automatic issue creation and syncing.
                You can skip this step and set it up later.
              </p>

              <q-toggle
                v-model="linearSetup.enabled"
                label="Enable Linear Integration"
                color="primary"
                class="q-mb-md"
              />

              <div
                v-if="linearSetup.enabled"
                class="q-gutter-md"
              >
                <q-input
                  v-model="linearSetup.apiKey"
                  label="Linear API Key"
                  hint="Get your API key from Linear settings"
                  outlined
                  type="password"
                >
                  <template #prepend>
                    <q-icon name="key" />
                  </template>
                </q-input>

                <q-input
                  v-model="linearSetup.teamId"
                  label="Team ID"
                  hint="Your Linear team identifier"
                  outlined
                >
                  <template #prepend>
                    <q-icon name="group" />
                  </template>
                </q-input>
              </div>

              <q-banner
                v-if="!linearSetup.enabled"
                class="q-mt-md bg-grey-3"
              >
                <template #avatar>
                  <q-icon name="info" />
                </template>
                You can enable Linear integration later in Settings.
              </q-banner>
            </div>
          </q-step>

          <!-- Step 5: Claude CLI Check -->
          <q-step
            :name="5"
            title="AI Setup"
            icon="psychology"
            :done="maxStepReached > 5 && step5Complete"
            :error="maxStepReached > 5 && !step5Complete"
          >
            <div class="q-pa-md">
              <h6 class="q-mt-none q-mb-md">
                AI-Powered Descriptions
              </h6>
              <p class="text-body1 q-mb-md">
                Unbroken QA Capture can use Claude AI to automatically generate bug descriptions
                from screenshots and context. This requires the Claude CLI to be installed.
              </p>

              <q-card
                flat
                bordered
                class="q-mb-md"
              >
                <q-card-section>
                  <div class="row items-center">
                    <div class="col">
                      <div class="text-h6">
                        Claude CLI Status
                      </div>
                    </div>
                    <div class="col-auto">
                      <q-btn
                        flat
                        round
                        dense
                        icon="refresh"
                        :loading="claudeChecking"
                        @click="checkClaudeStatus"
                      >
                        <q-tooltip>Refresh Status</q-tooltip>
                      </q-btn>
                    </div>
                  </div>

                  <q-separator class="q-my-md" />

                  <div v-if="claudeChecking">
                    <q-linear-progress
                      indeterminate
                      color="primary"
                    />
                    <p class="text-center q-mt-md">
                      Checking Claude CLI...
                    </p>
                  </div>

                  <div v-else-if="claudeStatus === 'Ready'">
                    <q-banner class="bg-positive text-white">
                      <template #avatar>
                        <q-icon
                          name="check_circle"
                          color="white"
                        />
                      </template>
                      <div>
                        <strong>Claude CLI is installed and ready!</strong>
                        <div class="text-caption q-mt-xs">
                          {{ claudeVersion }}
                        </div>
                      </div>
                    </q-banner>

                    <q-toggle
                      v-model="aiEnabled"
                      label="Enable AI-powered bug descriptions"
                      color="primary"
                      class="q-mt-md"
                    />
                  </div>

                  <div v-else-if="claudeStatus === 'NotAuthenticated'">
                    <q-banner class="bg-warning text-white">
                      <template #avatar>
                        <q-icon
                          name="warning"
                          color="white"
                        />
                      </template>
                      <div>
                        <strong>Claude CLI is installed but not authenticated</strong>
                        <div class="text-caption q-mt-xs">
                          {{ claudeMessage }}
                        </div>
                      </div>
                    </q-banner>

                    <p class="q-mt-md">
                      Run <code>claude</code> in your terminal to authenticate.
                    </p>
                  </div>

                  <div v-else>
                    <q-banner class="bg-grey-4">
                      <template #avatar>
                        <q-icon name="info" />
                      </template>
                      <div>
                        <strong>Claude CLI is not installed</strong>
                        <div class="text-caption q-mt-xs">
                          {{ claudeMessage }}
                        </div>
                      </div>
                    </q-banner>

                    <p class="q-mt-md">
                      To enable AI features, install the Claude CLI from
                      <a
                        href="https://claude.ai/download"
                        target="_blank"
                        rel="noopener noreferrer"
                      >claude.ai/download</a>
                    </p>
                  </div>
                </q-card-section>
              </q-card>
            </div>
          </q-step>

          <template #navigation>
            <q-stepper-navigation class="row q-gutter-md justify-between">
              <q-btn
                v-if="step > 1"
                flat
                color="primary"
                label="Back"
                icon="arrow_back"
                @click="(stepper as any)?.previous()"
              />
              <q-space />
              <q-btn
                v-if="step < 5"
                color="primary"
                label="Next"
                icon-right="arrow_forward"
                :disable="!canProceed"
                @click="nextStep"
              />
              <q-btn
                v-else
                color="positive"
                label="Finish Setup"
                icon-right="check"
                :loading="saving"
                :disable="!sessionsFolderPath"
                @click="completeSetup"
              >
                <q-tooltip v-if="!sessionsFolderPath">
                  Please select a sessions folder first (Step 2)
                </q-tooltip>
              </q-btn>
            </q-stepper-navigation>
          </template>
        </q-stepper>
      </q-card-section>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { useQuasar } from 'quasar'
import { useSettingsStore, SETTINGS_KEYS } from '@/stores/settings'
import * as tauri from '@/api/tauri'

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'complete': []
}>()

const $q = useQuasar()
const settingsStore = useSettingsStore()

const show = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
})

// Stepper state
const step = ref(1)
const maxStepReached = ref(1)
const stepper = ref()
const saving = ref(false)

// Step 2: Sessions folder
const sessionsFolderPath = ref('')

// Step 3: Hotkeys (use actual defaults from hotkey.rs)
const hotkeys = ref({
  toggleSession: 'Ctrl+Shift+Q',
  startBugCapture: 'PrintScreen',
  endBugCapture: 'F4',
  openQuickNotepad: 'Ctrl+Shift+N',
  openSessionNotepad: 'Ctrl+Shift+M',
})

// Step 4: Linear setup
const linearSetup = ref({
  enabled: false,
  apiKey: '',
  teamId: '',
})

// Step 5: Claude CLI
const claudeChecking = ref(false)
const claudeStatus = ref<string>('NotInstalled')
const claudeVersion = ref<string>('')
const claudeMessage = ref<string>('')
const aiEnabled = ref(false)

// Validation
const canProceed = computed(() => {
  switch (step.value) {
    case 1:
      return true
    case 2:
      return sessionsFolderPath.value !== ''
    case 3:
      return true
    case 4:
      if (linearSetup.value.enabled) {
        return linearSetup.value.apiKey !== '' && linearSetup.value.teamId !== ''
      }
      return true
    case 5:
      return true
    default:
      return false
  }
})

// Step completion status (for stepper header indicators)
const step1Complete = computed(() => true) // Welcome step is always complete
const step2Complete = computed(() => sessionsFolderPath.value !== '')
const step3Complete = computed(() => true) // Hotkeys step is always complete (defaults are fine)
const step4Complete = computed(() => {
  // Linear setup is optional, so it's complete if disabled OR if enabled with valid credentials
  if (!linearSetup.value.enabled) {
    return true
  }
  return linearSetup.value.apiKey !== '' && linearSetup.value.teamId !== ''
})
const step5Complete = computed(() => true) // AI setup is optional, always complete

// Step navigation
function nextStep() {
  if (canProceed.value) {
    step.value++
    if (step.value > maxStepReached.value) {
      maxStepReached.value = step.value
    }
    // Auto-check Claude status when reaching step 5
    if (step.value === 5) {
      checkClaudeStatus()
    }
  }
}

// Step 2: Select sessions folder
async function selectSessionsFolder() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select Sessions Root Folder',
    })
    if (selected) {
      sessionsFolderPath.value = selected as string
    }
  } catch (err) {
    console.error('Failed to select folder:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to select folder',
    })
  }
}

// Step 5: Check Claude CLI status
async function checkClaudeStatus() {
  claudeChecking.value = true
  try {
    // Use refresh instead of get to force a fresh check (not cached)
    const status = await tauri.refreshClaudeStatus()

    if (status.status === 'ready') {
      claudeStatus.value = 'Ready'
      claudeVersion.value = status.version ?? ''
      claudeMessage.value = ''
      aiEnabled.value = true
    } else if (status.status === 'notAuthenticated') {
      claudeStatus.value = 'NotAuthenticated'
      claudeVersion.value = status.version ?? ''
      claudeMessage.value = status.message ?? ''
      aiEnabled.value = false
    } else if (status.status === 'notInstalled') {
      claudeStatus.value = 'NotInstalled'
      claudeVersion.value = ''
      claudeMessage.value = status.message ?? ''
      aiEnabled.value = false
    }
  } catch (err) {
    console.error('Failed to check Claude status:', err)
    claudeStatus.value = 'NotInstalled'
    claudeMessage.value = err instanceof Error ? err.message : String(err)
    aiEnabled.value = false
  } finally {
    claudeChecking.value = false
  }
}

// Complete setup
async function completeSetup() {
  // Validate required fields before saving
  if (!sessionsFolderPath.value) {
    $q.notify({
      type: 'negative',
      message: 'Please select a sessions folder before completing setup',
      position: 'top',
    })
    return
  }

  saving.value = true
  try {
    // Save all settings
    await settingsStore.saveSetting(SETTINGS_KEYS.DEFAULT_SAVE_PATH, sessionsFolderPath.value)
    // Note: Hotkeys are read-only in wizard - backend uses hardcoded defaults from hotkey.rs
    // User can customize them later in Settings which will update the backend HotkeyConfig
    await settingsStore.saveSetting(SETTINGS_KEYS.AI_ENABLED, aiEnabled.value.toString())

    // Save Linear settings if enabled
    if (linearSetup.value.enabled) {
      await settingsStore.saveSetting('linear_api_key', linearSetup.value.apiKey)
      await settingsStore.saveSetting('linear_team_id', linearSetup.value.teamId)
    }

    // Mark setup as complete
    await tauri.markSetupComplete()

    $q.notify({
      type: 'positive',
      message: 'Setup completed successfully!',
      position: 'top',
    })

    emit('complete')
    show.value = false
  } catch (err) {
    console.error('Failed to complete setup:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to save settings',
      caption: err instanceof Error ? err.message : String(err),
    })
  } finally {
    saving.value = false
  }
}

onMounted(() => {
  // Initialize with any existing settings
  if (settingsStore.defaultSavePath) {
    sessionsFolderPath.value = settingsStore.defaultSavePath
  }
  // Hotkeys always show defaults - they're read-only in wizard
})
</script>

<style scoped>
.wizard-card {
  min-height: 80vh;
  display: flex;
  flex-direction: column;
}

.wizard-header {
  flex-shrink: 0;
}

.wizard-content {
  flex-grow: 1;
  overflow-y: auto;
}

ul {
  list-style-position: inside;
  padding-left: 0;
}

li {
  padding-left: 1em;
}

code {
  background-color: rgba(0, 0, 0, 0.05);
  padding: 2px 6px;
  border-radius: 3px;
  font-family: monospace;
}
</style>
