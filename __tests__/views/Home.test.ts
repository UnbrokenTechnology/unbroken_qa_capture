import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createRouter, createMemoryHistory } from 'vue-router'
import { Quasar } from 'quasar'
import Home from '@/views/Home.vue'

// Mock Tauri API
vi.mock('@/api/tauri', () => ({
  createBug: vi.fn(),
  getBug: vi.fn(),
  updateBug: vi.fn(),
  deleteBug: vi.fn(),
  listBugs: vi.fn().mockResolvedValue([]),
  getBugsBySession: vi.fn().mockResolvedValue([]),
}))

// Mock Tauri event listeners
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))

// Mock Tauri core invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}))

describe('Home View', () => {
  let pinia: ReturnType<typeof createPinia>
  let router: ReturnType<typeof createRouter>

  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)

    router = createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: '/', name: 'home', component: Home },
        { path: '/bug/:id', name: 'bug-detail', component: { template: '<div>bug detail</div>' } },
      ],
    })

    vi.clearAllMocks()
  })

  function mountHome() {
    return mount(Home, {
      global: {
        plugins: [pinia, router, Quasar],
        stubs: {
          QPage: { template: '<div><slot /></div>' },
          QCard: { template: '<div class="q-card"><slot /></div>' },
          QCardSection: { template: '<div><slot /></div>' },
          QBtn: { template: '<button class="q-btn" @click="$attrs.onClick && $attrs.onClick()">{{ $attrs.label }}<slot /></button>' },
          QIcon: { template: '<span />' },
          QList: { template: '<ul><slot /></ul>' },
          QItem: { template: '<li @click="$attrs.onClick && $attrs.onClick()"><slot /></li>' },
          QItemSection: { template: '<div><slot /></div>' },
          QItemLabel: { template: '<span><slot /></span>' },
        },
      },
    })
  }

  it('mounts without errors', async () => {
    const wrapper = mountHome()
    await flushPromises()
    expect(wrapper.vm).toBeDefined()
  })

  it('renders the app title', async () => {
    const wrapper = mountHome()
    await flushPromises()
    expect(wrapper.text()).toContain('Unbroken QA Capture')
  })

  it('renders the Bug List heading', async () => {
    const wrapper = mountHome()
    await flushPromises()
    expect(wrapper.text()).toContain('Bug List')
  })

  describe('empty state', () => {
    it('shows "No bugs available" when bug store is empty', async () => {
      const wrapper = mountHome()
      await flushPromises()
      expect(wrapper.text()).toContain('No bugs available')
    })

    it('shows "Load Sample Data" button when no bugs exist', async () => {
      const wrapper = mountHome()
      await flushPromises()
      expect(wrapper.text()).toContain('Load Sample Data')
    })

    it('does not show the bug list when empty', async () => {
      const wrapper = mountHome()
      await flushPromises()
      expect(wrapper.find('ul').exists()).toBe(false)
    })
  })

  describe('bug list', () => {
    it('shows bug list when bugs are available', async () => {
      const { useBugStore } = await import('@/stores/bug')
      const bugStore = useBugStore()
      bugStore.loadSampleData()

      const wrapper = mountHome()
      await flushPromises()

      // Sample data has 2 bugs, the list should appear
      expect(wrapper.find('ul').exists()).toBe(true)
    })

    it('hides empty state when bugs are present', async () => {
      const { useBugStore } = await import('@/stores/bug')
      const bugStore = useBugStore()
      bugStore.loadSampleData()

      const wrapper = mountHome()
      await flushPromises()

      expect(wrapper.text()).not.toContain('No bugs available')
    })

    it('renders bug titles in the list', async () => {
      const { useBugStore } = await import('@/stores/bug')
      const bugStore = useBugStore()
      bugStore.loadSampleData()

      const wrapper = mountHome()
      await flushPromises()

      expect(wrapper.text()).toContain('Login button not responding')
      expect(wrapper.text()).toContain('Data not saving in form')
    })

    it('renders bug type in the list', async () => {
      const { useBugStore } = await import('@/stores/bug')
      const bugStore = useBugStore()
      bugStore.loadSampleData()

      const wrapper = mountHome()
      await flushPromises()

      expect(wrapper.text()).toContain('UI')
    })
  })

  describe('interactions', () => {
    it('calls loadSampleData when Load Sample Data button is clicked', async () => {
      const { useBugStore } = await import('@/stores/bug')
      const bugStore = useBugStore()
      const loadSpy = vi.spyOn(bugStore, 'loadSampleData')

      const wrapper = mountHome()
      await flushPromises()

      const buttons = wrapper.findAll('button.q-btn')
      expect(buttons.length).toBeGreaterThan(0)
      await buttons[0]!.trigger('click')

      expect(loadSpy).toHaveBeenCalled()
    })

    it('navigates to bug-detail when a bug item is clicked', async () => {
      const { useBugStore } = await import('@/stores/bug')
      const bugStore = useBugStore()
      bugStore.loadSampleData()

      const wrapper = mountHome()
      await flushPromises()

      const pushSpy = vi.spyOn(router, 'push')

      const items = wrapper.findAll('li')
      expect(items.length).toBeGreaterThan(0)
      await items[0]!.trigger('click')

      expect(pushSpy).toHaveBeenCalledWith(
        expect.objectContaining({ name: 'bug-detail' })
      )
    })
  })
})
