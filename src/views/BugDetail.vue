<template>
  <q-page padding>
    <div
      v-if="loading"
      class="flex flex-center q-pa-xl"
    >
      <q-spinner
        color="primary"
        size="50px"
      />
    </div>

    <div
      v-else-if="!bug"
      class="flex flex-center q-pa-xl"
    >
      <div class="text-h6 text-grey">
        Bug not found
      </div>
    </div>

    <div
      v-else
      class="bug-detail"
    >
      <!-- Header with back button and title -->
      <div class="row items-center q-mb-md">
        <q-btn
          flat
          round
          dense
          icon="arrow_back"
          class="q-mr-md"
          @click="goBack"
        />
        <q-input
          v-model="titleDraft"
          class="text-h4 flex-1 bug-title-input"
          borderless
          dense
          placeholder="Enter bug title..."
          @blur="saveTitle"
        />
        <q-badge
          v-if="bug.status === 'capturing'"
          color="red"
          class="q-mr-md capturing-badge"
        >
          <q-icon
            name="fiber_manual_record"
            size="10px"
            class="q-mr-xs"
          />
          CAPTURING
        </q-badge>
        <q-btn
          v-if="bug.status === 'capturing'"
          color="secondary"
          icon="stop"
          label="End Bug Capture"
          class="q-mr-sm"
          :loading="endingCapture"
          @click="handleEndBugCapture"
        >
          <q-tooltip>Stop capturing screenshots for this bug</q-tooltip>
        </q-btn>
        <q-btn
          v-if="sessionStore.activeSession && bug.status !== 'capturing'"
          color="red"
          icon="fiber_manual_record"
          label="Resume Capture"
          class="q-mr-sm"
          @click="handleResumeCaptureForBug"
        >
          <q-tooltip>Resume capturing screenshots for this bug</q-tooltip>
        </q-btn>
        <q-btn
          color="secondary"
          icon="folder_open"
          label="Open Folder"
          class="q-mr-sm"
          @click="openBugFolder"
        >
          <q-tooltip>Open bug folder in file explorer</q-tooltip>
        </q-btn>
        <q-btn
          color="primary"
          icon="content_copy"
          label="Copy to Clipboard"
          :loading="copying"
          @click="copyToClipboard"
        />
      </div>

      <!-- Capture Summary Chips -->
      <div class="row q-gutter-sm q-mb-md">
        <q-chip
          v-if="screenshotCaptures.length > 0"
          icon="photo_camera"
          color="blue-1"
          text-color="blue-9"
          dense
        >
          {{ screenshotCaptures.length }} screenshot{{ screenshotCaptures.length !== 1 ? 's' : '' }}
        </q-chip>
        <q-chip
          v-if="videoCaptures.length > 0"
          icon="videocam"
          color="purple-1"
          text-color="purple-9"
          dense
        >
          {{ videoCaptures.length }} video{{ videoCaptures.length !== 1 ? 's' : '' }}
        </q-chip>
        <q-chip
          v-if="consoleParsed"
          icon="terminal"
          color="orange-1"
          text-color="orange-9"
          dense
        >
          Console parsed
        </q-chip>
        <q-chip
          v-if="bug.notes"
          icon="notes"
          color="green-1"
          text-color="green-9"
          dense
        >
          Has notes
        </q-chip>
        <q-chip
          :color="getBugStatusColor(bug.status)"
          text-color="white"
          dense
        >
          {{ bug.status }}
        </q-chip>
      </div>

      <!-- Bug Metadata Card -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            Bug Information
          </div>
          <div class="row q-col-gutter-md">
            <div class="col-12 col-md-6">
              <div class="text-caption text-grey-7">
                Type
              </div>
              <div class="text-body1">
                {{ bug.type }}
              </div>
            </div>
            <div class="col-12 col-md-6">
              <div class="text-caption text-grey-7">
                Folder
              </div>
              <div class="text-body1">
                {{ bug.folder_path }}
              </div>
            </div>
            <div
              v-if="bug.software_version"
              class="col-12 col-md-6"
            >
              <div class="text-caption text-grey-7">
                Software Version
              </div>
              <div class="text-body1">
                {{ bug.software_version }}
              </div>
            </div>
            <div
              v-if="bug.meeting_id"
              class="col-12 col-md-6"
            >
              <div class="text-caption text-grey-7">
                Meeting ID
              </div>
              <div
                v-if="isMeetingUrl(bug.meeting_id)"
                class="text-body1"
              >
                <a
                  href="#"
                  class="text-primary"
                  style="text-decoration: none;"
                  @click.prevent="openMeetingUrl(bug.meeting_id!)"
                >
                  <q-icon
                    name="open_in_new"
                    size="xs"
                    class="q-mr-xs"
                  />{{ bug.meeting_id }}
                </a>
              </div>
              <div
                v-else
                class="text-body1"
              >
                {{ bug.meeting_id }}
              </div>
            </div>
          </div>
        </q-card-section>
      </q-card>

      <!-- Description Card -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            Description
          </div>

          <q-input
            v-model="descriptionDraft"
            type="textarea"
            outlined
            autogrow
            :min-rows="3"
            placeholder="Describe the bug..."
            @blur="saveDescription"
          />

          <div
            v-if="bug.notes"
            class="q-mt-md"
          >
            <div class="text-subtitle2 text-grey-7 q-mb-sm">
              Notes
            </div>
            <div class="text-body2 whitespace-pre-wrap">
              {{ bug.notes }}
            </div>
          </div>

          <div
            v-if="bug.ai_description"
            class="q-mt-md"
          >
            <div class="text-subtitle2 text-primary q-mb-sm">
              AI Summary
            </div>
            <div class="text-body2 whitespace-pre-wrap">
              {{ bug.ai_description }}
            </div>
          </div>
        </q-card-section>
      </q-card>

      <!-- Captures Timeline Card -->
      <q-card
        v-if="allCaptures.length > 0"
        class="q-mb-md"
      >
        <q-card-section>
          <div class="row items-center q-mb-md">
            <q-icon
              name="timeline"
              size="sm"
              color="primary"
              class="q-mr-sm"
            />
            <span class="text-h6">All Captures ({{ allCaptures.length }})</span>
            <q-space />
            <q-btn-toggle
              v-model="captureViewMode"
              dense
              flat
              :options="[
                { label: 'Timeline', value: 'timeline' },
                { label: 'Carousel', value: 'carousel' },
              ]"
              color="primary"
            />
          </div>

          <!-- Timeline view -->
          <div v-if="captureViewMode === 'timeline'">
            <div
              v-for="(capture, index) in allCaptures"
              :key="capture.id"
              class="capture-timeline-item q-mb-sm"
            >
              <div class="row items-start q-gutter-sm">
                <!-- Timeline indicator -->
                <div class="col-auto flex column items-center">
                  <q-icon
                    :name="getCaptureIcon(capture)"
                    :color="getCaptureColor(capture)"
                    size="sm"
                  />
                  <div
                    v-if="index < allCaptures.length - 1"
                    class="timeline-connector"
                  />
                </div>

                <!-- Capture content -->
                <div class="col">
                  <div class="row items-center q-gutter-xs q-mb-xs">
                    <q-chip
                      :color="getCaptureColor(capture)"
                      text-color="white"
                      dense
                      size="sm"
                    >
                      {{ getCaptureLabel(capture) }}
                    </q-chip>
                    <span class="text-caption text-grey-6">{{ formatCaptureTime(capture.created_at) }}</span>
                    <q-space />
                    <q-btn
                      flat
                      round
                      dense
                      size="xs"
                      icon="swap_horiz"
                      color="grey-7"
                      @click="openReassignDialog(capture)"
                    >
                      <q-tooltip>Move to a different bug</q-tooltip>
                    </q-btn>
                  </div>

                  <!-- Screenshot preview -->
                  <div v-if="!isVideoPath(capture.file_path) && !capture.is_console_capture">
                    <div class="capture-thumbnail-row">
                      <q-img
                        :src="toAssetUrl(capture.annotated_path || capture.file_path)"
                        :alt="capture.file_name"
                        class="capture-thumbnail rounded-borders"
                        fit="cover"
                        @click="openAnnotator(capture.annotated_path || capture.file_path, getScreenshotIndex(capture))"
                      >
                        <template #error>
                          <div class="absolute-full flex flex-center bg-grey-3">
                            <q-icon
                              name="broken_image"
                              color="grey-6"
                            />
                          </div>
                        </template>
                      </q-img>
                      <div class="row items-center q-gutter-xs q-mt-xs">
                        <q-btn
                          flat
                          size="sm"
                          icon="edit"
                          label="Annotate"
                          color="primary"
                          @click="openAnnotator(capture.annotated_path || capture.file_path, getScreenshotIndex(capture))"
                        />
                        <q-btn
                          flat
                          size="sm"
                          icon="terminal"
                          label="Mark as Console"
                          color="grey-7"
                          @click="toggleConsoleCapture(capture.id, true)"
                        >
                          <q-tooltip>Tag this screenshot as a console capture for error/warning extraction</q-tooltip>
                        </q-btn>
                        <q-badge
                          v-if="capture.annotated_path"
                          color="blue"
                          label="Annotated"
                        />
                      </div>
                    </div>
                  </div>

                  <!-- Video preview -->
                  <div v-else-if="isVideoPath(capture.file_path)">
                    <div class="text-caption text-grey-6 q-mb-xs">
                      {{ capture.file_name }}
                    </div>
                    <VideoPlayer :file-path="capture.file_path" />
                  </div>

                  <!-- Console capture -->
                  <div v-else-if="capture.is_console_capture">
                    <div class="row items-center q-gutter-xs">
                      <div class="text-caption text-grey-6">
                        Console screenshot: {{ capture.file_name }}
                      </div>
                      <q-btn
                        flat
                        size="sm"
                        icon="cancel"
                        label="Unmark"
                        color="orange-7"
                        @click="toggleConsoleCapture(capture.id, false)"
                      >
                        <q-tooltip>Remove console capture tag</q-tooltip>
                      </q-btn>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Carousel view (screenshots only) -->
          <div v-else>
            <div
              v-if="screenshotCaptures.length === 0"
              class="text-body2 text-grey text-center q-py-md"
            >
              No screenshots to show in carousel view
            </div>
            <q-carousel
              v-else
              v-model="currentSlide"
              swipeable
              animated
              navigation
              infinite
              arrows
              control-color="primary"
              height="500px"
              class="bg-grey-2 rounded-borders"
            >
              <q-carousel-slide
                v-for="(capturePath, index) in screenshotCaptures"
                :key="index"
                :name="index"
                class="q-pa-none"
              >
                <div class="full-width full-height flex flex-center screenshot-slide">
                  <q-img
                    :src="toAssetUrl(capturePath)"
                    :alt="`Screenshot ${index + 1}`"
                    fit="contain"
                    class="full-width full-height"
                    :ratio="16/9"
                  >
                    <template #error>
                      <div class="absolute-full flex flex-center bg-grey-3">
                        <div class="text-center">
                          <q-icon
                            name="broken_image"
                            size="64px"
                            color="grey-6"
                          />
                          <div class="text-grey-7 q-mt-sm">
                            Failed to load image
                          </div>
                        </div>
                      </div>
                    </template>
                    <template #loading>
                      <div class="absolute-full flex flex-center">
                        <q-spinner
                          color="primary"
                          size="50px"
                        />
                      </div>
                    </template>
                  </q-img>
                  <div class="screenshot-slide-footer">
                    <span class="text-caption text-grey-6">{{ index + 1 }} / {{ screenshotCaptures.length }}</span>
                    <q-btn
                      color="primary"
                      icon="edit"
                      label="Annotate"
                      size="sm"
                      @click="openAnnotator(capturePath, index)"
                    />
                  </div>
                </div>
              </q-carousel-slide>
            </q-carousel>

            <!-- Videos shown below carousel -->
            <div
              v-if="videoCaptures.length > 0"
              class="q-mt-md"
            >
              <div class="text-subtitle2 q-mb-sm">
                Videos ({{ videoCaptures.length }})
              </div>
              <div class="column q-gutter-md">
                <div
                  v-for="(capturePath, index) in videoCaptures"
                  :key="index"
                >
                  <div class="text-caption text-grey-6 q-mb-xs">
                    {{ capturePath.split(/[\\/]/).pop() }}
                  </div>
                  <VideoPlayer :file-path="capturePath" />
                </div>
              </div>
            </div>
          </div>
        </q-card-section>
      </q-card>

      <!-- No captures info card (shown when bug has no captures yet) -->
      <q-card
        v-else-if="!loading"
        class="q-mb-md"
      >
        <q-card-section class="text-center q-py-lg">
          <q-icon
            name="photo_camera"
            size="48px"
            color="grey-4"
          />
          <div class="text-body2 text-grey-7 q-mt-sm">
            No captures yet
          </div>
          <div class="text-caption text-grey-5 q-mt-xs">
            Screenshots taken while capturing {{ bug.display_id }} will appear here
          </div>
          <q-btn
            v-if="sessionStore.activeSession && bug.status !== 'capturing'"
            color="primary"
            icon="fiber_manual_record"
            label="Resume Capture for This Bug"
            class="q-mt-md"
            @click="handleResumeCaptureForBug"
          />
        </q-card-section>
      </q-card>

      <!-- Parsed Console Output Card -->
      <q-card
        v-if="consoleParsed"
        class="q-mb-md"
      >
        <q-card-section>
          <div class="text-h6 q-mb-md">
            Parsed Console Output
          </div>

          <!-- Errors -->
          <div
            v-if="consoleParsed.errors.length > 0"
            class="q-mb-md"
          >
            <div class="text-subtitle2 text-negative q-mb-sm">
              Errors ({{ consoleParsed.errors.length }})
            </div>
            <div class="console-block bg-red-1 rounded-borders q-pa-sm">
              <div
                v-for="(err, i) in consoleParsed.errors"
                :key="i"
                class="console-line text-negative"
              >
                {{ err }}
              </div>
            </div>
          </div>

          <!-- Warnings -->
          <div
            v-if="consoleParsed.warnings.length > 0"
            class="q-mb-md"
          >
            <div class="text-subtitle2 text-warning q-mb-sm">
              Warnings ({{ consoleParsed.warnings.length }})
            </div>
            <div class="console-block bg-yellow-1 rounded-borders q-pa-sm">
              <div
                v-for="(warn, i) in consoleParsed.warnings"
                :key="i"
                class="console-line text-warning"
              >
                {{ warn }}
              </div>
            </div>
          </div>

          <!-- Logs -->
          <div v-if="consoleParsed.logs.length > 0">
            <div class="text-subtitle2 text-grey-7 q-mb-sm">
              Logs ({{ consoleParsed.logs.length }})
            </div>
            <div class="console-block bg-grey-2 rounded-borders q-pa-sm">
              <div
                v-for="(log, i) in consoleParsed.logs"
                :key="i"
                class="console-line text-grey-8"
              >
                {{ log }}
              </div>
            </div>
          </div>
        </q-card-section>
      </q-card>

      <!-- Environment Information Card -->
      <q-card
        v-if="environment"
        class="q-mb-md"
      >
        <q-card-section>
          <div class="text-h6 q-mb-md">
            Environment
          </div>
          <div class="row q-col-gutter-md">
            <div class="col-12 col-md-6">
              <div class="text-caption text-grey-7">
                Operating System
              </div>
              <div class="text-body1">
                {{ environment.os }}
              </div>
            </div>
            <div class="col-12 col-md-6">
              <div class="text-caption text-grey-7">
                Display Resolution
              </div>
              <div class="text-body1">
                {{ environment.display_resolution }}
              </div>
            </div>
            <div class="col-12 col-md-6">
              <div class="text-caption text-grey-7">
                DPI Scaling
              </div>
              <div class="text-body1">
                {{ environment.dpi_scaling }}
              </div>
            </div>
            <div class="col-12 col-md-6">
              <div class="text-caption text-grey-7">
                RAM
              </div>
              <div class="text-body1">
                {{ environment.ram }}
              </div>
            </div>
            <div class="col-12 col-md-6">
              <div class="text-caption text-grey-7">
                CPU
              </div>
              <div class="text-body1">
                {{ environment.cpu }}
              </div>
            </div>
            <div class="col-12 col-md-6">
              <div class="text-caption text-grey-7">
                Foreground App
              </div>
              <div class="text-body1">
                {{ environment.foreground_app }}
              </div>
            </div>
          </div>
        </q-card-section>
      </q-card>
    </div>

    <!-- Screenshot Annotator Dialog -->
    <ScreenshotAnnotator
      v-model="showAnnotator"
      :screenshot-path="selectedScreenshot"
      @saved="handleAnnotationSaved"
    />

    <!-- Reassign Capture Dialog -->
    <q-dialog
      v-model="showReassignDialog"
      persistent
    >
      <q-card style="min-width: 320px; max-width: 480px;">
        <q-card-section class="row items-center q-pb-none">
          <q-icon
            name="swap_horiz"
            size="md"
            color="primary"
            class="q-mr-sm"
          />
          <div class="text-h6">
            Move Capture to Another Bug
          </div>
        </q-card-section>
        <q-card-section>
          <p class="text-body2 text-grey-7">
            Select which bug this capture should be associated with.
          </p>
          <q-select
            v-model="reassignTargetBugId"
            :options="reassignBugOptions"
            option-label="label"
            option-value="value"
            emit-value
            map-options
            label="Move to bug"
            outlined
            dense
            class="q-mt-sm"
          />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn
            flat
            label="Cancel"
            color="grey"
            @click="showReassignDialog = false"
          />
          <q-btn
            unelevated
            label="Move"
            color="primary"
            icon="swap_horiz"
            :disable="!reassignTargetBugId || reassignTargetBugId === bug?.id"
            :loading="reassigning"
            @click="handleReassignCapture"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useBugStore } from '@/stores/bug'
