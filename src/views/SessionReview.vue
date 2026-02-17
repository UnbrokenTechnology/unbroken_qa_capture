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
        <q-btn
          v-if="finalizedBugs.length > 0"
          color="primary"
          icon="upload"
          label="Push to Linear"
          class="q-mr-md"
          :loading="isPushing"
          @click="showPushDialog = true"
        />
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

                    <!-- Capture Thumbnails -->
                    <div
                      v-if="bugCaptures[bug.id] && bugCaptures[bug.id]!.length > 0"
                      class="row q-gutter-xs q-mt-sm"
                    >
                      <div
                        v-for="capture in bugCaptures[bug.id]!.slice(0, 3)"
                        :key="capture.id"
                        class="thumbnail-container"
                      >
                        <!-- Video thumbnail: show video icon -->
                        <div
                          v-if="capture.file_type === 'video'"
                          class="thumbnail flex flex-center bg-grey-8 cursor-pointer"
                          @click.stop
                        >
                          <q-icon
                            name="videocam"
                            size="sm"
                            color="white"
                          />
                        </div>
                        <!-- Screenshot thumbnail -->
                        <q-img
                          v-else
                          :src="`file://${capture.file_path}`"
                          class="thumbnail"
                          fit="cover"
                          @click.stop="viewScreenshot(bug.id, getScreenshotIndex(bug.id, capture.id))"
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

              <!-- Actions Row -->
              <div class="row q-mb-md q-gutter-sm">
                <q-btn
                  color="primary"
                  icon="content_copy"
                  label="Copy to Clipboard"
                  outline
                  @click="copyBugToClipboard"
                />
                <q-btn
                  color="secondary"
                  icon="folder_open"
                  label="Open Folder"
                  outline
                  @click="openBugFolder"
                />
                <q-btn
                  color="negative"
                  icon="delete"
                  label="Delete Bug"
                  outline
                  @click="confirmDeleteBug"
                />
                <q-btn
                  v-if="selectedBug.status !== 'ready'"
                  color="positive"
                  icon="check_circle"
                  label="Mark Ready"
                  outline
                  @click="markBugReady"
                />
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

              <!-- Metadata Fields (Editable) -->
              <div class="q-mb-md">
                <div class="text-caption text-grey-7 q-mb-xs">
                  Metadata
                </div>
                <div class="row q-col-gutter-sm">
                  <div class="col-12 col-sm-6">
                    <q-input
                      :model-value="selectedBug.meeting_id || ''"
                      outlined
                      dense
                      label="Meeting ID / URL"
                      placeholder="e.g., Zoom meeting ID or URL"
                      @update:model-value="(val) => updateMetadata('meeting_id', String(val || ''))"
                    >
                      <template #prepend>
                        <q-icon name="videocam" />
                      </template>
                    </q-input>
                  </div>
                  <div class="col-12 col-sm-6">
                    <q-input
                      :model-value="selectedBug.software_version || ''"
                      outlined
                      dense
                      label="Software Version"
                      placeholder="e.g., 2.4.1"
                      @update:model-value="(val) => updateMetadata('software_version', String(val || ''))"
                    >
                      <template #prepend>
                        <q-icon name="info" />
                      </template>
                    </q-input>
                  </div>
                </div>
              </div>

              <!-- Notes (Editable) -->
              <div class="q-mb-md">
                <div class="text-caption text-grey-7 q-mb-xs">
                  Notes
                </div>
                <q-input
                  :model-value="selectedBug.notes || ''"
                  type="textarea"
                  outlined
                  autogrow
                  :min-rows="3"
                  placeholder="Add notes about this bug..."
                  @update:model-value="(val) => updateNotes(String(val || ''))"
                />
              </div>

              <!-- AI Description Generation -->
              <div class="q-mb-md">
                <div class="row items-center justify-between q-mb-xs">
                  <div class="text-caption text-grey-7">
                    AI-Generated Description
                  </div>
                  <div class="row q-gutter-sm">
                    <q-btn
                      v-if="!aiDescription && !isGenerating"
                      size="sm"
                      color="primary"
                      icon="psychology"
                      label="Generate Description"
                      :disable="!claudeAvailable"
                      @click="generateDescription"
                    />
                    <q-btn
                      v-if="aiDescription && !isGenerating"
                      size="sm"
                      color="secondary"
                      icon="auto_fix_high"
                      label="Refine"
                      :disable="!claudeAvailable"
                      @click="showRefineDialog = true"
                    />
                    <q-btn
                      v-if="aiDescription && !isGenerating"
                      size="sm"
                      color="positive"
                      icon="save"
                      label="Save"
                      @click="saveDescription"
                    />
                  </div>
                </div>

                <!-- Loading State -->
                <div
                  v-if="isGenerating"
                  class="q-pa-md text-center"
                >
                  <q-spinner
                    color="primary"
                    size="3em"
                  />
                  <div class="text-caption text-grey-7 q-mt-md">
                    Generating description with Claude...
                  </div>
                </div>

                <!-- Generated Description Editor -->
                <q-input
                  v-if="aiDescription && !isGenerating"
                  v-model="aiDescription"
                  type="textarea"
                  outlined
                  autogrow
                  :min-rows="6"
                  placeholder="AI-generated description will appear here..."
                  class="description-editor"
                />

                <!-- Claude Not Available Warning -->
                <div
                  v-if="!claudeAvailable && !isGenerating && !aiDescription"
                  class="q-pa-md bg-warning text-white rounded"
                >
                  <div class="row items-center">
                    <q-icon
                      name="warning"
                      size="sm"
                      class="q-mr-sm"
                    />
                    <div>
                      Claude CLI not available. {{ claudeStatusMessage }}
                    </div>
                  </div>
                </div>
              </div>

              <!-- Existing Description (if any) -->
              <div
                v-if="selectedBug.description"
                class="q-mb-md"
              >
                <div class="text-caption text-grey-7 q-mb-xs">
                  Saved Description
                </div>
                <div class="text-body2 whitespace-pre-wrap">
                  {{ selectedBug.description }}
                </div>
              </div>

              <!-- Screenshots -->
              <div
                v-if="selectedBugScreenshots.length > 0"
                class="q-mb-md"
              >
                <div class="text-caption text-grey-7 q-mb-xs">
                  Screenshots ({{ selectedBugScreenshots.length }})
                </div>
                <div class="row q-gutter-md">
                  <div
                    v-for="(capture, index) in selectedBugScreenshots"
                    :key="capture.id"
                    class="screenshot-container"
                    style="position: relative;"
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
                    <q-btn
                      :icon="capture.is_console_capture ? 'terminal' : 'radio_button_unchecked'"
                      :color="capture.is_console_capture ? 'primary' : 'grey-7'"
                      size="sm"
                      round
                      class="console-tag-btn"
                      @click.stop="toggleConsoleCapture(capture.id, !capture.is_console_capture)"
                    >
                      <q-tooltip>
                        {{ capture.is_console_capture ? 'Unmark as console capture' : 'Mark as console capture' }}
                      </q-tooltip>
                    </q-btn>
                  </div>
                </div>
              </div>

              <!-- Videos -->
              <div v-if="selectedBugVideos.length > 0">
                <div class="text-caption text-grey-7 q-mb-xs">
                  Videos ({{ selectedBugVideos.length }})
                </div>
                <div class="column q-gutter-md">
                  <div
                    v-for="capture in selectedBugVideos"
                    :key="capture.id"
                  >
                    <div class="text-caption text-grey-6 q-mb-xs">
                      {{ capture.file_name }}
                    </div>
                    <VideoPlayer :file-path="capture.file_path" />
                  </div>
                </div>
              </div>

              <!-- Parsed Console Output -->
              <div
                v-if="selectedBugConsoleParse"
                class="q-mt-md"
              >
                <div class="text-caption text-grey-7 q-mb-xs">
                  Parsed Console Output
                </div>

                <!-- Errors -->
                <div
                  v-if="selectedBugConsoleParse.errors.length > 0"
                  class="q-mb-sm"
                >
                  <div class="text-caption text-negative q-mb-xs">
                    Errors ({{ selectedBugConsoleParse.errors.length }})
                  </div>
                  <div class="console-output-block bg-red-1">
                    <div
                      v-for="(err, i) in selectedBugConsoleParse.errors"
                      :key="i"
                      class="console-line text-negative"
                    >
                      {{ err }}
                    </div>
                  </div>
                </div>

                <!-- Warnings -->
                <div
                  v-if="selectedBugConsoleParse.warnings.length > 0"
                  class="q-mb-sm"
                >
                  <div class="text-caption text-warning q-mb-xs">
                    Warnings ({{ selectedBugConsoleParse.warnings.length }})
                  </div>
                  <div class="console-output-block bg-yellow-1">
                    <div
                      v-for="(warn, i) in selectedBugConsoleParse.warnings"
                      :key="i"
                      class="console-line text-warning"
                    >
                      {{ warn }}
                    </div>
                  </div>
                </div>

                <!-- Logs -->
                <div v-if="selectedBugConsoleParse.logs.length > 0">
                  <div class="text-caption text-grey-7 q-mb-xs">
                    Logs ({{ selectedBugConsoleParse.logs.length }})
                  </div>
                  <div class="console-output-block bg-grey-2">
                    <div
                      v-for="(log, i) in selectedBugConsoleParse.logs"
                      :key="i"
                      class="console-line text-grey-8"
                    >
                      {{ log }}
                    </div>
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

      <!-- Bottom Action Bar -->
      <div
        v-if="sessionStore.activeSession && bugs.length > 0"
        class="row q-gutter-md q-mt-lg q-pa-md bg-grey-2 rounded-borders"
      >
        <q-btn
          color="primary"
          icon="auto_fix_high"
          label="Generate All Descriptions"
          :disable="!claudeAvailable || isGenerating"
          :loading="isGeneratingAll"
          @click="generateAllDescriptions"
        />
        <q-btn
          color="positive"
          icon="upload"
          label="Export to Linear"
          :disable="finalizedBugs.length === 0"
          @click="showPushDialog = true"
        />
        <q-btn
          color="secondary"
          icon="play_arrow"
          label="Resume Session"
          @click="resumeSession"
        />
        <q-space />
        <q-btn
          color="grey-7"
          icon="close"
          label="Close Session"
          outline
          @click="confirmCloseSession"
        />
      </div>
    </div>

    <!-- Refine Description Dialog -->
    <q-dialog v-model="showRefineDialog">
      <q-card style="min-width: 500px">
        <q-card-section>
          <div class="text-h6">
            Refine Description
          </div>
        </q-card-section>

        <q-card-section>
          <q-input
            v-model="refinementInstructions"
            type="textarea"
            outlined
            autogrow
            :min-rows="3"
            label="How would you like to refine the description?"
            placeholder="e.g., 'make steps more specific' or 'add technical details'"
          />
        </q-card-section>

        <q-card-actions align="right">
          <q-btn
            flat
            label="Cancel"
            color="grey"
            @click="showRefineDialog = false"
          />
          <q-btn
            label="Refine"
            color="primary"
            :disable="!refinementInstructions.trim()"
            @click="refineDescription"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- Full-size Screenshot Dialog -->
    <q-dialog
      v-model="showScreenshotDialog"
      maximized
    >
      <q-card class="bg-black">
        <q-card-section class="row items-center q-pb-none">
          <div class="text-h6 text-white">
            Screenshot {{ currentScreenshotIndex + 1 }} / {{ selectedBugScreenshots.length }}
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
            v-if="selectedBugScreenshots[currentScreenshotIndex]"
            :src="`file://${selectedBugScreenshots[currentScreenshotIndex]!.file_path}`"
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
          v-if="selectedBugScreenshots.length > 1"
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
            :disable="currentScreenshotIndex === selectedBugScreenshots.length - 1"
            @click="currentScreenshotIndex++"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- Push to Linear Dialog -->
    <q-dialog
      v-model="showPushDialog"
      persistent
    >
      <q-card style="min-width: 500px">
        <q-card-section>
          <div class="text-h6">
            Push Bugs to Linear
          </div>
        </q-card-section>

        <!-- Credentials Form -->
        <q-card-section v-if="!hasCredentials && !isPushing && !showPreview">
          <div class="text-body2 q-mb-md text-grey-7">
            Linear API credentials not found. Please configure them in Settings.
          </div>
          <q-btn
            outline
            color="primary"
            label="Open Settings"
            icon="settings"
            @click="$router.push('/settings')"
          />
        </q-card-section>

        <!-- Preview Mode -->
        <q-card-section v-if="showPreview && !isPushing">
          <div class="text-body2 q-mb-md">
            Preview of {{ ticketPreviews.length }} ticket{{ ticketPreviews.length > 1 ? 's' : '' }} to be created:
          </div>

          <q-scroll-area style="max-height: 400px">
            <q-list separator>
              <q-item
                v-for="preview in ticketPreviews"
                :key="preview.bugId"
                class="q-pa-md"
              >
                <q-item-section>
                  <q-item-label class="text-weight-bold">
                    {{ preview.title }}
                  </q-item-label>
                  <q-item-label caption>
                    <div class="q-mt-sm">
                      <strong>Labels:</strong> {{ preview.labels.join(', ') }}
                    </div>
                    <div class="q-mt-xs">
                      <strong>Attachments:</strong> {{ preview.attachmentCount }} screenshot{{ preview.attachmentCount !== 1 ? 's' : '' }}
                    </div>
                    <div class="q-mt-sm">
                      <strong>Description:</strong>
                    </div>
                    <div
                      class="q-mt-xs text-grey-8"
                      style="white-space: pre-wrap; max-height: 100px; overflow-y: auto; font-size: 12px;"
                    >
                      {{ preview.description.substring(0, 300) }}{{ preview.description.length > 300 ? '...' : '' }}
                    </div>
                  </q-item-label>
                </q-item-section>
              </q-item>
            </q-list>
          </q-scroll-area>
        </q-card-section>

        <!-- Progress Display -->
        <q-card-section v-if="isPushing || pushResults.length > 0">
          <div class="text-body2 q-mb-md">
            {{ isPushing ? 'Pushing bugs to Linear...' : 'Push complete!' }}
          </div>

          <!-- Progress bar -->
          <q-linear-progress
            v-if="isPushing"
            :value="pushProgress"
            color="primary"
            class="q-mb-md"
          />

          <!-- Results list -->
          <q-list
            separator
            class="rounded-borders"
          >
            <q-item
              v-for="result in pushResults"
              :key="result.bugId"
            >
              <q-item-section avatar>
                <q-icon
                  :name="result.success ? 'check_circle' : 'error'"
                  :color="result.success ? 'positive' : 'negative'"
                />
              </q-item-section>
              <q-item-section>
                <q-item-label>{{ result.bugTitle }}</q-item-label>
                <q-item-label
                  v-if="result.success"
                  caption
                >
                  {{ result.identifier }} - {{ result.url }}
                </q-item-label>
                <q-item-label
                  v-else
                  caption
                  class="text-negative"
                >
                  {{ result.error }}
                </q-item-label>
              </q-item-section>
              <q-item-section
                v-if="result.success"
                side
              >
                <q-btn
                  flat
                  dense
                  icon="open_in_new"
                  size="sm"
                  @click="openUrl(result.url!)"
                />
              </q-item-section>
            </q-item>
          </q-list>
        </q-card-section>

        <q-card-actions align="right">
          <q-btn
            v-if="!isPushing"
            flat
            label="Cancel"
            @click="closePushDialog"
          />
          <q-btn
            v-if="!hasCredentials && !isPushing && !showPreview"
            color="primary"
            label="Save & Push"
            @click="saveCredentialsAndPush"
          />
          <q-btn
            v-if="hasCredentials && !isPushing && !showPreview && pushResults.length === 0"
            outline
            color="secondary"
            label="Preview"
            icon="preview"
            @click="generatePreview"
          />
          <q-btn
            v-if="hasCredentials && !isPushing && !showPreview && pushResults.length === 0"
            color="primary"
            label="Push Now"
            icon="upload"
            @click="pushToLinear"
          />
          <q-btn
            v-if="showPreview && !isPushing"
            flat
            label="Back"
            @click="showPreview = false"
          />
          <q-btn
            v-if="showPreview && !isPushing"
            color="primary"
            label="Confirm & Push"
            icon="upload"
            @click="pushToLinear"
          />
          <q-btn
            v-if="!isPushing && pushResults.length > 0"
            color="primary"
            label="Close"
            @click="closePushDialog"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useQuasar } from 'quasar'
