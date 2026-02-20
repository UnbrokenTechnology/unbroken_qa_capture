<template>
  <q-page class="active-session-view q-pa-md">
    <div class="content-wrapper">
      <!-- Session Status Bar (hidden when first-run wizard is active) -->
      <q-card
        v-if="!showFirstRunWizard"
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

      <!-- Capture Folder Path -->
      <q-card
        v-if="captureFolderPath"
        flat
        bordered
        class="capture-folder-card q-mb-md"
      >
        <q-card-section class="q-pa-sm">
          <div class="text-caption text-grey-7 q-mb-xs">
            <q-icon
              name="folder"
              size="xs"
              class="q-mr-xs"
            />
            Save screenshots to this folder:
          </div>
          <div class="row items-center no-wrap">
            <div class="col text-body2 capture-folder-path ellipsis q-mr-sm">
              {{ captureFolderPath }}
              <q-tooltip>{{ captureFolderPath }}</q-tooltip>
            </div>
            <q-btn
              flat
              dense
              round
              size="sm"
              icon="content_copy"
              color="primary"
              @click="copyCaptureFolderPath"
            >
              <q-tooltip>Copy folder path to clipboard</q-tooltip>
            </q-btn>
          </div>
        </q-card-section>
      </q-card>

      <!-- Unsorted Screenshot Tray -->
      <q-card
        v-if="captureStore.unsortedCount > 0"
        flat
        bordered
        class="unsorted-tray q-mb-md"
      >
        <q-card-section class="q-py-sm">
          <div class="row items-center q-mb-sm">
            <q-icon
              name="photo_library"
              color="orange"
              size="sm"
              class="q-mr-xs"
            />
            <span class="text-subtitle2">Unsorted Screenshots ({{ captureStore.unsortedCount }})</span>
          </div>
          <div
            class="row q-gutter-sm"
            style="overflow-x: auto; flex-wrap: nowrap;"
          >
            <div
              v-for="capture in captureStore.unsortedCaptures"
              :key="capture.id"
              class="unsorted-thumb"
              style="flex-shrink: 0; position: relative;"
            >
              <q-img
                :src="toAssetUrl(capture.file_path)"
                style="width: 120px; height: 80px; border-radius: 4px; cursor: pointer;"
                fit="cover"
              />

              <!-- AI suggestion badge: suggests an existing bug -->
              <q-badge
                v-if="captureStore.getAiSuggestion(capture.id)?.suggestedBugId"
                color="blue"
                floating
                style="cursor: pointer; font-size: 10px; top: 2px; left: 2px;"
                @click="handleAcceptAiSuggestion(capture.id)"
              >
                AI: {{ captureStore.getAiSuggestion(capture.id)?.suggestedBugDisplayId }}
                <q-tooltip>{{ captureStore.getAiSuggestion(capture.id)?.reasoning }}</q-tooltip>
              </q-badge>

              <!-- AI suggestion badge: suggests creating a new bug -->
              <q-badge
                v-else-if="captureStore.getAiSuggestion(capture.id) && !captureStore.getAiSuggestion(capture.id)?.suggestedBugId"
                color="green"
                floating
                style="font-size: 10px; top: 2px; left: 2px;"
              >
                AI: New Bug?
                <q-tooltip>{{ captureStore.getAiSuggestion(capture.id)?.reasoning }}</q-tooltip>
              </q-badge>

              <!-- AI loading spinner while waiting for suggestion -->
              <q-spinner-dots
                v-else-if="captureStore.isAiPending(capture.id)"
                color="white"
                size="16px"
                style="position: absolute; top: 4px; left: 4px;"
              />

              <q-btn-dropdown
                flat
                dense
                icon="label"
                size="xs"
                color="white"
                style="position: absolute; bottom: 2px; right: 2px; background: rgba(0,0,0,0.5); border-radius: 4px;"
              >
                <q-list dense>
                  <q-item
                    v-for="bug in sessionBugs"
                    :key="bug.id"
                    clickable
                    v-close-popup
                    @click="handleAssignCapture(capture.id, bug.id)"
                  >
                    <q-item-section>{{ bug.display_id }}: {{ bug.title || 'Untitled' }}</q-item-section>
                  </q-item>
                  <q-separator />
                  <q-item
                    clickable
                    v-close-popup
                    @click="handleCreateBugFromCapture(capture)"
                  >
                    <q-item-section>
                      <q-item-label>
                        <q-icon
                          name="add"
                          size="xs"
                          class="q-mr-xs"
                        />New Bug from This
                      </q-item-label>
                    </q-item-section>
                  </q-item>
                </q-list>
              </q-btn-dropdown>
            </div>
          </div>
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
            :class="['bug-card', bug.status === 'capturing' ? 'bug-card--active' : '']"
            @click="handleBugClick(bug)"
          >
            <!-- Active capturing indicator strip -->
            <div
              v-if="bug.status === 'capturing'"
              class="active-capture-strip"
            />

            <!-- Thumbnail -->
            <q-item-section
              v-if="getBugThumbnail(bug)"
              thumbnail
              class="bug-thumbnail"
            >
              <img
                :src="getBugThumbnail(bug) || ''"
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
              <q-item-label class="text-subtitle2 row items-center q-gutter-xs">
                <span>{{ bug.display_id }}</span>
                <q-badge
                  v-if="bug.status === 'capturing'"
                  color="red"
                  class="q-ml-xs"
                >
                  <q-icon
                    name="fiber_manual_record"
                    size="10px"
                    class="q-mr-xs recording-dot"
                  />
                  CAPTURING
                </q-badge>
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

            <!-- Notes / Video indicators -->
            <q-item-section side>
              <div class="column items-center q-gutter-xs">
                <q-icon
                  v-if="bug.notes"
                  name="notes"
                  size="xs"
                  color="green-6"
                >
                  <q-tooltip>Has notes</q-tooltip>
                </q-icon>
                <q-icon
                  v-if="(captureStore.bugCaptureCounts.get(bug.id)?.videos ?? 0) > 0"
                  name="videocam"
                  size="xs"
                  color="purple-6"
                >
                  <q-tooltip>Has video</q-tooltip>
                </q-icon>
              </div>
            </q-item-section>

            <!-- Status Badge -->
            <q-item-section side>
              <div class="column items-end q-gutter-xs">
                <q-badge
                  :color="getBugStatusColor(bug.status)"
                  :label="bug.status === 'capturing' ? 'active' : bug.status"
                />
                <!-- "Add Screenshot" quick action when no bug is capturing -->
                <q-btn
                  v-if="!bugStore.isCapturing && bug.status !== 'capturing'"
                  flat
                  dense
                  size="xs"
                  icon="add_a_photo"
                  color="primary"
                  @click.stop="handleResumeCapture(bug)"
                >
                  <q-tooltip>Add more screenshots to this bug</q-tooltip>
                </q-btn>
              </div>
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
        >
          <q-tooltip>Start capturing a new bug (Ctrl+Alt+B)</q-tooltip>
        </q-btn>
        <q-btn
          unelevated
          color="secondary"
          icon="stop"
          label="End Bug Capture"
          class="full-width q-mb-sm"
          :disable="!bugStore.isCapturing"
          @click="handleEndBugCapture"
        >
          <q-tooltip>End the current bug capture (Ctrl+Alt+E)</q-tooltip>
        </q-btn>
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
import { ref, computed, onMounted, onUnmounted, watch, inject, type Ref } from 'vue'
import { useRouter } from 'vue-router'
import { useQuasar } from 'quasar'
import { useSessionStore } from '@/stores/session'
import { useBugStore } from '@/stores/bug'
import { useCaptureStore } from '@/stores/capture'
import SessionNotepad from '@/components/SessionNotepad.vue'
import { getCaptureFolderPath, getClaudeStatus } from '@/api/tauri'
import type { Bug as BackendBug, Capture } from '@/types/backend'
import { toAssetUrl } from '@/utils/paths'

