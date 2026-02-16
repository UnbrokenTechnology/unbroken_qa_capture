import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { Quasar } from 'quasar'
import FirstRunWizard from '@/components/FirstRunWizard.vue'
import type { ClaudeStatus } from '@/api/tauri'
import * as tauri from '@/api/tauri'

// Mock useQuasar
const mockNotify = vi.fn()
vi.mock('quasar', async () => {
  const actual = await vi.importActual('quasar')
  return {
    ...actual,
    useQuasar: () => ({
      notify: mockNotify,
      screen: {
        lt: {
          md: false
        }
      }
    })
  }
})

// Mock Tauri dialog API
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}))

// Mock Tauri API
vi.mock('@/api/tauri', () => ({
  getClaudeStatus: vi.fn(),
  markSetupComplete: vi.fn(),
  getSetting: vi.fn(),
  setSetting: vi.fn(),
  getAllSettings: vi.fn(),
  deleteSetting: vi.fn(),
}))

const mockReadyClaudeStatus: ClaudeStatus = {
  Ready: {
    version: 'claude-cli 1.0.0',
  },
}

const mockNotInstalledClaudeStatus: ClaudeStatus = {
  NotInstalled: {
    message: 'Claude CLI not found in PATH',
  },
}

describe('FirstRunWizard', () => {
  let pinia: ReturnType<typeof createPinia>
  let mockOpen: ReturnType<typeof vi.fn>

  beforeEach(async () => {
    pinia = createPinia()
    setActivePinia(pinia)

    // Get mocked functions
    const dialogModule = await import('@tauri-apps/plugin-dialog')
    mockOpen = vi.mocked(dialogModule.open)

    vi.clearAllMocks()

    // Setup default mocks
    vi.mocked(tauri.getClaudeStatus).mockResolvedValue(mockNotInstalledClaudeStatus)
    vi.mocked(tauri.markSetupComplete).mockResolvedValue(undefined)
    vi.mocked(tauri.getSetting).mockResolvedValue(null)
    vi.mocked(tauri.setSetting).mockResolvedValue(undefined)
    vi.mocked(tauri.getAllSettings).mockResolvedValue([])
    vi.mocked(tauri.deleteSetting).mockResolvedValue(undefined)
    mockOpen.mockResolvedValue(null)
  })

  afterEach(() => {
    // Clean up DOM after each test
    document.body.innerHTML = ''
  })

  const mountComponent = (props = {}) => {
    return mount(FirstRunWizard, {
      props: {
        modelValue: true,
        ...props,
      },
      global: {
        plugins: [
          pinia,
          [Quasar, {
            plugins: {}
          }]
        ],
      },
      attachTo: document.body,
    })
  }

  describe('Visibility', () => {
    it('should render when modelValue is true', async () => {
      mountComponent({ modelValue: true })
      await flushPromises()

      // q-dialog renders content in a portal, check document body
      const wizardCard = document.body.querySelector('.wizard-card')
      expect(wizardCard).toBeTruthy()
    })

    it('should not render when modelValue is false', async () => {
      mountComponent({ modelValue: false })
      await flushPromises()

      const wizardCard = document.body.querySelector('.wizard-card')
      expect(wizardCard).toBeFalsy()
    })
  })

  describe('Step 1: Welcome', () => {
    it('should display welcome message', async () => {
      mountComponent()
      await flushPromises()

      const bodyText = document.body.textContent || ''
      expect(bodyText).toContain('Welcome to Unbroken QA Capture!')
    })

    it('should show feature list', async () => {
      mountComponent()
      await flushPromises()

      const bodyText = document.body.textContent || ''
      expect(bodyText).toContain('Choose where to save your QA sessions')
      expect(bodyText).toContain('Configure keyboard shortcuts')
    })
  })

  describe('Step 2: Sessions Folder', () => {
    it('should display folder selection input', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      // Navigate to step 2
      const nextButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('label') === 'Next'
      )
      await nextButton!.trigger('click')
      await flushPromises()

      const bodyText = document.body.textContent || ''
      expect(bodyText).toContain('Choose Sessions Root Folder')
      const folderInput = wrapper.findAllComponents({ name: 'QInput' }).find(input =>
        input.props('label') === 'Sessions Folder Path'
      )
      expect(folderInput).toBeDefined()
    })

    it('should update folder path when folder is selected', async () => {
      mockOpen.mockResolvedValue('/test/sessions')

      const wrapper = mountComponent()
      await flushPromises()

      // Navigate to step 2
      let nextButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('label') === 'Next'
      )
      await nextButton!.trigger('click')
      await flushPromises()

      const browseButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('icon') === 'folder_open'
      )
      await browseButton!.trigger('click')
      await flushPromises()

      const bodyText = document.body.textContent || ''
      expect(bodyText).toContain('/test/sessions')
    })
  })

  describe('Step 3: Hotkeys', () => {
    it('should display default hotkeys', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      // Navigate to step 3
      mockOpen.mockResolvedValue('/test/sessions')
      let nextButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('label') === 'Next'
      )
      await nextButton!.trigger('click')
      await flushPromises()

      const browseButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('icon') === 'folder_open'
      )
      await browseButton!.trigger('click')
      await flushPromises()

      nextButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('label') === 'Next'
      )
      await nextButton!.trigger('click')
      await flushPromises()

      const bodyText = document.body.textContent || ''
      expect(bodyText).toContain('Configure Keyboard Shortcuts')
      expect(bodyText).toContain('Ctrl+Shift+B')
      expect(bodyText).toContain('Ctrl+Shift+S')
      expect(bodyText).toContain('Ctrl+Shift+E')
    })
  })

  describe('Step 5: Claude CLI Detection', () => {
    it('should auto-check Claude status when reaching step 5', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      // Navigate to step 5
      mockOpen.mockResolvedValue('/test/sessions')
      let nextButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('label') === 'Next'
      )
      await nextButton!.trigger('click')
      await flushPromises()

      const browseButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('icon') === 'folder_open'
      )
      await browseButton!.trigger('click')
      await flushPromises()

      for (let i = 0; i < 3; i++) {
        nextButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
          btn.props('label') === 'Next'
        )
        await nextButton!.trigger('click')
        await flushPromises()
      }

      expect(vi.mocked(tauri.getClaudeStatus)).toHaveBeenCalled()
    })

    it('should display Ready status when Claude CLI is installed and authenticated', async () => {
      vi.mocked(tauri.getClaudeStatus).mockResolvedValue(mockReadyClaudeStatus)

      const wrapper = mountComponent()
      await flushPromises()

      // Navigate to step 5
      mockOpen.mockResolvedValue('/test/sessions')
      let nextButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('label') === 'Next'
      )
      await nextButton!.trigger('click')
      await flushPromises()

      const browseButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('icon') === 'folder_open'
      )
      await browseButton!.trigger('click')
      await flushPromises()

      for (let i = 0; i < 3; i++) {
        nextButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
          btn.props('label') === 'Next'
        )
        await nextButton!.trigger('click')
        await flushPromises()
      }

      const bodyText = document.body.textContent || ''
      expect(bodyText).toContain('Claude CLI is installed and ready!')
      expect(bodyText).toContain('claude-cli 1.0.0')
    })

    it('should display NotInstalled status when Claude CLI is not found', async () => {
      vi.mocked(tauri.getClaudeStatus).mockResolvedValue(mockNotInstalledClaudeStatus)

      const wrapper = mountComponent()
      await flushPromises()

      // Navigate to step 5
      mockOpen.mockResolvedValue('/test/sessions')
      let nextButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('label') === 'Next'
      )
      await nextButton!.trigger('click')
      await flushPromises()

      const browseButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('icon') === 'folder_open'
      )
      await browseButton!.trigger('click')
      await flushPromises()

      for (let i = 0; i < 3; i++) {
        nextButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
          btn.props('label') === 'Next'
        )
        await nextButton!.trigger('click')
        await flushPromises()
      }

      const bodyText = document.body.textContent || ''
      expect(bodyText).toContain('Claude CLI is not installed')
    })
  })

  describe('Setup Completion', () => {
    it('should save all settings when finishing setup', async () => {
      mockOpen.mockResolvedValue('/test/sessions')
      vi.mocked(tauri.getClaudeStatus).mockResolvedValue(mockReadyClaudeStatus)

      const wrapper = mountComponent()
      await flushPromises()

      // Navigate through all steps
      let nextButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('label') === 'Next'
      )
      await nextButton!.trigger('click')
      await flushPromises()

      const browseButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('icon') === 'folder_open'
      )
      await browseButton!.trigger('click')
      await flushPromises()

      for (let i = 0; i < 3; i++) {
        nextButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
          btn.props('label') === 'Next'
        )
        await nextButton!.trigger('click')
        await flushPromises()
      }

      // Click Finish Setup
      const finishButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('label') === 'Finish Setup'
      )
      await finishButton!.trigger('click')
      await flushPromises()

      expect(vi.mocked(tauri.setSetting)).toHaveBeenCalledWith('default_save_path', '/test/sessions')
      expect(vi.mocked(tauri.setSetting)).toHaveBeenCalledWith('hotkey_capture', 'Ctrl+Shift+B')
      expect(vi.mocked(tauri.markSetupComplete)).toHaveBeenCalled()
    })

    it('should emit complete event when setup finishes', async () => {
      mockOpen.mockResolvedValue('/test/sessions')

      const wrapper = mountComponent()
      await flushPromises()

      // Navigate through all steps
      let nextButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('label') === 'Next'
      )
      await nextButton!.trigger('click')
      await flushPromises()

      const browseButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('icon') === 'folder_open'
      )
      await browseButton!.trigger('click')
      await flushPromises()

      for (let i = 0; i < 3; i++) {
        nextButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
          btn.props('label') === 'Next'
        )
        await nextButton!.trigger('click')
        await flushPromises()
      }

      // Click Finish Setup
      const finishButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('label') === 'Finish Setup'
      )
      await finishButton!.trigger('click')
      await flushPromises()

      expect(wrapper.emitted('complete')).toBeTruthy()
    })
  })

  describe('Navigation', () => {
    it('should show Back button on steps 2-5', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      // Step 1 - no back button
      let backButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('label') === 'Back'
      )
      expect(backButton).toBeUndefined()

      // Navigate to step 2
      const nextButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('label') === 'Next'
      )
      await nextButton!.trigger('click')
      await flushPromises()

      // Step 2 - should have back button
      backButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('label') === 'Back'
      )
      expect(backButton).toBeDefined()
    })
  })
})
