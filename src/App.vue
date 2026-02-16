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
    </q-header>

    <q-page-container>
      <router-view />
    </q-page-container>

    <!-- Session Status Widget -->
    <SessionStatusWidget
      :visible="showStatusWidget"
      @close="showStatusWidget = false"
    />
  </q-layout>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useRouter } from 'vue-router'
import { useTrayStore } from './stores/tray'
import SessionStatusWidget from './components/SessionStatusWidget.vue'

const router = useRouter()
const trayStore = useTrayStore()
const showStatusWidget = ref(true)

let unlistenHandlers: UnlistenFn[] = []

function toggleStatusWidget() {
  showStatusWidget.value = !showStatusWidget.value
}

onMounted(async () => {
  // Initialize tray to idle state
  await trayStore.setIdle()

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
})
</script>

<style scoped>
</style>