const router = useRouter()
const $q = useQuasar()
const sessionStore = useSessionStore()
const bugStore = useBugStore()
const captureStore = useCaptureStore()

const showFirstRunWizard = inject<Ref<boolean>>('showFirstRunWizard', ref(false))

// Local state
const notepadExpanded = ref(false)
const sessionDuration = ref('00:00')
const captureFolderPath = ref<string>('')

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

function getBugThumbnail(bug: BackendBug): string | null {
  return captureStore.bugCaptureCounts.get(bug.id)?.thumbnail ?? null
}

function getCaptureCountText(bug: BackendBug): string {
  const counts = captureStore.bugCaptureCounts.get(bug.id)
  if (!counts) return 'Loading...'

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
  if (!activeSession.value) {
    $q.notify({ type: 'warning', message: 'No active session', position: 'bottom-right' })
    return
  }

  try {
    await bugStore.startBugCapture({
      session_id: activeSession.value.id,
      status: 'capturing',
    })
  } catch (error) {
    console.error('Failed to start bug capture:', error)
    $q.notify({
      type: 'negative',
      message: 'Failed to start bug capture',
      caption: error instanceof Error ? error.message : String(error),
      position: 'bottom-right',
      timeout: 5000,
    })
  }
}

async function handleEndBugCapture() {
  if (!bugStore.activeBug) {
    $q.notify({ type: 'warning', message: 'No active bug capture', position: 'bottom-right' })
    return
  }

  try {
    await bugStore.completeBugCapture(bugStore.activeBug.id)
  } catch (error) {
    console.error('Failed to end bug capture:', error)
    $q.notify({
      type: 'negative',
      message: 'Failed to complete bug capture',
      caption: error instanceof Error ? error.message : String(error),
      position: 'bottom-right',
      timeout: 5000,
    })
  }
}

