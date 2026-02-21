<template>
  <div
    class="session-status-bar"
    :class="{ 'is-capturing': bugStore.isCapturing }"
  >
    <div class="drag-handle row items-center no-wrap q-px-sm q-gutter-xs">
      <!-- State indicator dot -->
      <div
        class="state-dot"
        :class="`bg-${stateColor}`"
      />

      <!-- Session Time -->
      <span class="time-display">{{ formattedSessionTime }}</span>

      <!-- Separator -->
      <span class="separator">|</span>

      <!-- Bug Count -->
      <q-badge
        :color="bugCount > 0 ? 'positive' : 'grey-6'"
        :label="`${bugCount} bug${bugCount !== 1 ? 's' : ''}`"
        class="badge-compact"
      />

      <!-- Current State -->
      <q-chip
        :color="stateColor"
        text-color="white"
        size="sm"
        dense
        class="state-chip"
      >
        {{ currentStateLabel }}
      </q-chip>

      <!-- Console Tag Mode Indicator -->
      <q-chip
        v-if="bugStore.tagNextScreenshotAsConsole"
        color="orange"
        text-color="white"
        size="sm"
        dense
        class="state-chip"
      >
        Console
      </q-chip>

      <q-space />

      <!-- Close button -->
      <q-btn
        icon="close"
        flat
        dense
        round
        size="xs"
        color="grey-6"
        class="close-btn"
        @click="closeWindow"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useSessionStore } from '../stores/session'
import { useBugStore } from '../stores/bug'

// Stores
const sessionStore = useSessionStore()
const bugStore = useBugStore()

// State
const sessionStartTime = ref<Date | null>(null)
const elapsedTime = ref(0)
let intervalId: number | null = null
let unlistenHandlers: UnlistenFn[] = []

// Computed
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
    return bugStore.bugCount
  }
  return bugStore.backendBugs.filter(bug => bug.session_id === sessionStore.activeSession?.id).length
})

const currentStateLabel = computed(() => {
  if (!sessionStore.isSessionActive) {
    return 'Idle'
  }

  if (bugStore.isCapturing && bugStore.activeBug) {
    return `Bug-${bugStore.activeBug.bug_number}`
  }

  return 'QA Mode'
})

const stateColor = computed(() => {
  if (!sessionStore.isSessionActive) {
    return 'grey'
  }

  if (bugStore.isCapturing) {
    return 'negative'
  }

  return 'primary'
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

async function closeWindow() {
  const appWindow = getCurrentWindow()
  await appWindow.close()
}

// Lifecycle
onMounted(async () => {
  // Setup stores and event listeners
  await sessionStore.setupEventListeners()
  await bugStore.setupEventListeners()

  // Load active session
  await sessionStore.loadActiveSession()

  // Start timer if session is active
  if (sessionStore.isSessionActive) {
    startTimer()
  }

  // Listen for session events so the widget stays in sync
  const unlistenCreated = await listen('session-created', () => { sessionStore.loadActiveSession() })
  const unlistenUpdated = await listen('session-updated', () => { sessionStore.loadActiveSession() })
  const unlistenDeleted = await listen('session-deleted', () => { sessionStore.loadActiveSession() })
  const unlistenStatusChanged = await listen('session-status-changed', () => { sessionStore.loadActiveSession() })

  // Handle the native close event for the frameless window
  const appWindow = getCurrentWindow()
  const unlistenClose = await appWindow.onCloseRequested(async () => {
    await appWindow.close()
  })

  unlistenHandlers = [unlistenCreated, unlistenUpdated, unlistenDeleted, unlistenStatusChanged, unlistenClose]
})

onUnmounted(() => {
  stopTimer()
  unlistenHandlers.forEach(u => u())
  sessionStore.cleanupEventListeners()
  bugStore.cleanupEventListeners()
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
.session-status-bar {
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  background: rgba(255, 255, 255, 0.95);
  border: 1px solid rgba(0, 0, 0, 0.12);
  border-radius: 8px;
}

.session-status-bar.is-capturing {
  border-color: #c10015;
  box-shadow: 0 0 0 1px rgba(193, 0, 21, 0.3);
}

.drag-handle {
  height: 100%;
  cursor: move;
  user-select: none;
  -webkit-app-region: drag;
}

/* Allow interactive elements to remain clickable */
.drag-handle .q-badge,
.drag-handle .q-chip,
.drag-handle .q-btn,
.drag-handle button {
  -webkit-app-region: no-drag;
}

.state-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.time-display {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  font-weight: 600;
  color: #333;
  letter-spacing: 0.5px;
}

.separator {
  color: #ccc;
  font-size: 12px;
}

.badge-compact {
  font-size: 11px;
  padding: 2px 6px;
}

.state-chip {
  font-size: 11px;
  height: 22px;
}

.close-btn {
  opacity: 0.5;
  transition: opacity 0.15s;
}

.close-btn:hover {
  opacity: 1;
}
</style>
