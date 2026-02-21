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
  invoke: vi.fn(),
  convertFileSrc: vi.fn((path: string) => `asset://localhost/${path}`),
}))

// Mock the tauri API module (used for getBugCaptures)
vi.mock('@/api/tauri', () => ({
  getBugCaptures: vi.fn().mockResolvedValue([]),
  getBug: vi.fn().mockResolvedValue(null),
  updateCaptureConsoleFlag: vi.fn().mockResolvedValue(undefined),
  updateBug: vi.fn().mockResolvedValue(undefined),
  updateBugType: vi.fn().mockResolvedValue(undefined),
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
})
