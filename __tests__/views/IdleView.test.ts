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
}))

// Mock Tauri event listeners
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))

// Mock Tauri core invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
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
})