import { useBugStore } from '@/stores/bug'
import { useSessionStore } from '@/stores/session'
import type { BugType, BugStatus, Capture, TicketingCredentials } from '@/types/backend'
import * as tauri from '@/api/tauri'
import { Notify } from 'quasar'
import VideoPlayer from '@/components/VideoPlayer.vue'

const router = useRouter()
const bugStore = useBugStore()
const sessionStore = useSessionStore()
const $q = useQuasar()

// State
const selectedBugId = ref<string | null>(null)
const bugCaptures = ref<Record<string, Capture[]>>({})
const showScreenshotDialog = ref(false)
const currentScreenshotIndex = ref(0)

// Linear push state
const showPushDialog = ref(false)
const isPushing = ref(false)
const hasCredentials = ref(false)
const showPreview = ref(false)
const linearCredentials = ref<TicketingCredentials>({
  api_key: '',
  team_id: null,
  workspace_id: null
})

interface PushResult {
  bugId: string
  bugTitle: string
  success: boolean
  identifier?: string
  url?: string
  error?: string
}

interface TicketPreview {
  bugId: string
  bugTitle: string
  title: string
  description: string
  labels: string[]
  attachmentCount: number
}

const pushResults = ref<PushResult[]>([])
const pushProgress = ref(0)
const ticketPreviews = ref<TicketPreview[]>([])

