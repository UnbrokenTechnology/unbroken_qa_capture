import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createRouter, createMemoryHistory } from 'vue-router'
import { Quasar } from 'quasar'
import SessionReview from '@/views/SessionReview.vue'
import { useBugStore } from '@/stores/bug'
import { useSessionStore } from '@/stores/session'
import type { Bug, Session, Capture } from '@/types/backend'
import * as tauri from '@/api/tauri'

// Create a mock notify function
const mockNotify = vi.fn()

// Mock useQuasar
vi.mock('quasar', async () => {
  const actual = await vi.importActual('quasar')
  return {
    ...actual,
    useQuasar: () => ({
      notify: mockNotify
    })
  }
})

// Mock Tauri core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
  convertFileSrc: vi.fn((path: string) => `asset://localhost/${path}`),
}))

// Mock Tauri API
vi.mock('@/api/tauri', () => ({
  getBugCaptures: vi.fn(),
  updateBug: vi.fn(),
  getBugsBySession: vi.fn(),
  getUnsortedCaptures: vi.fn().mockResolvedValue([]),
  assignCaptureToBug: vi.fn().mockResolvedValue(undefined),
  suggestCaptureAssignment: vi.fn().mockResolvedValue(undefined),
  ticketingGetCredentials: vi.fn(),
  ticketingSaveCredentials: vi.fn(),
  ticketingAuthenticate: vi.fn(),
  ticketingCreateTicket: vi.fn(),
  ticketingCheckConnection: vi.fn(),
  getClaudeStatus: vi.fn(),
  generateBugDescription: vi.fn(),
  refineBugDescription: vi.fn(),
  saveBugDescription: vi.fn()
}))

// Mock Tauri event listener
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {})
}))

const createMockSession = (id: string): Session => ({
  id,
  started_at: '2024-01-15T10:00:00Z',
  ended_at: null,
  status: 'active',
  folder_path: '/sessions/test-session',
  session_notes: 'Test session notes',
  environment_json: null,
  original_snip_path: null,
  created_at: '2024-01-15T10:00:00Z'
})

const createMockBug = (id: string, sessionId: string, displayId: string): Bug => ({
  id,
  session_id: sessionId,
  bug_number: 1,
  display_id: displayId,
  type: 'bug',
  title: `Test Bug ${id}`,
  notes: 'This is a test bug with some notes',
  description: 'Test description',
  ai_description: null,
  status: 'captured',
  meeting_id: null,
  software_version: '1.0.0',
  console_parse_json: null,
  metadata_json: null,
  folder_path: `/bugs/bug-${id}`,
  created_at: '2024-01-15T10:30:00Z',
  updated_at: '2024-01-15T10:30:00Z'
})

const createMockCapture = (id: string, bugId: string): Capture => ({
  id,
  bug_id: bugId,
  session_id: 'session-1',
  file_name: `screenshot-${id}.png`,
  file_path: `/captures/screenshot-${id}.png`,
  file_type: 'screenshot',
  annotated_path: null,
  file_size_bytes: 12345,
  is_console_capture: false,
  parsed_content: null,
  created_at: '2024-01-15T10:30:00Z'
})

