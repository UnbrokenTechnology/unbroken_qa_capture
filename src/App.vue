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
      <router-view />
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
import { useSettingsStore } from './stores/settings'
import SessionStatusWidget from './components/SessionStatusWidget.vue'
import SessionToolbar from './components/SessionToolbar.vue'
import FirstRunWizard from './components/FirstRunWizard.vue'
import * as tauri from './api/tauri'

const router = useRouter()
const trayStore = useTrayStore()
const sessionStore = useSessionStore()
const settingsStore = useSettingsStore()
const showStatusWidget = ref(true)
const showFirstRunWizard = ref(false)
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

  // Load active session if exists
  await sessionStore.loadActiveSession()

  // Setup session event listeners
  await sessionStore.setupEventListeners()

  // Listen for tray menu events
  const unlistenStartSession = await listen('tray-menu-start-session', () => {
    trayStore.toggleSession()
  })

  const unlistenNewBug = await listen('tray-menu-new-bug', () => {
    // Navigate to bug capture view (when implemented)
    console.log('New bug capture requested from tray')
    trayStore.setBugCapture()
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

// Watch for active session changes and navigate accordingly
watch(
  () => sessionStore.activeSession,
  (newSession, oldSession) => {
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
  },
  { immediate: true }
)
</script>

<style scoped>
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
