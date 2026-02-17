import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createRouter, createMemoryHistory } from 'vue-router'
import { Quasar } from 'quasar'
import HelpView from '@/views/HelpView.vue'
import { useSettingsStore } from '@/stores/settings'

// Mock Tauri API so loadAllSettings() doesn't fail in tests
vi.mock('@/api/tauri', () => ({
  getSetting: vi.fn().mockResolvedValue(null),
  setSetting: vi.fn().mockResolvedValue(undefined),
  getAllSettings: vi.fn().mockResolvedValue([]),
  deleteSetting: vi.fn().mockResolvedValue(undefined),
}))

// Stubs that allow slot content to be rendered
const quasarStubs = {
  QPage: { template: '<div><slot /></div>' },
  QCard: { template: '<div><slot /></div>' },
  QCardSection: { template: '<div><slot /></div>' },
  QBtn: { template: '<button class="q-btn" @click="$attrs.onClick && $attrs.onClick()"><slot /></button>' },
  QIcon: { template: '<span />' },
  QTimeline: { template: '<div><slot /></div>' },
  QTimelineEntry: { template: '<div><div>{{ title }}</div><slot /></div>', props: ['title', 'icon', 'color'] },
  QExpansionItem: { template: '<div>{{ label }}<slot /><slot name="default" /></div>', props: ['label', 'icon', 'headerClass', 'defaultOpened'] },
  QList: { template: '<ul><slot /></ul>' },
  QItem: { template: '<li><slot /></li>' },
  QItemSection: { template: '<div><slot /></div>' },
  QItemLabel: { template: '<span><slot /></span>' },
  QMarkupTable: { template: '<table><slot /></table>' },
  QBanner: { template: '<div><slot /><slot name="avatar" /></div>' },
}

describe('HelpView', () => {
  let pinia: ReturnType<typeof createPinia>
  let router: ReturnType<typeof createRouter>

  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)

    router = createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: '/', name: 'home', component: { template: '<div/>' } },
        { path: '/settings', name: 'settings', component: { template: '<div/>' } },
        { path: '/help', name: 'help', component: HelpView },
      ]
    })

    vi.clearAllMocks()
  })

  function mountHelp() {
    return mount(HelpView, {
      global: {
        plugins: [pinia, router, Quasar],
        stubs: quasarStubs,
      }
    })
  }

  it('component is defined and mounts', async () => {
    const wrapper = mountHelp()
    await flushPromises()
    expect(wrapper.vm).toBeDefined()
  })

  it('renders a Back button', async () => {
    const wrapper = mountHelp()
    await flushPromises()
    expect(wrapper.text()).toContain('Back')
  })

  it('renders the help page heading', async () => {
    const wrapper = mountHelp()
    await flushPromises()
    expect(wrapper.text()).toContain('Help / User Guide')
  })

  it('renders workflow section', async () => {
    const wrapper = mountHelp()
    await flushPromises()
    expect(wrapper.text()).toContain('Workflow')
  })

  it('renders hotkey reference section', async () => {
    const wrapper = mountHelp()
    await flushPromises()
    expect(wrapper.text()).toContain('Hotkey Reference')
  })

  it('renders key hotkeys in the reference table with default values from settings store', async () => {
    const wrapper = mountHelp()
    await flushPromises()
    const text = wrapper.text()
    expect(text).toContain('Toggle Session')
    expect(text).toContain('F5')
    expect(text).toContain('F7')
    expect(text).toContain('F9')
    expect(text).toContain('Ctrl+Shift+N')
    expect(text).toContain('Ctrl+Shift+M')
  })

  it('shows updated hotkeys when settings store is changed', async () => {
    const wrapper = mountHelp()
    await flushPromises()

    const store = useSettingsStore()
    store.setSetting('hotkey_toggle_session', 'F2')
    store.setSetting('hotkey_start_bug_capture', 'F3')
    store.setSetting('hotkey_end_bug_capture', 'F4')

    await flushPromises()

    const text = wrapper.text()
    expect(text).toContain('F2')
    expect(text).toContain('F3')
    expect(text).toContain('F4')
    // Old defaults should no longer appear for these keys
    expect(text).not.toContain('F5')
    expect(text).not.toContain('F7')
    expect(text).not.toContain('F9')
  })

  it('renders screenshot association section', async () => {
    const wrapper = mountHelp()
    await flushPromises()
    expect(wrapper.text()).toContain('Associating Screenshots with Bugs')
  })

  it('renders reviewing results section', async () => {
    const wrapper = mountHelp()
    await flushPromises()
    expect(wrapper.text()).toContain('Reviewing Results')
  })

  it('has goBack function that calls router.back', async () => {
    const mockBack = vi.fn()
    const localRouter = createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: '/', component: { template: '<div/>' } },
        { path: '/help', name: 'help', component: HelpView }
      ]
    })
    vi.spyOn(localRouter, 'back').mockImplementation(mockBack)

    const wrapper = mount(HelpView, {
      global: {
        plugins: [pinia, localRouter, Quasar],
        stubs: quasarStubs,
      }
    })

    await flushPromises()

    const vm = wrapper.vm as any
    expect(typeof vm.goBack).toBe('function')
    vm.goBack()
    expect(mockBack).toHaveBeenCalled()
  })

  it('navigates back when Escape key is pressed', async () => {
    const mockBack = vi.fn()
    const localRouter = createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: '/', component: { template: '<div/>' } },
        { path: '/help', name: 'help', component: HelpView }
      ]
    })
    vi.spyOn(localRouter, 'back').mockImplementation(mockBack)

    mount(HelpView, {
      global: {
        plugins: [pinia, localRouter, Quasar],
        stubs: quasarStubs,
      }
    })

    await flushPromises()

    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))

    expect(mockBack).toHaveBeenCalled()
  })

  it('has goToSettings function', async () => {
    const wrapper = mountHelp()
    await flushPromises()
    const vm = wrapper.vm as any
    expect(typeof vm.goToSettings).toBe('function')
  })
})