import { useSessionStore } from '@/stores/session'
import { invoke } from '@tauri-apps/api/core'
import { open as shellOpen } from '@tauri-apps/plugin-shell'
import { useQuasar } from 'quasar'
import ScreenshotAnnotator from '@/components/ScreenshotAnnotator.vue'
import VideoPlayer from '@/components/VideoPlayer.vue'
import * as tauri from '@/api/tauri'
import type { Capture, Environment } from '@/types/backend'
import { toAssetUrl } from '@/utils/paths'

interface ConsoleParsed {
  errors: string[]
  warnings: string[]
  logs: string[]
}

const VIDEO_EXTENSIONS = ['.mp4', '.webm', '.mkv', '.mov', '.avi']

function isVideoPath(path: string): boolean {
  const lower = path.toLowerCase()
  return VIDEO_EXTENSIONS.some(ext => lower.endsWith(ext))
}

const route = useRoute()
const router = useRouter()
const bugStore = useBugStore()
const sessionStore = useSessionStore()
const $q = useQuasar()

const currentSlide = ref(0)
const copying = ref(false)
const endingCapture = ref(false)
const showAnnotator = ref(false)
const selectedScreenshot = ref('')
const selectedScreenshotIndex = ref(0)
const loading = ref(false)
const captures = ref<Capture[]>([])
const captureViewMode = ref<'timeline' | 'carousel'>('timeline')
const descriptionDraft = ref('')
const titleDraft = ref('')

