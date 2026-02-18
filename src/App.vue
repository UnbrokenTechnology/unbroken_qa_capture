<template>
  <!-- Secondary windows (e.g. session-notes) render only the route view without the app shell -->
  <router-view v-if="isSecondaryWindow" />

  <!-- Main window: full app shell with header, status widget, notepads, etc. -->
  <q-layout
    v-else
    view="lHh Lpr lFf"
  >
    <q-header
      elevated
      class="bg-primary text-white"
    >
      <q-toolbar>
        <q-toolbar-title>
          Unbroken QA Capture
        </q-toolbar-title>

        <q-btn
          flat
          dense
          icon="sticky_note_2"
          @click="showQuickNotepad = !showQuickNotepad"
        >
          <q-tooltip>{{ showQuickNotepad ? 'Close' : 'Open' }} Quick Notepad (Ctrl+Alt+N)</q-tooltip>
        </q-btn>

        <q-btn
          flat
          dense
          :icon="showStatusWidget ? 'visibility_off' : 'visibility'"
          @click="toggleStatusWidget"
        >
          <q-tooltip>{{ showStatusWidget ? 'Hide' : 'Show' }} Status Widget</q-tooltip>
        </q-btn>
      </q-toolbar>

      <!-- Session Toolbar (hidden during first-run wizard) -->
      <SessionToolbar v-if="!showFirstRunWizard" />
    </q-header>

    <q-page-container>
      <!-- App initialization overlay: shown while checking for active session -->
      <div
        v-if="appInitializing"
        class="app-init-overlay"
      >
        <q-spinner-dots
          color="primary"
          size="3em"
        />
        <p class="text-body2 text-grey-7 q-mt-md">
          Checking for active session...
        </p>
      </div>
      <router-view v-else />
    </q-page-container>

    <!-- Session Status Widget -->
    <SessionStatusWidget
      :visible="showStatusWidget"
      @close="showStatusWidget = false"
    />

    <!-- First Run Wizard -->
    <FirstRunWizard
      v-model="showFirstRunWizard"
      @complete="onSetupComplete"
    />

    <!-- Quick Notepad (hotkey: Ctrl+Alt+N) -->
    <QuickNotepad
      :visible="showQuickNotepad"
      @close="showQuickNotepad = false"
    />
  </q-layout>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, provide } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useRouter } from 'vue-router'
import { useQuasar } from 'quasar'
import { useTrayStore } from './stores/tray'
import { useSessionStore } from './stores/session'
import { useBugStore } from './stores/bug'
import { useSettingsStore } from './stores/settings'
import SessionStatusWidget from './components/SessionStatusWidget.vue'
import SessionToolbar from './components/SessionToolbar.vue'
import FirstRunWizard from './components/FirstRunWizard.vue'
import QuickNotepad from './components/QuickNotepad.vue'
import * as tauri from './api/tauri'
import { useCaptureEventHandler } from './composables/useCaptureEventHandler'

const router = useRouter()
const $q = useQuasar()
const trayStore = useTrayStore()
const sessionStore = useSessionStore()
const bugStore = useBugStore()
const settingsStore = useSettingsStore()
// Secondary windows (e.g. session-notes) skip the full app shell.
// Treat undefined label (e.g. in test environments) as the main window.
const windowLabel = getCurrentWindow().label
const isSecondaryWindow = windowLabel !== undefined && windowLabel !== 'main'
// Status widget visibility: driven by persisted setting (default off)
const showStatusWidget = computed({
  get: () => settingsStore.showStatusWidget,
  set: (value: boolean) => {
    settingsStore.saveSetting('show_status_widget', value ? 'true' : 'false').catch((err) => {
      console.error('Failed to save status widget visibility:', err)
    })
  },
})
const showFirstRunWizard = ref(false)
const showQuickNotepad = ref(false)
// True until we've finished checking for an active session on startup
const appInitializing = ref(true)
provide('showFirstRunWizard', showFirstRunWizard)

let unlistenHandlers: UnlistenFn[] = []

// Routes that are valid sub-views when a session is active — do not redirect away from these
const activeSessionSubRoutes = new Set(['active-session', 'bug-detail', 'annotate'])

function toggleStatusWidget() {
  showStatusWidget.value = !showStatusWidget.value
}

async function onSetupComplete() {
  // Reload settings store so the app reflects the wizard-configured values
  await settingsStore.loadAllSettings()
  // Navigate to home view after setup
  router.push({ name: 'home' })
}

