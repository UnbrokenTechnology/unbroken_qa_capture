<template>
  <q-page class="idle-view q-pa-md">
    <div class="content-wrapper">
      <!-- App Branding -->
      <div class="app-branding text-center q-mb-xl">
        <q-icon
          name="bug_report"
          size="64px"
          color="primary"
          class="q-mb-md"
        />
        <h4 class="text-h4 q-my-none">
          Unbroken QA Capture
        </h4>
        <p class="text-subtitle2 text-grey-7">
          Professional QA Bug Reporting
        </p>
      </div>

      <!-- Claude AI Connection Banner -->
      <q-banner
        v-if="showClaudeBanner"
        rounded
        dense
        class="claude-banner q-mb-lg"
        :class="claudeBannerClass"
      >
        <template #avatar>
          <q-icon
            :name="claudeStatus?.status === 'notAuthenticated' ? 'warning' : 'info'"
            :color="claudeStatus?.status === 'notAuthenticated' ? 'warning' : 'grey-7'"
          />
        </template>
        <span v-if="claudeStatus?.status === 'notAuthenticated'">
          Claude Code is installed but not signed in. Run <code>claude</code> in your terminal to authenticate, or
          <a
            class="text-primary cursor-pointer"
            @click="handleSettingsClick"
          >open Settings</a> to check status.
        </span>
        <span v-else>
          AI features require Claude Code. Install it from
          <a
            href="https://claude.ai/download"
            target="_blank"
            class="text-primary"
          >claude.ai/download</a>, then run <code>claude</code> in your terminal to sign in. Or
          <a
            class="text-primary cursor-pointer"
            @click="handleSettingsClick"
          >open Settings</a> to check status.
        </span>
        <template #action>
          <q-btn
            flat
            dense
            round
            icon="close"
            size="sm"
            @click="dismissClaudeBanner"
          />
        </template>
      </q-banner>

      <!-- Active Profile Selector -->
      <div
        v-if="profileStore.profiles.length > 0"
        class="profile-section q-mb-md"
      >
        <q-select
          :model-value="profileStore.activeProfileId"
          :options="profileSelectOptions"
          label="Active Profile"
          outlined
          dense
          emit-value
          map-options
          clearable
          @update:model-value="onProfileChange"
        >
          <template #prepend>
            <q-icon
              name="manage_accounts"
              color="primary"
            />
          </template>
        </q-select>
      </div>

      <!-- Start Session Button -->
      <div class="start-session-section q-mb-xl">
        <q-btn
          unelevated
          color="primary"
          size="lg"
          class="start-session-btn full-width"
          :loading="sessionStore.isStartingSession"
          :disable="sessionStore.isStartingSession"
          @click="handleStartSession"
        >
          <template #loading>
            <q-spinner-dots
              size="1.5em"
              class="q-mr-sm"
            />
            Starting session...
          </template>
          <q-icon
            name="play_arrow"
            left
            size="sm"
          />
          Start Session
          <span class="hotkey-hint q-ml-sm text-caption">(Print Screen)</span>
        </q-btn>
      </div>

      <!-- Recent Sessions -->
      <div class="recent-sessions-section">
        <div class="section-header q-mb-md">
          <h6 class="text-h6 q-my-none">
            Recent Sessions
          </h6>
        </div>

        <div
          v-if="sessionStore.loading"
          class="text-center q-pa-md"
        >
          <q-spinner
            color="primary"
            size="3em"
          />
        </div>

        <div
          v-else-if="sessionStore.sessionSummaries.length === 0"
          class="no-sessions text-center q-pa-lg"
        >
          <q-icon
            name="inbox"
            size="48px"
            color="grey-5"
            class="q-mb-sm"
          />
          <p class="text-body2 text-grey-7">
            No recent sessions found
          </p>
          <p class="text-caption text-grey-6">
            Start a new session to begin capturing bugs
          </p>
        </div>

        <q-list
          v-else
          bordered
          separator
          class="sessions-list rounded-borders"
        >
          <q-item
            v-for="session in recentSessions"
            :key="session.id"
            clickable
            class="session-item"
            @click="handleSessionClick(session)"
          >
            <q-item-section>
              <q-item-label class="text-subtitle2">
                {{ formatSessionDate(session.started_at) }}
              </q-item-label>
              <q-item-label caption>
                <q-icon
                  name="bug_report"
                  size="14px"
                  class="q-mr-xs"
                />
                {{ session.bug_count }} bug{{ session.bug_count !== 1 ? 's' : '' }}
                <span class="q-mx-sm">•</span>
                {{ formatDuration(session) }}
              </q-item-label>
            </q-item-section>

            <q-item-section side>
              <q-badge
                :color="getStatusColor(session.status)"
                :label="session.status"
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

      <!-- Settings Link -->
      <div class="settings-section q-mt-xl text-center">
        <q-btn
          flat
          color="grey-8"
          icon="settings"
          label="Settings"
          @click="handleSettingsClick"
        />
      </div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { onMounted, computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useQuasar } from 'quasar'