// Reassign capture state
const showReassignDialog = ref(false)
const reassignCapture = ref<Capture | null>(null)
const reassignTargetBugId = ref<string | null>(null)
const reassigning = ref(false)

// Get bug ID from route params
const bugId = computed(() => route.params.id as string)

// Look up bug in backendBugs (the live store populated by ActiveSessionView)
const bug = computed(() =>
  bugStore.backendBugs.find(b => b.id === bugId.value) ?? null
)

// All captures sorted chronologically
const allCaptures = computed(() =>
  [...captures.value].sort((a, b) => a.created_at.localeCompare(b.created_at))
)

// Parse the console_parse_json from the bug
const consoleParsed = computed((): ConsoleParsed | null => {
  if (!bug.value?.console_parse_json) return null
  try {
    return JSON.parse(bug.value.console_parse_json) as ConsoleParsed
  } catch {
    return null
  }
})

// Parse environment from metadata_json
const environment = computed((): Environment | null => {
  if (!bug.value?.metadata_json) return null
  try {
    const meta = JSON.parse(bug.value.metadata_json) as { environment?: Environment }
    return meta.environment ?? null
  } catch {
    return null
  }
})

const screenshotCaptures = computed(() =>
  allCaptures.value
    .filter(c => !isVideoPath(c.file_path))
    .map(c => c.annotated_path || c.file_path)
)