onMounted(async () => {
  // Secondary windows (session-notes, annotation, etc.) don't need the main-window setup
  if (isSecondaryWindow) return

  // Initialize settings store so settings are available before any settings-dependent operations
  settingsStore.initialize()

  // Check if first-run setup is needed
  try {
    const setupComplete = await tauri.hasCompletedSetup()
    if (!setupComplete) {
      showFirstRunWizard.value = true
    }
  } catch (err) {
    console.error('Failed to check setup status:', err)
  }

  // Initialize tray to idle state
  await trayStore.setIdle()

  // Load active session if exists — this determines initial routing
  try {
    await sessionStore.loadActiveSession()
  } catch (err) {
    console.error('Failed to load active session:', err)
  }

  // If a session is active on startup, it was interrupted by a crash/restart.
  // Show a recovery dialog so the user can choose to resume or end it.
  if (sessionStore.isSessionActive && sessionStore.activeSession) {
    const crashedSession = sessionStore.activeSession
    await new Promise<void>((resolve) => {
      $q.dialog({
        title: 'Session Recovery',
        message: `A session was interrupted. Would you like to resume it or end it?\n\nSession started: ${new Date(crashedSession.started_at).toLocaleString()}`,
        persistent: true,
        ok: {
          label: 'Resume Session',
          color: 'primary',
          unelevated: true,
        },
        cancel: {
          label: 'End Session',
          color: 'negative',
          flat: true,
        },
      })
        .onOk(async () => {
          try {
            await sessionStore.resumeSession(crashedSession.id)
            await trayStore.setActive()
            router.push({ name: 'active-session' })
          } catch (err) {
            console.error('Failed to resume session:', err)
            $q.notify({
              type: 'negative',
              message: 'Failed to resume session',
              caption: err instanceof Error ? err.message : String(err),
              position: 'bottom-right',
              timeout: 5000,
            })
          }
          resolve()
        })
        .onCancel(async () => {
          try {
            await sessionStore.endSession(crashedSession.id)
            await trayStore.setIdle()
          } catch (err) {
            console.error('Failed to end crashed session:', err)
          }
          resolve()
        })
    })
  }

  // Reveal the UI now that initialization is complete
  appInitializing.value = false

  // Setup session event listeners
  await sessionStore.setupEventListeners()

  // Wire up screenshot capture event handler (in composable to avoid circular store deps)
  const { setup: setupCaptureHandler } = useCaptureEventHandler()
  const unlistenScreenshotCaptured = await setupCaptureHandler()

  // Listen for tray menu events
  const unlistenStartSession = await listen('tray-menu-start-session', async () => {
    if (sessionStore.isSessionActive) {
      // Session already running — just bring the window to the session view
      router.push({ name: 'active-session' })
    } else {
      // Start a new session; the activeSession watcher will navigate automatically
      try {
        await sessionStore.startSession()
      } catch (err) {
        console.error('Failed to start session from tray:', err)
        $q.notify({
          type: 'negative',
          message: 'Failed to start session',
          caption: err instanceof Error ? err.message : String(err),
          position: 'bottom-right',
          timeout: 5000,
        })
      }
    }
  })

  const unlistenNewBug = await listen('tray-menu-new-bug', async () => {
    if (sessionStore.isSessionActive && sessionStore.activeSessionId) {
      // Start bug capture in the current session and navigate to it
      try {
        const bug = await bugStore.startBugCapture({ session_id: sessionStore.activeSessionId, status: 'capturing' })
        await trayStore.setBugCapture(bug.display_id)
        router.push({ name: 'active-session' })
      } catch (err) {
        console.error('Failed to start bug capture from tray:', err)
      }
    } else {
      // No active session — navigate home so user can start one
      router.push({ name: 'home' })
    }
  })

  const unlistenSettings = await listen('tray-menu-settings', () => {
    router.push({ name: 'settings' })
  })

  // Tray "End Session" menu item (shown during active/bug states)
  const unlistenTrayEndSession = await listen('tray-menu-end-session', async () => {
    if (sessionStore.isSessionActive && sessionStore.activeSessionId) {
      $q.loading.show({ message: 'Ending session...' })
      try {
        await sessionStore.endSession(sessionStore.activeSessionId)
        await trayStore.setIdle()
        router.push({ name: 'home' })
      } catch (err) {
        console.error('Failed to end session from tray:', err)
        $q.notify({
          type: 'negative',
          message: 'Failed to end session',
          caption: err instanceof Error ? err.message : String(err),
          position: 'bottom-right',
          timeout: 5000,
        })
      } finally {
        $q.loading.hide()
      }
    }
  })

  // Tray "End Bug Capture (F4)" menu item (shown during bug capture state)
  const unlistenTrayEndBugCapture = await listen('tray-menu-end-bug-capture', async () => {
    if (bugStore.activeBug?.id) {
      try {
        await bugStore.completeBugCapture(bugStore.activeBug.id)
        await trayStore.setActive()
      } catch (err) {
        console.error('Failed to end bug capture from tray:', err)
        $q.notify({
          type: 'negative',
          message: 'Failed to end bug capture',
          caption: err instanceof Error ? err.message : String(err),
          position: 'bottom-right',
          timeout: 5000,
        })
      }
    }
  })

  // Tray "Open Review" menu item (shown during review state)
  const unlistenTrayOpenReview = await listen('tray-menu-open-review', () => {
    router.push({ name: 'session-review' })
  })

  // Tray "Help / User Guide" menu item (always visible)
  const unlistenTrayHelp = await listen('tray-menu-help', () => {
    router.push({ name: 'help' })
  })

  // When the window is restored from tray (icon click or "Open Main Window"),
  // navigate back to the correct screen — but preserve the user's current view
  // if they are already on a valid sub-view for the current session state.
  const unlistenWindowShown = await listen('tray-window-shown', () => {
    const currentRouteName = router.currentRoute.value.name
    if (sessionStore.isSessionActive) {
      // Don't redirect if already on an active-session sub-view
      if (!activeSessionSubRoutes.has(currentRouteName as string)) {
        router.push({ name: 'active-session' })
      }
    } else {
      if (currentRouteName !== 'home' && currentRouteName !== 'session-review' && currentRouteName !== 'settings') {
        router.push({ name: 'home' })
      }
    }
  })

  // Listen for global hotkey events from Rust HotkeyManager
  const unlistenHotkeyToggleSession = await listen('hotkey-toggle-session', async () => {
    if (sessionStore.isSessionActive && sessionStore.activeSessionId) {
      try {
        await sessionStore.endSession(sessionStore.activeSessionId)
        await trayStore.setIdle()
      } catch (err) {
        console.error('Failed to end session via hotkey:', err)
        $q.notify({
          type: 'negative',
          message: 'Failed to end session',
          caption: err instanceof Error ? err.message : String(err),
          position: 'bottom-right',
          timeout: 5000,
        })
      }
    } else {
      try {
        await sessionStore.startSession()
        await trayStore.setActive()
      } catch (err) {
        console.error('Failed to start session via hotkey:', err)
        // If the backend rejected start because a session is already active,
        // the frontend state is stale. Re-sync from the backend.
        const errMsg = err instanceof Error ? err.message : String(err)
        if (errMsg.includes('already active')) {
          try {
            await sessionStore.loadActiveSession()
          } catch (syncErr) {
            console.error('Failed to re-sync session state after start failure:', syncErr)
          }
        }
        $q.notify({
          type: 'negative',
          message: 'Failed to start session',
          caption: errMsg,
          position: 'bottom-right',
          timeout: 5000,
        })
      }
    }
  })

  const unlistenHotkeyStartBugCapture = await listen('hotkey-start-bug-capture', async () => {
    if (sessionStore.isSessionActive && sessionStore.activeSessionId) {
      try {
        const bug = await bugStore.startBugCapture({ session_id: sessionStore.activeSessionId, status: 'capturing' })
        await trayStore.setBugCapture(bug.display_id)
        router.push({ name: 'active-session' })
      } catch (err) {
        console.error('Failed to start bug capture via hotkey:', err)
        $q.notify({
          type: 'negative',
          message: 'Failed to start bug capture',
          caption: err instanceof Error ? err.message : String(err),
          position: 'bottom-right',
          timeout: 5000,
        })
      }
    } else {
      $q.notify({
        type: 'warning',
        message: 'No active session',
        caption: 'Start a session before capturing bugs',
        position: 'bottom-right',
        timeout: 3000,
      })
    }
  })

  const unlistenHotkeyEndBugCapture = await listen('hotkey-end-bug-capture', async () => {
    if (bugStore.activeBug?.id) {
      try {
        await bugStore.completeBugCapture(bugStore.activeBug.id)
        await trayStore.setActive()
      } catch (err) {
        console.error('Failed to complete bug capture via hotkey:', err)
        $q.notify({
          type: 'negative',
          message: 'Failed to complete bug capture',
          caption: err instanceof Error ? err.message : String(err),
          position: 'bottom-right',
          timeout: 5000,
        })
      }
    }
  })

  const unlistenHotkeyOpenQuickNotepad = await listen('hotkey-open-quick-notepad', () => {
    showQuickNotepad.value = !showQuickNotepad.value
  })

  const unlistenHotkeyOpenSessionNotepad = await listen('hotkey-open-session-notepad', async () => {
    try {
      await tauri.openSessionNotesWindow()
    } catch (err) {
      console.error('Failed to open session notes window:', err)
    }
  })

  unlistenHandlers = [
    unlistenScreenshotCaptured,
    unlistenStartSession,
    unlistenNewBug,
    unlistenSettings,
    unlistenTrayEndSession,
    unlistenTrayEndBugCapture,
    unlistenTrayOpenReview,
    unlistenTrayHelp,
    unlistenWindowShown,
    unlistenHotkeyToggleSession,
    unlistenHotkeyStartBugCapture,
    unlistenHotkeyEndBugCapture,
    unlistenHotkeyOpenQuickNotepad,
    unlistenHotkeyOpenSessionNotepad,
  ]
})