async function handleEndSession() {
  if (!activeSession.value) {
    $q.notify({ type: 'warning', message: 'No active session', position: 'bottom-right' })
    return
  }

  try {
    const sessionId = activeSession.value.id
    await sessionStore.endSession(sessionId)
    // Navigation will be handled by App.vue watching activeSession
    router.push({ name: 'session-review', params: { id: sessionId } })
  } catch (error) {
    console.error('Failed to end session:', error)
    $q.notify({
      type: 'negative',
      message: 'Failed to end session',
      caption: error instanceof Error ? error.message : String(error),
      position: 'bottom-right',
      timeout: 5000,
    })
  }
}

function handleBugClick(bug: BackendBug) {
  // Navigate to bug detail view
  router.push({ name: 'bug-detail', params: { id: bug.id } })
}

async function handleResumeCapture(bug: BackendBug) {
  // End any currently active capture first
  if (bugStore.isCapturing && bugStore.activeBug) {
    try {
      await bugStore.completeBugCapture(bugStore.activeBug.id)
    } catch (err) {
      console.error('Failed to complete current bug capture:', err)
    }
  }
  try {
    await bugStore.resumeBugCapture(bug)
    $q.notify({
      type: 'positive',
      icon: 'add_a_photo',
      message: `Now capturing screenshots for ${bug.display_id}`,
      caption: 'Press Print Screen to take screenshots, then click "End Bug Capture" when done',
      position: 'top',
      timeout: 5000,
    })
  } catch {
    $q.notify({
      type: 'negative',
      message: 'Failed to resume capture for bug',
      position: 'bottom-right',
      timeout: 3000,
    })
  }
}

async function loadCaptureFolderPath() {
  if (!activeSession.value?.folder_path) {
    captureFolderPath.value = ''
    return
  }
  try {
    captureFolderPath.value = await getCaptureFolderPath(activeSession.value.folder_path)
  } catch {
    captureFolderPath.value = ''
  }
}

async function copyCaptureFolderPath() {
  if (!captureFolderPath.value) return
  try {
    await navigator.clipboard.writeText(captureFolderPath.value)
    $q.notify({
      type: 'positive',
      icon: 'content_copy',
      message: 'Folder path copied to clipboard',
      position: 'top',
      timeout: 2000,
    })
  } catch {
    $q.notify({
      type: 'negative',
      message: 'Failed to copy to clipboard',
      position: 'bottom-right',
      timeout: 3000,
    })
  }
}

