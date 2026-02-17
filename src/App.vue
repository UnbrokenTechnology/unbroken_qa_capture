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
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useRouter } from 'vue-router'
import { useTrayStore } from './stores/tray'
import { useSessionStore } from './stores/session'
import SessionStatusWidget from './components/SessionStatusWidget.vue'
import SessionToolbar from './components/SessionToolbar.vue'
import FirstRunWizard from './components/FirstRunWizard.vue'
import * as tauri from './api/tauri'

const router = useRouter()
const trayStore = useTrayStore()
const sessionStore = useSessionStore()
const showStatusWidget = ref(true)
const showFirstRunWizard = ref(false)

let unlistenHandlers: UnlistenFn[] = []

function toggleStatusWidget() {
  showStatusWidget.value = !showStatusWidget.value
}

function onSetupComplete() {
  console.log('First-run setup completed')
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
</style>
