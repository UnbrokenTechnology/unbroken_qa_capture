import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createRouter, createMemoryHistory } from 'vue-router'
import { Quasar } from 'quasar'
import AnnotateView from '@/views/AnnotateView.vue'

// Use vi.hoisted to declare mockClose before vi.mock hoisting
const { mockClose } = vi.hoisted(() => ({
  mockClose: vi.fn().mockResolvedValue(undefined),
}))

// Mock Tauri window
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn().mockReturnValue({
    close: mockClose,
  }),
}))

// Mock Tauri core invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}))

// Mock Tauri event listeners
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))

// Mock the ScreenshotAnnotator component to avoid complex setup
vi.mock('@/components/ScreenshotAnnotator.vue', () => ({
  default: {
    name: 'ScreenshotAnnotator',
    props: ['screenshotPath'],
    emits: ['saved', 'close'],
    template:
      '<div class="screenshot-annotator" data-testid="screenshot-annotator"><slot /></div>',
  },
}))

describe('AnnotateView', () => {
  let pinia: ReturnType<typeof createPinia>
  let router: ReturnType<typeof createRouter>

  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)

    router = createRouter({
      history: createMemoryHistory(),
      routes: [
        {
          path: '/annotate',
          name: 'annotate',
          component: AnnotateView,
        },
      ],
    })

    vi.clearAllMocks()
  })

  function mountAnnotateView(query?: Record<string, string>) {
    return mount(AnnotateView, {
      global: {
        plugins: [pinia, router, Quasar],
        stubs: {
          QCard: { template: '<div class="q-card"><slot /></div>' },
          QCardSection: { template: '<div><slot /></div>' },
          QCardActions: { template: '<div><slot /></div>' },
          QBtn: {
            template:
              '<button class="q-btn" @click="$attrs.onClick && $attrs.onClick()">{{ $attrs.label }}<slot /></button>',
          },
        },
      },
      props: {},
      ...(query ? { attachTo: document.body } : {}),
    })
  }

  it('mounts without errors', async () => {
    const wrapper = mountAnnotateView()
    await flushPromises()
    expect(wrapper.vm).toBeDefined()
  })

  describe('error state (no image path)', () => {
    it('shows error state when no image query param is provided', async () => {
      const wrapper = mountAnnotateView()
      await flushPromises()

      expect(wrapper.text()).toContain('Error')
    })

    it('shows "No image path provided" error message', async () => {
      const wrapper = mountAnnotateView()
      await flushPromises()

      expect(wrapper.text()).toContain('No image path provided')
    })

    it('shows a Close button in the error state', async () => {
      const wrapper = mountAnnotateView()
      await flushPromises()

      const buttons = wrapper.findAll('button.q-btn')
      expect(buttons.length).toBeGreaterThan(0)
      expect(buttons[0]!.text()).toContain('Close')
    })

    it('does not render ScreenshotAnnotator when no image path', async () => {
      const wrapper = mountAnnotateView()
      await flushPromises()

      expect(wrapper.find('[data-testid="screenshot-annotator"]').exists()).toBe(false)
    })

    it('calls window close when Close button is clicked in error state', async () => {
      const wrapper = mountAnnotateView()
      await flushPromises()

      const buttons = wrapper.findAll('button.q-btn')
      await buttons[0]!.trigger('click')
      await flushPromises()

      expect(mockClose).toHaveBeenCalled()
    })
  })

  describe('normal state (with image path)', () => {
    it('renders ScreenshotAnnotator when image path is provided via route', async () => {
      // Navigate to annotate route with an image query param
      await router.push({ path: '/annotate', query: { image: encodeURIComponent('/path/to/image.png') } })
      await router.isReady()

      const wrapper = mountAnnotateView()
      await flushPromises()

      expect(wrapper.find('[data-testid="screenshot-annotator"]').exists()).toBe(true)
    })

    it('passes the decoded image path to ScreenshotAnnotator', async () => {
      const imagePath = '/path/to/screenshot.png'
      await router.push({ path: '/annotate', query: { image: encodeURIComponent(imagePath) } })
      await router.isReady()

      const wrapper = mountAnnotateView()
      await flushPromises()

      const annotator = wrapper.findComponent({ name: 'ScreenshotAnnotator' })
      expect(annotator.exists()).toBe(true)
      expect(annotator.props('screenshotPath')).toBe(imagePath)
    })

    it('does not show the error state when image path is provided', async () => {
      await router.push({ path: '/annotate', query: { image: encodeURIComponent('/img.png') } })
      await router.isReady()

      const wrapper = mountAnnotateView()
      await flushPromises()

      expect(wrapper.text()).not.toContain('No image path provided')
    })
  })

  describe('close handler', () => {
    it('closes the window when ScreenshotAnnotator emits close', async () => {
      await router.push({ path: '/annotate', query: { image: encodeURIComponent('/img.png') } })
      await router.isReady()

      const wrapper = mountAnnotateView()
      await flushPromises()

      const annotator = wrapper.findComponent({ name: 'ScreenshotAnnotator' })
      await annotator.vm.$emit('close')
      await flushPromises()

      expect(mockClose).toHaveBeenCalled()
    })

    it('closes the window when ScreenshotAnnotator emits saved', async () => {
      await router.push({ path: '/annotate', query: { image: encodeURIComponent('/img.png') } })
      await router.isReady()

      const wrapper = mountAnnotateView()
      await flushPromises()

      const annotator = wrapper.findComponent({ name: 'ScreenshotAnnotator' })
      await annotator.vm.$emit('saved', '/path/to/annotated.png')
      await flushPromises()

      expect(mockClose).toHaveBeenCalled()
    })
  })
})