onUnmounted(() => {
  // Clean up event listeners
  unlistenHandlers.forEach(unlisten => unlisten())
  sessionStore.cleanupEventListeners()
  bugStore.cleanupEventListeners()
})

// Keep tray state in sync reactively with session/bug store state.
// This ensures the tray is always consistent even if an event handler fails or is skipped.
watch(
  () => [sessionStore.isSessionActive, bugStore.activeBug?.id] as const,
  ([sessionActive, activeBugId], [prevSessionActive, prevActiveBugId]) => {
    // Skip during initialization — tray is set to idle in onMounted before this runs
    if (appInitializing.value) return

    if (activeBugId && activeBugId !== prevActiveBugId) {
      // Bug capture just became active (or changed)
      trayStore.setBugCapture(bugStore.activeBug?.display_id ?? activeBugId).catch(err => {
        console.error('Failed to sync tray to bug state:', err)
      })
    } else if (!activeBugId && prevActiveBugId && sessionActive) {
      // Bug capture ended but session still active
      trayStore.setActive().catch(err => {
        console.error('Failed to sync tray to active state:', err)
      })
    } else if (sessionActive && !prevSessionActive) {
      // Session just became active
      trayStore.setActive().catch(err => {
        console.error('Failed to sync tray to active state:', err)
      })
    } else if (!sessionActive && prevSessionActive) {
      // Session just ended
      trayStore.setIdle().catch(err => {
        console.error('Failed to sync tray to idle state:', err)
      })
    }
  }
)

