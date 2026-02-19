import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { Quasar } from 'quasar'
import QuickNotepad from '@/components/QuickNotepad.vue'
import { useBugStore } from '@/stores/bug'
import type { Bug as BackendBug } from '@/types/backend'
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
    updateTrayIcon: vi.fn(),
    updateTrayTooltip: vi.fn(),
  }
})

// Mock Tauri event listeners
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))

const mockActiveBug: BackendBug = {
  id: 'bug-1',
  session_id: 'session-1',
  bug_number: 1,
  display_id: 'BUG-001',
  type: 'bug',
  title: 'Test Bug',
  notes: null,
  description: 'Test description',
  ai_description: null,
  status: 'capturing',
  meeting_id: null,
  software_version: '1.0.0',
  console_parse_json: null,
  metadata_json: null,
  folder_path: '/test/bugs/bug1',
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
}

describe('QuickNotepad.vue', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    vi.mocked(tauri.getBugNotes).mockResolvedValue('') // Default to empty notes
    vi.mocked(tauri.updateBugNotes).mockResolvedValue(undefined)
    mockSetAlwaysOnTop.mockResolvedValue(undefined)
  })

  afterEach(() => {
    vi.clearAllTimers()
  })

  const mountComponent = (props = {}) => {
    return mount(QuickNotepad, {
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
    expect(wrapper.find('.quick-notepad').exists()).toBe(true)
    expect(wrapper.text()).toContain('Quick Notes')
  })

  it('does not render when visible is false', () => {
    const wrapper = mountComponent({ visible: false })
    expect(wrapper.find('.quick-notepad').exists()).toBe(false)
  })

  it('sets window to always-on-top on mount', async () => {
    mountComponent()
    await flushPromises()
    expect(mockSetAlwaysOnTop).toHaveBeenCalledWith(true)
  })

  it('loads notes for active bug on mount', async () => {
    const bugStore = useBugStore()
    bugStore.activeBug = mockActiveBug

    vi.mocked(tauri.getBugNotes).mockResolvedValue('Existing notes content')

    const wrapper = mountComponent()
    await flushPromises()

    expect(vi.mocked(tauri.getBugNotes)).toHaveBeenCalledWith('bug-1')

    // Check that textarea has the loaded notes
    const textarea = wrapper.find('textarea')
    expect(textarea.element.value).toBe('Existing notes content')
  })

  it('shows "No active bug" message when no bug is active', () => {
    const bugStore = useBugStore()
    bugStore.activeBug = null

    const wrapper = mountComponent()
    expect(wrapper.text()).toContain('No active bug')
  })

  it('disables textarea when no active bug', () => {
    const bugStore = useBugStore()
    bugStore.activeBug = null

    const wrapper = mountComponent()
    const input = wrapper.findComponent({ name: 'QInput' })
    expect(input.props('disable')).toBe(true)
  })

  it('enables textarea when active bug exists', () => {
    const bugStore = useBugStore()
    bugStore.activeBug = mockActiveBug

    const wrapper = mountComponent()
    const input = wrapper.findComponent({ name: 'QInput' })
    expect(input.props('disable')).toBe(false)
  })

  it('displays bug display ID when active bug exists', async () => {
    const bugStore = useBugStore()
    bugStore.activeBug = mockActiveBug

    const wrapper = mountComponent()
    await flushPromises()

    expect(wrapper.text()).toContain('BUG-001')
  })

  it('auto-saves notes with debounce when typing', async () => {
    vi.useFakeTimers()

    const bugStore = useBugStore()
    bugStore.activeBug = mockActiveBug

    const wrapper = mountComponent()
    await flushPromises()

    // Type in the textarea
    const textarea = wrapper.find('textarea')
    await textarea.setValue('New notes content')

    // Should not save immediately
    expect(vi.mocked(tauri.updateBugNotes)).not.toHaveBeenCalled()

    // Fast-forward 500ms (debounce time)
    vi.advanceTimersByTime(500)
    await flushPromises()

    // Should now have saved
    expect(vi.mocked(tauri.updateBugNotes)).toHaveBeenCalledWith(
      'bug-1',
      'New notes content'
    )

    vi.useRealTimers()
  })

  it('shows "Saving..." status while saving', async () => {
    vi.useFakeTimers()

    const bugStore = useBugStore()
    bugStore.activeBug = mockActiveBug

    // Make save take a while
    let resolveSave: () => void
    vi.mocked(tauri.updateBugNotes).mockReturnValue(
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

    const bugStore = useBugStore()
    bugStore.activeBug = mockActiveBug

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

    const bugStore = useBugStore()
    bugStore.activeBug = mockActiveBug

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

    const bugStore = useBugStore()
    bugStore.activeBug = mockActiveBug

    vi.mocked(tauri.updateBugNotes).mockRejectedValue(new Error('Save failed'))

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

  it('loads notes when active bug changes', async () => {
    const bugStore = useBugStore()
    bugStore.activeBug = mockActiveBug

    vi.mocked(tauri.getBugNotes).mockResolvedValue('Bug 1 notes')

    const wrapper = mountComponent()
    await flushPromises()

    expect(vi.mocked(tauri.getBugNotes)).toHaveBeenCalledWith('bug-1')

    // Change to a different bug
    const newBug: BackendBug = {
      ...mockActiveBug,
      id: 'bug-2',
      bug_number: 2,
      display_id: 'BUG-002',
      folder_path: '/test/bugs/bug2',
    }

    vi.mocked(tauri.getBugNotes).mockResolvedValue('Bug 2 notes')
    bugStore.activeBug = newBug
    await flushPromises()

    // Should load notes for new bug
    expect(vi.mocked(tauri.getBugNotes)).toHaveBeenCalledWith('bug-2')

    const textarea = wrapper.find('textarea')
    expect(textarea.element.value).toBe('Bug 2 notes')
  })

  it('saves notes for old bug when switching bugs', async () => {
    vi.useFakeTimers()

    const bugStore = useBugStore()
    bugStore.activeBug = mockActiveBug

    vi.mocked(tauri.getBugNotes).mockResolvedValue('Bug 1 notes')

    const wrapper = mountComponent()
    await flushPromises()

    // Type in the textarea
    const textarea = wrapper.find('textarea')
    await textarea.setValue('Modified bug 1 notes')

    // Change to a different bug before debounce completes
    const newBug: BackendBug = {
      ...mockActiveBug,
      id: 'bug-2',
      bug_number: 2,
      display_id: 'BUG-002',
      folder_path: '/test/bugs/bug2',
    }

    vi.mocked(tauri.getBugNotes).mockResolvedValue('Bug 2 notes')
    bugStore.activeBug = newBug
    await flushPromises()

    // Should have saved the old bug's notes
    expect(vi.mocked(tauri.updateBugNotes)).toHaveBeenCalledWith(
      'bug-1',
      'Modified bug 1 notes'
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

    const card = wrapper.find('.quick-notepad')
    const style = card.attributes('style')

    expect(style).toContain('left: 200px')
    expect(style).toContain('top: 300px')
  })

  it('handles empty notes gracefully', async () => {
    const bugStore = useBugStore()
    bugStore.activeBug = mockActiveBug

    vi.mocked(tauri.getBugNotes).mockResolvedValue('')

    const wrapper = mountComponent()
    await flushPromises()

    const textarea = wrapper.find('textarea')
    expect(textarea.element.value).toBe('')
  })

  it('handles notes loading error gracefully', async () => {
    const bugStore = useBugStore()
    bugStore.activeBug = mockActiveBug

    vi.mocked(tauri.getBugNotes).mockRejectedValue(new Error('Failed to load'))

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

    const bugStore = useBugStore()
    bugStore.activeBug = mockActiveBug

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
    expect(vi.mocked(tauri.updateBugNotes)).not.toHaveBeenCalled()

    // Complete the debounce
    vi.advanceTimersByTime(500)
    await flushPromises()

    // Should save only once with the final value
    expect(vi.mocked(tauri.updateBugNotes)).toHaveBeenCalledTimes(1)
    expect(vi.mocked(tauri.updateBugNotes)).toHaveBeenCalledWith(
      'bug-1',
      'ABC'
    )

    vi.useRealTimers()
  })

  it('clears notes when active bug becomes null', async () => {
    const bugStore = useBugStore()
    bugStore.activeBug = mockActiveBug

    vi.mocked(tauri.getBugNotes).mockResolvedValue('Some notes')

    const wrapper = mountComponent()
    await flushPromises()

    // Should have loaded notes
    const textarea = wrapper.find('textarea')
    expect(textarea.element.value).toBe('Some notes')

    // Clear active bug
    bugStore.activeBug = null
    await flushPromises()

    // Notes should be cleared
    expect(textarea.element.value).toBe('')
  })

  describe('draggable behavior', () => {
    it('moves the notepad when dragged via the header', async () => {
      const wrapper = mountComponent({ initialX: 100, initialY: 100 })
      await flushPromises()

      const card = wrapper.find('.quick-notepad')

      // Simulate mousedown on card (header area)
      await card.trigger('mousedown', { clientX: 150, clientY: 150 })

      // Simulate mousemove on document
      const mousemoveEvent = new MouseEvent('mousemove', { clientX: 200, clientY: 180 })
      document.dispatchEvent(mousemoveEvent)
      await wrapper.vm.$nextTick()

      // Card should have moved by (50, 30)
      const style = card.attributes('style') ?? ''
      expect(style).toContain('left: 150px')
      expect(style).toContain('top: 130px')

      // Simulate mouseup to stop dragging
      document.dispatchEvent(new MouseEvent('mouseup'))
    })

    it('stops moving after mouseup', async () => {
      const wrapper = mountComponent({ initialX: 100, initialY: 100 })
      await flushPromises()

      const card = wrapper.find('.quick-notepad')

      await card.trigger('mousedown', { clientX: 150, clientY: 150 })
      document.dispatchEvent(new MouseEvent('mousemove', { clientX: 200, clientY: 200 }))
      await wrapper.vm.$nextTick()
      document.dispatchEvent(new MouseEvent('mouseup'))

      // Now move again — should not affect position
      document.dispatchEvent(new MouseEvent('mousemove', { clientX: 300, clientY: 300 }))
      await wrapper.vm.$nextTick()

      const style = card.attributes('style') ?? ''
      expect(style).toContain('left: 150px')
      expect(style).toContain('top: 150px')
    })
  })

  describe('console tag toggle', () => {
    it('renders the console tag toggle', async () => {
      const bugStore = useBugStore()
      bugStore.activeBug = mockActiveBug

      const wrapper = mountComponent()
      await flushPromises()

      expect(wrapper.text()).toContain('Tag next screenshot as console')
    })

    it('toggle is disabled when no active bug', async () => {
      const bugStore = useBugStore()
      bugStore.activeBug = null

      const wrapper = mountComponent()
      await flushPromises()

      const toggle = wrapper.findComponent({ name: 'QToggle' })
      expect(toggle.props('disable')).toBe(true)
    })

    it('toggle is enabled when active bug exists', async () => {
      const bugStore = useBugStore()
      bugStore.activeBug = mockActiveBug

      const wrapper = mountComponent()
      await flushPromises()

      const toggle = wrapper.findComponent({ name: 'QToggle' })
      expect(toggle.props('disable')).toBe(false)
    })

    it('updates bug store tagNextScreenshotAsConsole when toggled', async () => {
      const bugStore = useBugStore()
      bugStore.activeBug = mockActiveBug

      const wrapper = mountComponent()
      await flushPromises()

      expect(bugStore.tagNextScreenshotAsConsole).toBe(false)

      const toggle = wrapper.findComponent({ name: 'QToggle' })
      await toggle.trigger('click')
      await wrapper.vm.$nextTick()

      // Toggle state is reflected in store
      expect(bugStore.tagNextScreenshotAsConsole).toBe(true)
    })

    it('resets console tag toggle when bug changes', async () => {
      const bugStore = useBugStore()
      bugStore.activeBug = mockActiveBug
      bugStore.setTagNextScreenshotAsConsole(true)

      mountComponent()
      await flushPromises()

      // Change bug
      const newBug: BackendBug = { ...mockActiveBug, id: 'bug-2', display_id: 'BUG-002', folder_path: '/test/bugs/bug2' }
      bugStore.activeBug = newBug
      await flushPromises()

      // Console tag should be reset
      expect(bugStore.tagNextScreenshotAsConsole).toBe(false)
    })
  })

  describe('meeting ID pre-population', () => {
    it('pre-populates meeting ID from lastSessionMeetingId when new bug has no meeting_id', async () => {
      const bugStore = useBugStore()
      bugStore.setLastSessionMeetingId('previous-meeting-123')

      const newBug: BackendBug = { ...mockActiveBug, meeting_id: null }
      bugStore.activeBug = newBug

      vi.mocked(tauri.getBugNotes).mockResolvedValue('')

      const wrapper = mountComponent()
      await flushPromises()

      // The meeting ID input should show the last session's meeting ID
      const inputs = wrapper.findAllComponents({ name: 'QInput' })
      const meetingInput = inputs.find(i => i.props('label') === 'Meeting ID / URL')
      expect(meetingInput).toBeDefined()
      expect((meetingInput!.element as HTMLElement).querySelector('input')?.value).toBe('previous-meeting-123')
    })

    it('uses the bug own meeting_id over lastSessionMeetingId when set', async () => {
      const bugStore = useBugStore()
      bugStore.setLastSessionMeetingId('previous-meeting-123')

      const bugWithMeeting: BackendBug = { ...mockActiveBug, meeting_id: 'current-meeting-456' }
      bugStore.activeBug = bugWithMeeting

      vi.mocked(tauri.getBugNotes).mockResolvedValue('')

      const wrapper = mountComponent()
      await flushPromises()

      const inputs = wrapper.findAllComponents({ name: 'QInput' })
      const meetingInput = inputs.find(i => i.props('label') === 'Meeting ID / URL')
      expect((meetingInput!.element as HTMLElement).querySelector('input')?.value).toBe('current-meeting-456')
    })
  })

  describe('always-on-top management', () => {
    it('sets always-on-top to false when visible becomes false', async () => {
      const wrapper = mountComponent({ visible: true })
      await flushPromises()

      // Clear initial call
      mockSetAlwaysOnTop.mockClear()

      // Hide the notepad
      await wrapper.setProps({ visible: false })
      await flushPromises()

      expect(mockSetAlwaysOnTop).toHaveBeenCalledWith(false)
    })

    it('sets always-on-top to true when visible becomes true', async () => {
      const wrapper = mountComponent({ visible: false })
      await flushPromises()

      mockSetAlwaysOnTop.mockClear()

      await wrapper.setProps({ visible: true })
      await flushPromises()

      expect(mockSetAlwaysOnTop).toHaveBeenCalledWith(true)
    })
  })
})
