<template>
  <q-page class="active-session-view q-pa-md">
    <div class="content-wrapper">
      <!-- Session Status Bar -->
      <q-card
        flat
        bordered
        class="status-bar q-mb-md"
      >
        <q-card-section class="row items-center q-pa-sm">
          <div class="col-auto q-mr-md">
            <q-icon
              name="schedule"
              size="sm"
              color="primary"
            />
            <span class="text-body2 q-ml-xs">{{ sessionDuration }}</span>
          </div>

          <div class="col-auto q-mr-md">
            <q-icon
              name="bug_report"
              size="sm"
              color="primary"
            />
            <span class="text-body2 q-ml-xs">{{ bugCount }} bug{{ bugCount !== 1 ? 's' : '' }}</span>
          </div>

          <q-space />

          <q-badge
            :color="currentStateBadgeColor"
            :label="currentStateLabel"
          />
        </q-card-section>
      </q-card>

      <!-- Bug Card List -->
      <div class="bug-list-section q-mb-md">
        <div class="section-header q-mb-sm">
          <h6 class="text-h6 q-my-none">
            Captured Bugs
          </h6>
        </div>

        <div
          v-if="bugStore.loading"
          class="text-center q-pa-md"
        >
          <q-spinner
            color="primary"
            size="3em"
          />
        </div>

        <div
          v-else-if="sessionBugs.length === 0"
          class="no-bugs text-center q-pa-lg"
        >
          <q-icon
            name="inbox"
            size="48px"
            color="grey-5"
            class="q-mb-sm"
          />
          <p class="text-body2 text-grey-7">
            No bugs captured yet
          </p>
          <p class="text-caption text-grey-6">
            Press Print Screen to start capturing a bug
          </p>
        </div>

        <q-list
          v-else
          bordered
          separator
          class="bug-cards-list rounded-borders"
        >
          <q-item
            v-for="bug in sessionBugs"
            :key="bug.id"
            clickable
            class="bug-card"
            @click="handleBugClick(bug)"
          >
            <!-- Thumbnail -->
            <q-item-section
              v-if="getBugThumbnail()"
              thumbnail
              class="bug-thumbnail"
            >
              <img
                :src="getBugThumbnail() || ''"
                alt="Bug screenshot"
              >
            </q-item-section>
            <q-item-section
              v-else
              thumbnail
              class="bug-thumbnail-placeholder"
            >
              <q-icon
                name="image"
                size="md"
                color="grey-5"
              />
            </q-item-section>

            <!-- Bug Info -->
            <q-item-section>
              <q-item-label class="text-subtitle2">
                {{ bug.display_id }}
              </q-item-label>
              <q-item-label caption>
                {{ getCaptureCountText(bug) }}
              </q-item-label>
              <q-item-label
                v-if="bug.notes"
                caption
                class="text-grey-7 notes-preview"
              >
                {{ getNotesPreview(bug.notes) }}
              </q-item-label>
            </q-item-section>

            <!-- Status Badge -->
            <q-item-section side>
              <q-badge
                :color="getBugStatusColor(bug.status)"
                :label="bug.status"
              />
            </q-item-section>

            <q-item-section side>
              <q-icon
                name="chevron_right"
                color="grey-7"
              />
            </q-item-section>
          </q-item>
        </q-list>
      </div>

      <!-- Session Controls -->
      <div class="session-controls q-mb-md">
        <q-btn
          unelevated
          color="primary"
          icon="add"
          label="New Bug Capture"
          class="full-width q-mb-sm"
          @click="handleNewBugCapture"
        />
        <q-btn
          unelevated
          color="secondary"
          icon="stop"
          label="End Bug Capture"
          class="full-width q-mb-sm"
          :disable="!bugStore.isCapturing"
          @click="handleEndBugCapture"
        />
        <q-btn
          unelevated
          color="orange"
          icon="done"
          label="End Session"
          class="full-width"
          @click="handleEndSession"
        />
      </div>

      <!-- Session Notepad (Collapsible) -->
      <q-expansion-item
        v-model="notepadExpanded"
        icon="notes"
        label="Session Notes"
        class="session-notepad-expansion rounded-borders"
        bordered
      >
        <q-card>
          <q-card-section>
            <SessionNotepad :visible="true" />
          </q-card-section>
        </q-card>
      </q-expansion-item>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useSessionStore } from '@/stores/session'
import { useBugStore } from '@/stores/bug'
import SessionNotepad from '@/components/SessionNotepad.vue'
import type { Bug as BackendBug } from '@/types/backend'

const router = useRouter()
const sessionStore = useSessionStore()
const bugStore = useBugStore()

// Local state
const notepadExpanded = ref(false)
const sessionDuration = ref('00:00')
const bugCaptureCounts = ref<Map<string, { screenshots: number; videos: number }>>(new Map())

let durationInterval: number | null = null

// Computed
const activeSession = computed(() => sessionStore.activeSession)
const sessionBugs = computed(() => bugStore.backendBugs)
const bugCount = computed(() => sessionBugs.value.length)

const currentStateLabel = computed(() => {
  if (bugStore.isCapturing) {
    const activeBug = bugStore.activeBug
    return activeBug ? `Capturing ${activeBug.display_id}` : 'Bug Capture Mode'
  }
  return 'QA Mode'
})

const currentStateBadgeColor = computed(() => {
  return bugStore.isCapturing ? 'red' : 'green'
})

