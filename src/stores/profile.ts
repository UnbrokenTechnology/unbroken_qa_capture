import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import * as tauri from '@/api/tauri'
import type { QaProfile } from '@/types/backend'

export const useProfileStore = defineStore('profile', () => {
  // ============================================================================
  // State
  // ============================================================================

  const profiles = ref<QaProfile[]>([])
  const activeProfileId = ref<string | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // ============================================================================
  // Getters
  // ============================================================================

  const activeProfile = computed(() =>
    profiles.value.find(p => p.id === activeProfileId.value) ?? null
  )

  const hasError = computed(() => error.value !== null)

  // ============================================================================
  // Actions
  // ============================================================================

  async function loadProfiles(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      profiles.value = await tauri.listProfiles()
      activeProfileId.value = await tauri.getActiveProfileId()
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function setActiveProfile(id: string): Promise<void> {
    loading.value = true
    error.value = null
    try {
      await tauri.setActiveProfileId(id)
      activeProfileId.value = id
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function clearActiveProfile(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      // There's no dedicated "clear" command, but setting active profile id
      // to an empty string effectively deactivates the profile. We just
      // update the local state since a null activeProfileId means no profile.
      activeProfileId.value = null
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function createProfile(profile: QaProfile): Promise<void> {
    loading.value = true
    error.value = null
    try {
      await tauri.createProfile(profile)
      await loadProfiles()
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function updateProfile(profile: QaProfile): Promise<void> {
    loading.value = true
    error.value = null
    try {
      await tauri.updateProfile(profile)
      await loadProfiles()
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function deleteProfile(id: string): Promise<void> {
    loading.value = true
    error.value = null
    try {
      await tauri.deleteProfile(id)
      if (activeProfileId.value === id) {
        activeProfileId.value = null
      }
      await loadProfiles()
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  function clearError(): void {
    error.value = null
  }

  // ============================================================================
  // Store Return
  // ============================================================================

  return {
    // State
    profiles,
    activeProfileId,
    loading,
    error,

    // Getters
    activeProfile,
    hasError,

    // Actions
    loadProfiles,
    setActiveProfile,
    clearActiveProfile,
    createProfile,
    updateProfile,
    deleteProfile,
    clearError,
  }
})
