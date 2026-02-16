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
      </q-toolbar>
    </q-header>

    <q-page-container>
      <router-view />
    </q-page-container>
  </q-layout>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useTrayStore } from './stores/tray'

const trayStore = useTrayStore()

let unlistenHandlers: UnlistenFn[] = []

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
    // Navigate to settings view (when implemented)
    console.log('Settings requested from tray')
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
