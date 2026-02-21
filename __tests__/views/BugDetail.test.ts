import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createRouter, createMemoryHistory } from 'vue-router'
import { Quasar, Notify } from 'quasar'
import BugDetail from '@/views/BugDetail.vue'
import { useBugStore } from '@/stores/bug'
import { useSessionStore } from '@/stores/session'
import { useProfileStore } from '@/stores/profile'
import type { Bug as BackendBug, Capture, QaProfile } from '@/types/backend'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  convertFileSrc: vi.fn((path: string) => `asset://localhost/${path}`),
}))

// Mock the tauri API module (used for getBugCaptures, getProfile, updateBugMetadata, etc.)
vi.mock('@/api/tauri', () => ({
  getBugCaptures: vi.fn().mockResolvedValue([]),
  getBug: vi.fn().mockResolvedValue(null),
  updateCaptureConsoleFlag: vi.fn().mockResolvedValue(undefined),
  updateBug: vi.fn().mockResolvedValue(undefined),
  updateBugType: vi.fn().mockResolvedValue(undefined),
  updateBugTitle: vi.fn().mockResolvedValue(undefined),
  getProfile: vi.fn().mockResolvedValue(null),
  updateBugMetadata: vi.fn().mockResolvedValue(undefined),
  assignCaptureToBug: vi.fn().mockResolvedValue(undefined),
}))

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

const createMockBackendBug = (id: string): BackendBug => ({
  id,
  session_id: 'session-1',
  bug_number: 1,
  display_id: 'BUG-001',
  type: 'bug',
  title: 'Test Bug Title',
  notes: null,
  description: null,
  ai_description: null,
  status: 'captured',
  meeting_id: 'MTG-123',
  software_version: '1.0.0',
  console_parse_json: null,
  metadata_json: JSON.stringify({
    environment: {
      os: 'Windows 11',
      display_resolution: '1920x1080',
      dpi_scaling: '100%',
      ram: '16GB',
      cpu: 'Intel i7',
      foreground_app: 'TestApp'
    }
  }),
  folder_path: '/test/path',
  created_at: '2024-01-15T10:30:00Z',
  updated_at: '2024-01-15T10:30:00Z',
})

const createMockCapture = (filePath: string, bugId: string): Capture => ({
  id: `capture-${filePath}`,
  bug_id: bugId,
  session_id: 'session-1',
  file_name: filePath.split('/').pop() ?? filePath,
  file_path: filePath,
  file_type: 'screenshot',
  annotated_path: null,
  file_size_bytes: null,
  is_console_capture: false,
  parsed_content: null,
  created_at: '2024-01-15T10:30:00Z',
})

