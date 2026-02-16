import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

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

  // Getters
  const currentState = computed(() => state.value)
  const currentTooltip = computed(() => tooltip.value)
  const isSessionActive = computed(() => sessionActive.value)

  // Actions
  async function setState(newState: TrayState) {
    state.value = newState

    // Update tooltip based on state
    const tooltips: Record<TrayState, string> = {
      idle: 'Unbroken QA Capture - Idle',
      active: 'Unbroken QA Capture - Session Active',
      bug: 'Unbroken QA Capture - Capturing Bug',
      review: 'Unbroken QA Capture - Review Mode'
    }

    tooltip.value = tooltips[newState]

    // Update the tray icon via Tauri backend
    try {
      await invoke('update_tray_icon', { state: newState })
      await invoke('update_tray_tooltip', { tooltip: tooltip.value })
    } catch (error) {
      console.error('Failed to update tray:', error)
    }
  }

  async function setIdle() {
    sessionActive.value = false
    await setState('idle')
  }

  async function setActive() {
    sessionActive.value = true
    await setState('active')
  }

  async function setBugCapture() {
    await setState('bug')
  }

  async function setReview() {
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
