/**
 * Tests for global hotkey event listeners wired in App.vue.
 *
 * Strategy: intercept calls to listen() and capture the event name + callback.
 * Then fire each callback to verify the correct store action is invoked.
 */
import { describe, it, expect, beforeEach, vi, type Mock } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { Quasar } from 'quasar'
import { createRouter, createMemoryHistory } from 'vue-router'
import App from '@/App.vue'
import * as tauri from '@/api/tauri'

// ─── Captured listener map ───────────────────────────────────────────────────
// listen() is async; each call registers ONE event name → async callback.
// We collect them all so tests can fire individual hotkeys.
type ListenerCallback = (...args: unknown[]) => Promise<void> | void
const capturedListeners: Record<string, ListenerCallback> = {}

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((eventName: string, cb: ListenerCallback) => {
    capturedListeners[eventName] = cb
    return Promise.resolve(() => {})
  }),
}))

// ─── Tauri API mock ───────────────────────────────────────────────────────────
vi.mock('@/api/tauri', () => ({
  hasCompletedSetup: vi.fn(),
  getClaudeStatus: vi.fn(),
  markSetupComplete: vi.fn(),
  getSetting: vi.fn(),
  setSetting: vi.fn(),
  getAllSettings: vi.fn(),
  deleteSetting: vi.fn(),
  getActiveSession: vi.fn(),
  // Session commands called by the stores
  createSession: vi.fn(),
  getSession: vi.fn(),
  updateSession: vi.fn(),
  deleteSession: vi.fn(),
  listSessions: vi.fn(),
  getSessionSummaries: vi.fn(),
  updateSessionStatus: vi.fn(),
  // Bug commands called by the stores
  createBug: vi.fn(),
  getBug: vi.fn(),
  updateBug: vi.fn(),
  deleteBug: vi.fn(),
  listBugs: vi.fn(),
  getBugsBySession: vi.fn(),
  // Tray commands
  updateTrayIcon: vi.fn(),
  updateTrayTooltip: vi.fn(),
}))

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => ({ setAlwaysOnTop: vi.fn() })),
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const mockNotify = vi.fn()
vi.mock('quasar', async () => {
  const actual = await vi.importActual('quasar')
  return {
    ...actual,
    useQuasar: () => ({
      notify: mockNotify,
      screen: { lt: { md: false } },
    }),
  }
})

// ─── Helpers ──────────────────────────────────────────────────────────────────

function buildRouter() {
  return createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', name: 'home', component: { template: '<div>Home</div>' } },
      { path: '/active-session', name: 'active-session', component: { template: '<div>Active</div>' } },
      { path: '/settings', name: 'settings', component: { template: '<div>Settings</div>' } },
    ],
  })
}

async function mountApp(pinia: ReturnType<typeof createPinia>, router: ReturnType<typeof buildRouter>) {
  const wrapper = mount(App, {
    global: {
      plugins: [pinia, router, [Quasar, { plugins: {} }]],
    },
    attachTo: document.body,
  })
  await flushPromises()
  return wrapper
}

