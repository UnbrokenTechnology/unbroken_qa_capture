<template>
  <q-page padding>
    <div class="session-review">
      <!-- Header -->
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
          Session Review
        </div>
        <div
          v-if="sessionStore.activeSession"
          class="text-caption text-grey-7"
        >
          {{ formatDate(sessionStore.activeSession.started_at) }}
        </div>
      </div>

      <!-- Empty state -->
      <div
        v-if="!sessionStore.activeSession"
        class="flex flex-center q-pa-xl"
      >
        <div class="text-h6 text-grey">
          No active session
        </div>
      </div>

      <!-- Master-Detail Layout -->
      <div
        v-else
        class="row q-col-gutter-md"
      >
        <!-- Left Panel: Bug List -->
        <div class="col-12 col-md-4">
          <q-card>
            <q-card-section>
              <div class="text-h6 q-mb-md">
                Bugs ({{ bugs.length }})
              </div>

              <!-- Empty state for bug list -->
              <div
                v-if="bugs.length === 0"
                class="text-center q-pa-md text-grey-7"
              >
                No bugs in this session
              </div>

              <!-- Bug Cards List -->
              <q-list
                v-else
                separator
              >
                <q-item
                  v-for="bug in bugs"
                  :key="bug.id"
                  clickable
                  :active="selectedBugId === bug.id"
                  class="bug-card"
                  @click="selectBug(bug.id)"
                >
                  <q-item-section>
                    <!-- Bug ID and Type -->
                    <div class="row items-center q-mb-xs">
                      <q-badge
                        :color="getBugTypeColor(bug.type)"
                        class="q-mr-sm"
                      >
                        {{ bug.display_id }}
                      </q-badge>
                      <q-chip
                        :color="getBugTypeColor(bug.type)"
                        text-color="white"
                        dense
                        size="sm"
                      >
                        {{ bug.type }}
                      </q-chip>
                    </div>

                    <!-- Bug Title/Notes Snippet -->
                    <q-item-label class="text-body2 q-mb-xs">
                      {{ bug.title || 'Untitled Bug' }}
                    </q-item-label>

                    <!-- Notes Snippet -->
                    <q-item-label
                      v-if="bug.notes"
                      caption
                      lines="2"
                    >
                      {{ bug.notes }}
                    </q-item-label>

                    <!-- Folder Path -->
                    <q-item-label
                      caption
                      class="text-grey-6 q-mt-xs"
                    >
                      <q-icon
                        name="folder"
                        size="xs"
                      />
                      {{ bug.folder_path }}
                    </q-item-label>

                    <!-- Screenshot Thumbnails -->
                    <div
                      v-if="bugCaptures[bug.id] && bugCaptures[bug.id]!.length > 0"
                      class="row q-gutter-xs q-mt-sm"
                    >
                      <div
                        v-for="(capture, index) in bugCaptures[bug.id]!.slice(0, 3)"
                        :key="capture.id"
                        class="thumbnail-container"
                      >
                        <q-img
                          :src="`file://${capture.file_path}`"
                          class="thumbnail"
                          fit="cover"
                          @click.stop="viewScreenshot(bug.id, index)"
                        >
                          <template #error>
                            <div class="absolute-full flex flex-center bg-grey-3">
                              <q-icon
                                name="image_not_supported"
                                size="sm"
                                color="grey-6"
                              />
                            </div>
                          </template>
                        </q-img>
                      </div>
                      <div
                        v-if="bugCaptures[bug.id]!.length > 3"
                        class="thumbnail-container more-indicator"
                      >
                        +{{ bugCaptures[bug.id]!.length - 3 }}
                      </div>
                    </div>
                  </q-item-section>

                  <q-item-section side>
                    <q-icon
                      name="chevron_right"
                      color="grey-5"
                    />
                  </q-item-section>
                </q-item>
              </q-list>
            </q-card-section>
          </q-card>
        </div>

        <!-- Right Panel: Bug Detail -->
        <div class="col-12 col-md-8">
          <q-card v-if="selectedBug">
            <q-card-section>
              <div class="text-h5 q-mb-md">
                {{ selectedBug.title || 'Untitled Bug' }}
              </div>

              <!-- Bug Type Editor -->
              <div class="q-mb-md">
                <div class="text-caption text-grey-7 q-mb-xs">
                  Bug Type
                </div>
                <q-btn-group spread>
                  <q-btn
                    :color="selectedBug.type === 'bug' ? 'negative' : 'grey-3'"
                    :text-color="selectedBug.type === 'bug' ? 'white' : 'grey-8'"
                    label="Bug"
                    @click="updateBugType('bug')"
                  />
                  <q-btn
                    :color="selectedBug.type === 'feature' ? 'primary' : 'grey-3'"
                    :text-color="selectedBug.type === 'feature' ? 'white' : 'grey-8'"
                    label="Feature"
                    @click="updateBugType('feature')"
                  />
                  <q-btn
                    :color="selectedBug.type === 'feedback' ? 'warning' : 'grey-3'"
                    :text-color="selectedBug.type === 'feedback' ? 'white' : 'grey-8'"
                    label="Feedback"
                    @click="updateBugType('feedback')"
                  />
                </q-btn-group>
              </div>

              <!-- Bug Metadata -->
              <div class="row q-col-gutter-md q-mb-md">
                <div class="col-12 col-sm-6">
                  <div class="text-caption text-grey-7">
                    Bug ID
                  </div>
                  <div class="text-body1">
                    {{ selectedBug.display_id }}
                  </div>
                </div>
                <div class="col-12 col-sm-6">
                  <div class="text-caption text-grey-7">
                    Status
                  </div>
                  <div class="text-body1">
                    <q-chip
                      :color="getBugStatusColor(selectedBug.status)"
                      text-color="white"
                      dense
                    >
                      {{ selectedBug.status }}
                    </q-chip>
                  </div>
                </div>
                <div class="col-12">
                  <div class="text-caption text-grey-7">
                    Folder Path
                  </div>
                  <div class="text-body2">
                    {{ selectedBug.folder_path }}
                  </div>
                </div>
              </div>

              <!-- Notes -->
              <div
                v-if="selectedBug.notes"
                class="q-mb-md"
              >
                <div class="text-caption text-grey-7 q-mb-xs">
                  Notes
                </div>
                <div class="text-body2 whitespace-pre-wrap">
                  {{ selectedBug.notes }}
                </div>
              </div>

              <!-- Description -->
              <div
                v-if="selectedBug.description"
                class="q-mb-md"
              >
                <div class="text-caption text-grey-7 q-mb-xs">
                  Description
                </div>
                <div class="text-body2 whitespace-pre-wrap">
                  {{ selectedBug.description }}
                </div>
              </div>

              <!-- Screenshots -->
              <div v-if="selectedBugCaptures.length > 0">
                <div class="text-caption text-grey-7 q-mb-xs">
                  Screenshots ({{ selectedBugCaptures.length }})
                </div>
                <div class="row q-gutter-md">
                  <div
                    v-for="(capture, index) in selectedBugCaptures"
                    :key="capture.id"
                    class="screenshot-container"
                  >
                    <q-img
                      :src="`file://${capture.file_path}`"
                      class="screenshot"
                      fit="contain"
                      @click="viewScreenshot(selectedBug.id, index)"
                    >
                      <template #error>
                        <div class="absolute-full flex flex-center bg-grey-3">
                          <q-icon
                            name="image_not_supported"
                            size="lg"
                            color="grey-6"
                          />
                        </div>
                      </template>
                    </q-img>
                  </div>
                </div>
              </div>
            </q-card-section>
          </q-card>

          <!-- No bug selected state -->
          <q-card v-else>
            <q-card-section>
              <div class="text-center text-grey-7 q-pa-xl">
                Select a bug to view details
              </div>
            </q-card-section>
          </q-card>
        </div>
      </div>
    </div>

    <!-- Full-size Screenshot Dialog -->
    <q-dialog
      v-model="showScreenshotDialog"
      maximized
    >
      <q-card class="bg-black">
        <q-card-section class="row items-center q-pb-none">
          <div class="text-h6 text-white">
            Screenshot {{ currentScreenshotIndex + 1 }} / {{ selectedBugCaptures.length }}
          </div>
          <q-space />
          <q-btn
            icon="close"
            flat
            round
            dense
            color="white"
            @click="showScreenshotDialog = false"
          />
        </q-card-section>

        <q-card-section class="flex flex-center full-height">
          <q-img
            v-if="selectedBugCaptures[currentScreenshotIndex]"
            :src="`file://${selectedBugCaptures[currentScreenshotIndex]!.file_path}`"
            fit="contain"
            class="full-width"
          >
            <template #error>
              <div class="absolute-full flex flex-center bg-grey-9">
                <div class="text-center">
                  <q-icon
                    name="image_not_supported"
                    size="xl"
                    color="grey-6"
                  />
                  <div class="text-grey-6 q-mt-md">
                    Unable to load image
                  </div>
                </div>
              </div>
            </template>
          </q-img>
        </q-card-section>

        <q-card-actions
          v-if="selectedBugCaptures.length > 1"
          class="row justify-center"
        >
          <q-btn
            icon="chevron_left"
            flat
            round
            color="white"
            :disable="currentScreenshotIndex === 0"
            @click="currentScreenshotIndex--"
          />
          <q-btn
            icon="chevron_right"
            flat
            round
            color="white"
            :disable="currentScreenshotIndex === selectedBugCaptures.length - 1"
            @click="currentScreenshotIndex++"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useBugStore } from '@/stores/bug'