const videoCaptures = computed(() =>
  allCaptures.value.filter(c => isVideoPath(c.file_path)).map(c => c.file_path)
)

// For the reassign dialog: list all other bugs in the session
const reassignBugOptions = computed(() => {
  return bugStore.backendBugs
    .filter(b => b.id !== bug.value?.id)
    .map(b => ({
      label: b.display_id + (b.title ? ` — ${b.title}` : ''),
      value: b.id,
    }))
})

// Get the 0-based index of a screenshot capture within the screenshotCaptures list
function getScreenshotIndex(capture: Capture): number {
  const screenshotOnly = allCaptures.value.filter(c => !isVideoPath(c.file_path))
  return screenshotOnly.findIndex(c => c.id === capture.id)
}

function getCaptureIcon(capture: Capture): string {
  if (isVideoPath(capture.file_path)) return 'videocam'
  if (capture.is_console_capture) return 'terminal'
  return 'photo_camera'
}

function getCaptureColor(capture: Capture): string {
  if (isVideoPath(capture.file_path)) return 'purple'
  if (capture.is_console_capture) return 'orange'
  return 'blue'
}

function getCaptureLabel(capture: Capture): string {
  if (isVideoPath(capture.file_path)) return 'Video'
  if (capture.is_console_capture) return 'Console'
  return 'Screenshot'
}

