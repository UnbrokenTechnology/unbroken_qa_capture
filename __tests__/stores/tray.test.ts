import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useTrayStore } from '@/stores/tray'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
  convertFileSrc: vi.fn((path: string) => `asset://localhost/${path}`),
}))

describe('Tray Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('should initialize with idle state', () => {
    const store = useTrayStore()
    expect(store.state).toBe('idle')
    expect(store.sessionActive).toBe(false)
    expect(store.tooltip).toBe('Unbroken QA Capture - Idle')
  })

  describe('setState', () => {
    it('should update state to active', async () => {
      const store = useTrayStore()
      await store.setState('active')

      expect(store.state).toBe('active')
      expect(store.tooltip).toBe('Unbroken QA Capture - Session Active')
    })

    it('should update state to bug', async () => {
      const store = useTrayStore()
      await store.setState('bug')

      expect(store.state).toBe('bug')
      expect(store.tooltip).toBe('Unbroken QA Capture - Capturing Bug')
    })

    it('should update state to review', async () => {
      const store = useTrayStore()
      await store.setState('review')

      expect(store.state).toBe('review')
      expect(store.tooltip).toBe('Unbroken QA Capture - Review Mode')
    })

    it('should update state back to idle', async () => {
      const store = useTrayStore()
      await store.setState('active')
      await store.setState('idle')

      expect(store.state).toBe('idle')
      expect(store.tooltip).toBe('Unbroken QA Capture - Idle')
    })
  })

  describe('setIdle', () => {
    it('should set state to idle and mark session as inactive', async () => {
      const store = useTrayStore()
      store.sessionActive = true

      await store.setIdle()

      expect(store.state).toBe('idle')
      expect(store.sessionActive).toBe(false)
      expect(store.tooltip).toBe('Unbroken QA Capture - Idle')
    })
  })

  describe('setActive', () => {
    it('should set state to active and mark session as active', async () => {
      const store = useTrayStore()

      await store.setActive()

      expect(store.state).toBe('active')
      expect(store.sessionActive).toBe(true)
      expect(store.tooltip).toBe('Unbroken QA Capture - Session Active')
    })
  })

  describe('setBugCapture', () => {
    it('should set state to bug capture', async () => {
      const store = useTrayStore()

      await store.setBugCapture()

      expect(store.state).toBe('bug')
      expect(store.tooltip).toBe('Unbroken QA Capture - Capturing Bug')
    })
  })

  describe('setReview', () => {
    it('should set state to review', async () => {
      const store = useTrayStore()

      await store.setReview()

      expect(store.state).toBe('review')
      expect(store.tooltip).toBe('Unbroken QA Capture - Review Mode')
    })
  })

  describe('setCustomTooltip', () => {
    it('should set custom tooltip text', async () => {
      const store = useTrayStore()
      const customText = 'Custom Status Message'

      await store.setCustomTooltip(customText)

      expect(store.tooltip).toBe(customText)
    })
  })

  describe('toggleSession', () => {
    it('should start session when idle', async () => {
      const store = useTrayStore()
      expect(store.sessionActive).toBe(false)

      await store.toggleSession()

      expect(store.sessionActive).toBe(true)
      expect(store.state).toBe('active')
    })

    it('should end session when active', async () => {
      const store = useTrayStore()
      await store.setActive()
      expect(store.sessionActive).toBe(true)

      await store.toggleSession()

      expect(store.sessionActive).toBe(false)
      expect(store.state).toBe('idle')
    })

    it('should toggle session multiple times', async () => {
      const store = useTrayStore()

      // Start
      await store.toggleSession()
      expect(store.sessionActive).toBe(true)
      expect(store.state).toBe('active')

      // End
      await store.toggleSession()
      expect(store.sessionActive).toBe(false)
      expect(store.state).toBe('idle')

      // Start again
      await store.toggleSession()
      expect(store.sessionActive).toBe(true)
      expect(store.state).toBe('active')
    })
  })

  describe('State transitions', () => {
    it('should handle workflow: idle -> active -> bug -> review -> idle', async () => {
      const store = useTrayStore()

      // Start at idle
      expect(store.state).toBe('idle')

      // Start session
      await store.setActive()
      expect(store.state).toBe('active')
      expect(store.sessionActive).toBe(true)

      // Capture bug
      await store.setBugCapture()
      expect(store.state).toBe('bug')

      // Enter review
      await store.setReview()
      expect(store.state).toBe('review')

      // End session
      await store.setIdle()
      expect(store.state).toBe('idle')
      expect(store.sessionActive).toBe(false)
    })

  })

  describe('setBugCapture with bugId', () => {
    it('should set currentBugId when bugId is provided', async () => {
      const store = useTrayStore()

      await store.setBugCapture('bug-42')

      expect(store.currentBugId).toBe('bug-42')
      expect(store.state).toBe('bug')
    })

    it('should clear currentBugId when transitioning away from bug state', async () => {
      const store = useTrayStore()
      await store.setBugCapture('bug-42')
      expect(store.currentBugId).toBe('bug-42')

      await store.setIdle()
      expect(store.currentBugId).toBeNull()
    })
  })
})
