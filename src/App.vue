<template>
  <q-layout view="lHh Lpr lFf">
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
  </q-layout>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, provide } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useRouter } from 'vue-router'
import { useTrayStore } from './stores/tray'
import { useSessionStore } from './stores/session'
import { useBugStore } from './stores/bug'
import { useSettingsStore } from './stores/settings'
import SessionStatusWidget from './components/SessionStatusWidget.vue'
import SessionToolbar from './components/SessionToolbar.vue'
import FirstRunWizard from './components/FirstRunWizard.vue'
import * as tauri from './api/tauri'

const router = useRouter()
const trayStore = useTrayStore()
const sessionStore = useSessionStore()
const bugStore = useBugStore()
const settingsStore = useSettingsStore()
const showStatusWidget = ref(true)
const showFirstRunWizard = ref(false)
// True until we've finished checking for an active session on startup
const appInitializing = ref(true)
provide('showFirstRunWizard', showFirstRunWizard)

let unlistenHandlers: UnlistenFn[] = []

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

  // Now that we know whether a session is active, perform initial navigation
  if (sessionStore.isSessionActive) {
    router.push({ name: 'active-session' })
  }

  // Reveal the UI now that initialization is complete
  appInitializing.value = false

  // Setup session event listeners
  await sessionStore.setupEventListeners()

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
      }
    }
  })

  const unlistenNewBug = await listen('tray-menu-new-bug', async () => {
    if (sessionStore.isSessionActive && sessionStore.activeSessionId) {
      // Start bug capture in the current session and navigate to it
      try {
        await bugStore.startBugCapture({ session_id: sessionStore.activeSessionId, status: 'capturing' })
        await trayStore.setBugCapture()
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

  unlistenHandlers = [unlistenStartSession, unlistenNewBug, unlistenSettings]
})

onUnmounted(() => {
  // Clean up event listeners
  unlistenHandlers.forEach(unlisten => unlisten())
  sessionStore.cleanupEventListeners()
})

// Watch for active session changes and navigate accordingly (after init)
watch(
  () => sessionStore.activeSession,
  (newSession, oldSession) => {
    // Don't navigate during initial load — handled above in onMounted
    if (appInitializing.value) return

    // Don't navigate if we're already on the correct page
    const currentRouteName = router.currentRoute.value.name

    if (newSession && newSession.status === 'active') {
      // Session started or became active - navigate to active session view
      if (currentRouteName !== 'active-session') {
        router.push({ name: 'active-session' })
      }
    } else if (oldSession && !newSession) {
      // Session ended - navigate to home/idle view
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

/* Smooth font rendering with Segoe UI Variable */
:deep(*) {
  font-family: 'Segoe UI Variable', 'Segoe UI', system-ui, -apple-system, sans-serif;
  -webkit-font-smoothing: antialiased;
}
</style>
