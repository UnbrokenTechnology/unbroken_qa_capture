import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createRouter, createMemoryHistory } from 'vue-router'
import { Quasar, Notify } from 'quasar'
import BugDetail from '@/views/BugDetail.vue'
import { useBugStore } from '@/stores/bug'
import type { Bug as BackendBug, Capture } from '@/types/backend'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Mock the tauri API module (used for getBugCaptures)
vi.mock('@/api/tauri', () => ({
  getBugCaptures: vi.fn().mockResolvedValue([]),
  getBug: vi.fn().mockResolvedValue(null),
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

    expect(wrapper.text()).toContain('Test Bug Title')
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

  it('should call copy_bug_to_clipboard command with correct folder path', async () => {
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
      folderPath: '/test/path'
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
    expect(wrapper.text()).toContain('Test Bug Title')
  })
})