import { useSessionStore } from '@/stores/session'
import type { BugType, BugStatus, Capture } from '@/types/backend'
import * as tauri from '@/api/tauri'

const router = useRouter()
const bugStore = useBugStore()
const sessionStore = useSessionStore()

// State
const selectedBugId = ref<string | null>(null)
const bugCaptures = ref<Record<string, Capture[]>>({})
const showScreenshotDialog = ref(false)
const currentScreenshotIndex = ref(0)

// Computed
const bugs = computed(() => bugStore.backendBugs)

const selectedBug = computed(() => {
  if (!selectedBugId.value) return null
  return bugs.value.find(b => b.id === selectedBugId.value) || null
})

const selectedBugCaptures = computed(() => {
  if (!selectedBugId.value) return []
  return bugCaptures.value[selectedBugId.value] || []
})

// Methods
function goBack() {
  router.back()
}

function selectBug(id: string) {
  selectedBugId.value = id
  loadBugCaptures(id)
}

async function loadBugCaptures(bugId: string) {
  try {
    // Load captures for this bug if not already loaded
    if (!bugCaptures.value[bugId]) {
      const captures = await tauri.getBugCaptures(bugId)
      bugCaptures.value[bugId] = captures.filter(c => c.file_type === 'screenshot')
    }
  } catch (err) {
    console.error('Failed to load bug captures:', err)
    bugCaptures.value[bugId] = []
  }
}