// AI Description Generation State
const aiDescription = ref<string>('')
const isGenerating = ref(false)
const isGeneratingAll = ref(false)
const claudeAvailable = ref(false)
const claudeStatusMessage = ref('')
const showRefineDialog = ref(false)
const refinementInstructions = ref('')

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

const selectedBugScreenshots = computed(() =>
  selectedBugCaptures.value.filter(c => c.file_type === 'screenshot')
)

const selectedBugVideos = computed(() =>
  selectedBugCaptures.value.filter(c => c.file_type === 'video')
)

// Only bugs that are "ready" status can be pushed
const finalizedBugs = computed(() => {
  return bugs.value.filter(b => b.status === 'ready' || b.status === 'reviewed')
})

// Parsed console data for the selected bug
const selectedBugConsoleParse = computed((): tauri.ConsoleParsed | null => {
  if (!selectedBug.value?.console_parse_json) return null
  try {
    return JSON.parse(selectedBug.value.console_parse_json) as tauri.ConsoleParsed
  } catch {
    return null
  }
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
      bugCaptures.value[bugId] = captures
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

async function updateNotes(notes: string) {
  if (!selectedBug.value) return

  try {
    await bugStore.updateBackendBug(selectedBug.value.id, { notes })
  } catch (err) {
    console.error('Failed to update notes:', err)
  }
}

async function updateMetadata(field: 'meeting_id' | 'software_version', value: string) {
  if (!selectedBug.value) return

  try {
    await bugStore.updateBackendBug(selectedBug.value.id, {
      [field]: value || null
    })
  } catch (err) {
    console.error('Failed to update metadata:', err)
  }
}

async function toggleConsoleCapture(captureId: string, isConsole: boolean) {
  try {
    await tauri.updateCaptureConsoleFlag(captureId, isConsole)

    // Reload captures for selected bug
    if (selectedBugId.value) {
      const captures = await tauri.getBugCaptures(selectedBugId.value)
      bugCaptures.value[selectedBugId.value] = captures

      // If marking as console, parse the screenshot and save results to the bug
      if (isConsole && claudeAvailable.value && selectedBug.value) {
        const markedCapture = captures.find(c => c.id === captureId)
        if (markedCapture) {
          try {
            // Use annotated_path if available, otherwise file_path
            const pathToParse = markedCapture.annotated_path || markedCapture.file_path

            $q.notify({
              type: 'info',
              message: 'Parsing console screenshot with Claude...',
              position: 'top',
              timeout: 2000
            })

            const parsedResult = await tauri.parseConsoleScreenshot(pathToParse)

            // Merge with any existing parses for this bug
            const existingJson = selectedBug.value.console_parse_json
            const merged: tauri.ConsoleParsed = existingJson
              ? (() => {
                  const existing = JSON.parse(existingJson) as tauri.ConsoleParsed
                  return {
                    errors: [...existing.errors, ...parsedResult.errors],
                    warnings: [...existing.warnings, ...parsedResult.warnings],
                    logs: [...existing.logs, ...parsedResult.logs]
                  }
                })()
              : parsedResult

            // Persist to database
            await tauri.updateBugConsoleParse(selectedBug.value.id, merged)

            // Update local state
            const bug = bugStore.backendBugs.find(b => b.id === selectedBug.value!.id)
            if (bug) {
              bug.console_parse_json = JSON.stringify(merged)
            }

            $q.notify({
              type: 'positive',
              message: `Parsed: ${parsedResult.errors.length} error(s), ${parsedResult.warnings.length} warning(s), ${parsedResult.logs.length} log(s)`,
              position: 'top',
              timeout: 3000
            })
          } catch (parseErr) {
            console.error('Failed to parse console screenshot:', parseErr)
            $q.notify({
              type: 'warning',
              message: 'Console marked but parsing failed (Claude may be unavailable)',
              position: 'top',
              timeout: 3000
            })
          }
        }
      }
    }

    $q.notify({
      type: 'positive',
      message: isConsole ? 'Marked as console capture' : 'Unmarked as console capture',
      position: 'top',
      timeout: 1000
    })
  } catch (err) {
    console.error('Failed to toggle console capture:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to update console capture status',
      position: 'top'
    })
  }
}

function viewScreenshot(bugId: string, screenshotIndex: number) {
  if (selectedBugId.value !== bugId) {
    selectBug(bugId)
  }
  currentScreenshotIndex.value = screenshotIndex
  showScreenshotDialog.value = true
}

/** Return the index of `capture` within the screenshots-only list for the given bug. */
function getScreenshotIndex(bugId: string, captureId: string): number {
  const screenshots = (bugCaptures.value[bugId] || []).filter(c => c.file_type === 'screenshot')
  return screenshots.findIndex(c => c.id === captureId)
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

// Linear push methods
async function checkCredentials() {
  try {
    const creds = await tauri.ticketingGetCredentials()
    if (creds && creds.api_key) {
      hasCredentials.value = true
      linearCredentials.value = creds
    } else {
      hasCredentials.value = false
    }
  } catch (err) {
    console.error('Failed to check credentials:', err)
    hasCredentials.value = false
  }
}

async function saveCredentialsAndPush() {
  if (!linearCredentials.value.api_key) {
    Notify.create({
      type: 'negative',
      message: 'API Key is required'
    })
    return
  }

  try {
    // Save credentials
    await tauri.ticketingSaveCredentials(linearCredentials.value)

    // Authenticate
    await tauri.ticketingAuthenticate(linearCredentials.value)

    hasCredentials.value = true

    Notify.create({
      type: 'positive',
      message: 'Credentials saved successfully'
    })

    // Start push
    await pushToLinear()
  } catch (err) {
    console.error('Authentication failed:', err)
    Notify.create({
      type: 'negative',
      message: `Authentication failed: ${err}`
    })
  }
}

async function generatePreview() {
  showPreview.value = true
  ticketPreviews.value = []

  const bugsToProcess = finalizedBugs.value

  for (const bug of bugsToProcess) {
    // Get bug captures for attachments
    const captures = bugCaptures.value[bug.id] || []
    const attachmentPaths = captures
      .filter(c => c.file_type === 'screenshot')
      .map(c => c.file_path)

    // Read description from description.md if it exists
    const description = bug.description || bug.ai_description || bug.notes || 'No description available'

    ticketPreviews.value.push({
      bugId: bug.id,
      bugTitle: bug.title || `Bug ${bug.display_id}`,
      title: bug.title || `Bug ${bug.display_id}`,
      description,
      labels: [bug.type],
      attachmentCount: attachmentPaths.length
    })
  }
}

async function pushToLinear() {
  isPushing.value = true
  pushResults.value = []
  pushProgress.value = 0
  showPreview.value = false

  const bugsToProcess = finalizedBugs.value
  const totalBugs = bugsToProcess.length

  for (let i = 0; i < bugsToProcess.length; i++) {
    const bug = bugsToProcess[i]
    if (!bug) continue

    try {
      // Get bug captures for attachments
      const captures = bugCaptures.value[bug.id] || []
      const attachmentPaths = captures
        .filter(c => c.file_type === 'screenshot')
        .map(c => c.file_path)

      // Read description from description.md if it exists
      let description = bug.description || bug.ai_description || bug.notes || 'No description available'

      // Create ticket request
      const request = {
        title: bug.title || `Bug ${bug.display_id}`,
        description,
        attachments: attachmentPaths,
        labels: [bug.type]
      }

      // Push to Linear
      const response = await tauri.ticketingCreateTicket(request)

      pushResults.value.push({
        bugId: bug.id,
        bugTitle: bug.title || `Bug ${bug.display_id}`,
        success: true,
        identifier: response.identifier,
        url: response.url
      })

      Notify.create({
        type: 'positive',
        message: `Pushed ${bug.display_id} to Linear`,
        timeout: 1000
      })
    } catch (err) {
      console.error(`Failed to push bug ${bug.id}:`, err)
      pushResults.value.push({
        bugId: bug.id,
        bugTitle: bug.title || `Bug ${bug.display_id}`,
        success: false,
        error: String(err)
      })
    }

    // Update progress
    pushProgress.value = (i + 1) / totalBugs
  }

  isPushing.value = false

  // Show summary notification
  const successCount = pushResults.value.filter(r => r.success).length
  const failCount = pushResults.value.length - successCount

  if (failCount === 0) {
    Notify.create({
      type: 'positive',
      message: `Successfully pushed ${successCount} bug${successCount > 1 ? 's' : ''} to Linear`
    })
  } else {
    Notify.create({
      type: 'warning',
      message: `Pushed ${successCount} bug${successCount > 1 ? 's' : ''}, ${failCount} failed`
    })
  }
}

function closePushDialog() {
  showPushDialog.value = false
  showPreview.value = false
  pushResults.value = []
  ticketPreviews.value = []
  pushProgress.value = 0
}

function openUrl(url: string) {
  window.open(url, '_blank')
}

// AI Description Generation Methods
async function checkClaudeStatus() {
  try {
    const status = await tauri.getClaudeStatus()
    if (status.status === 'ready') {
      claudeAvailable.value = true
      claudeStatusMessage.value = `Claude CLI v${status.version} is ready`
    } else if (status.status === 'notAuthenticated') {
      claudeAvailable.value = false
      claudeStatusMessage.value = status.message ?? 'Claude CLI not authenticated'
    } else if (status.status === 'notInstalled') {
      claudeAvailable.value = false
      claudeStatusMessage.value = status.message ?? 'Claude CLI not installed'
    }
  } catch (err) {
    console.error('Failed to check Claude status:', err)
    claudeAvailable.value = false
    claudeStatusMessage.value = 'Failed to check Claude CLI status'
  }
}

async function generateDescription() {
  if (!selectedBug.value) return

  try {
    isGenerating.value = true

    // Gather screenshot paths for the bug
    const screenshotPaths = selectedBugCaptures.value.map(c => c.file_path)

    // Build bug context
    const bugContext: tauri.BugContext = {
      bug_id: selectedBug.value.id,
      bug_type: selectedBug.value.type,
      notes: selectedBug.value.notes || undefined,
      screenshot_paths: screenshotPaths,
      metadata: {
        display_id: selectedBug.value.display_id,
        status: selectedBug.value.status,
      }
    }

    // Call Claude CLI
    const response = await tauri.generateBugDescription(bugContext)
    aiDescription.value = response.text

    $q.notify({
      type: 'positive',
      message: 'Description generated successfully',
      position: 'top'
    })
  } catch (err) {
    console.error('Failed to generate description:', err)
    $q.notify({
      type: 'negative',
      message: `Failed to generate description: ${err}`,
      position: 'top'
    })
  } finally {
    isGenerating.value = false
  }
}

async function refineDescription() {
  if (!selectedBug.value || !aiDescription.value) return

  try {
    showRefineDialog.value = false
    isGenerating.value = true

    const response = await tauri.refineBugDescription(
      aiDescription.value,
      refinementInstructions.value,
      selectedBug.value.id
    )
    aiDescription.value = response.text
    refinementInstructions.value = ''

    $q.notify({
      type: 'positive',
      message: 'Description refined successfully',
      position: 'top'
    })
  } catch (err) {
    console.error('Failed to refine description:', err)
    $q.notify({
      type: 'negative',
      message: `Failed to refine description: ${err}`,
      position: 'top'
    })
  } finally {
    isGenerating.value = false
  }
}

async function saveDescription() {
  if (!selectedBug.value || !aiDescription.value) return

  try {
    await tauri.saveBugDescription(selectedBug.value.folder_path, aiDescription.value)

    $q.notify({
      type: 'positive',
      message: 'Description saved to description.md',
      position: 'top'
    })
  } catch (err) {
    console.error('Failed to save description:', err)
    $q.notify({
      type: 'negative',
      message: `Failed to save description: ${err}`,
      position: 'top'
    })
  }
}

async function generateAllDescriptions() {
  if (bugs.value.length === 0) return

  isGeneratingAll.value = true

  try {
    let successCount = 0
    let failCount = 0

    for (const bug of bugs.value) {
      try {
        // Load captures for this bug
        await loadBugCaptures(bug.id)
        const captures = bugCaptures.value[bug.id] || []
        const screenshotPaths = captures
          .filter(c => c.file_type === 'screenshot')
          .map(c => c.file_path)

        // Build bug context
        const bugContext: tauri.BugContext = {
          bug_id: bug.id,
          bug_type: bug.type,
          notes: bug.notes || undefined,
          screenshot_paths: screenshotPaths,
          metadata: {
            display_id: bug.display_id,
            status: bug.status,
          }
        }

        // Generate description
        const response = await tauri.generateBugDescription(bugContext)

        // Save description
        await tauri.saveBugDescription(bug.folder_path, response.text)

        successCount++
      } catch (err) {
        console.error(`Failed to generate description for bug ${bug.id}:`, err)
        failCount++
      }
    }

    if (failCount === 0) {
      $q.notify({
        type: 'positive',
        message: `Generated descriptions for all ${successCount} bugs`,
        position: 'top'
      })
    } else {
      $q.notify({
        type: 'warning',
        message: `Generated ${successCount} descriptions, ${failCount} failed`,
        position: 'top'
      })
    }
  } catch (err) {
    console.error('Failed to generate all descriptions:', err)
    $q.notify({
      type: 'negative',
      message: `Failed to generate descriptions: ${err}`,
      position: 'top'
    })
  } finally {
    isGeneratingAll.value = false
  }
}

function copyBugToClipboard() {
  if (!selectedBug.value) return

  // Build markdown format
  const captures = selectedBugCaptures.value
  const description = selectedBug.value.description || selectedBug.value.ai_description || selectedBug.value.notes || 'No description available'

  let markdown = `# ${selectedBug.value.title || 'Untitled Bug'}\n\n`
  markdown += `**Bug ID:** ${selectedBug.value.display_id}\n`
  markdown += `**Type:** ${selectedBug.value.type}\n`
  markdown += `**Status:** ${selectedBug.value.status}\n`
  markdown += `**Folder:** ${selectedBug.value.folder_path}\n\n`
  markdown += `## Description\n\n${description}\n\n`

  if (selectedBug.value.notes) {
    markdown += `## Notes\n\n${selectedBug.value.notes}\n\n`
  }

  if (captures.length > 0) {
    markdown += `## Screenshots\n\n`
    captures.forEach((capture, index) => {
      markdown += `- Screenshot ${index + 1}: ${capture.file_path}\n`
    })
  }

  // Copy to clipboard
  navigator.clipboard.writeText(markdown).then(() => {
    $q.notify({
      type: 'positive',
      message: 'Copied bug details to clipboard as markdown',
      position: 'top'
    })
  }).catch(err => {
    console.error('Failed to copy to clipboard:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to copy to clipboard',
      position: 'top'
    })
  })
}

async function openBugFolder() {
  if (!selectedBug.value) return

  try {
    await tauri.openBugFolder(selectedBug.value.folder_path)
  } catch (err) {
    console.error('Failed to open folder:', err)
    $q.notify({
      type: 'negative',
      message: `Failed to open folder: ${err}`,
      position: 'top'
    })
  }
}

function confirmDeleteBug() {
  if (!selectedBug.value) return

  $q.dialog({
    title: 'Delete Bug',
    message: `Are you sure you want to delete ${selectedBug.value.display_id}? This will remove all captures and data.`,
    cancel: true,
    persistent: true
  }).onOk(async () => {
    await deleteBug()
  })
}

async function deleteBug() {
  if (!selectedBug.value) return

  const bugToDelete = selectedBug.value

  try {
    await bugStore.deleteBug(bugToDelete.id)

    $q.notify({
      type: 'positive',
      message: 'Bug deleted successfully',
      position: 'top'
    })

    // Select another bug if available
    if (bugs.value.length > 0) {
      selectBug(bugs.value[0]!.id)
    } else {
      selectedBugId.value = null
    }
  } catch (err) {
    console.error('Failed to delete bug:', err)
    $q.notify({
      type: 'negative',
      message: `Failed to delete bug: ${err}`,
      position: 'top'
    })
  }
}

async function markBugReady() {
  if (!selectedBug.value) return

  try {
    await bugStore.updateBackendBug(selectedBug.value.id, { status: 'ready' })

    $q.notify({
      type: 'positive',
      message: 'Bug marked as ready',
      position: 'top'
    })
  } catch (err) {
    console.error('Failed to mark bug as ready:', err)
    $q.notify({
      type: 'negative',
      message: `Failed to mark bug as ready: ${err}`,
      position: 'top'
    })
  }
}

async function resumeSession() {
  if (!sessionStore.activeSession) return

  try {
    await sessionStore.updateSessionStatus(sessionStore.activeSession.id, 'active')

    $q.notify({
      type: 'positive',
      message: 'Session resumed',
      position: 'top'
    })

    // Navigate back to active session view
    router.push('/')
  } catch (err) {
    console.error('Failed to resume session:', err)
    $q.notify({
      type: 'negative',
      message: `Failed to resume session: ${err}`,
      position: 'top'
    })
  }
}

function confirmCloseSession() {
  if (!sessionStore.activeSession) return

  $q.dialog({
    title: 'Close Session',
    message: 'Are you sure you want to close this session? You can view it later from the history.',
    cancel: true,
    persistent: true
  }).onOk(async () => {
    await closeSession()
  })
}

async function closeSession() {
  if (!sessionStore.activeSession) return

  try {
    await sessionStore.endSession(sessionStore.activeSession.id)

    $q.notify({
      type: 'positive',
      message: 'Session closed',
      position: 'top'
    })

    // Navigate back to main view
    router.push('/')
  } catch (err) {
    console.error('Failed to close session:', err)
    $q.notify({
      type: 'negative',
      message: `Failed to close session: ${err}`,
      position: 'top'
    })
  }
}

// Lifecycle
onMounted(async () => {
  // Check Claude CLI status
  await checkClaudeStatus()

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

  // Check for stored credentials
  await checkCredentials()
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
  position: relative;
}

.console-tag-btn {
  position: absolute;
  top: 8px;
  right: 8px;
  z-index: 10;
}

.screenshot {
  width: 100%;
  height: 100%;
}

.whitespace-pre-wrap {
  white-space: pre-wrap;
}

.description-editor {
  font-family: monospace;
  font-size: 14px;
}

.console-output-block {
  border-radius: 4px;
  padding: 8px;
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

.rounded {
  border-radius: 4px;
}
</style>
