<template>
  <q-page class="q-pa-md">
    <div
      class="help-container"
      style="max-width: 860px; margin: 0 auto;"
    >
      <div class="row items-center q-mb-md">
        <q-btn
          flat
          dense
          icon="arrow_back"
          label="Back"
          class="q-mr-md"
          @click="goBack"
        />
        <h1 class="text-h4 q-my-none">
          Help / User Guide
        </h1>
      </div>

      <!-- Overview -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-sm">
            <q-icon
              name="info"
              class="q-mr-sm"
            />
            Overview
          </div>
          <p class="text-body1 q-mb-none">
            Unbroken QA Capture streamlines manual QA testing by letting you stay
            focused on the app under test. Use global hotkeys to start sessions,
            capture bugs, and annotate screenshots — all without switching windows.
            At the end of a session, review your bugs and get AI-drafted descriptions
            ready for your ticketing workflow.
          </p>
        </q-card-section>
      </q-card>

      <!-- Workflow Steps -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            <q-icon
              name="play_circle"
              class="q-mr-sm"
            />
            Workflow
          </div>

          <q-timeline color="primary">
            <q-timeline-entry
              title="1. Start a Session"
              icon="play_arrow"
              color="positive"
            >
              <p>
                Press <kbd>{{ settingsStore.hotkeyToggleSession }}</kbd> to start a QA session.
                You can also click <strong>Start Session</strong> from the system tray
                menu or the home screen. A session tracks all bugs you capture until you
                end it.
              </p>
            </q-timeline-entry>

            <q-timeline-entry
              title="2. Capture a Bug"
              icon="bug_report"
              color="negative"
            >
              <p>
                When you spot a bug, press <kbd>{{ settingsStore.hotkeyStartBugCapture }}</kbd> to begin a bug
                capture. The app automatically records a screenshot. You can take
                multiple screenshots during a single bug capture — each press of
                <kbd>{{ settingsStore.hotkeyStartBugCapture }}</kbd> adds another screenshot to the current bug.
              </p>
            </q-timeline-entry>

            <q-timeline-entry
              title="3. Annotate Screenshots"
              icon="edit"
              color="warning"
            >
              <p>
                After capturing a screenshot, the Annotator opens automatically
                (if enabled in Settings). Use the annotation toolbar to draw
                arrows, add labels, and highlight areas of interest. Click
                <strong>Save</strong> when done — the annotated version replaces
                the original in the bug folder.
              </p>
            </q-timeline-entry>

            <q-timeline-entry
              title="4. End Bug Capture"
              icon="check_circle"
              color="primary"
            >
              <p>
                Press <kbd>{{ settingsStore.hotkeyEndBugCapture }}</kbd> to end the current
                bug capture. The bug is saved with all its screenshots. You can
                also end a bug capture from the system tray menu.
              </p>
            </q-timeline-entry>

            <q-timeline-entry
              title="5. Capture More Bugs"
              icon="repeat"
              color="secondary"
            >
              <p>
                Repeat steps 2–4 for each bug you find. All bugs are associated
                with the active session and stored together in the session folder.
              </p>
            </q-timeline-entry>

            <q-timeline-entry
              title="6. End Session &amp; Review"
              icon="summarize"
              color="purple"
            >
              <p>
                Press <kbd>{{ settingsStore.hotkeyToggleSession }}</kbd> again to end the session, or use the system
                tray menu. The app navigates to the Session Review page where
                you can see all captured bugs, read AI-drafted descriptions, and
                copy formatted bug reports to your clipboard for pasting into
                your ticketing system.
              </p>
            </q-timeline-entry>
          </q-timeline>
        </q-card-section>
      </q-card>

      <!-- Screenshot Association -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-sm">
            <q-icon
              name="photo_library"
              class="q-mr-sm"
            />
            Associating Screenshots with Bugs
          </div>
          <p class="text-body2">
            Screenshots are automatically associated with the <strong>active bug</strong>
            at the time they are taken:
          </p>
          <ul class="text-body2">
            <li>Start a bug capture ({{ settingsStore.hotkeyStartBugCapture }} or tray menu)</li>
            <li>
              Every screenshot taken while the bug is active is saved into that
              bug's folder
            </li>
            <li>End the bug capture ({{ settingsStore.hotkeyEndBugCapture }}) before starting the next one</li>
            <li>
              Screenshots taken <em>outside</em> an active bug capture are not
              associated with any bug — start a capture first
            </li>
          </ul>
          <q-banner
            dense
            class="bg-blue-1 text-blue-9 q-mt-sm"
          >
            <template #avatar>
              <q-icon
                name="tips_and_updates"
                color="blue-7"
              />
            </template>
            Tip: One bug capture per bug. End the current bug capture before
            starting a new one so each bug has its own screenshots.
          </q-banner>
        </q-card-section>
      </q-card>

      <!-- Hotkey Reference -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            <q-icon
              name="keyboard"
              class="q-mr-sm"
            />
            Hotkey Reference
          </div>
          <q-markup-table flat>
            <thead>
              <tr>
                <th class="text-left">
                  Action
                </th>
                <th class="text-left">
                  Default Hotkey
                </th>
                <th class="text-left">
                  Description
                </th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td>Toggle Session</td>
                <td><kbd>{{ settingsStore.hotkeyToggleSession }}</kbd></td>
                <td>Start a new session, or end the active session</td>
              </tr>
              <tr>
                <td>Start Bug Capture</td>
                <td><kbd>{{ settingsStore.hotkeyStartBugCapture }}</kbd></td>
                <td>Begin a new bug capture and take the first screenshot</td>
              </tr>
              <tr>
                <td>End Bug Capture</td>
                <td><kbd>{{ settingsStore.hotkeyEndBugCapture }}</kbd></td>
                <td>Finish the current bug capture</td>
              </tr>
              <tr>
                <td>Quick Notepad</td>
                <td><kbd>{{ settingsStore.hotkeyOpenQuickNotepad }}</kbd></td>
                <td>Open a floating notepad for quick notes (not tied to a session)</td>
              </tr>
              <tr>
                <td>Session Notepad</td>
                <td><kbd>{{ settingsStore.hotkeyOpenSessionNotepad }}</kbd></td>
                <td>Open the session notepad for notes associated with the current session</td>
              </tr>
            </tbody>
          </q-markup-table>
          <p class="text-caption text-grey-6 q-mt-sm">
            Hotkeys can be customised in
            <q-btn
              flat
              dense
              size="sm"
              label="Settings"
              icon="settings"
              @click="goToSettings"
            />
          </p>
        </q-card-section>
      </q-card>

      <!-- Features -->
      <q-expansion-item
        icon="star"
        label="Features"
        header-class="text-h6"
        class="q-mb-md shadow-1"
        default-opened
      >
        <q-card>
          <q-card-section>
            <q-list separator>
              <q-item>
                <q-item-section avatar>
                  <q-icon
                    name="screenshot_monitor"
                    color="primary"
                  />
                </q-item-section>
                <q-item-section>
                  <q-item-label class="text-weight-medium">
                    Automatic Screenshot Capture
                  </q-item-label>
                  <q-item-label caption>
                    Screenshots are captured automatically using Print Screen and
                    organised into per-bug folders.
                  </q-item-label>
                </q-item-section>
              </q-item>

              <q-item>
                <q-item-section avatar>
                  <q-icon
                    name="draw"
                    color="warning"
                  />
                </q-item-section>
                <q-item-section>
                  <q-item-label class="text-weight-medium">
                    Screenshot Annotation
                  </q-item-label>
                  <q-item-label caption>
                    Annotate screenshots with arrows, text labels, and highlights
                    directly in the app. Enable auto-open in Settings.
                  </q-item-label>
                </q-item-section>
              </q-item>

              <q-item>
                <q-item-section avatar>
                  <q-icon
                    name="smart_toy"
                    color="purple"
                  />
                </q-item-section>
                <q-item-section>
                  <q-item-label class="text-weight-medium">
                    AI-Drafted Bug Descriptions
                  </q-item-label>
                  <q-item-label caption>
                    Claude AI automatically drafts structured bug descriptions
                    based on captured screenshots and metadata. Requires Claude
                    CLI installed. Works without it too — descriptions can be
                    written manually.
                  </q-item-label>
                </q-item-section>
              </q-item>

              <q-item>
                <q-item-section avatar>
                  <q-icon
                    name="content_copy"
                    color="teal"
                  />
                </q-item-section>
                <q-item-section>
                  <q-item-label class="text-weight-medium">
                    One-Click Copy to Clipboard
                  </q-item-label>
                  <q-item-label caption>
                    Copy formatted bug reports to your clipboard from the Session
                    Review page and paste them directly into Linear, Jira, or any
                    other ticketing tool.
                  </q-item-label>
                </q-item-section>
              </q-item>

              <q-item>
                <q-item-section avatar>
                  <q-icon
                    name="folder_open"
                    color="orange"
                  />
                </q-item-section>
                <q-item-section>
                  <q-item-label class="text-weight-medium">
                    Organised Session Folders
                  </q-item-label>
                  <q-item-label caption>
                    Every session and bug is stored as a folder on disk — no
                    proprietary database. All data stays local, with no cloud
                    sync or telemetry.
                  </q-item-label>
                </q-item-section>
              </q-item>

              <q-item>
                <q-item-section avatar>
                  <q-icon
                    name="system_update_alt"
                    color="indigo"
                  />
                </q-item-section>
                <q-item-section>
                  <q-item-label class="text-weight-medium">
                    System Tray Integration
                  </q-item-label>
                  <q-item-label caption>
                    The app lives in the system tray. Start sessions, capture bugs,
                    and end sessions without ever switching away from the app under
                    test.
                  </q-item-label>
                </q-item-section>
              </q-item>
            </q-list>
          </q-card-section>
        </q-card>
      </q-expansion-item>

      <!-- Tips -->
      <q-expansion-item
        icon="lightbulb"
        label="Tips &amp; Best Practices"
        header-class="text-h6"
        class="q-mb-md shadow-1"
      >
        <q-card>
          <q-card-section>
            <q-list separator>
              <q-item>
                <q-item-section avatar>
                  <q-icon
                    name="speed"
                    color="green"
                  />
                </q-item-section>
                <q-item-section>
                  <q-item-label class="text-weight-medium">
                    Stay in your testing flow
                  </q-item-label>
                  <q-item-label caption>
                    Use only hotkeys during testing. Review and annotate bugs
                    after the session when you are no longer time-pressured.
                  </q-item-label>
                </q-item-section>
              </q-item>

              <q-item>
                <q-item-section avatar>
                  <q-icon
                    name="note_alt"
                    color="blue"
                  />
                </q-item-section>
                <q-item-section>
                  <q-item-label class="text-weight-medium">
                    Use the Quick Notepad for context
                  </q-item-label>
                  <q-item-label caption>
                    Press Ctrl+Shift+N to jot a quick note mid-test (e.g. "this
                    only happens on first login"). Claude will use these notes
                    when drafting bug descriptions.
                  </q-item-label>
                </q-item-section>
              </q-item>

              <q-item>
                <q-item-section avatar>
                  <q-icon
                    name="bug_report"
                    color="red"
                  />
                </q-item-section>
                <q-item-section>
                  <q-item-label class="text-weight-medium">
                    One bug per capture
                  </q-item-label>
                  <q-item-label caption>
                    Start a new bug capture for each distinct bug. This keeps
                    screenshots and AI descriptions focused and avoids confusion
                    during review.
                  </q-item-label>
                </q-item-section>
              </q-item>

              <q-item>
                <q-item-section avatar>
                  <q-icon
                    name="folder"
                    color="orange"
                  />
                </q-item-section>
                <q-item-section>
                  <q-item-label class="text-weight-medium">
                    Set your sessions folder
                  </q-item-label>
                  <q-item-label caption>
                    Configure a dedicated folder for sessions in Settings. This
                    makes it easy to find all your captured data later.
                  </q-item-label>
                </q-item-section>
              </q-item>
            </q-list>
          </q-card-section>
        </q-card>
      </q-expansion-item>

      <!-- Reviewing Results -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-sm">
            <q-icon
              name="rate_review"
              class="q-mr-sm"
            />
            Reviewing Results
          </div>
          <p class="text-body2">
            After ending a session, the <strong>Session Review</strong> page shows
            all bugs captured:
          </p>
          <ul class="text-body2">
            <li>Click a bug to view its screenshots and AI-drafted description</li>
            <li>Edit the description if needed</li>
            <li>Use <strong>Copy to Clipboard</strong> to copy the formatted report</li>
            <li>Use <strong>Open Folder</strong> to access the raw files on disk</li>
            <li>Navigate back with the Back button or <kbd>Escape</kbd></li>
          </ul>
        </q-card-section>
      </q-card>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useSettingsStore } from '../stores/settings'

const router = useRouter()
const settingsStore = useSettingsStore()

function goBack() {
  router.back()
}

function goToSettings() {
  router.push({ name: 'settings' })
}

function onKeyDown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    goBack()
  }
}

onMounted(() => {
  document.addEventListener('keydown', onKeyDown)
  settingsStore.loadAllSettings().catch(() => {/* uses defaults if backend unavailable */})
})

onUnmounted(() => {
  document.removeEventListener('keydown', onKeyDown)
})
</script>

<style scoped>
kbd {
  display: inline-block;
  padding: 2px 6px;
  font-size: 0.85em;
  font-family: monospace;
  background-color: #f1f3f5;
  border: 1px solid #ced4da;
  border-radius: 4px;
  box-shadow: 0 1px 0 rgba(0,0,0,.2);
}
</style>
