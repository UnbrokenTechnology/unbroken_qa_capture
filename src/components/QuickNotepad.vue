<template>
  <q-card
    v-if="visible"
    class="quick-notepad"
    :style="widgetStyle"
    @mousedown="onCardMousedown"
  >
    <q-card-section class="q-pa-sm">
      <div class="row items-center q-mb-sm drag-handle">
        <div class="col">
          <span class="text-subtitle2">Quick Notes</span>
          <span
            v-if="activeBug"
            class="text-caption q-ml-sm text-grey-7"
          >
            {{ activeBug.display_id }}
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

      <!-- Metadata Fields -->
      <div class="row q-col-gutter-sm q-mb-sm">
        <div class="col-12">
          <q-input
            v-model="localMeetingId"
            outlined
            dense
            label="Meeting ID / URL"
            placeholder="e.g., Zoom meeting ID or URL"
            :disable="!activeBug"
            @update:model-value="onMeetingIdChanged"
            @mousedown.stop
          >
            <template #prepend>
              <q-icon name="videocam" />
            </template>
          </q-input>
        </div>
        <div class="col-12">
          <q-input
            v-model="localSoftwareVersion"
            outlined
            dense
            label="Software Version"
            placeholder="e.g., 2.4.1"
            :disable="!activeBug"
            @update:model-value="onSoftwareVersionChanged"
            @mousedown.stop
          >
            <template #prepend>
              <q-icon name="info" />
            </template>
          </q-input>
        </div>
      </div>

      <!-- Notes Field -->
      <q-input
        v-model="localNotes"
        class="notepad-input"
        type="textarea"
        outlined
        label="Notes"
        placeholder="Type your notes here... (Auto-saves)"
        :rows="6"
        :disable="!activeBug"
        @update:model-value="onNotesChanged"
        @mousedown.stop
      />

      <!-- Console Tag Toggle -->
      <div class="row items-center q-mt-sm">
        <q-toggle
          :model-value="bugStore.tagNextScreenshotAsConsole"
          :disable="!activeBug"
          label="Tag next screenshot as console"
          dense
          size="sm"
          color="primary"
          @update:model-value="bugStore.setTagNextScreenshotAsConsole($event)"
        />
      </div>

      <div
        v-if="!activeBug"
        class="text-caption text-grey-7 q-mt-xs"
      >
        No active bug. Start capturing a bug to take notes.
      </div>
    </q-card-section>
  </q-card>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useBugStore } from '../stores/bug'
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
const bugStore = useBugStore()

// State
const position = ref({ x: props.initialX, y: props.initialY })
const localNotes = ref('')
const localMeetingId = ref('')
const localSoftwareVersion = ref('')
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

const activeBug = computed(() => bugStore.activeBug)

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
  if (!activeBug.value) {
    localNotes.value = ''
    localMeetingId.value = ''
    localSoftwareVersion.value = ''
    return
  }

  try {
    const notes = await tauri.getBugNotes(
      activeBug.value.id,
      activeBug.value.folder_path
    )
    localNotes.value = notes

    // Pre-populate meeting ID: use this bug's value if set, else carry over from store's lastMeetingId
    if (activeBug.value.meeting_id) {
      localMeetingId.value = activeBug.value.meeting_id
    } else if (bugStore.lastSessionMeetingId) {
      localMeetingId.value = bugStore.lastSessionMeetingId
    } else {
      localMeetingId.value = ''
    }

    localSoftwareVersion.value = activeBug.value.software_version || ''
  } catch (error) {
    console.error('Failed to load notes:', error)
    localNotes.value = ''
    localMeetingId.value = ''
    localSoftwareVersion.value = ''
  }
}

async function saveNotes() {
  if (!activeBug.value) {
    return
  }

  saveStatus.value = 'saving'
  saveError.value = null

  try {
    await tauri.updateBugNotes(
      activeBug.value.id,
      activeBug.value.folder_path,
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
    console.error('Failed to save notes:', error)
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

async function saveMetadata(field: 'meeting_id' | 'software_version', value: string) {
  if (!activeBug.value) {
    return
  }

  saveStatus.value = 'saving'
  saveError.value = null

  try {
    await bugStore.updateBackendBug(activeBug.value.id, {
      [field]: value || null
    })
    // Track meeting ID for pre-population on next bug
    if (field === 'meeting_id' && value) {
      bugStore.setLastSessionMeetingId(value)
    }
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
    console.error('Failed to save metadata:', error)
  }
}

function onMeetingIdChanged() {
  // Debounce auto-save
  if (saveDebounceTimeout !== null) {
    clearTimeout(saveDebounceTimeout)
  }

  saveDebounceTimeout = window.setTimeout(() => {
    saveMetadata('meeting_id', localMeetingId.value)
  }, 500)
}

function onSoftwareVersionChanged() {
  // Debounce auto-save
  if (saveDebounceTimeout !== null) {
    clearTimeout(saveDebounceTimeout)
  }

  saveDebounceTimeout = window.setTimeout(() => {
    saveMetadata('software_version', localSoftwareVersion.value)
  }, 500)
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

// Reset console tag toggle when bug changes (it's per-capture, not persistent)
watch(
  () => activeBug.value?.id,
  () => {
    bugStore.setTagNextScreenshotAsConsole(false)
  }
)

// Watch for active bug changes
watch(
  () => activeBug.value,
  async (_newBug, oldBug) => {
    // Save notes and metadata for the old bug before switching
    if (oldBug && saveDebounceTimeout !== null) {
      clearTimeout(saveDebounceTimeout)
      // Save with the old bug's data
      try {
        await tauri.updateBugNotes(
          oldBug.id,
          oldBug.folder_path,
          localNotes.value
        )

        // Save metadata changes
        const updates: { meeting_id?: string; software_version?: string } = {}
        if (localMeetingId.value !== (oldBug.meeting_id || '')) {
          updates.meeting_id = localMeetingId.value || undefined
          // Track for pre-population
          if (localMeetingId.value) {
            bugStore.setLastSessionMeetingId(localMeetingId.value)
          }
        }
        if (localSoftwareVersion.value !== (oldBug.software_version || '')) {
          updates.software_version = localSoftwareVersion.value || undefined
        }
        if (Object.keys(updates).length > 0) {
          await bugStore.updateBackendBug(oldBug.id, updates)
        }
      } catch (error) {
        console.error('Failed to save notes/metadata for old bug:', error)
      }
    }

    // Load notes and metadata for the new bug
    await loadNotes()
  }
)

// Watch visible prop to manage always-on-top state
watch(
  () => props.visible,
  async (isVisible) => {
    await setWindowAlwaysOnTop(isVisible)
  }
)
</script>

<style scoped>
.quick-notepad {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  border-radius: 8px;
  cursor: default;
}

.quick-notepad:hover {
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