function getBugStatusColor(status: string): string {
  switch (status) {
    case 'capturing': return 'red'
    case 'captured': return 'orange'
    case 'reviewed': return 'blue'
    case 'ready': return 'green'
    default: return 'grey'
  }
}

function formatCaptureTime(isoString: string): string {
  try {
    const d = new Date(isoString)
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
  } catch {
    return ''
  }
}

// Navigate back to bug list
function goBack() {
  router.back()
}

// Save title to database on blur
async function saveTitle() {
  if (!bug.value) return
  const newTitle = titleDraft.value.trim()
  const current = bug.value.title ?? ''
  if (newTitle === current) return

  try {
    await tauri.updateBugTitle(bug.value.id, newTitle)
    // Patch local store so the header reflects the new title immediately
    const localBug = bugStore.backendBugs.find(b => b.id === bug.value!.id)
    if (localBug) {
      localBug.title = newTitle || null
    }
  } catch (err) {
    console.error('Failed to save title:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to save title',
      position: 'top',
      timeout: 3000,
    })
  }
}

// Save description to database on blur
async function saveDescription() {
  if (!bug.value) return
  const newDescription = descriptionDraft.value.trim()
  const current = bug.value.description ?? ''
  if (newDescription === current) return

  try {
    await bugStore.updateBackendBug(bug.value.id, { description: newDescription })
  } catch (err) {
    console.error('Failed to save description:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to save description',
      position: 'top',
      timeout: 3000,
    })
  }
}

