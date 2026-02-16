import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { Quasar } from 'quasar'
import SessionNotepad from '@/components/SessionNotepad.vue'
import { useSessionStore } from '@/stores/session'
import type { Session } from '@/types/backend'
import * as tauri from '@/api/tauri'

// Mock Tauri window API
const mockSetAlwaysOnTop = vi.fn().mockResolvedValue(undefined)

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => ({
    setAlwaysOnTop: mockSetAlwaysOnTop,
  })),
}))

// Mock Tauri API
vi.mock('@/api/tauri', () => {
  return {
    createSession: vi.fn(),
    getSession: vi.fn(),
    updateSession: vi.fn(),
    deleteSession: vi.fn(),
    listSessions: vi.fn(),
    getActiveSession: vi.fn(),
    getSessionSummaries: vi.fn(),
    updateSessionStatus: vi.fn(),
    createBug: vi.fn(),
    getBug: vi.fn(),
    updateBug: vi.fn(),
    deleteBug: vi.fn(),
    listBugs: vi.fn(),
    getBugsBySession: vi.fn(),
    getBugNotes: vi.fn(),
    updateBugNotes: vi.fn(),
    getSessionNotes: vi.fn(),
    updateSessionNotes: vi.fn(),
  }
})

// Mock Tauri event listeners
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))

const mockActiveSession: Session = {
  id: 'session-1',
  started_at: new Date('2026-02-16T10:00:00Z').toISOString(),
  ended_at: null,
  status: 'active',
  folder_path: '/test/sessions/session1',
  session_notes: null,
  environment_json: null,
  original_snip_path: null,
  created_at: new Date().toISOString(),
}

