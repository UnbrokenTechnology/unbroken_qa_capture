<template>
  <q-card
    v-if="visible"
    class="session-status-widget"
    :style="widgetStyle"
  >
    <q-card-section class="q-pa-sm">
      <div class="row items-center q-gutter-xs">
        <!-- Session Time -->
        <div class="col-auto">
          <q-icon
            name="schedule"
            size="sm"
          />
          <span class="q-ml-xs text-caption">{{ formattedSessionTime }}</span>
        </div>

        <!-- Bug Count -->
        <div class="col-auto">
          <q-badge
            :color="bugCount > 0 ? 'positive' : 'grey'"
            :label="bugCount"
          />
          <span class="q-ml-xs text-caption">bugs</span>
        </div>

        <!-- Current State -->
        <div class="col-auto">
          <q-chip
            :color="stateColor"
            text-color="white"
            size="sm"
            dense
          >
            {{ currentStateLabel }}
          </q-chip>
        </div>

        <!-- Close button -->
        <div class="col-auto">
          <q-btn
            icon="close"
            flat
            dense
            size="sm"
            @click="$emit('close')"
          />
        </div>
      </div>
    </q-card-section>
  </q-card>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useSessionStore } from '../stores/session'
import { useBugStore } from '../stores/bug'
import { getCurrentWindow } from '@tauri-apps/api/window'

// Props
interface Props {
  visible?: boolean
  initialX?: number
  initialY?: number
}

const props = withDefaults(defineProps<Props>(), {
  visible: true,
  initialX: 20,
  initialY: 20
})

// Emits
defineEmits<{
  close: []
}>()

// Stores
const sessionStore = useSessionStore()
const bugStore = useBugStore()

// State
const position = ref({ x: props.initialX, y: props.initialY })
const sessionStartTime = ref<Date | null>(null)
const elapsedTime = ref(0)
let intervalId: number | null = null

// Computed
const widgetStyle = computed(() => ({
  position: 'fixed',
  top: `${position.value.y}px`,
  left: `${position.value.x}px`,
  zIndex: 9999,
  cursor: 'move',
  minWidth: '300px'
}))

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
  // Count bugs for active session
  return bugStore.backendBugs.filter(bug => bug.session_id === sessionStore.activeSession?.id).length
})

const currentStateLabel = computed(() => {
  if (!sessionStore.isSessionActive) {
    return 'Idle'
  }

  if (bugStore.isCapturing && bugStore.activeBug) {
    return `Capturing Bug-${bugStore.activeBug.bug_number}`
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

async function setupDraggable() {
  try {
    const appWindow = getCurrentWindow()
    await appWindow.setAlwaysOnTop(true)
  } catch (error) {
    console.error('Failed to set window always-on-top:', error)
  }
}

// Lifecycle
onMounted(async () => {
  // Setup stores
  await sessionStore.setupEventListeners()
  await bugStore.setupEventListeners()

  // Load active session
  await sessionStore.loadActiveSession()

  // Start timer if session is active
  if (sessionStore.isSessionActive) {
    startTimer()
  }

  // Setup window properties
  await setupDraggable()
})

onUnmounted(() => {
  stopTimer()
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
.session-status-widget {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  border-radius: 8px;
}

.session-status-widget:hover {
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.2);
}
</style>
