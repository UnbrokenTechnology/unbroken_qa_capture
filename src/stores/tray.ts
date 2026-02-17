import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// currentBugId tracks the active bug capture ID so the tray menu label can
// display it (e.g. "End Bug Capture Bug-05 (F4)") per PRD Section 13.

export type TrayState = 'idle' | 'active' | 'bug' | 'review'

export interface TrayStatus {
  state: TrayState
  tooltip: string
  sessionActive: boolean
}

export const useTrayStore = defineStore('tray', () => {
  // State
  const state = ref<TrayState>('idle')
  const tooltip = ref<string>('Unbroken QA Capture - Idle')
  const sessionActive = ref<boolean>(false)
  const currentBugId = ref<string | null>(null)

  // Getters
  const currentState = computed(() => state.value)
  const currentTooltip = computed(() => tooltip.value)
  const isSessionActive = computed(() => sessionActive.value)

  // Actions
  async function setState(newState: TrayState, bugId?: string | null) {
    state.value = newState

    // Update tooltip based on state
    const tooltips: Record<TrayState, string> = {
      idle: 'Unbroken QA Capture - Idle',
      active: 'Unbroken QA Capture - Session Active',
      bug: 'Unbroken QA Capture - Capturing Bug',
      review: 'Unbroken QA Capture - Review Mode'
    }

    tooltip.value = tooltips[newState]

    // Rebuild tray context menu for the new state, then update tooltip.
    // update_tray_menu replaces all menu items to match PRD Section 13.
    try {
      await invoke('update_tray_menu', {
        state: newState,
        bugId: bugId ?? null
      })
      await invoke('update_tray_tooltip', { tooltip: tooltip.value })
    } catch (error) {
      console.error('Failed to update tray:', error)
    }
  }

  async function setIdle() {
    sessionActive.value = false
    currentBugId.value = null
    await setState('idle')
  }

  async function setActive() {
    sessionActive.value = true
    currentBugId.value = null
    await setState('active')
  }

  async function setBugCapture(bugId?: string | null) {
    currentBugId.value = bugId ?? null
    await setState('bug', bugId)
  }

  async function setReview() {
    currentBugId.value = null
    await setState('review')
  }

  async function setCustomTooltip(text: string) {
    tooltip.value = text
    try {
      await invoke('update_tray_tooltip', { tooltip: text })
    } catch (error) {
      console.error('Failed to update tray tooltip:', error)
    }
  }

  function toggleSession() {
    if (sessionActive.value) {
      setIdle()
    } else {
      setActive()
    }
  }

  return {
    // State
    state,
    tooltip,
    sessionActive,
    currentBugId,

    // Getters
    currentState,
    currentTooltip,
    isSessionActive,

    // Actions
    setState,
    setIdle,
    setActive,
    setBugCapture,
    setReview,
    setCustomTooltip,
    toggleSession
  }
})