async function fireHotkey(eventName: string) {
  const cb = capturedListeners[eventName]
  expect(cb, `listener for "${eventName}" was not registered`).toBeDefined()
  await cb!()
  await flushPromises()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

describe('Hotkey event listeners (App.vue)', () => {
  let pinia: ReturnType<typeof createPinia>
  let router: ReturnType<typeof buildRouter>

  beforeEach(async () => {
    // Reset captured listeners before each test
    for (const key of Object.keys(capturedListeners)) {
      delete capturedListeners[key]
    }

    pinia = createPinia()
    setActivePinia(pinia)
    router = buildRouter()
    vi.clearAllMocks()
    mockNotify.mockClear()

    // Default mocks
    vi.mocked(tauri.hasCompletedSetup).mockResolvedValue(true)
    vi.mocked(tauri.getSetting).mockResolvedValue(null)
    vi.mocked(tauri.setSetting).mockResolvedValue(undefined)
    vi.mocked(tauri.getAllSettings).mockResolvedValue([])
    vi.mocked(tauri.deleteSetting).mockResolvedValue(undefined)
    vi.mocked(tauri.getActiveSession).mockResolvedValue(null)
    vi.mocked(tauri.updateTrayIcon).mockResolvedValue(undefined)
    vi.mocked(tauri.updateTrayTooltip).mockResolvedValue(undefined)
  })

  it('registers all 5 hotkey event listeners on mount', async () => {
    await mountApp(pinia, router)

    expect(capturedListeners['hotkey-toggle-session']).toBeDefined()
    expect(capturedListeners['hotkey-start-bug-capture']).toBeDefined()
    expect(capturedListeners['hotkey-end-bug-capture']).toBeDefined()
    expect(capturedListeners['hotkey-open-quick-notepad']).toBeDefined()
    expect(capturedListeners['hotkey-open-session-notepad']).toBeDefined()
  })

  describe('hotkey-toggle-session', () => {
    it('starts a session when no session is active', async () => {
      const mockSession = {
        id: 'session-1',
        status: 'active' as const,
        started_at: new Date().toISOString(),
        ended_at: null,
        folder_path: '/tmp/session-1',
        session_notes: null,
        environment_json: null,
        original_snip_path: null,
        created_at: new Date().toISOString(),
      }
      vi.mocked(tauri.getActiveSession).mockResolvedValue(null)
      // sessionStore.startSession -> createSession -> tauri.createSession
      vi.mocked(tauri.createSession).mockResolvedValue(mockSession)

      await mountApp(pinia, router)
      await fireHotkey('hotkey-toggle-session')

      expect(tauri.createSession).toHaveBeenCalled()
    })

    it('ends the active session when one is running', async () => {
      const mockSession = {
        id: 'session-1',
        status: 'active' as const,
        started_at: new Date().toISOString(),
        ended_at: null,
        folder_path: '/tmp/session-1',
        session_notes: null,
        environment_json: null,
        original_snip_path: null,
        created_at: new Date().toISOString(),
      }
      vi.mocked(tauri.getActiveSession).mockResolvedValue(mockSession)
      // sessionStore.endSession calls invoke('end_session') directly
      const { invoke } = await import('@tauri-apps/api/core')
      vi.mocked(invoke as Mock).mockResolvedValue(undefined)

      await mountApp(pinia, router)
      await fireHotkey('hotkey-toggle-session')

      // endSession calls invoke('end_session', { sessionId })
      expect(invoke).toHaveBeenCalledWith('end_session', expect.objectContaining({ sessionId: 'session-1' }))
    })

    it('shows an error notification when starting a session fails', async () => {
      vi.mocked(tauri.getActiveSession).mockResolvedValue(null)
      vi.mocked(tauri.createSession).mockRejectedValue(new Error('DB error'))

      await mountApp(pinia, router)
      await fireHotkey('hotkey-toggle-session')

      expect(mockNotify).toHaveBeenCalledWith(
        expect.objectContaining({ type: 'negative', message: 'Failed to start session' })
      )
    })
  })

  describe('hotkey-start-bug-capture', () => {
    it('starts bug capture when a session is active', async () => {
      const mockSession = {
        id: 'session-1',
        status: 'active' as const,
        started_at: new Date().toISOString(),
        ended_at: null,
        folder_path: '/tmp/session-1',
        session_notes: null,
        environment_json: null,
        original_snip_path: null,
        created_at: new Date().toISOString(),
      }
      const mockBug = {
        id: 'bug-1',
        session_id: 'session-1',
        bug_number: 1,
        display_id: 'BUG-001',
        type: 'bug' as const,
        title: null,
        notes: null,
        description: null,
        ai_description: null,
        status: 'capturing' as const,
        meeting_id: null,
        software_version: null,
        console_parse_json: null,
        metadata_json: null,
        folder_path: '/tmp/session-1/bug-1',
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      }
      vi.mocked(tauri.getActiveSession).mockResolvedValue(mockSession)
      // createBug is called by bugStore.startBugCapture -> createBug -> tauri.createBug
      vi.mocked(tauri.createBug).mockResolvedValue(mockBug)

      await mountApp(pinia, router)
      await fireHotkey('hotkey-start-bug-capture')

      // The store calls tauri.createBug with the bug data including session_id
      expect(tauri.createBug).toHaveBeenCalledWith(
        expect.objectContaining({ session_id: 'session-1' })
      )
    })

    it('shows a warning when no session is active', async () => {
      vi.mocked(tauri.getActiveSession).mockResolvedValue(null)

      await mountApp(pinia, router)
      await fireHotkey('hotkey-start-bug-capture')

      expect(mockNotify).toHaveBeenCalledWith(
        expect.objectContaining({ type: 'warning', message: 'No active session' })
      )
    })
  })

  describe('hotkey-end-bug-capture', () => {
    it('completes bug capture when a bug is active', async () => {
      const mockSession = {
        id: 'session-1',
        status: 'active' as const,
        started_at: new Date().toISOString(),
        ended_at: null,
        folder_path: '/tmp/session-1',
        session_notes: null,
        environment_json: null,
        original_snip_path: null,
        created_at: new Date().toISOString(),
      }
      const mockBug = {
        id: 'bug-1',
        session_id: 'session-1',
        bug_number: 1,
        display_id: 'BUG-001',
        type: 'bug' as const,
        title: null,
        notes: null,
        description: null,
        ai_description: null,
        status: 'capturing' as const,
        meeting_id: null,
        software_version: null,
        console_parse_json: null,
        metadata_json: null,
        folder_path: '/tmp/session-1/bug-1',
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      }
      vi.mocked(tauri.getActiveSession).mockResolvedValue(mockSession)
      // createBug is called by bugStore.startBugCapture
      vi.mocked(tauri.createBug).mockResolvedValue(mockBug)
      // updateBug is called by bugStore.completeBugCapture -> updateBackendBug({status:'captured'})
      vi.mocked(tauri.updateBug).mockResolvedValue(undefined)

      await mountApp(pinia, router)

      // First start a bug capture to set activeBug in the store
      await fireHotkey('hotkey-start-bug-capture')
      await fireHotkey('hotkey-end-bug-capture')

      // completeBugCapture calls updateBackendBug(id, { status: 'captured' })
      // which calls tauri.updateBug(id, { status: 'captured' })
      expect(tauri.updateBug).toHaveBeenCalledWith(
        'bug-1',
        expect.objectContaining({ status: 'captured' })
      )
    })

    it('does nothing when no bug is active', async () => {
      vi.mocked(tauri.getActiveSession).mockResolvedValue(null)
      vi.mocked(tauri.updateBug).mockResolvedValue(undefined)

      await mountApp(pinia, router)
      await fireHotkey('hotkey-end-bug-capture')

      // updateBug (which calls end_bug_capture) should not be called when there is no active bug
      expect(tauri.updateBug).not.toHaveBeenCalled()
    })
  })

  describe('hotkey-open-quick-notepad', () => {
    it('toggles the QuickNotepad panel on each hotkey press', async () => {
      vi.mocked(tauri.getActiveSession).mockResolvedValue(null)

      const wrapper = await mountApp(pinia, router)

      // Initially hidden
      expect(wrapper.html()).not.toContain('Quick Notes')

      // First press: show
      await fireHotkey('hotkey-open-quick-notepad')
      await wrapper.vm.$nextTick()
      expect(wrapper.html()).toContain('Quick Notes')

      // Second press: hide
      await fireHotkey('hotkey-open-quick-notepad')
      await wrapper.vm.$nextTick()
      expect(wrapper.html()).not.toContain('Quick Notes')
    })
  })

  describe('hotkey-open-session-notepad', () => {
    it('toggles the SessionNotepad panel on each hotkey press', async () => {
      vi.mocked(tauri.getActiveSession).mockResolvedValue(null)

      const wrapper = await mountApp(pinia, router)

      // Initially hidden
      expect(wrapper.html()).not.toContain('Session Notes')

      // First press: show
      await fireHotkey('hotkey-open-session-notepad')
      await wrapper.vm.$nextTick()
      expect(wrapper.html()).toContain('Session Notes')

      // Second press: hide
      await fireHotkey('hotkey-open-session-notepad')
      await wrapper.vm.$nextTick()
      expect(wrapper.html()).not.toContain('Session Notes')
    })
  })
})
