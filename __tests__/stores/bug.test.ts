import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useBugStore, type Bug } from '@/stores/bug'

describe('Bug Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  const createMockBug = (id: string, title: string): Bug => ({
    id,
    title,
    bug_type: 'UI',
    description_steps: 'Test steps',
    description_expected: 'Expected result',
    description_actual: 'Actual result',
    metadata: {
      environment: {
        os: 'Windows 11',
        display_resolution: '1920x1080',
        dpi_scaling: '100%',
        ram: '16GB',
        cpu: 'Intel i7',
        foreground_app: 'TestApp'
      },
      console_captures: [],
      custom_fields: {}
    },
    folder_path: '/test/path',
    captures: []
  })

  it('should initialize with empty bugs array', () => {
    const store = useBugStore()
    expect(store.bugs).toEqual([])
    expect(store.bugCount).toBe(0)
  })

  it('should add a new bug', () => {
    const store = useBugStore()
    const bug = createMockBug('1', 'Test Bug')

    store.addBug(bug)

    expect(store.bugs).toHaveLength(1)
    expect(store.bugs[0]).toEqual(bug)
    expect(store.bugCount).toBe(1)
  })

  it('should update existing bug when adding with same ID', () => {
    const store = useBugStore()
    const bug1 = createMockBug('1', 'Original Title')
    const bug2 = createMockBug('1', 'Updated Title')

    store.addBug(bug1)
    store.addBug(bug2)

    expect(store.bugs).toHaveLength(1)
    expect(store.bugs[0]?.title).toBe('Updated Title')
  })

  it('should get bug by ID', () => {
    const store = useBugStore()
    const bug = createMockBug('1', 'Test Bug')

    store.addBug(bug)

    const foundBug = store.getBugById('1')
    expect(foundBug).toEqual(bug)

    const notFound = store.getBugById('999')
    expect(notFound).toBeUndefined()
  })

  it('should remove a bug', () => {
    const store = useBugStore()
    const bug1 = createMockBug('1', 'Bug 1')
    const bug2 = createMockBug('2', 'Bug 2')

    store.addBug(bug1)
    store.addBug(bug2)

    store.removeBug('1')

    expect(store.bugs).toHaveLength(1)
    expect(store.bugs[0]?.id).toBe('2')
  })

  it('should update a bug', () => {
    const store = useBugStore()
    const bug = createMockBug('1', 'Original Title')

    store.addBug(bug)
    store.updateBug('1', { title: 'Updated Title', bug_type: 'Data' })

    const updated = store.getBugById('1')
    expect(updated?.title).toBe('Updated Title')
    expect(updated?.bug_type).toBe('Data')
  })

  it('should set current bug', () => {
    const store = useBugStore()
    const bug = createMockBug('1', 'Test Bug')

    store.addBug(bug)
    store.setCurrentBug('1')

    expect(store.currentBug).toEqual(bug)
  })

  it('should clear current bug when removed', () => {
    const store = useBugStore()
    const bug = createMockBug('1', 'Test Bug')

    store.addBug(bug)
    store.setCurrentBug('1')
    store.removeBug('1')

    expect(store.currentBug).toBeNull()
  })

  it('should clear all bugs', () => {
    const store = useBugStore()
    const bug1 = createMockBug('1', 'Bug 1')
    const bug2 = createMockBug('2', 'Bug 2')

    store.addBug(bug1)
    store.addBug(bug2)
    store.setCurrentBug('1')

    store.clearBugs()

    expect(store.bugs).toEqual([])
    expect(store.bugCount).toBe(0)
    expect(store.currentBug).toBeNull()
  })

  it('should load bugs from array', () => {
    const store = useBugStore()
    const bugs = [
      createMockBug('1', 'Bug 1'),
      createMockBug('2', 'Bug 2'),
      createMockBug('3', 'Bug 3')
    ]

    store.loadBugs(bugs)

    expect(store.bugs).toEqual(bugs)
    expect(store.bugCount).toBe(3)
  })

  it('should load sample data', () => {
    const store = useBugStore()

    store.loadSampleData()

    expect(store.bugCount).toBeGreaterThan(0)
    expect(store.bugs[0]).toHaveProperty('title')
    expect(store.bugs[0]).toHaveProperty('bug_type')
    expect(store.bugs[0]).toHaveProperty('metadata')
  })
})