async function updateBugType(type: BugType) {
  if (!selectedBug.value) return

  try {
    await bugStore.updateBackendBug(selectedBug.value.id, { type })
  } catch (err) {
    console.error('Failed to update bug type:', err)
  }
}

function viewScreenshot(bugId: string, index: number) {
  if (selectedBugId.value !== bugId) {
    selectBug(bugId)
  }
  currentScreenshotIndex.value = index
  showScreenshotDialog.value = true
}

function getBugTypeColor(type: BugType): string {
  switch (type) {
    case 'bug': return 'negative'
    case 'feature': return 'primary'
    case 'feedback': return 'warning'
    default: return 'grey'
  }
}

function getBugStatusColor(status: BugStatus): string {
  switch (status) {
    case 'capturing': return 'info'
    case 'captured': return 'primary'
    case 'reviewed': return 'positive'
    case 'ready': return 'positive'
    default: return 'grey'
  }
}

function formatDate(dateStr: string): string {
  const date = new Date(dateStr)
  return date.toLocaleString()
}

// Lifecycle
onMounted(async () => {
  // Load bugs for the active session
  if (sessionStore.activeSession) {
    await bugStore.loadBugsBySession(sessionStore.activeSession.id)

    // Preload captures for all bugs
    for (const bug of bugs.value) {
      await loadBugCaptures(bug.id)
    }

    // Select first bug by default
    if (bugs.value.length > 0) {
      selectBug(bugs.value[0]!.id)
    }
  }
})
</script>

<style scoped>
.session-review {
  max-width: 1400px;
  margin: 0 auto;
}

.bug-card {
  border-radius: 4px;
  transition: background-color 0.2s;
}

.bug-card:hover {
  background-color: rgba(0, 0, 0, 0.02);
}

.thumbnail-container {
  width: 60px;
  height: 60px;
  border-radius: 4px;
  overflow: hidden;
  cursor: pointer;
  border: 1px solid #e0e0e0;
}

.thumbnail {
  width: 100%;
  height: 100%;
}

.more-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #f5f5f5;
  font-size: 12px;
  color: #666;
  cursor: default;
}

.screenshot-container {
  width: 200px;
  height: 200px;
  border-radius: 4px;
  overflow: hidden;
  cursor: pointer;
  border: 1px solid #e0e0e0;
}

.screenshot {
  width: 100%;
  height: 100%;
}

.whitespace-pre-wrap {
  white-space: pre-wrap;
}
</style>