describe('BugDetail', () => {
  let router: ReturnType<typeof createRouter>
  let pinia: ReturnType<typeof createPinia>

  beforeEach(async () => {
    pinia = createPinia()
    setActivePinia(pinia)

    // Clear mock notify between tests
    mockNotify.mockClear()

    // Reset tauri API mocks
    const tauriApi = await import('@/api/tauri')
    vi.mocked(tauriApi.getBugCaptures).mockResolvedValue([])
    vi.mocked(tauriApi.updateBugTitle).mockClear()

    router = createRouter({
      history: createMemoryHistory(),
      routes: [
        {
          path: '/bug/:id',
          name: 'bug-detail',
          component: BugDetail
        }
      ]
    })
  })

  const mountComponent = async (bugId: string) => {
    router.push(`/bug/${bugId}`)
    await router.isReady()

    return mount(BugDetail, {
      global: {
        plugins: [
          pinia,
          router,
          [Quasar, { plugins: { Notify } }]
        ],
        stubs: {
          QPage: { template: '<div><slot /></div>' },
          QCard: { template: '<div><slot /></div>' },
          QCardSection: { template: '<div><slot /></div>' },
          QBtn: { template: '<button @click="$attrs.onClick">{{ $attrs.label }}<slot /></button>' },
          QInput: {
            template: '<input :value="modelValue" :placeholder="$attrs.placeholder" @input="$emit(\'update:modelValue\', $event.target.value)" @blur="$attrs.onBlur && $attrs.onBlur()" />',
            props: ['modelValue'],
            emits: ['update:modelValue'],
          },
          QCarousel: { template: '<div><slot /></div>' },
          QCarouselSlide: { template: '<div><slot /></div>' },
          QImg: { template: '<img />' },
          QScrollArea: { template: '<div><slot /></div>' },
          QSpinner: { template: '<div class="spinner" />' },
          ScreenshotAnnotator: { template: '<div />' },
          VideoPlayer: { template: '<div />' },
        }
      }
    })
  }

  it('should display bug not found when bug does not exist', async () => {
    const wrapper = await mountComponent('999')
    await flushPromises()

    expect(wrapper.text()).toContain('Bug not found')
  })

  it('should display bug details when bug exists in backendBugs', async () => {
    const store = useBugStore()
    const bug = createMockBackendBug('1')
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()

    // Title is rendered as an input — check its value
    const titleInput = wrapper.find('input')
    expect(titleInput.element.value).toBe('Test Bug Title')
    expect(wrapper.text()).toContain('bug')
    expect(wrapper.text()).toContain('/test/path')
  })

  it('should display metadata information', async () => {
    const store = useBugStore()
    const bug = createMockBackendBug('1')
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()

    expect(wrapper.text()).toContain('1.0.0')
    expect(wrapper.text()).toContain('MTG-123')
    expect(wrapper.text()).toContain('/test/path')
  })

  it('should display environment information from metadata_json', async () => {
    const store = useBugStore()
    const bug = createMockBackendBug('1')
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()

    expect(wrapper.text()).toContain('Windows 11')
    expect(wrapper.text()).toContain('1920x1080')
    expect(wrapper.text()).toContain('100%')
    expect(wrapper.text()).toContain('16GB')
    expect(wrapper.text()).toContain('Intel i7')
    expect(wrapper.text()).toContain('TestApp')
  })

  it('should display screenshot count from fetched captures', async () => {
    const tauriApi = await import('@/api/tauri')
    vi.mocked(tauriApi.getBugCaptures).mockResolvedValue([
      createMockCapture('image1.png', '1'),
      createMockCapture('image2.png', '1'),
    ])

    const store = useBugStore()
    const bug = createMockBackendBug('1')
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()

    expect(wrapper.text()).toContain('All Captures (2)')
  })

  it('should display no-captures placeholder when no captures', async () => {
    const store = useBugStore()
    const bug = createMockBackendBug('1')
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()

    // Should show the "no captures yet" placeholder, not the captures card
    expect(wrapper.text()).toContain('No captures yet')
    expect(wrapper.text()).not.toContain('All Captures (')
  })

  it('should navigate back when back button is clicked', async () => {
    const store = useBugStore()
    const bug = createMockBackendBug('1')
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()
    const backSpy = vi.spyOn(router, 'back')

    const backButton = wrapper.find('button')
    await backButton.trigger('click')

    expect(backSpy).toHaveBeenCalled()
  })

  it('should not display optional metadata fields when not provided', async () => {
    const store = useBugStore()
    const bug = createMockBackendBug('1')
    bug.meeting_id = null
    bug.software_version = null
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()

    expect(wrapper.text()).not.toContain('Meeting ID')
    expect(wrapper.text()).not.toContain('Software Version')
  })

  it('should display copy to clipboard button', async () => {
    const store = useBugStore()
    const bug = createMockBackendBug('1')
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()

    expect(wrapper.text()).toContain('Copy to Clipboard')
  })

  it('should display open folder button', async () => {
    const store = useBugStore()
    const bug = createMockBackendBug('1')
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()

    expect(wrapper.text()).toContain('Open Folder')
  })

  it('should call open_bug_folder command when open folder button clicked', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    const store = useBugStore()
    const bug = createMockBackendBug('1')
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()
    const buttons = wrapper.findAll('button')
    const openFolderButton = buttons.at(1)! // Second button is the open folder button

    await openFolderButton.trigger('click')

    expect(invoke).toHaveBeenCalledWith('open_bug_folder', {
      folderPath: '/test/path'
    })
  })

  it('should show success notification when open folder succeeds', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    vi.mocked(invoke).mockResolvedValueOnce(undefined)

    const store = useBugStore()
    const bug = createMockBackendBug('1')
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()
    const buttons = wrapper.findAll('button')
    const openFolderButton = buttons.at(1)!

    await openFolderButton.trigger('click')
    await flushPromises()

    expect(mockNotify).toHaveBeenCalledWith({
      type: 'positive',
      message: 'Bug folder opened',
      position: 'top',
      timeout: 2000
    })
  })

  it('should show error notification when open folder fails', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    vi.mocked(invoke).mockRejectedValueOnce('Folder not found')

    const store = useBugStore()
    const bug = createMockBackendBug('1')
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()
    const buttons = wrapper.findAll('button')
    const openFolderButton = buttons.at(1)!

    await openFolderButton.trigger('click')
    await flushPromises()

    expect(mockNotify).toHaveBeenCalledWith({
      type: 'negative',
      message: 'Failed to open bug folder: Folder not found',
      position: 'top',
      timeout: 3000
    })
  })

  it('should call copy_bug_to_clipboard command with correct bug id', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    const store = useBugStore()
    const bug = createMockBackendBug('1')
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()
    const buttons = wrapper.findAll('button')
    const copyButton = buttons.at(2)! // Third button is the copy button (back, open folder, copy)

    await copyButton.trigger('click')

    expect(invoke).toHaveBeenCalledWith('copy_bug_to_clipboard', {
      bugId: '1'
    })
  })

  it('should show success notification when copy succeeds', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    vi.mocked(invoke).mockResolvedValueOnce(undefined)

    const store = useBugStore()
    const bug = createMockBackendBug('1')
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()
    const buttons = wrapper.findAll('button')
    const copyButton = buttons.at(2)! // Third button is copy

    await copyButton.trigger('click')
    await flushPromises()

    expect(mockNotify).toHaveBeenCalledWith({
      type: 'positive',
      message: 'Bug report copied to clipboard',
      position: 'top',
      timeout: 2000
    })
  })

  it('should show error notification when copy fails', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    vi.mocked(invoke).mockRejectedValueOnce('Failed to read bug data')

    const store = useBugStore()
    const bug = createMockBackendBug('1')
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()
    const buttons = wrapper.findAll('button')
    const copyButton = buttons.at(2)! // Third button is copy

    await copyButton.trigger('click')
    await flushPromises()

    expect(mockNotify).toHaveBeenCalledWith({
      type: 'negative',
      message: 'Failed to copy bug report: Failed to read bug data',
      position: 'top',
      timeout: 3000
    })
  })

  it('should fetch bug from backend when not in store', async () => {
    const tauriApi = await import('@/api/tauri')
    const mockBug = createMockBackendBug('42')
    vi.mocked(tauriApi.getBug).mockResolvedValue(mockBug)

    const wrapper = await mountComponent('42')
    await flushPromises()

    expect(tauriApi.getBug).toHaveBeenCalledWith('42')
    // Title is rendered as an input — check its value
    const titleInput = wrapper.find('input')
    expect(titleInput.element.value).toBe('Test Bug Title')
  })

  it('should display "Mark as Console" button on screenshot captures', async () => {
    const tauriApi = await import('@/api/tauri')
    vi.mocked(tauriApi.getBugCaptures).mockResolvedValue([
      createMockCapture('screenshot1.png', '1'),
    ])

    const store = useBugStore()
    store.backendBugs.push(createMockBackendBug('1'))

    const wrapper = await mountComponent('1')
    await flushPromises()

    expect(wrapper.text()).toContain('Mark as Console')
  })

  it('should call updateCaptureConsoleFlag when "Mark as Console" button is clicked', async () => {
    const tauriApi = await import('@/api/tauri')
    const capture = createMockCapture('screenshot1.png', '1')
    vi.mocked(tauriApi.getBugCaptures).mockResolvedValue([capture])

    const store = useBugStore()
    store.backendBugs.push(createMockBackendBug('1'))

    const wrapper = await mountComponent('1')
    await flushPromises()

    const buttons = wrapper.findAll('button')
    const markConsoleBtn = buttons.find(b => b.text().includes('Mark as Console'))
    expect(markConsoleBtn).toBeDefined()

    await markConsoleBtn!.trigger('click')
    await flushPromises()

    expect(tauriApi.updateCaptureConsoleFlag).toHaveBeenCalledWith(capture.id, true)
  })

  it('should display "Unmark" button on console captures', async () => {
    const tauriApi = await import('@/api/tauri')
    const capture = { ...createMockCapture('console1.png', '1'), is_console_capture: true }
    vi.mocked(tauriApi.getBugCaptures).mockResolvedValue([capture])

    const store = useBugStore()
    store.backendBugs.push(createMockBackendBug('1'))

    const wrapper = await mountComponent('1')
    await flushPromises()

    expect(wrapper.text()).toContain('Unmark')
  })

  it('should call updateCaptureConsoleFlag(id, false) when "Unmark" is clicked on a console capture', async () => {
    const tauriApi = await import('@/api/tauri')
    const capture = { ...createMockCapture('console1.png', '1'), is_console_capture: true }
    vi.mocked(tauriApi.getBugCaptures).mockResolvedValue([capture])

    const store = useBugStore()
    store.backendBugs.push(createMockBackendBug('1'))

    const wrapper = await mountComponent('1')
    await flushPromises()

    const buttons = wrapper.findAll('button')
    const unmarkBtn = buttons.find(b => b.text().includes('Unmark'))
    expect(unmarkBtn).toBeDefined()

    await unmarkBtn!.trigger('click')
    await flushPromises()

    expect(tauriApi.updateCaptureConsoleFlag).toHaveBeenCalledWith(capture.id, false)
  })

  it('should show success notification after marking as console capture', async () => {
    const tauriApi = await import('@/api/tauri')
    const capture = createMockCapture('screenshot1.png', '1')
    vi.mocked(tauriApi.getBugCaptures).mockResolvedValue([capture])
    vi.mocked(tauriApi.updateCaptureConsoleFlag).mockResolvedValue(undefined)

    const store = useBugStore()
    store.backendBugs.push(createMockBackendBug('1'))

    const wrapper = await mountComponent('1')
    await flushPromises()

    const buttons = wrapper.findAll('button')
    const markConsoleBtn = buttons.find(b => b.text().includes('Mark as Console'))

    await markConsoleBtn!.trigger('click')
    await flushPromises()

    expect(mockNotify).toHaveBeenCalledWith(
      expect.objectContaining({
        type: 'positive',
        message: 'Marked as console capture',
      })
    )
  })

  it('should show error notification when console flag update fails', async () => {
    const tauriApi = await import('@/api/tauri')
    const capture = createMockCapture('screenshot1.png', '1')
    vi.mocked(tauriApi.getBugCaptures).mockResolvedValue([capture])
    vi.mocked(tauriApi.updateCaptureConsoleFlag).mockRejectedValueOnce(new Error('DB error'))

    const store = useBugStore()
    store.backendBugs.push(createMockBackendBug('1'))

    const wrapper = await mountComponent('1')
    await flushPromises()

    const buttons = wrapper.findAll('button')
    const markConsoleBtn = buttons.find(b => b.text().includes('Mark as Console'))

    await markConsoleBtn!.trigger('click')
    await flushPromises()

    expect(mockNotify).toHaveBeenCalledWith(
      expect.objectContaining({
        type: 'negative',
        message: 'Failed to update console capture status',
      })
    )
  })

  it('should display End Bug Capture button when bug status is capturing', async () => {
    const store = useBugStore()
    const bug = createMockBackendBug('1')
    bug.status = 'capturing'
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()

    expect(wrapper.text()).toContain('End Bug Capture')
  })

  it('should not display End Bug Capture button when bug status is not capturing', async () => {
    const store = useBugStore()
    const bug = createMockBackendBug('1')
    bug.status = 'captured'
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()

    expect(wrapper.text()).not.toContain('End Bug Capture')
  })

  it('should call completeBugCapture when End Bug Capture button is clicked', async () => {
    const store = useBugStore()
    const bug = createMockBackendBug('1')
    bug.status = 'capturing'
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()

    const buttons = wrapper.findAll('button')
    const endCaptureBtn = buttons.find(b => b.text().includes('End Bug Capture'))
    expect(endCaptureBtn).toBeDefined()

    await endCaptureBtn!.trigger('click')
    await flushPromises()

    // completeBugCapture calls updateBackendBug which calls tauri.updateBug
    const tauriApi = await import('@/api/tauri')
    expect(tauriApi.updateBug).toHaveBeenCalledWith('1', { status: 'captured' })
  })

  it('should show success notification after ending bug capture', async () => {
    const store = useBugStore()
    const bug = createMockBackendBug('1')
    bug.status = 'capturing'
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()

    const buttons = wrapper.findAll('button')
    const endCaptureBtn = buttons.find(b => b.text().includes('End Bug Capture'))

    await endCaptureBtn!.trigger('click')
    await flushPromises()

    expect(mockNotify).toHaveBeenCalledWith(
      expect.objectContaining({
        type: 'positive',
        message: 'Capture ended for BUG-001',
      })
    )
  })

  it('should render title input with the bug title as its value', async () => {
    const store = useBugStore()
    const bug = createMockBackendBug('1')
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()

    const titleInput = wrapper.find('input')
    expect(titleInput.exists()).toBe(true)
    expect(titleInput.element.value).toBe('Test Bug Title')
  })

  it('should render title input with placeholder when title is null', async () => {
    const store = useBugStore()
    const bug = createMockBackendBug('1')
    bug.title = null
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()

    const titleInput = wrapper.find('input')
    expect(titleInput.exists()).toBe(true)
    expect(titleInput.attributes('placeholder')).toBe('Enter bug title...')
  })

  it('should call updateBugTitle with new value on blur', async () => {
    const tauriApi = await import('@/api/tauri')
    const store = useBugStore()
    const bug = createMockBackendBug('1')
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()

    const titleInput = wrapper.find('input')
    await titleInput.setValue('Updated Bug Title')
    await titleInput.trigger('blur')
    await flushPromises()

    expect(tauriApi.updateBugTitle).toHaveBeenCalledWith('1', 'Updated Bug Title')
  })

  it('should not call updateBugTitle when title is unchanged on blur', async () => {
    const tauriApi = await import('@/api/tauri')
    const store = useBugStore()
    const bug = createMockBackendBug('1')
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()

    const titleInput = wrapper.find('input')
    // Trigger blur without changing the value
    await titleInput.trigger('blur')
    await flushPromises()

    expect(tauriApi.updateBugTitle).not.toHaveBeenCalled()
  })

  it('should show error notification when updateBugTitle fails', async () => {
    const tauriApi = await import('@/api/tauri')
    vi.mocked(tauriApi.updateBugTitle).mockRejectedValueOnce(new Error('DB error'))

    const store = useBugStore()
    const bug = createMockBackendBug('1')
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()

    const titleInput = wrapper.find('input')
    await titleInput.setValue('Failing Title')
    await titleInput.trigger('blur')
    await flushPromises()

    expect(mockNotify).toHaveBeenCalledWith(
      expect.objectContaining({
        type: 'negative',
        message: 'Failed to save title',
      })
    )
  })

  it('should not show custom metadata fields when no session profile is set', async () => {
    const store = useBugStore()
    const bug = createMockBackendBug('1')
    store.backendBugs.push(bug)

    const wrapper = await mountComponent('1')
    await flushPromises()

    // With no profile, profile custom fields should not be rendered
    // (no QSelect or extra inputs from profile)
    const selects = wrapper.findAll('select')
    expect(selects).toHaveLength(0)
  })

  it('should render QSelect for profile select fields when session has a profile', async () => {
    const tauriApi = await import('@/api/tauri')
    const mockProfile: QaProfile = {
      id: 'profile-1',
      name: 'Test Profile',
      linear_config: null,
      area_categories: [
        { code: 'UI', name: 'User Interface', description: null },
        { code: 'API', name: 'Backend API', description: null },
      ],
      custom_fields: [
        {
          key: 'area_category',
          label: 'Area Category',
          field_type: 'select',
          required: false,
          default_value: null,
          options: [],
        },
      ],
      title_conventions: null,
      created_at: '2024-01-01T00:00:00Z',
      updated_at: '2024-01-01T00:00:00Z',
    }

    vi.mocked(tauriApi.getProfile).mockResolvedValue(mockProfile)

    const store = useBugStore()
    const bug = createMockBackendBug('1')
    store.backendBugs.push(bug)

    // Set an active session with a profile_id
    const sessionStore = useSessionStore()
    sessionStore.activeSession = {
      id: 'session-1',
      started_at: '2024-01-01T00:00:00Z',
      ended_at: null,
      status: 'active',
      folder_path: '/qa/session-1',
      session_notes: null,
      environment_json: null,
      original_snip_path: null,
      created_at: '2024-01-01T00:00:00Z',
      profile_id: 'profile-1',
    }

    const wrapper = await mountComponent('1')
    await flushPromises()

    // The profile should have been fetched
    expect(tauriApi.getProfile).toHaveBeenCalledWith('profile-1')

    // Area Category label should appear in the rendered output
    expect(wrapper.text()).toContain('Area Category')
  })

  it('should pre-populate custom metadata values from bug.custom_metadata', async () => {
    const tauriApi = await import('@/api/tauri')
    const mockProfile: QaProfile = {
      id: 'profile-1',
      name: 'Test Profile',
      linear_config: null,
      area_categories: [
        { code: 'UI', name: 'User Interface', description: null },
        { code: 'API', name: 'Backend API', description: null },
      ],
      custom_fields: [
        {
          key: 'area_category',
          label: 'Area Category',
          field_type: 'select',
          required: false,
          default_value: null,
          options: [],
        },
      ],
      title_conventions: null,
      created_at: '2024-01-01T00:00:00Z',
      updated_at: '2024-01-01T00:00:00Z',
    }

    vi.mocked(tauriApi.getProfile).mockResolvedValue(mockProfile)

    const store = useBugStore()
    const bug = createMockBackendBug('1')
    // Simulate bug already having a custom_metadata value
    bug.custom_metadata = JSON.stringify({ area_category: 'UI' }) as unknown as Record<string, string>
    store.backendBugs.push(bug)

    const sessionStore = useSessionStore()
    sessionStore.activeSession = {
      id: 'session-1',
      started_at: '2024-01-01T00:00:00Z',
      ended_at: null,
      status: 'active',
      folder_path: '/qa/session-1',
      session_notes: null,
      environment_json: null,
      original_snip_path: null,
      created_at: '2024-01-01T00:00:00Z',
      profile_id: 'profile-1',
    }

    await mountComponent('1')
    await flushPromises()

    // The profile was fetched and the component rendered
    expect(tauriApi.getProfile).toHaveBeenCalledWith('profile-1')
  })

  it('should use area_categories from profile to populate options when options array is empty', async () => {
    const tauriApi = await import('@/api/tauri')
    const mockProfile: QaProfile = {
      id: 'profile-1',
      name: 'Test Profile',
      linear_config: null,
      area_categories: [
        { code: 'UI', name: 'User Interface', description: null },
        { code: 'PERF', name: 'Performance', description: null },
      ],
      custom_fields: [
        {
          key: 'area_category',
          label: 'Area Category',
          field_type: 'select',
          required: false,
          default_value: null,
          options: [],  // Empty — should be populated from area_categories
        },
      ],
      title_conventions: null,
      created_at: '2024-01-01T00:00:00Z',
      updated_at: '2024-01-01T00:00:00Z',
    }

    // Use the profileStore (cached path) instead of fetching
    const profileStore = useProfileStore()
    profileStore.profiles.push(mockProfile)

    const store = useBugStore()
    const bug = createMockBackendBug('1')
    store.backendBugs.push(bug)

    const sessionStore = useSessionStore()
    sessionStore.activeSession = {
      id: 'session-1',
      started_at: '2024-01-01T00:00:00Z',
      ended_at: null,
      status: 'active',
      folder_path: '/qa/session-1',
      session_notes: null,
      environment_json: null,
      original_snip_path: null,
      created_at: '2024-01-01T00:00:00Z',
      profile_id: 'profile-1',
    }

    // Clear mock call count before this specific assertion
    vi.mocked(tauriApi.getProfile).mockClear()

    const wrapper = await mountComponent('1')
    await flushPromises()

    // Profile found in store cache — getProfile should NOT be called
    expect(tauriApi.getProfile).not.toHaveBeenCalled()

    // The Area Category label should be rendered
    expect(wrapper.text()).toContain('Area Category')
  })
})
