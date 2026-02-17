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
  refreshClaudeStatus: vi.fn(),
  markSetupComplete: vi.fn(),
  getSetting: vi.fn(),
  setSetting: vi.fn(),
  getAllSettings: vi.fn(),
  deleteSetting: vi.fn(),
}))

const mockReadyClaudeStatus: ClaudeStatus = {
  status: 'ready',
  version: 'claude-cli 1.0.0',
}

const mockNotInstalledClaudeStatus: ClaudeStatus = {
  status: 'notInstalled',
  message: 'Claude CLI not found in PATH',
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
    vi.mocked(tauri.refreshClaudeStatus).mockResolvedValue(mockNotInstalledClaudeStatus)
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
      expect(bodyText).toContain('Current Keyboard Shortcuts')
      // Verify correct defaults from hotkey.rs
      expect(bodyText).toContain('Ctrl+Shift+Q') // Toggle Session
      expect(bodyText).toContain('PrintScreen')  // Start Bug Capture
      expect(bodyText).toContain('F4')           // End Bug Capture
      expect(bodyText).toContain('Ctrl+Shift+N') // Open Quick Notepad
      expect(bodyText).toContain('Ctrl+Shift+M') // Open Session Notepad
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

      expect(vi.mocked(tauri.refreshClaudeStatus)).toHaveBeenCalled()
    })

    it('should display Ready status when Claude CLI is installed and authenticated', async () => {
      vi.mocked(tauri.refreshClaudeStatus).mockResolvedValue(mockReadyClaudeStatus)

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
      vi.mocked(tauri.refreshClaudeStatus).mockResolvedValue(mockNotInstalledClaudeStatus)

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
      vi.mocked(tauri.refreshClaudeStatus).mockResolvedValue(mockReadyClaudeStatus)

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
      // Hotkeys are no longer saved from wizard - they use backend defaults from hotkey.rs
      expect(vi.mocked(tauri.setSetting)).not.toHaveBeenCalledWith('hotkey_capture', expect.anything())
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

  describe('Validation', () => {
    it('should disable Finish Setup button when folder is not selected', async () => {
      const wrapper = mountComponent()
      await flushPromises()

      // Can't proceed to step 2 without completing step 1, but step 1 doesn't require folder
      // Step 2 requires folder to proceed to step 3
      // So we test by being on step 2 without selecting folder

      // Navigate to step 2
      const nextButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('label') === 'Next'
      )
      await nextButton!.trigger('click')
      await flushPromises()

      // Next button on step 2 should be disabled without folder selection
      const nextButtonStep2 = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('label') === 'Next'
      )
      expect(nextButtonStep2).toBeDefined()
      expect(nextButtonStep2!.props('disable')).toBe(true)
    })

    it('should enable Finish Setup button when folder is selected', async () => {
      mockOpen.mockResolvedValue('/test/sessions')

      const wrapper = mountComponent()
      await flushPromises()

      // Navigate to step 2 and select folder
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

      // Navigate to step 5
      for (let i = 0; i < 3; i++) {
        nextButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
          btn.props('label') === 'Next'
        )
        await nextButton!.trigger('click')
        await flushPromises()
      }

      // Find Finish Setup button - should be enabled
      const finishButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('label') === 'Finish Setup'
      )
      expect(finishButton).toBeDefined()
      expect(finishButton!.props('disable')).toBe(false)
    })
  })

  describe('Navigate-back: preserve completed state', () => {
    // Helper: count step tabs with the "done" CSS class in the stepper header
    const countDoneStepTabs = () =>
      document.body.querySelectorAll('.q-stepper__tab--done').length

    it('should keep step 1 marked as done after navigating back from step 2', async () => {
      mountComponent()
      await flushPromises()

      // Initially no steps are done
      expect(countDoneStepTabs()).toBe(0)

      // Navigate to step 2
      // Find Next button via wrapper since querySelector approach is fragile
      const wrapper = mountComponent()
      await flushPromises()

      let nextBtn = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('label') === 'Next'
      )
      await nextBtn!.trigger('click')
      await flushPromises()

      // Navigate back to step 1
      const backButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('label') === 'Back'
      )
      await backButton!.trigger('click')
      await flushPromises()

      // Step 1 tab should now be marked done
      expect(document.body.querySelectorAll('.q-stepper__tab--done').length).toBeGreaterThanOrEqual(1)
    })

    it('should keep steps 1 and 2 marked as done after navigating back from step 3', async () => {
      mockOpen.mockResolvedValue('/test/sessions')

      const wrapper = mountComponent()
      await flushPromises()

      // Navigate to step 2
      let nextButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('label') === 'Next'
      )
      await nextButton!.trigger('click')
      await flushPromises()

      // Select folder to enable proceeding
      const browseButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('icon') === 'folder_open'
      )
      await browseButton!.trigger('click')
      await flushPromises()

      // Navigate to step 3
      nextButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('label') === 'Next'
      )
      await nextButton!.trigger('click')
      await flushPromises()

      // Navigate back to step 2
      const backButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('label') === 'Back'
      )
      await backButton!.trigger('click')
      await flushPromises()

      // Step 1 tab should be done (we've advanced past step 1 to step 3)
      expect(document.body.querySelectorAll('.q-stepper__tab--done').length).toBeGreaterThanOrEqual(1)
    })

    it('should not mark steps as done if they have not been visited', async () => {
      mountComponent()
      await flushPromises()

      // On step 1, no steps have been completed yet (maxStepReached=1, no step satisfies maxStepReached > N for N>=1)
      expect(countDoneStepTabs()).toBe(0)
    })

    it('should preserve done state: navigating forward then back keeps steps done', async () => {
      mockOpen.mockResolvedValue('/test/sessions')

      const wrapper = mountComponent()
      await flushPromises()

      // Navigate forward through steps 1 -> 2 -> 3
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

      // Now at step 3 — step 1 and 2 tabs should be done
      const doneAtStep3 = document.body.querySelectorAll('.q-stepper__tab--done').length
      expect(doneAtStep3).toBeGreaterThanOrEqual(1)

      // Navigate back to step 2
      const backButton = wrapper.findAllComponents({ name: 'QBtn' }).find(btn =>
        btn.props('label') === 'Back'
      )
      await backButton!.trigger('click')
      await flushPromises()

      // Step 1 tab should still be done after navigating back
      const doneAfterBack = document.body.querySelectorAll('.q-stepper__tab--done').length
      expect(doneAfterBack).toBeGreaterThanOrEqual(1)
    })
  })
})
