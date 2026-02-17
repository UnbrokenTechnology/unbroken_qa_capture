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
        <div class="text-h4 flex-1">
          {{ bug.title || bug.display_id }}
        </div>
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
                  <q-icon name="open_in_new" size="xs" class="q-mr-xs" />{{ bug.meeting_id }}
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

          <div
            v-if="bug.description"
            class="text-body2 whitespace-pre-wrap"
          >
            {{ bug.description }}
          </div>

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

          <div
            v-if="!bug.description && !bug.notes && !bug.ai_description"
            class="text-body2 text-grey"
          >
            No description provided.
          </div>
        </q-card-section>
      </q-card>

      <!-- Screenshots Card -->
      <q-card
        v-if="screenshotCaptures.length > 0"
        class="q-mb-md"
      >
        <q-card-section>
          <div class="row items-center q-mb-md">
            <q-icon
              name="photo_camera"
              size="sm"
              color="primary"
              class="q-mr-sm"
            />
            <span class="text-h6">Screenshots ({{ screenshotCaptures.length }})</span>
            <q-space />
            <q-badge
              color="primary"
              outline
            >
              Associated with {{ bug.display_id }}
            </q-badge>
          </div>
          <q-carousel
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
              v-for="(capture, index) in screenshotCaptures"
              :key="index"
              :name="index"
              class="q-pa-none"
            >
              <div class="full-width full-height flex flex-center screenshot-slide">
                <q-img
                  :src="capture"
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
                    @click="openAnnotator(capture, index)"
                  />
                </div>
              </div>
            </q-carousel-slide>
          </q-carousel>
        </q-card-section>
      </q-card>

      <!-- No captures info card (shown when bug has no screenshots yet) -->
      <q-card
        v-else-if="!loading && captures.length === 0"
        class="q-mb-md"
      >
        <q-card-section class="text-center q-py-lg">
          <q-icon
            name="photo_camera"
            size="48px"
            color="grey-4"
          />
          <div class="text-body2 text-grey-7 q-mt-sm">
            No screenshots yet
          </div>
          <div class="text-caption text-grey-5 q-mt-xs">
            Screenshots taken while capturing {{ bug.display_id }} will appear here
          </div>
        </q-card-section>
      </q-card>

      <!-- Videos Card -->
      <q-card
        v-if="videoCaptures.length > 0"
        class="q-mb-md"
      >
        <q-card-section>
          <div class="text-h6 q-mb-md">
            Videos ({{ videoCaptures.length }})
          </div>
          <div class="column q-gutter-md">
            <div
              v-for="(capture, index) in videoCaptures"
              :key="index"
            >
              <div class="text-caption text-grey-6 q-mb-xs">
                {{ capture.split(/[\\/]/).pop() }}
              </div>
              <VideoPlayer :file-path="capture" />
            </div>
          </div>
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
const showAnnotator = ref(false)
const selectedScreenshot = ref('')
const selectedScreenshotIndex = ref(0)
const loading = ref(false)
const captures = ref<Capture[]>([])

// Get bug ID from route params
const bugId = computed(() => route.params.id as string)

// Look up bug in backendBugs (the live store populated by ActiveSessionView)
const bug = computed(() =>
  bugStore.backendBugs.find(b => b.id === bugId.value) ?? null
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
  captures.value.filter(c => !isVideoPath(c.file_path)).map(c => c.file_path)
)

const videoCaptures = computed(() =>
  captures.value.filter(c => isVideoPath(c.file_path)).map(c => c.file_path)
)

// Navigate back to bug list
function goBack() {
  router.back()
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
      folderPath: bug.value.folder_path
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

async function refreshCaptures() {
  const id = bugId.value
  if (!id) return
  try {
    captures.value = await tauri.getBugCaptures(id)
  } catch (err) {
    console.error('Failed to load captures:', err)
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

  // Load captures for this bug
  await refreshCaptures()
})

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
</style>