function isMeetingUrl(value: string | null | undefined): boolean {
  if (!value) return false
  return value.startsWith('http://') || value.startsWith('https://')
}

function openMeetingUrl(url: string) {
  shellOpen(url).catch((err) => {
    console.error('Failed to open URL:', err)
  })
}

// Open bug folder in file explorer
async function openBugFolder() {
  if (!bug.value) return

  try {
    await invoke('open_bug_folder', {
      folderPath: bug.value.folder_path
    })

    $q.notify({
      type: 'positive',
      message: 'Bug folder opened',
      position: 'top',
      timeout: 2000
    })
  } catch (error) {
    $q.notify({
      type: 'negative',
      message: `Failed to open bug folder: ${error}`,
      position: 'top',
      timeout: 3000
    })
  }
}

// Copy bug to clipboard
async function copyToClipboard() {
  if (!bug.value) return

  copying.value = true
  try {
    await invoke('copy_bug_to_clipboard', {
      bugId: bug.value.id
    })

    $q.notify({
      type: 'positive',
      message: 'Bug report copied to clipboard',
      position: 'top',
      timeout: 2000
    })
  } catch (error) {
    $q.notify({
      type: 'negative',
      message: `Failed to copy bug report: ${error}`,
      position: 'top',
      timeout: 3000
    })
  } finally {
    copying.value = false
  }
}

