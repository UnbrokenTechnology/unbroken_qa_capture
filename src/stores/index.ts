/**
 * Store initialization module
 *
 * This module provides a centralized way to initialize all Pinia stores
 * and set up their event listeners when the app starts.
 */

import { useSessionStore } from './session'
import { useBugStore } from './bug'
import { useSettingsStore } from './settings'

/**
 * Initialize all stores
 *
 * Call this function once when the app starts (in main.ts or App.vue)
 * to set up event listeners and load initial data.
 */
export async function initializeStores(): Promise<void> {
  try {
    // Initialize settings first (needed by other stores)
    const settingsStore = useSettingsStore()
    settingsStore.initialize()

    // Set up event listeners for session store
    const sessionStore = useSessionStore()
    await sessionStore.setupEventListeners()

    // Set up event listeners for bug store
    const bugStore = useBugStore()
    await bugStore.setupEventListeners()

    // Load initial data
    await Promise.all([
      sessionStore.loadActiveSession().catch(err => {
        console.warn('Failed to load active session:', err)
      }),
      sessionStore.loadSessionSummaries().catch(err => {
        console.warn('Failed to load session summaries:', err)
      }),
    ])

    console.log('Stores initialized successfully')
  } catch (error) {
    console.error('Failed to initialize stores:', error)
    throw error
  }
}

/**
 * Cleanup all stores
 *
 * Call this function when the app is closing to properly
 * clean up event listeners and save any pending state.
 */
export function cleanupStores(): void {
  try {
    const sessionStore = useSessionStore()
    sessionStore.cleanupEventListeners()

    const bugStore = useBugStore()
    bugStore.cleanupEventListeners()

    console.log('Stores cleaned up successfully')
  } catch (error) {
    console.error('Failed to cleanup stores:', error)
  }
}

// Re-export all stores for convenience
export { useSessionStore } from './session'
export { useBugStore } from './bug'
export { useSettingsStore } from './settings'
export { useTrayStore } from './tray'

// Re-export types
export type { Bug, BugMetadata, Environment } from './bug'
export type { TrayState, TrayStatus } from './tray'
export type { SettingsKey } from './settings'
export { SETTINGS_KEYS } from './settings'