describe('SessionReview', () => {
  let router: ReturnType<typeof createRouter>
  let pinia: ReturnType<typeof createPinia>

  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)

    router = createRouter({
      history: createMemoryHistory(),
      routes: [
        {
          path: '/session-review',
          name: 'session-review',
          component: SessionReview
        }
      ]
    })

    // Reset mocks
    vi.clearAllMocks()

    // Set default mock implementations
    vi.mocked(tauri.ticketingGetCredentials).mockResolvedValue(null)
    vi.mocked(tauri.getClaudeStatus).mockResolvedValue({
      status: 'notInstalled', message: 'Claude CLI not installed'
    })
  })

  const mountComponent = async () => {
    router.push('/session-review')
    await router.isReady()

    return mount(SessionReview, {
      global: {
        plugins: [
          pinia,
          router,
          Quasar
        ],
        stubs: {
          QPage: { template: '<div><slot /></div>' },
          QCard: { template: '<div><slot /></div>' },
          QCardSection: { template: '<div><slot /></div>' },
          QCardActions: { template: '<div><slot /></div>' },
          QBtn: { template: '<button @click="$attrs.onClick"><slot />{{ $attrs.label }}</button>' },
          QBtnGroup: { template: '<div><slot /></div>' },
          QList: { template: '<div><slot /></div>' },
          QItem: { template: '<div @click="$attrs.onClick" :class="{ active: $attrs.active }"><slot /></div>' },
          QItemSection: { template: '<div><slot /></div>' },
          QItemLabel: { template: '<div><slot /></div>' },
          QBadge: { template: '<span><slot /></span>' },
          QChip: { template: '<span><slot /></span>' },
          QIcon: { template: '<span />' },
          QImg: { template: '<img :src="$attrs.src" @click="$attrs.onClick" />' },
          QDialog: { template: '<div v-if="$attrs.modelValue"><slot /></div>' },
          QSpace: { template: '<div />' },
          QInput: { template: '<input :value="$attrs.modelValue" @input="$emit(\'update:modelValue\', $event.target.value)" />' },
          QSpinner: { template: '<div class="spinner">Loading...</div>' }
        }
      }
    })
  }

  it('should show "No session found" when there is no active session and no session ID in route', async () => {
    const wrapper = await mountComponent()
    await flushPromises()

    expect(wrapper.text()).toContain('No session found')
  })

  it('should display session review header with active session', async () => {
    const sessionStore = useSessionStore()
    const session = createMockSession('session-1')
    sessionStore.activeSession = session

    vi.mocked(tauri.getBugsBySession).mockResolvedValue([])

    const wrapper = await mountComponent()
    await flushPromises()

    expect(wrapper.text()).toContain('Session Review')
  })

  it('should display bug list when session has bugs', async () => {
    const sessionStore = useSessionStore()
    const session = createMockSession('session-1')
    sessionStore.activeSession = session

    const bug1 = createMockBug('bug-1', 'session-1', 'BUG-001')
    const bug2 = createMockBug('bug-2', 'session-1', 'BUG-002')

    vi.mocked(tauri.getBugsBySession).mockResolvedValue([bug1, bug2])
    vi.mocked(tauri.getBugCaptures).mockResolvedValue([])

    const wrapper = await mountComponent()
    await flushPromises()

    expect(wrapper.text()).toContain('Bugs (2)')
    expect(wrapper.text()).toContain('Test Bug bug-1')
    expect(wrapper.text()).toContain('Test Bug bug-2')
  })

  it('should show empty state when session has no bugs', async () => {
    const sessionStore = useSessionStore()
    const session = createMockSession('session-1')
    sessionStore.activeSession = session

    vi.mocked(tauri.getBugsBySession).mockResolvedValue([])

    const wrapper = await mountComponent()
    await flushPromises()

    expect(wrapper.text()).toContain('No bugs in this session')
  })

  it('should display bug card with correct information', async () => {
    const sessionStore = useSessionStore()
    const session = createMockSession('session-1')
    sessionStore.activeSession = session

    const bug = createMockBug('bug-1', 'session-1', 'BUG-001')

    vi.mocked(tauri.getBugsBySession).mockResolvedValue([bug])
    vi.mocked(tauri.getBugCaptures).mockResolvedValue([])

    const wrapper = await mountComponent()
    await flushPromises()

    expect(wrapper.text()).toContain('BUG-001')
    expect(wrapper.text()).toContain('Test Bug bug-1')
    expect(wrapper.text()).toContain('This is a test bug with some notes')
    expect(wrapper.text()).toContain('/bugs/bug-bug-1')
  })

  it('should display screenshot thumbnails when bug has captures', async () => {
    const sessionStore = useSessionStore()
    const session = createMockSession('session-1')
    sessionStore.activeSession = session

    const bug = createMockBug('bug-1', 'session-1', 'BUG-001')

    const captures = [
      createMockCapture('cap-1', 'bug-1'),
      createMockCapture('cap-2', 'bug-1')
    ]

    vi.mocked(tauri.getBugsBySession).mockResolvedValue([bug])
    vi.mocked(tauri.getBugCaptures).mockResolvedValue(captures)

    const wrapper = await mountComponent()
    await flushPromises()

    const images = wrapper.findAll('img')
    expect(images.length).toBeGreaterThan(0)
  })

  it('should select bug when clicked', async () => {
    const sessionStore = useSessionStore()
    const session = createMockSession('session-1')
    sessionStore.activeSession = session

    const bug = createMockBug('bug-1', 'session-1', 'BUG-001')

    vi.mocked(tauri.getBugsBySession).mockResolvedValue([bug])
    vi.mocked(tauri.getBugCaptures).mockResolvedValue([])

    const wrapper = await mountComponent()
    await flushPromises()

    const bugItem = wrapper.find('.bug-card')
    await bugItem.trigger('click')
    await flushPromises()

    expect(wrapper.text()).toContain('Bug Type')
  })

  it('should display bug detail panel when bug is selected', async () => {
    const sessionStore = useSessionStore()
    const session = createMockSession('session-1')
    sessionStore.activeSession = session

    const bug = createMockBug('bug-1', 'session-1', 'BUG-001')

    vi.mocked(tauri.getBugsBySession).mockResolvedValue([bug])
    vi.mocked(tauri.getBugCaptures).mockResolvedValue([])

    const wrapper = await mountComponent()
    await flushPromises()

    const bugItem = wrapper.find('.bug-card')
    await bugItem.trigger('click')
    await flushPromises()

    expect(wrapper.text()).toContain('Test Bug bug-1')
    expect(wrapper.text()).toContain('Bug Type')
    expect(wrapper.text()).toContain('Bug ID')
    expect(wrapper.text()).toContain('Status')
    expect(wrapper.text()).toContain('Folder Path')
  })

  it('should show "Select a bug to view details" when no bug is selected', async () => {
    const sessionStore = useSessionStore()
    const session = createMockSession('session-1')
    sessionStore.activeSession = session

    vi.mocked(tauri.getBugsBySession).mockResolvedValue([])

    const wrapper = await mountComponent()
    await flushPromises()

    expect(wrapper.text()).toContain('Select a bug to view details')
  })

  it('should call updateBugType when bug type button is clicked', async () => {
    const sessionStore = useSessionStore()
    const bugStore = useBugStore()
    const session = createMockSession('session-1')
    sessionStore.activeSession = session

    const bug = createMockBug('bug-1', 'session-1', 'BUG-001')

    vi.mocked(tauri.getBugsBySession).mockResolvedValue([bug])
    vi.mocked(tauri.getBugCaptures).mockResolvedValue([])
    const updateSpy = vi.spyOn(bugStore, 'updateBackendBug').mockResolvedValue(undefined)

    const wrapper = await mountComponent()
    await flushPromises()

    const bugItem = wrapper.find('.bug-card')
    await bugItem.trigger('click')
    await flushPromises()

    const buttons = wrapper.findAll('button')
    const featureButton = buttons.find(btn => btn.text().includes('Feature'))
    expect(featureButton).toBeDefined()

    if (featureButton) {
      await featureButton.trigger('click')
      await flushPromises()

      expect(updateSpy).toHaveBeenCalledWith('bug-1', { type: 'feature' })
    }
  })

  it('should navigate back when back button is clicked', async () => {
    const sessionStore = useSessionStore()
    const session = createMockSession('session-1')
    sessionStore.activeSession = session

    vi.mocked(tauri.getBugsBySession).mockResolvedValue([])

    const wrapper = await mountComponent()
    await flushPromises()

    const backSpy = vi.spyOn(router, 'back')

    const backButton = wrapper.find('button')
    await backButton.trigger('click')

    expect(backSpy).toHaveBeenCalled()
  })

  it('should reload bug captures when Refresh Captures is clicked', async () => {
    const sessionStore = useSessionStore()
    const session = createMockSession('session-1')
    sessionStore.activeSession = session

    const bug = createMockBug('bug-1', 'session-1', 'BUG-001')
    const initialCaptures = [createMockCapture('cap-1', 'bug-1')]
    const refreshedCaptures = [
      createMockCapture('cap-1', 'bug-1'),
      createMockCapture('cap-2', 'bug-1')
    ]

    vi.mocked(tauri.getBugsBySession).mockResolvedValue([bug])
    vi.mocked(tauri.getBugCaptures)
      .mockResolvedValueOnce(initialCaptures)
      .mockResolvedValueOnce(refreshedCaptures)

    const wrapper = await mountComponent()
    await flushPromises()

    // Verify initial load happened
    expect(tauri.getBugCaptures).toHaveBeenCalledTimes(1)

    // Click the Refresh Captures button
    const refreshButton = wrapper.findAll('button').find(btn => btn.text().includes('Refresh Captures'))
    expect(refreshButton).toBeDefined()

    if (refreshButton) {
      await refreshButton.trigger('click')
      await flushPromises()

      // Should have reloaded captures
      expect(tauri.getBugCaptures).toHaveBeenCalledTimes(2)
    }
  })

  it('should load bug captures on mount', async () => {
    const sessionStore = useSessionStore()
    const session = createMockSession('session-1')
    sessionStore.activeSession = session

    const bug = createMockBug('bug-1', 'session-1', 'BUG-001')

    const captures = [createMockCapture('cap-1', 'bug-1')]

    vi.mocked(tauri.getBugsBySession).mockResolvedValue([bug])
    vi.mocked(tauri.getBugCaptures).mockResolvedValue(captures)

    await mountComponent()
    await flushPromises()

    expect(tauri.getBugCaptures).toHaveBeenCalledWith('bug-1')
  })

  it('should auto-select first bug on mount when bugs exist', async () => {
    const sessionStore = useSessionStore()
    const session = createMockSession('session-1')
    sessionStore.activeSession = session

    const bug = createMockBug('bug-1', 'session-1', 'BUG-001')

    vi.mocked(tauri.getBugsBySession).mockResolvedValue([bug])
    vi.mocked(tauri.getBugCaptures).mockResolvedValue([])

    const wrapper = await mountComponent()
    await flushPromises()

    // Bug detail panel should be visible
    expect(wrapper.text()).toContain('Bug Type')
  })

  it('should display bug notes in detail panel', async () => {
    const sessionStore = useSessionStore()
    const session = createMockSession('session-1')
    sessionStore.activeSession = session

    const bug = createMockBug('bug-1', 'session-1', 'BUG-001')
    bug.notes = 'These are detailed bug notes'

    vi.mocked(tauri.getBugsBySession).mockResolvedValue([bug])
    vi.mocked(tauri.getBugCaptures).mockResolvedValue([])

    const wrapper = await mountComponent()
    await flushPromises()

    expect(wrapper.text()).toContain('These are detailed bug notes')
  })

  it('should display bug description in detail panel', async () => {
    const sessionStore = useSessionStore()
    const session = createMockSession('session-1')
    sessionStore.activeSession = session

    const bug = createMockBug('bug-1', 'session-1', 'BUG-001')
    bug.description = 'This is the bug description'

    vi.mocked(tauri.getBugsBySession).mockResolvedValue([bug])
    vi.mocked(tauri.getBugCaptures).mockResolvedValue([])

    const wrapper = await mountComponent()
    await flushPromises()

    expect(wrapper.text()).toContain('This is the bug description')
  })

  describe('AI Description Generation', () => {
    it('should check Claude status on mount', async () => {
      const sessionStore = useSessionStore()
      const session = createMockSession('session-1')
      sessionStore.activeSession = session

      vi.mocked(tauri.getClaudeStatus).mockResolvedValue({
        status: 'ready', version: '1.0.0'
      })
      vi.mocked(tauri.getBugsBySession).mockResolvedValue([])

      await mountComponent()
      await flushPromises()

      expect(tauri.getClaudeStatus).toHaveBeenCalled()
    })

    it('should show Generate Description button when Claude is available', async () => {
      const sessionStore = useSessionStore()
      const session = createMockSession('session-1')
      sessionStore.activeSession = session

      const bug = createMockBug('bug-1', 'session-1', 'BUG-001')

      vi.mocked(tauri.getClaudeStatus).mockResolvedValue({
        status: 'ready', version: '1.0.0'
      })
      vi.mocked(tauri.getBugsBySession).mockResolvedValue([bug])
      vi.mocked(tauri.getBugCaptures).mockResolvedValue([])

      const wrapper = await mountComponent()
      await flushPromises()

      expect(wrapper.text()).toContain('Generate Description')
    })

    it('should show warning when Claude is not available', async () => {
      const sessionStore = useSessionStore()
      const session = createMockSession('session-1')
      sessionStore.activeSession = session

      const bug = createMockBug('bug-1', 'session-1', 'BUG-001')

      vi.mocked(tauri.getClaudeStatus).mockResolvedValue({
        status: 'notInstalled', message: 'Claude CLI not found'
      })
      vi.mocked(tauri.getBugsBySession).mockResolvedValue([bug])
      vi.mocked(tauri.getBugCaptures).mockResolvedValue([])

      const wrapper = await mountComponent()
      await flushPromises()

      expect(wrapper.text()).toContain('Claude CLI not available')
      expect(wrapper.text()).toContain('Claude CLI not found')
    })

    it('should generate description when Generate button is clicked', async () => {
      const sessionStore = useSessionStore()
      const session = createMockSession('session-1')
      sessionStore.activeSession = session

      const bug = createMockBug('bug-1', 'session-1', 'BUG-001')
      const capture = createMockCapture('cap-1', 'bug-1')

      vi.mocked(tauri.getClaudeStatus).mockResolvedValue({
        status: 'ready', version: '1.0.0'
      })
      vi.mocked(tauri.getBugsBySession).mockResolvedValue([bug])
      vi.mocked(tauri.getBugCaptures).mockResolvedValue([capture])
      vi.mocked(tauri.generateBugDescription).mockResolvedValue({
        content: 'AI-generated description text',
        task: 'DescribeBug',
        bug_id: 'bug-1'
      })

      const wrapper = await mountComponent()
      await flushPromises()

      const generateButton = wrapper.findAll('button').find(btn => btn.text().includes('Generate Description'))
      expect(generateButton).toBeDefined()

      if (generateButton) {
        await generateButton.trigger('click')
        await flushPromises()

        expect(tauri.generateBugDescription).toHaveBeenCalledWith({
          bug_id: 'bug-1',
          bug_type: 'bug',
          notes: 'This is a test bug with some notes',
          screenshot_paths: ['/captures/screenshot-cap-1.png'],
          metadata: {
            display_id: 'BUG-001',
            status: 'captured'
          }
        })
      }
    })

    it('should show loading state while generating description', async () => {
      const sessionStore = useSessionStore()
      const session = createMockSession('session-1')
      sessionStore.activeSession = session

      const bug = createMockBug('bug-1', 'session-1', 'BUG-001')

      vi.mocked(tauri.getClaudeStatus).mockResolvedValue({
        status: 'ready', version: '1.0.0'
      })
      vi.mocked(tauri.getBugsBySession).mockResolvedValue([bug])
      vi.mocked(tauri.getBugCaptures).mockResolvedValue([])

      let resolveGenerate: (value: any) => void
      const generatePromise = new Promise((resolve) => {
        resolveGenerate = resolve
      })
      vi.mocked(tauri.generateBugDescription).mockReturnValue(generatePromise as any)

      const wrapper = await mountComponent()
      await flushPromises()

      const generateButton = wrapper.findAll('button').find(btn => btn.text().includes('Generate Description'))
      if (generateButton) {
        await generateButton.trigger('click')
        await flushPromises()

        expect(wrapper.text()).toContain('Generating description with Claude')

        resolveGenerate!({
          content: 'Generated description',
          task: 'DescribeBug'
        })
        await flushPromises()
      }
    })

    it('should show Refine and Save buttons after description is generated', async () => {
      const sessionStore = useSessionStore()
      const session = createMockSession('session-1')
      sessionStore.activeSession = session

      const bug = createMockBug('bug-1', 'session-1', 'BUG-001')

      vi.mocked(tauri.getClaudeStatus).mockResolvedValue({
        status: 'ready', version: '1.0.0'
      })
      vi.mocked(tauri.getBugsBySession).mockResolvedValue([bug])
      vi.mocked(tauri.getBugCaptures).mockResolvedValue([])
      vi.mocked(tauri.generateBugDescription).mockResolvedValue({
        content: 'AI-generated description',
        task: 'DescribeBug'
      })

      const wrapper = await mountComponent()
      await flushPromises()

      const generateButton = wrapper.findAll('button').find(btn => btn.text().includes('Generate Description'))
      if (generateButton) {
        await generateButton.trigger('click')
        await flushPromises()

        expect(wrapper.text()).toContain('Refine')
        expect(wrapper.text()).toContain('Save')
      }
    })

    it('should save description when Save button is clicked', async () => {
      const sessionStore = useSessionStore()
      const session = createMockSession('session-1')
      sessionStore.activeSession = session

      const bug = createMockBug('bug-1', 'session-1', 'BUG-001')

      vi.mocked(tauri.getClaudeStatus).mockResolvedValue({
        status: 'ready', version: '1.0.0'
      })
      vi.mocked(tauri.getBugsBySession).mockResolvedValue([bug])
      vi.mocked(tauri.getBugCaptures).mockResolvedValue([])
      vi.mocked(tauri.generateBugDescription).mockResolvedValue({
        content: 'AI-generated description',
        task: 'DescribeBug'
      })
      vi.mocked(tauri.saveBugDescription).mockResolvedValue(undefined)

      const wrapper = await mountComponent()
      await flushPromises()

      const generateButton = wrapper.findAll('button').find(btn => btn.text().includes('Generate Description'))
      if (generateButton) {
        await generateButton.trigger('click')
        await flushPromises()

        const saveButton = wrapper.findAll('button').find(btn => btn.text().includes('Save'))
        if (saveButton) {
          await saveButton.trigger('click')
          await flushPromises()

          expect(tauri.saveBugDescription).toHaveBeenCalledWith(
            '/bugs/bug-bug-1',
            'AI-generated description'
          )
        }
      }
    })

    it('should show Refresh Captures button in actions row when bug is selected', async () => {
      const sessionStore = useSessionStore()
      const session = createMockSession('session-1')
      sessionStore.activeSession = session

      const bug = createMockBug('bug-1', 'session-1', 'BUG-001')

      vi.mocked(tauri.getBugsBySession).mockResolvedValue([bug])
      vi.mocked(tauri.getBugCaptures).mockResolvedValue([])

      const wrapper = await mountComponent()
      await flushPromises()

      expect(wrapper.text()).toContain('Refresh Captures')
    })

    it('should refine description when Refine button is clicked', async () => {
      const sessionStore = useSessionStore()
      const session = createMockSession('session-1')
      sessionStore.activeSession = session

      const bug = createMockBug('bug-1', 'session-1', 'BUG-001')

      vi.mocked(tauri.getClaudeStatus).mockResolvedValue({
        status: 'ready', version: '1.0.0'
      })
      vi.mocked(tauri.getBugsBySession).mockResolvedValue([bug])
      vi.mocked(tauri.getBugCaptures).mockResolvedValue([])
      vi.mocked(tauri.generateBugDescription).mockResolvedValue({
        content: 'Initial description',
        task: 'DescribeBug'
      })
      vi.mocked(tauri.refineBugDescription).mockResolvedValue({
        content: 'Refined description',
        task: 'RefineDescription'
      })

      const wrapper = await mountComponent()
      await flushPromises()

      // Generate description first
      const generateButton = wrapper.findAll('button').find(btn => btn.text().includes('Generate Description'))
      if (generateButton) {
        await generateButton.trigger('click')
        await flushPromises()

        // Click Refine button
        const refineButton = wrapper.findAll('button').find(btn => btn.text().includes('Refine'))
        if (refineButton) {
          await refineButton.trigger('click')
          await flushPromises()

          // Refine dialog should be visible
          expect(wrapper.html()).toContain('How would you like to refine')
        }
      }
    })
  })
})