import { useSessionStore } from '@/stores/session'
import { useProfileStore } from '@/stores/profile'
import { getClaudeStatus } from '@/api/tauri'
import type { ClaudeStatus } from '@/api/tauri'
import type { SessionSummary } from '@/types/backend'

const router = useRouter()
const $q = useQuasar()
const sessionStore = useSessionStore()
const profileStore = useProfileStore()

const claudeStatus = ref<ClaudeStatus | null>(null)
const claudeBannerDismissed = ref(false)

// Profile selector options
const profileSelectOptions = computed(() =>
  profileStore.profiles.map(p => ({ label: p.name, value: p.id }))
)

async function onProfileChange(value: string | null): Promise<void> {
  try {
    if (value) {
      await profileStore.setActiveProfile(value)
    } else {
      await profileStore.clearActiveProfile()
    }
  } catch (error) {
    console.error('Failed to switch profile:', error)
    $q.notify({
      type: 'negative',
      message: 'Failed to switch profile',
      caption: error instanceof Error ? error.message : String(error),
      position: 'bottom-right',
      timeout: 3000,
    })
  }
}

const showClaudeBanner = computed(() =>
  claudeStatus.value !== null &&
  claudeStatus.value.status !== 'ready' &&
  !claudeBannerDismissed.value
)

const claudeBannerClass = computed(() =>
  claudeStatus.value?.status === 'notAuthenticated'
    ? 'bg-warning-light'
    : 'bg-grey-3'
)

function dismissClaudeBanner() {
  claudeBannerDismissed.value = true
}

// Limit to 10 most recent sessions
const recentSessions = computed(() =>
  sessionStore.sessionSummaries.slice(0, 10)
)

onMounted(async () => {
  // Load recent session summaries, profiles, and check Claude status in parallel
  const sessionsPromise = sessionStore.loadSessionSummaries().catch((error) => {
    console.error('Failed to load session summaries:', error)
    $q.notify({
      type: 'negative',
      message: 'Failed to load recent sessions',
      caption: error instanceof Error ? error.message : String(error),
      position: 'bottom-right',
      timeout: 4000,
    })
  })

  const profilesPromise = profileStore.loadProfiles().catch((err) => {
    console.error('Failed to load profiles:', err)
  })

  const claudePromise = getClaudeStatus().then((status) => {
    claudeStatus.value = status
  }).catch((err) => {
    console.error('Failed to check Claude status:', err)
  })

  await Promise.all([sessionsPromise, profilesPromise, claudePromise])
})

async function handleStartSession() {
  try {
    await sessionStore.startSession(profileStore.activeProfileId ?? null)
    // Navigation will be handled by App.vue watching activeSession
  } catch (error) {
    console.error('Failed to start session:', error)
    $q.notify({
      type: 'negative',
      message: 'Failed to start session',
      caption: error instanceof Error ? error.message : String(error),
      position: 'bottom-right',
      timeout: 5000,
    })
  }
}

