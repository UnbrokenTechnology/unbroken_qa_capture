import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createRouter, createMemoryHistory } from 'vue-router'
import { Quasar, Notify } from 'quasar'
import IdleView from '@/views/IdleView.vue'
import * as tauri from '@/api/tauri'

// Mock useQuasar
const mockNotify = vi.fn()
vi.mock('quasar', async () => {
  const actual = await vi.importActual('quasar')
  return {
    ...actual,
    useQuasar: () => ({
      notify: mockNotify,
    })
  }
})

// Mock Tauri API
vi.mock('@/api/tauri', () => ({
  createSession: vi.fn(),
  getActiveSession: vi.fn().mockResolvedValue(null),
  getSessionSummaries: vi.fn().mockResolvedValue([]),
  getClaudeStatus: vi.fn().mockResolvedValue({ status: 'ready', version: 'Claude Code' }),
  listProfiles: vi.fn().mockResolvedValue([]),
  getActiveProfileId: vi.fn().mockResolvedValue(null),
  setActiveProfileId: vi.fn().mockResolvedValue(undefined),
}))

// Mock Tauri event listeners
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))

// Mock Tauri core invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
  convertFileSrc: vi.fn((path: string) => `asset://localhost/${path}`),
}))

const mockSession = {
  id: 'session-1',
  started_at: '2024-01-01T10:00:00Z',
  ended_at: null,
  status: 'active' as const,
  folder_path: '/test/sessions/session1',
  session_notes: null,
  environment_json: null,
  original_snip_path: null,
  created_at: '2024-01-01T10:00:00Z',
  profile_id: null,
}