// Resume capturing screenshots for this bug (when a session is active but this bug is not being captured)
async function handleResumeCaptureForBug() {
  if (!bug.value) return
  // If another bug is currently capturing, end it first
  if (bugStore.isCapturing && bugStore.activeBug) {
    try {
      await bugStore.completeBugCapture(bugStore.activeBug.id)
    } catch (err) {
      console.error('Failed to complete current bug capture:', err)
    }
  }
  // Set this bug as the active capturing bug
  try {
    await bugStore.resumeBugCapture(bug.value)
    $q.notify({
      type: 'positive',
      icon: 'fiber_manual_record',
      message: `Now capturing screenshots for ${bug.value.display_id}`,
      caption: 'Press Print Screen to take screenshots',
      position: 'top',
      timeout: 4000,
    })
  } catch {
    $q.notify({
      type: 'negative',
      message: 'Failed to resume capture',
      position: 'bottom-right',
      timeout: 3000,
    })
  }
}

// End capturing screenshots for this bug
async function handleEndBugCapture() {
  if (!bug.value) return
  endingCapture.value = true
  try {
    await bugStore.completeBugCapture(bug.value.id)
    $q.notify({
      type: 'positive',
      icon: 'stop',
      message: `Capture ended for ${bug.value.display_id}`,
      position: 'top',
      timeout: 3000,
    })
  } catch {
    $q.notify({
      type: 'negative',
      message: 'Failed to end bug capture',
      position: 'bottom-right',
      timeout: 3000,
    })
  } finally {
    endingCapture.value = false
  }
}

// Open screenshot annotator
function openAnnotator(screenshotPath: string, index: number) {
  selectedScreenshot.value = screenshotPath
  selectedScreenshotIndex.value = index
  showAnnotator.value = true
}

// Handle annotation saved
function handleAnnotationSaved(annotatedPath: string) {
  $q.notify({
    type: 'positive',
    message: 'Annotated screenshot saved successfully',
    position: 'top',
    timeout: 2000
  })

  console.log('Annotated screenshot saved to:', annotatedPath)
}

// Open the reassign dialog for a specific capture
function openReassignDialog(capture: Capture) {
  if (reassignBugOptions.value.length === 0) {
    $q.notify({
      type: 'info',
      message: 'No other bugs in this session to move the capture to',
      position: 'top',
      timeout: 3000,
    })
    return
  }
  reassignCapture.value = capture
  reassignTargetBugId.value = null
  showReassignDialog.value = true
}

// Execute the reassignment
async function handleReassignCapture() {
  if (!reassignCapture.value || !reassignTargetBugId.value) return
  reassigning.value = true
  try {
    await tauri.assignCaptureToBug(reassignCapture.value.id, reassignTargetBugId.value)
    const targetBug = bugStore.backendBugs.find(b => b.id === reassignTargetBugId.value)
    $q.notify({
      type: 'positive',
      icon: 'swap_horiz',
      message: `Capture moved to ${targetBug?.display_id ?? 'bug'}`,
      position: 'top',
      timeout: 3000,
    })
    showReassignDialog.value = false
    await refreshCaptures()
  } catch (err) {
    $q.notify({
      type: 'negative',
      message: 'Failed to move capture',
      caption: err instanceof Error ? err.message : String(err),
      position: 'bottom-right',
      timeout: 4000,
    })
  } finally {
    reassigning.value = false
  }
}