describe('SessionNotepad.vue', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    vi.mocked(tauri.getSessionNotes).mockResolvedValue('') // Default to empty notes
    vi.mocked(tauri.updateSessionNotes).mockResolvedValue(undefined)
    mockSetAlwaysOnTop.mockResolvedValue(undefined)
  })

  afterEach(() => {
    vi.clearAllTimers()
  })

  const mountComponent = (props = {}) => {
    return mount(SessionNotepad, {
      props: {
        visible: true,
        ...props,
      },
      global: {
        plugins: [Quasar],
      },
    })
  }

  it('renders the component when visible', () => {
    const wrapper = mountComponent()
    expect(wrapper.find('.session-notepad').exists()).toBe(true)
    expect(wrapper.text()).toContain('Session Notes')
  })

  it('does not render when visible is false', () => {
    const wrapper = mountComponent({ visible: false })
    expect(wrapper.find('.session-notepad').exists()).toBe(false)
  })

  it('sets window to always-on-top on mount', async () => {
    mountComponent()
    await flushPromises()
    expect(mockSetAlwaysOnTop).toHaveBeenCalledWith(true)
  })

  it('loads notes for active session on mount', async () => {
    const sessionStore = useSessionStore()
    sessionStore.activeSession = mockActiveSession

    vi.mocked(tauri.getSessionNotes).mockResolvedValue('Existing session notes content')

    const wrapper = mountComponent()
    await flushPromises()

    expect(vi.mocked(tauri.getSessionNotes)).toHaveBeenCalledWith('session-1', '/test/sessions/session1')

    // Check that textarea has the loaded notes
    const textarea = wrapper.find('textarea')
    expect(textarea.element.value).toBe('Existing session notes content')
  })

  it('shows "No active session" message when no session is active', () => {
    const sessionStore = useSessionStore()
    sessionStore.activeSession = null

    const wrapper = mountComponent()
    expect(wrapper.text()).toContain('No active session')
  })

  it('disables textarea when no active session', () => {
    const sessionStore = useSessionStore()
    sessionStore.activeSession = null

    const wrapper = mountComponent()
    const input = wrapper.findComponent({ name: 'QInput' })
    expect(input.props('disable')).toBe(true)
  })

  it('enables textarea when active session exists', () => {
    const sessionStore = useSessionStore()
    sessionStore.activeSession = mockActiveSession

    const wrapper = mountComponent()
    const input = wrapper.findComponent({ name: 'QInput' })
    expect(input.props('disable')).toBe(false)
  })

  it('displays session date when active session exists', async () => {
    const sessionStore = useSessionStore()
    sessionStore.activeSession = mockActiveSession

    const wrapper = mountComponent()
    await flushPromises()

    // Should display formatted date (2/16/2026 or similar depending on locale)
    expect(wrapper.text()).toMatch(/\d{1,2}\/\d{1,2}\/\d{4}/)
  })

  it('auto-saves notes with debounce when typing', async () => {
    vi.useFakeTimers()

    const sessionStore = useSessionStore()
    sessionStore.activeSession = mockActiveSession

    const wrapper = mountComponent()
    await flushPromises()

    // Type in the textarea
    const textarea = wrapper.find('textarea')
    await textarea.setValue('New session notes content')

    // Should not save immediately
    expect(vi.mocked(tauri.updateSessionNotes)).not.toHaveBeenCalled()

    // Fast-forward 500ms (debounce time)
    vi.advanceTimersByTime(500)
    await flushPromises()

    // Should now have saved
    expect(vi.mocked(tauri.updateSessionNotes)).toHaveBeenCalledWith(
      'session-1',
      '/test/sessions/session1',
      'New session notes content'
    )

    vi.useRealTimers()
  })

  it('shows "Saving..." status while saving', async () => {
    vi.useFakeTimers()

    const sessionStore = useSessionStore()
    sessionStore.activeSession = mockActiveSession

    // Make save take a while
    let resolveSave: () => void
    vi.mocked(tauri.updateSessionNotes).mockReturnValue(
      new Promise((resolve) => {
        resolveSave = resolve as () => void
      })
    )

    const wrapper = mountComponent()
    await flushPromises()

    // Type in the textarea
    const textarea = wrapper.find('textarea')
    await textarea.setValue('New notes')

    // Fast-forward debounce
    vi.advanceTimersByTime(500)
    await flushPromises()

    // Should show "Saving..." status
    expect(wrapper.text()).toContain('Saving...')

    // Complete the save
    resolveSave!()
    await flushPromises()

    vi.useRealTimers()
  })

  it('shows "Saved" status after successful save', async () => {
    vi.useFakeTimers()

    const sessionStore = useSessionStore()
    sessionStore.activeSession = mockActiveSession

    const wrapper = mountComponent()
    await flushPromises()

    // Type in the textarea
    const textarea = wrapper.find('textarea')
    await textarea.setValue('New notes')

    // Fast-forward debounce
    vi.advanceTimersByTime(500)
    await flushPromises()

    // Should show "Saved" status
    expect(wrapper.text()).toContain('Saved')

    vi.useRealTimers()
  })

  it('clears "Saved" status after 2 seconds', async () => {
    vi.useFakeTimers()

    const sessionStore = useSessionStore()
    sessionStore.activeSession = mockActiveSession

    const wrapper = mountComponent()
    await flushPromises()

    // Type in the textarea
    const textarea = wrapper.find('textarea')
    await textarea.setValue('New notes')

    // Fast-forward debounce
    vi.advanceTimersByTime(500)
    await flushPromises()

    // Should show "Saved" status
    expect(wrapper.text()).toContain('Saved')

    // Fast-forward 2 seconds
    vi.advanceTimersByTime(2000)
    await flushPromises()

    // "Saved" status should be cleared
    expect(wrapper.text()).not.toContain('Saved')

    vi.useRealTimers()
  })

  it('shows "Error" status when save fails', async () => {
    vi.useFakeTimers()

    const sessionStore = useSessionStore()
    sessionStore.activeSession = mockActiveSession

    vi.mocked(tauri.updateSessionNotes).mockRejectedValue(new Error('Save failed'))

    const wrapper = mountComponent()
    await flushPromises()

    // Type in the textarea
    const textarea = wrapper.find('textarea')
    await textarea.setValue('New notes')

    // Fast-forward debounce
    vi.advanceTimersByTime(500)
    await flushPromises()

    // Should show "Error" status
    expect(wrapper.text()).toContain('Error')

    vi.useRealTimers()
  })

  it('loads notes when active session changes', async () => {
    const sessionStore = useSessionStore()
    sessionStore.activeSession = mockActiveSession

    vi.mocked(tauri.getSessionNotes).mockResolvedValue('Session 1 notes')

    const wrapper = mountComponent()
    await flushPromises()

    expect(vi.mocked(tauri.getSessionNotes)).toHaveBeenCalledWith('session-1', '/test/sessions/session1')

    // Change to a different session
    const newSession: Session = {
      ...mockActiveSession,
      id: 'session-2',
      folder_path: '/test/sessions/session2',
      started_at: new Date('2026-02-17T10:00:00Z').toISOString(),
    }

    vi.mocked(tauri.getSessionNotes).mockResolvedValue('Session 2 notes')
    sessionStore.activeSession = newSession
    await flushPromises()

    // Should load notes for new session
    expect(vi.mocked(tauri.getSessionNotes)).toHaveBeenCalledWith('session-2', '/test/sessions/session2')

    const textarea = wrapper.find('textarea')
    expect(textarea.element.value).toBe('Session 2 notes')
  })

  it('saves notes for old session when switching sessions', async () => {
    vi.useFakeTimers()

    const sessionStore = useSessionStore()
    sessionStore.activeSession = mockActiveSession

    vi.mocked(tauri.getSessionNotes).mockResolvedValue('Session 1 notes')

    const wrapper = mountComponent()
    await flushPromises()

    // Type in the textarea
    const textarea = wrapper.find('textarea')
    await textarea.setValue('Modified session 1 notes')

    // Change to a different session before debounce completes
    const newSession: Session = {
      ...mockActiveSession,
      id: 'session-2',
      folder_path: '/test/sessions/session2',
      started_at: new Date('2026-02-17T10:00:00Z').toISOString(),
    }

    vi.mocked(tauri.getSessionNotes).mockResolvedValue('Session 2 notes')
    sessionStore.activeSession = newSession
    await flushPromises()

    // Should have saved the old session's notes
    expect(vi.mocked(tauri.updateSessionNotes)).toHaveBeenCalledWith(
      'session-1',
      '/test/sessions/session1',
      'Modified session 1 notes'
    )

    vi.useRealTimers()
  })

  it('emits close event when close button is clicked', async () => {
    const wrapper = mountComponent()
    await flushPromises()

    const closeButton = wrapper.findAll('button').find((btn) =>
      btn.html().includes('close')
    )
    expect(closeButton).toBeDefined()

    await closeButton!.trigger('click')

    expect(wrapper.emitted()).toHaveProperty('close')
    expect(wrapper.emitted('close')).toHaveLength(1)
  })

  it('applies custom position from props', () => {
    const wrapper = mountComponent({
      initialX: 200,
      initialY: 300,
    })

    const card = wrapper.find('.session-notepad')
    const style = card.attributes('style')

    expect(style).toContain('left: 200px')
    expect(style).toContain('top: 300px')
  })

  it('handles empty notes gracefully', async () => {
    const sessionStore = useSessionStore()
    sessionStore.activeSession = mockActiveSession

    vi.mocked(tauri.getSessionNotes).mockResolvedValue('')

    const wrapper = mountComponent()
    await flushPromises()

    const textarea = wrapper.find('textarea')
    expect(textarea.element.value).toBe('')
  })

  it('handles notes loading error gracefully', async () => {
    const sessionStore = useSessionStore()
    sessionStore.activeSession = mockActiveSession

    vi.mocked(tauri.getSessionNotes).mockRejectedValue(new Error('Failed to load'))

    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

    const wrapper = mountComponent()
    await flushPromises()

    // Should not crash, should show empty notes
    const textarea = wrapper.find('textarea')
    expect(textarea.element.value).toBe('')

    expect(consoleSpy).toHaveBeenCalled()
    consoleSpy.mockRestore()
  })

  it('debounces multiple rapid changes correctly', async () => {
    vi.useFakeTimers()

    const sessionStore = useSessionStore()
    sessionStore.activeSession = mockActiveSession

    const wrapper = mountComponent()
    await flushPromises()

    const textarea = wrapper.find('textarea')

    // Rapid typing
    await textarea.setValue('A')
    vi.advanceTimersByTime(100)

    await textarea.setValue('AB')
    vi.advanceTimersByTime(100)

    await textarea.setValue('ABC')
    vi.advanceTimersByTime(100)

    // Should not have saved yet
    expect(vi.mocked(tauri.updateSessionNotes)).not.toHaveBeenCalled()

    // Complete the debounce
    vi.advanceTimersByTime(500)
    await flushPromises()

    // Should save only once with the final value
    expect(vi.mocked(tauri.updateSessionNotes)).toHaveBeenCalledTimes(1)
    expect(vi.mocked(tauri.updateSessionNotes)).toHaveBeenCalledWith(
      'session-1',
      '/test/sessions/session1',
      'ABC'
    )

    vi.useRealTimers()
  })

  it('clears notes when active session becomes null', async () => {
    const sessionStore = useSessionStore()
    sessionStore.activeSession = mockActiveSession

    vi.mocked(tauri.getSessionNotes).mockResolvedValue('Some notes')

    const wrapper = mountComponent()
    await flushPromises()

    // Should have loaded notes
    const textarea = wrapper.find('textarea')
    expect(textarea.element.value).toBe('Some notes')

    // Clear active session
    sessionStore.activeSession = null
    await flushPromises()

    // Notes should be cleared
    expect(textarea.element.value).toBe('')
  })

  it('does not attempt to load notes when session has no folder_path', async () => {
    const sessionStore = useSessionStore()
    sessionStore.activeSession = {
      ...mockActiveSession,
      folder_path: '',
    }

    const wrapper = mountComponent()
    await flushPromises()

    // Should not call getSessionNotes
    expect(vi.mocked(tauri.getSessionNotes)).not.toHaveBeenCalled()

    const textarea = wrapper.find('textarea')
    expect(textarea.element.value).toBe('')
  })

  it('does not attempt to save notes when session has no folder_path', async () => {
    vi.useFakeTimers()

    const sessionStore = useSessionStore()
    sessionStore.activeSession = {
      ...mockActiveSession,
      folder_path: '',
    }

    const wrapper = mountComponent()
    await flushPromises()

    // Type in the textarea
    const textarea = wrapper.find('textarea')
    await textarea.setValue('New notes')

    // Fast-forward debounce
    vi.advanceTimersByTime(500)
    await flushPromises()

    // Should not have attempted to save
    expect(vi.mocked(tauri.updateSessionNotes)).not.toHaveBeenCalled()

    vi.useRealTimers()
  })

  it('does not attempt to save old session notes when old session has no folder_path', async () => {
    vi.useFakeTimers()

    const sessionStore = useSessionStore()
    sessionStore.activeSession = {
      ...mockActiveSession,
      folder_path: '',
    }

    const wrapper = mountComponent()
    await flushPromises()

    // Type in the textarea
    const textarea = wrapper.find('textarea')
    await textarea.setValue('Modified notes')

    // Change to a different session
    const newSession: Session = {
      ...mockActiveSession,
      id: 'session-2',
      folder_path: '/test/sessions/session2',
    }

    vi.mocked(tauri.getSessionNotes).mockResolvedValue('Session 2 notes')
    sessionStore.activeSession = newSession
    await flushPromises()

    // Should not have attempted to save the old session's notes (since it had no folder_path)
    expect(vi.mocked(tauri.updateSessionNotes)).not.toHaveBeenCalled()

    vi.useRealTimers()
  })
})