async function loadSessionBugs() {
  if (!activeSession.value) {
    return
  }

  try {
    await bugStore.loadBugsBySession(activeSession.value.id)
    await captureStore.refreshAll(activeSession.value.id, sessionBugs.value)

    // Recover activeBug after a crash/restart: the Rust backend restores active_bug
    // in resume_session, but Pinia state is ephemeral. If a bug is still in 'capturing'
    // status and we don't have an activeBug reference, restore it from the loaded list.
    if (bugStore.activeBug === null) {
      const capturingBug = sessionBugs.value.find(b => b.status === 'capturing')
      if (capturingBug) {
        bugStore.setActiveBug(capturingBug)
      }
    }
  } catch (err) {
    console.error('Failed to load session bugs:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to load bugs for this session',
      caption: err instanceof Error ? err.message : String(err),
      position: 'bottom-right',
      timeout: 4000,
    })
  }
}

// Lifecycle
onMounted(async () => {
  // Load bugs and capture folder path for the active session
  await Promise.all([loadSessionBugs(), loadCaptureFolderPath()])

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
      await Promise.all([loadSessionBugs(), loadCaptureFolderPath()])
    }
  }
)

// Watch for screenshot capture events to show notifications and update counts
watch(
  () => sessionStore.lastScreenshotEvent,
  async (event) => {
    if (!event) return

    const session = activeSession.value
    if (!session) return

    if (event.bugDisplayId) {
      // Screenshot was captured for an active bug — show success toast
      $q.notify({
        type: 'positive',
        icon: 'photo_camera',
        message: `Screenshot saved to ${event.bugDisplayId}`,
        position: 'top',
        timeout: 3000,
      })
      // Refresh capture counts for the active bug
      const activeBug = bugStore.activeBug
      if (activeBug) {
        await captureStore.loadBugCaptureCounts([activeBug])
      }
    } else {
      // Screenshot taken with no active bug — non-blocking toast instead of modal
      // Refresh unsorted captures so the tray updates
      await captureStore.loadUnsortedCaptures(session.id)

      // Auto-trigger AI suggestion for the newly arrived unsorted capture
      try {
        const claudeStatus = await getClaudeStatus()
        if (claudeStatus.status === 'ready' && bugStore.backendBugs.length > 0) {
          const latest = captureStore.unsortedCaptures[captureStore.unsortedCaptures.length - 1]
          if (latest) {
            void captureStore.requestAiSuggestion(latest.id, session.id)
          }
        }
      } catch {
        // Non-fatal: AI suggestion is best-effort
      }

      $q.notify({
        type: 'warning',
        icon: 'photo_camera',
        message: 'Screenshot saved (unsorted)',
        caption: 'Assign it to a bug from the tray below',
        position: 'top',
        timeout: 5000,
        actions: [
          {
            label: 'New Bug',
            color: 'white',
            handler: () => {
              // Find the most recently added unsorted capture
              const latestCapture = captureStore.unsortedCaptures[captureStore.unsortedCaptures.length - 1]
              if (latestCapture) {
                void handleCreateBugFromCapture(latestCapture)
              }
            },
          },
        ],
      })
    }
  }
)

async function handleAcceptAiSuggestion(captureId: string) {
  try {
    await captureStore.acceptAiSuggestion(captureId)
    // Refresh counts after assignment
    if (sessionStore.activeSession) {
      await captureStore.refreshAll(sessionStore.activeSession.id, bugStore.backendBugs)
    }
  } catch (err) {
    console.error('Failed to accept AI suggestion:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to accept AI suggestion',
      position: 'bottom-right',
      timeout: 4000,
    })
  }
}