function handleSessionClick(session: SessionSummary) {
  router.push({ name: 'session-review', params: { sessionId: session.id } })
}

function handleSettingsClick() {
  router.push({ name: 'settings' })
}

function formatSessionDate(isoString: string): string {
  try {
    const date = new Date(isoString)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24))

    if (diffDays === 0) {
      return `Today at ${date.toLocaleTimeString('en-US', {
        hour: 'numeric',
        minute: '2-digit',
        hour12: true
      })}`
    } else if (diffDays === 1) {
      return `Yesterday at ${date.toLocaleTimeString('en-US', {
        hour: 'numeric',
        minute: '2-digit',
        hour12: true
      })}`
    } else if (diffDays < 7) {
      return date.toLocaleDateString('en-US', {
        weekday: 'long',
        hour: 'numeric',
        minute: '2-digit',
        hour12: true
      })
    } else {
      return date.toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
        year: date.getFullYear() !== now.getFullYear() ? 'numeric' : undefined
      })
    }
  } catch {
    return isoString
  }
}

function formatDuration(session: SessionSummary): string {
  if (!session.ended_at) {
    return 'In progress'
  }

  try {
    const start = new Date(session.started_at)
    const end = new Date(session.ended_at)
    const durationMs = end.getTime() - start.getTime()
    const minutes = Math.floor(durationMs / (1000 * 60))
    const hours = Math.floor(minutes / 60)
    const remainingMinutes = minutes % 60

    if (hours > 0) {
      return `${hours}h ${remainingMinutes}m`
    } else {
      return `${minutes}m`
    }
  } catch {
    return 'Unknown'
  }
}

function getStatusColor(status: string): string {
  switch (status) {
    case 'active':
      return 'green'
    case 'ended':
      return 'orange'
    case 'reviewed':
      return 'blue'
    case 'synced':
      return 'purple'
    default:
      return 'grey'
  }
}
</script>

<style scoped>
.idle-view {
  display: flex;
  justify-content: center;
  align-items: flex-start;
  min-height: 100vh;
  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
}

.content-wrapper {
  max-width: 600px;
  width: 100%;
  padding: 2rem 1rem;
}

.app-branding {
  animation: fadeIn 0.5s ease-in;
}

.start-session-btn {
  font-size: 1.1rem;
  padding: 1rem 2rem;
  font-weight: 600;
  letter-spacing: 0.5px;
  transition: transform 0.2s ease;
  border-radius: 8px;
}

.start-session-btn:hover {
  transform: translateY(-2px);
}

.hotkey-hint {
  opacity: 0.7;
  font-weight: normal;
}

.sessions-list {
  background: white;
  border-radius: 8px;
  animation: slideUp 0.3s ease-out;
}

.session-item {
  transition: background-color 0.2s ease;
}

.session-item:hover {
  background-color: #f5f5f5;
}

.no-sessions {
  background: white;
  border-radius: 8px;
  animation: fadeIn 0.3s ease-in;
}

.claude-banner {
  animation: fadeIn 0.3s ease-in;
}

.claude-banner code {
  background: rgba(0, 0, 0, 0.08);
  padding: 1px 4px;
  border-radius: 3px;
  font-size: 0.9em;
}

.bg-warning-light {
  background: #fff8e1;
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
    padding: 0.75rem 0.5rem;
  }
  .app-branding .q-icon {
    font-size: 40px !important;
  }
  .start-session-btn {
    font-size: 0.95rem;
    padding: 0.6rem 1rem;
  }
}

/* Comfortable mode: 600x800px */
@media (min-width: 441px) and (max-width: 660px) {
  .content-wrapper {
    padding: 1.5rem 1rem;
  }
}

/* Full mode: 1000x800px+ */
@media (min-width: 1000px) {
  .content-wrapper {
    max-width: 800px;
    padding: 2.5rem 2rem;
  }
}
</style>
