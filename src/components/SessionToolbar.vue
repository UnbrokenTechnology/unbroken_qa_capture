<template>
  <q-toolbar
    v-if="sessionStore.isSessionActive"
    class="session-toolbar bg-secondary text-white"
  >
    <q-toolbar-title class="text-subtitle2">
      <div class="row items-center q-gutter-sm">
        <!-- Session Icon -->
        <q-icon
          name="play_circle"
          size="sm"
        />

        <!-- Session ID/Name -->
        <span>Session: {{ sessionDisplayId }}</span>

        <!-- Session Time -->
        <q-chip
          color="primary"
          text-color="white"
          size="sm"
          dense
        >
          <q-icon
            name="schedule"
            size="xs"
            class="q-mr-xs"
          />
          {{ formattedSessionTime }}
        </q-chip>

        <!-- Bug Count -->
        <q-chip
          :color="bugCount > 0 ? 'positive' : 'grey'"
          text-color="white"
          size="sm"
          dense
        >
          <q-icon
            name="bug_report"
            size="xs"
            class="q-mr-xs"
          />
          {{ bugCount }} {{ bugCount === 1 ? 'bug' : 'bugs' }}
        </q-chip>
      </div>
    </q-toolbar-title>

    <q-space />

    <!-- Open Folder Button -->
    <q-btn
      flat
      dense
      icon="folder_open"
      label="Open Folder"
      @click="openSessionFolder"
    >
      <q-tooltip>Open session folder</q-tooltip>
    </q-btn>
  </q-toolbar>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useSessionStore } from '../stores/session'
import { useBugStore } from '../stores/bug'
import { invoke } from '@tauri-apps/api/core'
import { useQuasar } from 'quasar'

// Stores
const sessionStore = useSessionStore()
const bugStore = useBugStore()
const $q = useQuasar()

// State
const sessionStartTime = ref<Date | null>(null)
const elapsedTime = ref(0)
let intervalId: number | null = null

// Computed
const sessionDisplayId = computed(() => {
  if (!sessionStore.activeSession) {
    return 'N/A'
  }
  // Use first 8 characters of session ID for display
  return sessionStore.activeSession.id.substring(0, 8)
})

const formattedSessionTime = computed(() => {
  if (!sessionStore.isSessionActive) {
    return '00:00:00'
  }

  const hours = Math.floor(elapsedTime.value / 3600)
  const minutes = Math.floor((elapsedTime.value % 3600) / 60)
  const seconds = elapsedTime.value % 60

  return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`
})

const bugCount = computed(() => {
  if (!sessionStore.activeSession) {
    return 0
  }
  // Count bugs for active session
  return bugStore.backendBugs.filter(bug => bug.session_id === sessionStore.activeSession?.id).length
})

// Methods
function updateElapsedTime() {
  if (sessionStartTime.value) {
    const now = new Date()
    elapsedTime.value = Math.floor((now.getTime() - sessionStartTime.value.getTime()) / 1000)
  }
}

function startTimer() {
  if (intervalId !== null) {
    clearInterval(intervalId)
  }

  if (sessionStore.activeSession?.started_at) {
    sessionStartTime.value = new Date(sessionStore.activeSession.started_at)
    updateElapsedTime()
    intervalId = window.setInterval(updateElapsedTime, 1000)
  }
}

function stopTimer() {
  if (intervalId !== null) {
    clearInterval(intervalId)
    intervalId = null
  }
  sessionStartTime.value = null
  elapsedTime.value = 0
}

async function openSessionFolder() {
  if (!sessionStore.activeSession?.folder_path) {
    $q.notify({
      type: 'warning',
      message: 'No folder path available for active session',
      position: 'top',
      timeout: 2000
    })
    return
  }

  try {
    await invoke('open_session_folder', {
      folderPath: sessionStore.activeSession.folder_path
    })

    $q.notify({
      type: 'positive',
      message: 'Session folder opened',
      position: 'top',
      timeout: 2000
    })
  } catch (error) {
    $q.notify({
      type: 'negative',
      message: `Failed to open session folder: ${error}`,
      position: 'top',
      timeout: 3000
    })
  }
}

// Lifecycle
onMounted(() => {
  // Start timer if session is active
  if (sessionStore.isSessionActive) {
    startTimer()
  }
})

onUnmounted(() => {
  stopTimer()
})

// Watch for session state changes
watch(() => sessionStore.isSessionActive, (isActive) => {
  if (isActive) {
    startTimer()
  } else {
    stopTimer()
  }
})

watch(() => sessionStore.activeSession?.started_at, () => {
  if (sessionStore.isSessionActive) {
    startTimer()
  }
})
</script>

<style scoped>
.session-toolbar {
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}
</style>