async function handleAssignCapture(captureId: string, bugId: string) {
  try {
    await captureStore.assignCaptureToBug(captureId, bugId)
    // Refresh the bug's capture count
    const bug = sessionBugs.value.find(b => b.id === bugId)
    if (bug) {
      await captureStore.loadBugCaptureCounts([bug])
      $q.notify({
        type: 'positive',
        icon: 'photo_camera',
        message: `Screenshot assigned to ${bug.display_id}`,
        position: 'top',
        timeout: 3000,
      })
    }
  } catch (err) {
    console.error('Failed to assign capture to bug:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to assign screenshot to bug',
      position: 'bottom-right',
      timeout: 4000,
    })
  }
}

async function handleCreateBugFromCapture(capture: Capture) {
  if (!activeSession.value) return

  try {
    // Create a new bug, then assign the capture to it
    const newBug = await bugStore.startBugCapture({
      session_id: activeSession.value.id,
      status: 'capturing',
    })
    await captureStore.assignCaptureToBug(capture.id, newBug.id)
    await bugStore.completeBugCapture(newBug.id)
    await captureStore.loadBugCaptureCounts([newBug])
    $q.notify({
      type: 'positive',
      icon: 'bug_report',
      message: `New bug ${newBug.display_id} created with screenshot`,
      position: 'top',
      timeout: 3000,
    })
  } catch (err) {
    console.error('Failed to create bug from capture:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to create bug from screenshot',
      caption: err instanceof Error ? err.message : String(err),
      position: 'bottom-right',
      timeout: 4000,
    })
  }
}
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
  border-radius: 8px;
  animation: fadeIn 0.3s ease-in;
}

.capture-folder-card {
  background: white;
  border-radius: 8px;
  animation: fadeIn 0.3s ease-in;
}

.capture-folder-path {
  font-family: monospace;
  font-size: 0.8rem;
  color: #444;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.unsorted-tray {
  background: white;
  border-radius: 8px;
  animation: fadeIn 0.3s ease-in;
}

.unsorted-thumb {
  transition: transform 0.15s ease;
}

.unsorted-thumb:hover {
  transform: scale(1.05);
}

.bug-list-section {
  animation: slideUp 0.3s ease-out;
}

.bug-cards-list {
  background: white;
  border-radius: 8px;
  max-height: 400px;
  overflow-y: auto;
  /* Smooth scrolling for 30-bug list performance */
  scroll-behavior: smooth;
  -webkit-overflow-scrolling: touch;
}

.bug-card {
  transition: background-color 0.15s ease;
  position: relative;
  overflow: hidden;
}

.bug-card:hover {
  background-color: #f5f5f5;
}

.bug-card--active {
  background-color: #fff8f8;
  border-left: 3px solid #f44336;
}

.bug-card--active:hover {
  background-color: #fff0f0;
}

.active-capture-strip {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 4px;
  background-color: #f44336;
  animation: pulse-strip 1.5s ease-in-out infinite;
}

@keyframes pulse-strip {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

.recording-dot {
  animation: pulse-dot 1.5s ease-in-out infinite;
}

@keyframes pulse-dot {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
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
  border-radius: 8px;
  animation: fadeIn 0.5s ease-in;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes slideUp {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

/* Compact mode: 400x500px */
@media (max-width: 440px) {
  .content-wrapper {
    padding: 0.5rem 0.25rem;
  }
  .bug-thumbnail,
  .bug-thumbnail-placeholder {
    width: 50px;
    height: 38px;
    min-width: 50px;
  }
  .bug-cards-list {
    max-height: 260px;
  }
  .status-bar .q-card-section {
    flex-wrap: wrap;
    gap: 4px;
  }
}

/* Comfortable mode: 600x800px */
@media (min-width: 441px) and (max-width: 660px) {
  .content-wrapper {
    padding: 0.75rem;
  }
  .bug-thumbnail,
  .bug-thumbnail-placeholder {
    width: 60px;
    height: 45px;
    min-width: 60px;
  }
  .bug-cards-list {
    max-height: 340px;
  }
}

/* Full mode: 1000x800px+ */
@media (min-width: 1000px) {
  .content-wrapper {
    max-width: 900px;
    padding: 1.5rem 2rem;
  }
  .bug-cards-list {
    max-height: 500px;
  }
}
</style>
