<template>
  <div class="session-notes-window">
    <div class="drag-handle row items-center q-px-sm q-py-xs">
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
      </div>
    </div>

    <div class="q-px-sm q-pb-sm">
      <q-input
        v-model="localNotes"
        class="notepad-input"
        type="textarea"
        outlined
        placeholder="Type your session notes here... (Auto-saves)"
        :rows="10"
        :disable="!activeSession"
        @update:model-value="onNotesChanged"
      />

      <div
        v-if="!activeSession"
        class="text-caption text-grey-7 q-mt-xs"
      >
        No active session. Start a session to take notes.
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useSessionStore } from '../stores/session'
import * as tauri from '../api/tauri'

const sessionStore = useSessionStore()

// State
const localNotes = ref('')
const saveStatus = ref<'idle' | 'saving' | 'saved' | 'error'>('idle')
let saveDebounceTimeout: number | null = null
let savedStatusTimeout: number | null = null
let unlistenHandlers: UnlistenFn[] = []

// Computed
const activeSession = computed(() => sessionStore.activeSession)

const sessionDisplayName = computed(() => {
  if (!activeSession.value) return ''
  const date = new Date(activeSession.value.started_at)
  return date.toLocaleDateString()
})

const saveStatusColor = computed(() => {
  switch (saveStatus.value) {
    case 'saving': return 'primary'
    case 'saved': return 'positive'
    case 'error': return 'negative'
    default: return 'grey'
  }
})

const saveStatusLabel = computed(() => {
  switch (saveStatus.value) {
    case 'saving': return 'Saving...'
    case 'saved': return 'Saved'
    case 'error': return 'Error'
    default: return ''
  }
})

// Note I/O
async function loadNotes() {
  if (!activeSession.value?.folder_path?.trim()) {
    localNotes.value = ''
    return
  }
  try {
    localNotes.value = await tauri.getSessionNotes(
      activeSession.value.id,
      activeSession.value.folder_path
    )
  } catch (error) {
    console.error('Failed to load session notes:', error)
    localNotes.value = ''
  }
}

async function saveNotes() {
  if (!activeSession.value?.folder_path?.trim()) return

  saveStatus.value = 'saving'
  try {
    await tauri.updateSessionNotes(
      activeSession.value.id,
      activeSession.value.folder_path,
      localNotes.value
    )
    saveStatus.value = 'saved'
    if (savedStatusTimeout !== null) clearTimeout(savedStatusTimeout)
    savedStatusTimeout = window.setTimeout(() => {
      if (saveStatus.value === 'saved') saveStatus.value = 'idle'
    }, 2000)
  } catch (error) {
    saveStatus.value = 'error'
    console.error('Failed to save session notes:', error)
  }
}

function onNotesChanged() {
  if (saveDebounceTimeout !== null) clearTimeout(saveDebounceTimeout)
  saveDebounceTimeout = window.setTimeout(() => { saveNotes() }, 500)
}

// Lifecycle
onMounted(async () => {
  await sessionStore.loadActiveSession()
  await loadNotes()
  await sessionStore.setupEventListeners()

  // Listen for session events that may arrive while this window is open
  const unlistenCreated = await listen('session-created', () => { sessionStore.loadActiveSession() })
  const unlistenUpdated = await listen('session-updated', () => { sessionStore.loadActiveSession() })
  const unlistenDeleted = await listen('session-deleted', () => { sessionStore.loadActiveSession() })
  const unlistenStatusChanged = await listen('session-status-changed', () => { sessionStore.loadActiveSession() })

  // Ensure the native X close button works by explicitly handling the close event
  const appWindow = getCurrentWindow()
  const unlistenClose = await appWindow.onCloseRequested(async () => {
    await appWindow.close()
  })

  unlistenHandlers = [unlistenCreated, unlistenUpdated, unlistenDeleted, unlistenStatusChanged, unlistenClose]
})

onUnmounted(() => {
  if (saveDebounceTimeout !== null) clearTimeout(saveDebounceTimeout)
  if (savedStatusTimeout !== null) clearTimeout(savedStatusTimeout)
  unlistenHandlers.forEach(u => u())
  sessionStore.cleanupEventListeners()
})

// When session changes, save old notes and load new ones
watch(
  () => activeSession.value,
  async (_newSession, oldSession) => {
    if (oldSession?.folder_path?.trim() && saveDebounceTimeout !== null) {
      clearTimeout(saveDebounceTimeout)
      try {
        await tauri.updateSessionNotes(oldSession.id, oldSession.folder_path, localNotes.value)
      } catch (error) {
        console.error('Failed to save notes for old session:', error)
      }
    }
    await loadNotes()
  }
)
</script>

<style scoped>
.session-notes-window {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #ffffff;
}

.drag-handle {
  cursor: move;
  user-select: none;
  background: #f5f5f5;
  border-bottom: 1px solid #e0e0e0;
  min-height: 36px;
  -webkit-app-region: drag;
}

/* Allow buttons/chips in the drag handle to still be clickable */
.drag-handle .q-chip,
.drag-handle button {
  -webkit-app-region: no-drag;
}

.notepad-input :deep(textarea) {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 14px;
  line-height: 1.5;
  resize: none;
}
</style>