async function refreshCaptures() {
  const id = bugId.value
  if (!id) return
  try {
    captures.value = await tauri.getBugCaptures(id)
  } catch (err) {
    console.error('Failed to load captures:', err)
  }
}

async function toggleConsoleCapture(captureId: string, isConsole: boolean) {
  try {
    await tauri.updateCaptureConsoleFlag(captureId, isConsole)
    await refreshCaptures()
    $q.notify({
      type: 'positive',
      message: isConsole ? 'Marked as console capture' : 'Unmarked as console capture',
      position: 'top',
      timeout: 1500,
    })
  } catch (err) {
    console.error('Failed to toggle console capture:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to update console capture status',
      position: 'top',
      timeout: 3000,
    })
  }
}

// Load bug data on mount — handles direct navigation/page reload
onMounted(async () => {
  const id = bugId.value
  if (!id) return

  // If bug isn't in the store yet, fetch it from the backend
  if (!bug.value) {
    loading.value = true
    try {
      await bugStore.loadBug(id)
    } catch (err) {
      console.error('Failed to load bug:', err)
    } finally {
      loading.value = false
    }
  }

  // Initialize title and description drafts from the loaded bug
  titleDraft.value = bug.value?.title ?? ''
  descriptionDraft.value = bug.value?.description ?? ''

  // Load captures for this bug
  await refreshCaptures()
})

// Sync title draft when bug data changes (e.g. after store load)
watch(
  () => bug.value?.title,
  (newVal) => {
    titleDraft.value = newVal ?? ''
  }
)

// Sync description draft when bug data changes (e.g. after store load)
watch(
  () => bug.value?.description,
  (newVal) => {
    descriptionDraft.value = newVal ?? ''
  }
)

// Auto-refresh captures when a new screenshot is taken for this bug
watch(
  () => sessionStore.lastScreenshotEvent,
  async (event) => {
    if (!event) return
    // Only refresh if the screenshot belongs to the bug we're viewing
    const activeBug = bugStore.activeBug
    if (activeBug?.id === bugId.value) {
      await refreshCaptures()
      $q.notify({
        type: 'positive',
        icon: 'photo_camera',
        message: 'New screenshot added',
        position: 'top',
        timeout: 2000,
      })
    }
  }
)
</script>

<style scoped>
.bug-detail {
  max-width: 1200px;
  margin: 0 auto;
}

.whitespace-pre-wrap {
  white-space: pre-wrap;
}

.flex-center {
  display: flex;
  justify-content: center;
  align-items: center;
}

.flex-1 {
  flex: 1;
}

.bug-title-input {
  font-size: inherit;
  font-weight: inherit;
  line-height: inherit;
}

.screenshot-slide {
  position: relative;
  flex-direction: column;
}

.screenshot-slide-footer {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: rgba(0, 0, 0, 0.5);
  color: white;
  z-index: 10;
}

.console-block {
  max-height: 200px;
  overflow-y: auto;
}

.console-line {
  font-family: monospace;
  font-size: 12px;
  white-space: pre-wrap;
  word-break: break-all;
  line-height: 1.4;
}

.capturing-badge {
  animation: pulse-badge 1.5s ease-in-out infinite;
}

@keyframes pulse-badge {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.screenshot-slide-footer .q-btn {
  color: white !important;
}

/* Carousel arrow visibility on dark screenshot backgrounds */
:deep(.q-carousel .q-carousel__arrow .q-btn) {
  background: rgba(0, 0, 0, 0.45) !important;
  color: white !important;
  border-radius: 50%;
}

:deep(.q-carousel .q-carousel__arrow .q-btn:hover) {
  background: rgba(0, 0, 0, 0.7) !important;
}

/* Timeline styles */
.capture-timeline-item {
  position: relative;
}

.timeline-connector {
  width: 2px;
  flex: 1;
  min-height: 16px;
  background: #e0e0e0;
  margin: 2px 0;
}

.capture-thumbnail-row {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
}

.capture-thumbnail {
  width: 200px;
  height: 130px;
  cursor: pointer;
  border: 1px solid #e0e0e0;
  transition: opacity 0.15s ease;
}

.capture-thumbnail:hover {
  opacity: 0.85;
}
</style>
