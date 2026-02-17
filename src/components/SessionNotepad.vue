<template>
  <q-card
    v-if="visible"
    class="session-notepad"
    :style="widgetStyle"
    @mousedown="onCardMousedown"
  >
    <q-card-section class="q-pa-sm">
      <div class="row items-center q-mb-sm drag-handle">
        <div class="col">
          <span class="text-subtitle2">Session Notes</span>
          <span
            v-if="activeSession"
            class="text-caption q-ml-sm text-grey-7"
          >
            {{ sessionDisplayName }}
          </span>
        </div>
        <div class="col-auto">
          <q-chip
            v-if="saveStatus !== 'idle'"
            :color="saveStatusColor"
            text-color="white"
            size="sm"
            dense
          >
            {{ saveStatusLabel }}
          </q-chip>
          <q-btn
            icon="close"
            flat
            dense
            size="sm"
            @click="$emit('close')"
          />
        </div>
      </div>

      <q-input
        v-model="localNotes"
        class="notepad-input"
        type="textarea"
        outlined
        placeholder="Type your session notes here... (Auto-saves)"
        :rows="8"
        :disable="!activeSession"
        @update:model-value="onNotesChanged"
        @mousedown.stop
      />

      <div
        v-if="!activeSession"
        class="text-caption text-grey-7 q-mt-xs"
      >
        No active session. Start a session to take notes.
      </div>
    </q-card-section>
  </q-card>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useSessionStore } from '../stores/session'
import { getCurrentWindow } from '@tauri-apps/api/window'
import * as tauri from '../api/tauri'

// Props
interface Props {
  visible?: boolean
  initialX?: number
  initialY?: number
}

const props = withDefaults(defineProps<Props>(), {
  visible: true,
  initialX: 100,
  initialY: 100
})

// Emits
defineEmits<{
  close: []
}>()

// Stores
const sessionStore = useSessionStore()

// State
const position = ref({ x: props.initialX, y: props.initialY })
const localNotes = ref('')
const saveStatus = ref<'idle' | 'saving' | 'saved' | 'error'>('idle')
const saveError = ref<string | null>(null)
let saveDebounceTimeout: number | null = null
let savedStatusTimeout: number | null = null

// Drag state
let isDragging = false
let dragStartX = 0
let dragStartY = 0
let positionAtDragStart = { x: 0, y: 0 }

// Computed
const widgetStyle = computed(() => ({
  position: 'fixed',
  top: `${position.value.y}px`,
  left: `${position.value.x}px`,
  zIndex: 9999,
  minWidth: '400px',
  maxWidth: '600px'
}))

const activeSession = computed(() => sessionStore.activeSession)

const sessionDisplayName = computed(() => {
  if (!activeSession.value) return ''
  const date = new Date(activeSession.value.started_at)
  return date.toLocaleDateString()
})

const saveStatusColor = computed(() => {
  switch (saveStatus.value) {
    case 'saving':
      return 'primary'
    case 'saved':
      return 'positive'
    case 'error':
      return 'negative'
    default:
      return 'grey'
  }
})

const saveStatusLabel = computed(() => {
  switch (saveStatus.value) {
    case 'saving':
      return 'Saving...'
    case 'saved':
      return 'Saved'
    case 'error':
      return 'Error'
    default:
      return ''
  }
})

// Drag handlers
function onCardMousedown(event: MouseEvent) {
  // Only drag from the header area (not from input fields — they stop propagation)
  isDragging = true
  dragStartX = event.clientX
  dragStartY = event.clientY
  positionAtDragStart = { ...position.value }

  document.addEventListener('mousemove', onMousemove)
  document.addEventListener('mouseup', onMouseup)
  event.preventDefault()
}

function onMousemove(event: MouseEvent) {
  if (!isDragging) return
  const dx = event.clientX - dragStartX
  const dy = event.clientY - dragStartY
  position.value = {
    x: positionAtDragStart.x + dx,
    y: positionAtDragStart.y + dy
  }
}

function onMouseup() {
  isDragging = false
  document.removeEventListener('mousemove', onMousemove)
  document.removeEventListener('mouseup', onMouseup)
}

// Methods
async function loadNotes() {
  if (!activeSession.value || !activeSession.value.folder_path || activeSession.value.folder_path.trim() === '') {
    localNotes.value = ''
    return
  }

  try {
    const notes = await tauri.getSessionNotes(
      activeSession.value.id,
      activeSession.value.folder_path
    )
    localNotes.value = notes
  } catch (error) {
    console.error('Failed to load session notes:', error)
    localNotes.value = ''
  }
}

async function saveNotes() {
  if (!activeSession.value || !activeSession.value.folder_path || activeSession.value.folder_path.trim() === '') {
    return
  }

  saveStatus.value = 'saving'
  saveError.value = null

  try {
    await tauri.updateSessionNotes(
      activeSession.value.id,
      activeSession.value.folder_path,
      localNotes.value
    )
    saveStatus.value = 'saved'

    // Clear "Saved" status after 2 seconds
    if (savedStatusTimeout !== null) {
      clearTimeout(savedStatusTimeout)
    }
    savedStatusTimeout = window.setTimeout(() => {
      if (saveStatus.value === 'saved') {
        saveStatus.value = 'idle'
      }
    }, 2000)
  } catch (error) {
    saveStatus.value = 'error'
    saveError.value = error instanceof Error ? error.message : String(error)
    console.error('Failed to save session notes:', error)
  }
}

function onNotesChanged() {
  // Debounce auto-save
  if (saveDebounceTimeout !== null) {
    clearTimeout(saveDebounceTimeout)
  }

  saveDebounceTimeout = window.setTimeout(() => {
    saveNotes()
  }, 500) // 500ms debounce
}

async function setWindowAlwaysOnTop(value: boolean) {
  try {
    const appWindow = getCurrentWindow()
    await appWindow.setAlwaysOnTop(value)
  } catch (error) {
    console.error('Failed to set window always-on-top:', error)
  }
}

// Lifecycle
onMounted(async () => {
  await setWindowAlwaysOnTop(true)
  await loadNotes()
})

onUnmounted(() => {
  if (saveDebounceTimeout !== null) {
    clearTimeout(saveDebounceTimeout)
  }
  if (savedStatusTimeout !== null) {
    clearTimeout(savedStatusTimeout)
  }
  // Clean up drag listeners if somehow left behind
  document.removeEventListener('mousemove', onMousemove)
  document.removeEventListener('mouseup', onMouseup)
})

// Watch visible prop to manage always-on-top state
watch(
  () => props.visible,
  async (isVisible) => {
    await setWindowAlwaysOnTop(isVisible)
  }
)

// Watch for active session changes
watch(
  () => activeSession.value,
  async (_newSession, oldSession) => {
    // Save notes for the old session before switching
    if (oldSession && oldSession.folder_path && oldSession.folder_path.trim() !== '' && saveDebounceTimeout !== null) {
      clearTimeout(saveDebounceTimeout)
      // Save with the old session's data
      try {
        await tauri.updateSessionNotes(
          oldSession.id,
          oldSession.folder_path,
          localNotes.value
        )
      } catch (error) {
        console.error('Failed to save notes for old session:', error)
      }
    }

    // Load notes for the new session
    await loadNotes()
  }
)
</script>

<style scoped>
.session-notepad {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  border-radius: 8px;
  cursor: default;
}

.session-notepad:hover {
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.2);
}

.drag-handle {
  cursor: move;
  user-select: none;
}

.notepad-input :deep(textarea) {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 14px;
  line-height: 1.5;
}
</style>