// Methods
function updateSessionDuration() {
  if (!activeSession.value || !activeSession.value.started_at) {
    sessionDuration.value = '00:00'
    return
  }

  try {
    const startTime = new Date(activeSession.value.started_at)
    const now = new Date()
    const durationMs = now.getTime() - startTime.getTime()

    const totalSeconds = Math.floor(durationMs / 1000)
    const hours = Math.floor(totalSeconds / 3600)
    const minutes = Math.floor((totalSeconds % 3600) / 60)
    const seconds = totalSeconds % 60

    if (hours > 0) {
      sessionDuration.value = `${hours}:${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`
    } else {
      sessionDuration.value = `${minutes}:${String(seconds).padStart(2, '0')}`
    }
  } catch (error) {
    console.error('Failed to calculate session duration:', error)
    sessionDuration.value = '00:00'
  }
}

function getBugThumbnail(): string | null {
  // In the real implementation, this would fetch the first screenshot from captures
  // For now, return null (will show placeholder icon)
  return null
}

function getCaptureCountText(bug: BackendBug): string {
  // In a full implementation, we'd fetch captures and count them
  // For now, return a placeholder
  const counts = bugCaptureCounts.value.get(bug.id) || { screenshots: 0, videos: 0 }

  const parts: string[] = []
  if (counts.screenshots > 0) {
    parts.push(`${counts.screenshots} screenshot${counts.screenshots !== 1 ? 's' : ''}`)
  }
  if (counts.videos > 0) {
    parts.push(`${counts.videos} video${counts.videos !== 1 ? 's' : ''}`)
  }

  return parts.length > 0 ? parts.join(', ') : 'No captures yet'
}

function getNotesPreview(notes: string): string {
  if (!notes) return ''
  return notes.length > 50 ? notes.substring(0, 50) + '...' : notes
}

function getBugStatusColor(status: string): string {
  switch (status) {
    case 'capturing':
      return 'red'
    case 'captured':
      return 'orange'
    case 'reviewed':
      return 'blue'
    case 'ready':
      return 'green'
    default:
      return 'grey'
  }
}

async function handleNewBugCapture() {
  try {
    // Start a new bug capture
    if (!activeSession.value) {
      console.error('No active session')
      return
    }

    await bugStore.startBugCapture({
      session_id: activeSession.value.id,
      status: 'capturing',
    })
  } catch (error) {
    console.error('Failed to start bug capture:', error)
  }
}

async function handleEndBugCapture() {
  try {
    if (!bugStore.activeBug) {
      console.error('No active bug capture')
      return
    }

    await bugStore.completeBugCapture(bugStore.activeBug.id)
  } catch (error) {
    console.error('Failed to end bug capture:', error)
  }
}

async function handleEndSession() {
  try {
    if (!activeSession.value) {
      console.error('No active session')
      return
    }

    await sessionStore.endSession(activeSession.value.id)
    // Navigation will be handled by App.vue watching activeSession
    router.push({ name: 'session-review', params: { id: activeSession.value.id } })
  } catch (error) {
    console.error('Failed to end session:', error)
  }
}

function handleBugClick(bug: BackendBug) {
  // Navigate to bug detail view
  router.push({ name: 'bug-detail', params: { id: bug.id } })
}

async function loadSessionBugs() {
  if (!activeSession.value) {
    return
  }

  try {
    await bugStore.loadBugsBySession(activeSession.value.id)
  } catch (error) {
    console.error('Failed to load session bugs:', error)
  }
}

// Lifecycle
onMounted(async () => {
  // Load bugs for the active session
  await loadSessionBugs()

  // Start duration timer
  updateSessionDuration()
  durationInterval = window.setInterval(updateSessionDuration, 1000)

  // Setup event listeners for real-time updates
  await bugStore.setupEventListeners()
})

onUnmounted(() => {
  if (durationInterval !== null) {
    clearInterval(durationInterval)
  }
  bugStore.cleanupEventListeners()
})

// Watch for session changes
watch(
  () => activeSession.value,
  async (newSession) => {
    if (newSession) {
      await loadSessionBugs()
    }
  }
)
</script>

<style scoped>
.active-session-view {
  display: flex;
  justify-content: center;
  align-items: flex-start;
  min-height: 100vh;
  background: linear-gradient(135deg, #f5f7fa 0%, #e3e7ed 100%);
}

.content-wrapper {
  max-width: 600px;
  width: 100%;
  padding: 1rem;
}

.status-bar {
  background: white;
  animation: fadeIn 0.3s ease-in;
}

.bug-list-section {
  animation: slideUp 0.3s ease-out;
}

.bug-cards-list {
  background: white;
  max-height: 400px;
  overflow-y: auto;
}

.bug-card {
  transition: background-color 0.2s ease;
}

.bug-card:hover {
  background-color: #f5f5f5;
}

.bug-thumbnail {
  width: 80px;
  height: 60px;
  min-width: 80px;
}

.bug-thumbnail img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 4px;
}

.bug-thumbnail-placeholder {
  width: 80px;
  height: 60px;
  min-width: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f5f5f5;
  border-radius: 4px;
}

.notes-preview {
  font-style: italic;
  margin-top: 4px;
}

.no-bugs {
  background: white;
  border-radius: 8px;
  border: 1px solid #e0e0e0;
}

.session-controls {
  animation: fadeIn 0.4s ease-in;
}

.session-notepad-expansion {
  background: white;
  animation: fadeIn 0.5s ease-in;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Responsive design for 400x600px minimum */
@media (max-width: 600px) {
  .content-wrapper {
    padding: 0.5rem;
  }

  .status-bar .q-card-section {
    flex-direction: column;
    align-items: flex-start;
  }

  .bug-thumbnail,
  .bug-thumbnail-placeholder {
    width: 60px;
    height: 45px;
    min-width: 60px;
  }
}
</style>