// Watch for active session changes and navigate accordingly (after init)
// Only navigate on session transitions (null→active or active→null), not on updates
// to an already-active session (e.g. session metadata refreshed via backend events).
watch(
  () => sessionStore.activeSession?.id,
  (newId, oldId) => {
    // Secondary windows manage their own view; don't navigate them
    if (isSecondaryWindow) return

    // Don't navigate during initial load — handled above in onMounted
    if (appInitializing.value) return

    const currentRouteName = router.currentRoute.value.name

    if (newId && !oldId) {
      // Session just started — navigate to active session view
      if (!activeSessionSubRoutes.has(currentRouteName as string)) {
        router.push({ name: 'active-session' })
      }
    } else if (!newId && oldId) {
      // Session ended — navigate to home/idle view
      if (currentRouteName !== 'home' && currentRouteName !== 'session-review') {
        router.push({ name: 'home' })
      }
    }
  }
)
</script>

<style scoped>
.app-init-overlay {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: calc(100vh - 100px);
  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
}

/* Windows 11 Mica/Acrylic effect on header */
:deep(.q-header) {
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  background: rgba(46, 49, 146, 0.92) !important;
}

/* Rounded corners on cards and dialogs for Windows 11 feel */
:deep(.q-card) {
  border-radius: 8px;
}

:deep(.q-dialog .q-card) {
  border-radius: 12px;
}

/* Smooth font rendering — font-family is set by $typography-font-family in quasar-variables.sass */
:deep(*) {
  -webkit-font-smoothing: antialiased;
}
</style>
