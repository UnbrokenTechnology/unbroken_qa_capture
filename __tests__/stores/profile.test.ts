import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useProfileStore } from '@/stores/profile'
import type { QaProfile } from '@/types/backend'

// Mock Tauri API
vi.mock('@/api/tauri', () => ({
  listProfiles: vi.fn(),
  getProfile: vi.fn(),
  createProfile: vi.fn(),
  updateProfile: vi.fn(),
  deleteProfile: vi.fn(),
  getActiveProfileId: vi.fn(),
  setActiveProfileId: vi.fn(),
}))

import * as tauri from '@/api/tauri'

const makeProfile = (id: string, name: string): QaProfile => ({
  id,
  name,
  linear_config: null,
  area_categories: [],
  custom_fields: [],
  title_conventions: null,
  created_at: '2024-01-01T10:00:00Z',
  updated_at: '2024-01-01T10:00:00Z',
})

describe('Profile Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('initializes with empty state', () => {
    const store = useProfileStore()
    expect(store.profiles).toEqual([])
    expect(store.activeProfileId).toBeNull()
    expect(store.activeProfile).toBeNull()
    expect(store.loading).toBe(false)
    expect(store.error).toBeNull()
    expect(store.hasError).toBe(false)
  })

  describe('loadProfiles', () => {
    it('loads profiles and active profile ID from backend', async () => {
      const store = useProfileStore()
      const profiles = [makeProfile('p-1', 'Project A'), makeProfile('p-2', 'Project B')]
      vi.mocked(tauri.listProfiles).mockResolvedValue(profiles)
      vi.mocked(tauri.getActiveProfileId).mockResolvedValue('p-1')

      await store.loadProfiles()

      expect(store.profiles).toEqual(profiles)
      expect(store.activeProfileId).toBe('p-1')
      expect(store.loading).toBe(false)
    })

    it('sets activeProfile computed from activeProfileId', async () => {
      const store = useProfileStore()
      const profiles = [makeProfile('p-1', 'Project A')]
      vi.mocked(tauri.listProfiles).mockResolvedValue(profiles)
      vi.mocked(tauri.getActiveProfileId).mockResolvedValue('p-1')

      await store.loadProfiles()

      expect(store.activeProfile).toEqual(profiles[0])
    })

    it('returns null activeProfile when no active profile ID', async () => {
      const store = useProfileStore()
      vi.mocked(tauri.listProfiles).mockResolvedValue([makeProfile('p-1', 'A')])
      vi.mocked(tauri.getActiveProfileId).mockResolvedValue(null)

      await store.loadProfiles()

      expect(store.activeProfile).toBeNull()
    })

    it('handles errors and sets error state', async () => {
      const store = useProfileStore()
      vi.mocked(tauri.listProfiles).mockRejectedValue(new Error('DB failure'))
      vi.mocked(tauri.getActiveProfileId).mockResolvedValue(null)

      await expect(store.loadProfiles()).rejects.toThrow('DB failure')
      expect(store.error).toBe('DB failure')
      expect(store.loading).toBe(false)
    })
  })

  describe('setActiveProfile', () => {
    it('calls backend and updates local activeProfileId', async () => {
      const store = useProfileStore()
      vi.mocked(tauri.setActiveProfileId).mockResolvedValue()

      await store.setActiveProfile('p-1')

      expect(tauri.setActiveProfileId).toHaveBeenCalledWith('p-1')
      expect(store.activeProfileId).toBe('p-1')
    })

    it('handles errors', async () => {
      const store = useProfileStore()
      vi.mocked(tauri.setActiveProfileId).mockRejectedValue(new Error('Not found'))

      await expect(store.setActiveProfile('p-bad')).rejects.toThrow('Not found')
      expect(store.error).toBe('Not found')
    })
  })

  describe('clearActiveProfile', () => {
    it('clears activeProfileId locally', async () => {
      const store = useProfileStore()
      store.activeProfileId = 'p-1'

      await store.clearActiveProfile()

      expect(store.activeProfileId).toBeNull()
    })
  })

  describe('createProfile', () => {
    it('creates profile and reloads list', async () => {
      const store = useProfileStore()
      const newProfile = makeProfile('p-new', 'New Project')
      vi.mocked(tauri.createProfile).mockResolvedValue()
      vi.mocked(tauri.listProfiles).mockResolvedValue([newProfile])
      vi.mocked(tauri.getActiveProfileId).mockResolvedValue(null)

      await store.createProfile(newProfile)

      expect(tauri.createProfile).toHaveBeenCalledWith(newProfile)
      expect(store.profiles).toEqual([newProfile])
    })

    it('handles errors', async () => {
      const store = useProfileStore()
      vi.mocked(tauri.createProfile).mockRejectedValue(new Error('Duplicate name'))

      await expect(store.createProfile(makeProfile('x', 'X'))).rejects.toThrow('Duplicate name')
      expect(store.error).toBe('Duplicate name')
    })
  })

  describe('updateProfile', () => {
    it('updates profile and reloads list', async () => {
      const store = useProfileStore()
      const original = makeProfile('p-1', 'Old Name')
      const updated = { ...original, name: 'New Name' }
      store.profiles = [original]
      vi.mocked(tauri.updateProfile).mockResolvedValue()
      vi.mocked(tauri.listProfiles).mockResolvedValue([updated])
      vi.mocked(tauri.getActiveProfileId).mockResolvedValue(null)

      await store.updateProfile(updated)

      expect(tauri.updateProfile).toHaveBeenCalledWith(updated)
      expect(store.profiles[0]?.name).toBe('New Name')
    })

    it('handles errors', async () => {
      const store = useProfileStore()
      vi.mocked(tauri.updateProfile).mockRejectedValue(new Error('Profile not found'))

      await expect(store.updateProfile(makeProfile('missing', 'X'))).rejects.toThrow('Profile not found')
      expect(store.error).toBe('Profile not found')
    })
  })

  describe('deleteProfile', () => {
    it('deletes profile and reloads list', async () => {
      const store = useProfileStore()
      const profile = makeProfile('p-1', 'To Delete')
      store.profiles = [profile]
      vi.mocked(tauri.deleteProfile).mockResolvedValue()
      vi.mocked(tauri.listProfiles).mockResolvedValue([])
      vi.mocked(tauri.getActiveProfileId).mockResolvedValue(null)

      await store.deleteProfile('p-1')

      expect(tauri.deleteProfile).toHaveBeenCalledWith('p-1')
      expect(store.profiles).toEqual([])
    })

    it('clears activeProfileId when deleting the active profile', async () => {
      const store = useProfileStore()
      store.activeProfileId = 'p-1'
      vi.mocked(tauri.deleteProfile).mockResolvedValue()
      vi.mocked(tauri.listProfiles).mockResolvedValue([])
      vi.mocked(tauri.getActiveProfileId).mockResolvedValue(null)

      await store.deleteProfile('p-1')

      expect(store.activeProfileId).toBeNull()
    })

    it('does not clear activeProfileId when deleting a non-active profile', async () => {
      const store = useProfileStore()
      store.activeProfileId = 'p-1'
      vi.mocked(tauri.deleteProfile).mockResolvedValue()
      vi.mocked(tauri.listProfiles).mockResolvedValue([makeProfile('p-1', 'Active')])
      vi.mocked(tauri.getActiveProfileId).mockResolvedValue('p-1')

      await store.deleteProfile('p-2')

      expect(store.activeProfileId).toBe('p-1')
    })

    it('handles errors', async () => {
      const store = useProfileStore()
      vi.mocked(tauri.deleteProfile).mockRejectedValue(new Error('Cannot delete'))

      await expect(store.deleteProfile('p-bad')).rejects.toThrow('Cannot delete')
      expect(store.error).toBe('Cannot delete')
    })
  })

  describe('clearError', () => {
    it('clears the error state', () => {
      const store = useProfileStore()
      store.error = 'Test error'

      store.clearError()

      expect(store.error).toBeNull()
      expect(store.hasError).toBe(false)
    })
  })

  describe('activeProfile computed', () => {
    it('returns null when profiles list is empty', () => {
      const store = useProfileStore()
      store.activeProfileId = 'p-1'
      expect(store.activeProfile).toBeNull()
    })

    it('returns the profile matching activeProfileId', () => {
      const store = useProfileStore()
      const p1 = makeProfile('p-1', 'Alpha')
      const p2 = makeProfile('p-2', 'Beta')
      store.profiles = [p1, p2]
      store.activeProfileId = 'p-2'
      expect(store.activeProfile).toEqual(p2)
    })

    it('returns null when activeProfileId does not match any profile', () => {
      const store = useProfileStore()
      store.profiles = [makeProfile('p-1', 'Alpha')]
      store.activeProfileId = 'p-999'
      expect(store.activeProfile).toBeNull()
    })
  })
})