describe('IdleView', () => {
  let pinia: ReturnType<typeof createPinia>
  let router: ReturnType<typeof createRouter>

  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)

    router = createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: '/', name: 'home', component: IdleView },
        { path: '/active-session', name: 'active-session', component: { template: '<div>active</div>' } },
      ]
    })

    vi.clearAllMocks()
    vi.mocked(tauri.getSessionSummaries).mockResolvedValue([])
    vi.mocked(tauri.getActiveSession).mockResolvedValue(null)
    vi.mocked(tauri.getClaudeStatus).mockResolvedValue({ status: 'ready', version: 'Claude Code' })
    vi.mocked(tauri.listProfiles).mockResolvedValue([])
    vi.mocked(tauri.getActiveProfileId).mockResolvedValue(null)
    vi.mocked(tauri.setActiveProfileId).mockResolvedValue(undefined)
  })

  function mountIdleView() {
    return mount(IdleView, {
      global: {
        plugins: [pinia, router, [Quasar, { plugins: { Notify } }]],
        stubs: {
          QPage: { template: '<div><slot /></div>' },
          QBtn: { template: '<button class="q-btn" @click="$attrs.onClick && $attrs.onClick()"><slot /></button>' },
          QIcon: { template: '<span />' },
          QList: { template: '<ul><slot /></ul>' },
          QItem: { template: '<li @click="$attrs.onClick && $attrs.onClick()"><slot /></li>' },
          QItemSection: { template: '<div><slot /></div>' },
          QItemLabel: { template: '<span><slot /></span>' },
          QBadge: { template: '<span />' },
          QSpinner: { template: '<div />' },
          QChip: { template: '<span />' },
          QBanner: { template: '<div class="q-banner" v-bind="$attrs"><slot name="avatar" /><slot /><slot name="action" /></div>' },
          QSelect: { template: '<div class="q-select"><slot name="prepend" /><slot /></div>', props: ['modelValue', 'options', 'label', 'emitValue', 'mapOptions', 'clearable', 'outlined', 'dense'] },
        }
      }
    })
  }

  describe('handleStartSession', () => {
    it('shows a negative notification when session start fails', async () => {
      vi.mocked(tauri.createSession).mockRejectedValue(new Error('Session manager not initialized'))

      const wrapper = mountIdleView()
      await flushPromises()

      // Find the Start Session button and trigger it
      const buttons = wrapper.findAll('button.q-btn')
      // First button should be the Start Session button
      const startBtn = buttons[0]
      expect(startBtn).toBeDefined()

      await startBtn!.trigger('click')
      await flushPromises()

      expect(mockNotify).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'negative',
          message: 'Failed to start session',
        })
      )
    })

    it('includes the error message in the notification caption', async () => {
      const errorMessage = 'Database connection failed'
      vi.mocked(tauri.createSession).mockRejectedValue(new Error(errorMessage))

      const wrapper = mountIdleView()
      await flushPromises()

      const startBtn = wrapper.findAll('button.q-btn')[0]
      await startBtn!.trigger('click')
      await flushPromises()

      expect(mockNotify).toHaveBeenCalledWith(
        expect.objectContaining({
          caption: errorMessage,
        })
      )
    })

    it('does not show a negative notification on successful session start', async () => {
      vi.mocked(tauri.createSession).mockResolvedValue(mockSession)

      const wrapper = mountIdleView()
      await flushPromises()

      const startBtn = wrapper.findAll('button.q-btn')[0]
      await startBtn!.trigger('click')
      await flushPromises()

      expect(mockNotify).not.toHaveBeenCalledWith(
        expect.objectContaining({ type: 'negative' })
      )
    })
  })

  describe('session store error state', () => {
    it('sets error in session store when start fails', async () => {
      vi.mocked(tauri.createSession).mockRejectedValue(new Error('Backend error'))

      const { useSessionStore } = await import('@/stores/session')
      const sessionStore = useSessionStore()

      const wrapper = mountIdleView()
      await flushPromises()

      const startBtn = wrapper.findAll('button.q-btn')[0]
      await startBtn!.trigger('click')
      await flushPromises()

      expect(sessionStore.error).toBe('Backend error')
    })

    it('clears error state at start of each session attempt', async () => {
      const { useSessionStore } = await import('@/stores/session')
      const sessionStore = useSessionStore()

      // First attempt fails
      vi.mocked(tauri.createSession).mockRejectedValue(new Error('First error'))
      const wrapper = mountIdleView()
      await flushPromises()

      const startBtn = wrapper.findAll('button.q-btn')[0]
      await startBtn!.trigger('click')
      await flushPromises()
      expect(sessionStore.error).toBe('First error')

      // Second attempt succeeds — error should be cleared
      vi.mocked(tauri.createSession).mockResolvedValue(mockSession)
      await startBtn!.trigger('click')
      await flushPromises()

      expect(sessionStore.error).toBeNull()
    })
  })

  describe('profile selector', () => {
    it('does not render profile selector when no profiles exist', async () => {
      vi.mocked(tauri.listProfiles).mockResolvedValue([])

      const wrapper = mountIdleView()
      await flushPromises()

      expect(wrapper.find('.profile-section').exists()).toBe(false)
    })

    it('renders profile selector when profiles are available', async () => {
      vi.mocked(tauri.listProfiles).mockResolvedValue([
        {
          id: 'profile-1',
          name: 'Contio MeetingOS',
          linear_config: null,
          area_categories: [],
          custom_fields: [],
          title_conventions: null,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      ])
      vi.mocked(tauri.getActiveProfileId).mockResolvedValue('profile-1')

      const wrapper = mountIdleView()
      await flushPromises()

      expect(wrapper.find('.profile-section').exists()).toBe(true)
      expect(wrapper.find('.q-select').exists()).toBe(true)
    })

    it('passes active profile_id to startSession', async () => {
      vi.mocked(tauri.listProfiles).mockResolvedValue([
        {
          id: 'profile-1',
          name: 'Contio MeetingOS',
          linear_config: null,
          area_categories: [],
          custom_fields: [],
          title_conventions: null,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      ])
      vi.mocked(tauri.getActiveProfileId).mockResolvedValue('profile-1')
      vi.mocked(tauri.createSession).mockResolvedValue({ ...mockSession, profile_id: 'profile-1' })

      const wrapper = mountIdleView()
      await flushPromises()

      const startBtn = wrapper.findAll('button.q-btn')[0]
      await startBtn!.trigger('click')
      await flushPromises()

      expect(tauri.createSession).toHaveBeenCalledWith(
        expect.objectContaining({ profile_id: 'profile-1' })
      )
    })

    it('passes null profile_id when no active profile', async () => {
      vi.mocked(tauri.listProfiles).mockResolvedValue([])
      vi.mocked(tauri.getActiveProfileId).mockResolvedValue(null)
      vi.mocked(tauri.createSession).mockResolvedValue(mockSession)

      const wrapper = mountIdleView()
      await flushPromises()

      const startBtn = wrapper.findAll('button.q-btn')[0]
      await startBtn!.trigger('click')
      await flushPromises()

      expect(tauri.createSession).toHaveBeenCalledWith(
        expect.objectContaining({ profile_id: null })
      )
    })
  })

  describe('Claude connection banner', () => {
    it('does not show banner when Claude status is ready', async () => {
      vi.mocked(tauri.getClaudeStatus).mockResolvedValue({ status: 'ready', version: 'Claude Code' })

      const wrapper = mountIdleView()
      await flushPromises()

      expect(wrapper.find('.claude-banner').exists()).toBe(false)
    })

    it('shows banner when Claude is not installed', async () => {
      vi.mocked(tauri.getClaudeStatus).mockResolvedValue({
        status: 'notInstalled',
        message: 'Claude Code not found on this system',
      })

      const wrapper = mountIdleView()
      await flushPromises()

      const banner = wrapper.find('.claude-banner')
      expect(banner.exists()).toBe(true)
      expect(banner.text()).toContain('AI features require Claude Code')
    })

    it('shows banner when Claude is not authenticated', async () => {
      vi.mocked(tauri.getClaudeStatus).mockResolvedValue({
        status: 'notAuthenticated',
        version: 'Claude Code',
        message: 'Claude Code is installed but not signed in',
      })

      const wrapper = mountIdleView()
      await flushPromises()

      const banner = wrapper.find('.claude-banner')
      expect(banner.exists()).toBe(true)
      expect(banner.text()).toContain('not signed in')
    })

    it('dismisses banner when close button is clicked', async () => {
      vi.mocked(tauri.getClaudeStatus).mockResolvedValue({
        status: 'notInstalled',
        message: 'Claude Code not found on this system',
      })

      const wrapper = mountIdleView()
      await flushPromises()

      expect(wrapper.find('.claude-banner').exists()).toBe(true)

      // The close button is inside the banner's action slot
      const closeBtn = wrapper.find('.claude-banner button.q-btn')
      expect(closeBtn.exists()).toBe(true)
      await closeBtn.trigger('click')
      await flushPromises()

      expect(wrapper.find('.claude-banner').exists()).toBe(false)
    })
  })
})
