import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createRouter, createMemoryHistory } from 'vue-router'
import { Quasar } from 'quasar'
import BugDetail from '@/views/BugDetail.vue'
import { useBugStore, type Bug } from '@/stores/bug'

const createMockBug = (id: string): Bug => ({
  id,
  title: 'Test Bug Title',
  bug_type: 'UI',
  description_steps: 'Step 1\nStep 2\nStep 3',
  description_expected: 'Expected behavior',
  description_actual: 'Actual behavior',
  metadata: {
    meeting_id: 'MTG-123',
    software_version: '1.0.0',
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
  captures: ['image1.png', 'image2.png'],
  console_output: 'Error: Test error message',
  created_at: '2024-01-15T10:30:00Z'
})

describe('BugDetail', () => {
  let router: ReturnType<typeof createRouter>
  let pinia: ReturnType<typeof createPinia>

  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)

    router = createRouter({
      history: createMemoryHistory(),
      routes: [
        {
          path: '/bug/:id',
          name: 'bug-detail',
          component: BugDetail
        }
      ]
    })
  })

  const mountComponent = async (bugId: string) => {
    router.push(`/bug/${bugId}`)
    await router.isReady()

    return mount(BugDetail, {
      global: {
        plugins: [pinia, router, Quasar],
        stubs: {
          QPage: { template: '<div><slot /></div>' },
          QCard: { template: '<div><slot /></div>' },
          QCardSection: { template: '<div><slot /></div>' },
          QBtn: { template: '<button @click="$attrs.onClick"><slot /></button>' },
          QCarousel: { template: '<div><slot /></div>' },
          QCarouselSlide: { template: '<div><slot /></div>' },
          QImg: { template: '<img />' },
          QScrollArea: { template: '<div><slot /></div>' }
        }
      }
    })
  }

  it('should display bug not found when bug does not exist', async () => {
    const wrapper = await mountComponent('999')

    expect(wrapper.text()).toContain('Bug not found')
  })

  it('should display bug details when bug exists', async () => {
    const store = useBugStore()
    const bug = createMockBug('1')
    store.addBug(bug)

    const wrapper = await mountComponent('1')

    expect(wrapper.text()).toContain('Test Bug Title')
    expect(wrapper.text()).toContain('UI')
    expect(wrapper.text()).toContain('Step 1')
    expect(wrapper.text()).toContain('Expected behavior')
    expect(wrapper.text()).toContain('Actual behavior')
  })

  it('should display metadata information', async () => {
    const store = useBugStore()
    const bug = createMockBug('1')
    store.addBug(bug)

    const wrapper = await mountComponent('1')

    expect(wrapper.text()).toContain('1.0.0')
    expect(wrapper.text()).toContain('MTG-123')
    expect(wrapper.text()).toContain('/test/path')
  })

  it('should display environment information', async () => {
    const store = useBugStore()
    const bug = createMockBug('1')
    store.addBug(bug)

    const wrapper = await mountComponent('1')

    expect(wrapper.text()).toContain('Windows 11')
    expect(wrapper.text()).toContain('1920x1080')
    expect(wrapper.text()).toContain('100%')
    expect(wrapper.text()).toContain('16GB')
    expect(wrapper.text()).toContain('Intel i7')
    expect(wrapper.text()).toContain('TestApp')
  })

  it('should display console output when available', async () => {
    const store = useBugStore()
    const bug = createMockBug('1')
    store.addBug(bug)

    const wrapper = await mountComponent('1')

    expect(wrapper.text()).toContain('Console Output')
    expect(wrapper.text()).toContain('Error: Test error message')
  })

  it('should not display console output card when not available', async () => {
    const store = useBugStore()
    const bug = createMockBug('1')
    bug.console_output = undefined
    store.addBug(bug)

    const wrapper = await mountComponent('1')

    expect(wrapper.text()).not.toContain('Console Output')
  })

  it('should display screenshot count', async () => {
    const store = useBugStore()
    const bug = createMockBug('1')
    store.addBug(bug)

    const wrapper = await mountComponent('1')

    expect(wrapper.text()).toContain('Screenshots (2)')
  })

  it('should not display screenshots section when no captures', async () => {
    const store = useBugStore()
    const bug = createMockBug('1')
    bug.captures = []
    store.addBug(bug)

    const wrapper = await mountComponent('1')

    expect(wrapper.text()).not.toContain('Screenshots')
  })

  it('should navigate back when back button is clicked', async () => {
    const store = useBugStore()
    const bug = createMockBug('1')
    store.addBug(bug)

    const wrapper = await mountComponent('1')
    const backSpy = vi.spyOn(router, 'back')

    const backButton = wrapper.find('button')
    await backButton.trigger('click')

    expect(backSpy).toHaveBeenCalled()
  })

  it('should not display optional metadata fields when not provided', async () => {
    const store = useBugStore()
    const bug = createMockBug('1')
    bug.metadata.meeting_id = undefined
    bug.metadata.software_version = undefined
    store.addBug(bug)

    const wrapper = await mountComponent('1')

    expect(wrapper.text()).not.toContain('Meeting ID')
    expect(wrapper.text()).not.toContain('Software Version')
  })
})
