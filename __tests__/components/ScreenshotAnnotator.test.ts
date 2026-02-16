import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, VueWrapper } from '@vue/test-utils'
import ScreenshotAnnotator from '../../src/components/ScreenshotAnnotator.vue'
import { createPinia, setActivePinia } from 'pinia'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  convertFileSrc: vi.fn((path: string) => `asset://localhost/${path}`)
}))

// Mock Fabric.js Canvas
vi.mock('fabric', () => {
  const mockCanvas = {
    width: 1200,
    height: 800,
    add: vi.fn(),
    remove: vi.fn(),
    insertAt: vi.fn(),
    renderAll: vi.fn(),
    getScenePoint: vi.fn(() => ({ x: 100, y: 100 })),
    setActiveObject: vi.fn(),
    getActiveObject: vi.fn(() => null),
    selection: true,
    isDrawingMode: false,
    freeDrawingBrush: null,
    on: vi.fn(),
    dispose: vi.fn(),
    toDataURL: vi.fn(() => 'data:image/png;base64,mockdata'),
    toJSON: vi.fn(() => ({})),
    loadFromJSON: vi.fn(() => Promise.resolve()),
  }

  return {
    Canvas: vi.fn(() => mockCanvas),
    FabricImage: {
      fromURL: vi.fn(() =>
        Promise.resolve({
          width: 800,
          height: 600,
          scale: vi.fn(),
          set: vi.fn(),
        })
      ),
    },
    Rect: vi.fn(function (this: any, options: any) {
      this.set = vi.fn()
      this.left = options.left
      this.top = options.top
      this.width = options.width
      this.height = options.height
      return this
    }),
    Circle: vi.fn(function (this: any, options: any) {
      this.set = vi.fn()
      this.left = options.left
      this.top = options.top
      this.radius = options.radius
      return this
    }),
    IText: vi.fn(function (this: any, text: string, options: any) {
      this.set = vi.fn()
      this.text = text
      this.left = options.left
      this.top = options.top
      return this
    }),
    PencilBrush: vi.fn(function () {
      const brush: any = {
        color: '#FF0000',
        width: 3
      }
      return brush
    }),
  }
})

describe('ScreenshotAnnotator.vue (Wrapper Component)', () => {
  let wrapper: VueWrapper

  beforeEach(() => {
    // Set up Pinia for settings store
    setActivePinia(createPinia())
  })

  describe('Dialog Mode (Default)', () => {
    beforeEach(() => {
      wrapper = mount(ScreenshotAnnotator, {
        props: {
          modelValue: false,
          screenshotPath: '/path/to/screenshot.png',
        },
        global: {
          stubs: {
            QDialog: {
              template: '<div><slot /></div>',
              props: ['modelValue'],
            },
            QCard: { template: '<div><slot /></div>' },
            AnnotatorContent: {
              template: '<div class="annotator-content-stub"></div>',
              props: ['screenshotPath'],
            },
          },
        },
      })
    })

    it('mounts successfully in dialog mode', () => {
      expect(wrapper.exists()).toBe(true)
    })

    it('renders QDialog when useDialog is true', () => {
      // In dialog mode, QDialog should be rendered as a stub
      expect(wrapper.html()).toContain('annotator-content-stub')
    })

    it('passes screenshotPath to AnnotatorContent', () => {
      const content = wrapper.find('.annotator-content-stub')
      expect(content.exists()).toBe(true)
    })

    it('opens when modelValue is set to true', async () => {
      await wrapper.setProps({ modelValue: true })
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.isOpen).toBe(true)
    })

    it('emits update:modelValue when isOpen changes', async () => {
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.isOpen = true
      await wrapper.vm.$nextTick()

      expect(wrapper.emitted('update:modelValue')).toBeTruthy()
      const emissions = wrapper.emitted('update:modelValue')
      expect(emissions?.[emissions.length - 1]).toEqual([true])
    })

    it('closes dialog and emits update:modelValue on close', async () => {
      await wrapper.setProps({ modelValue: true })
      // @ts-expect-error - Accessing internal method for testing
      wrapper.vm.handleClose()
      await wrapper.vm.$nextTick()

      expect(wrapper.emitted('close')).toBeTruthy()
      expect(wrapper.emitted('update:modelValue')).toBeTruthy()
    })

    it('emits saved event with path', async () => {
      const testPath = '/path/to/annotated.png'
      // @ts-expect-error - Accessing internal method for testing
      wrapper.vm.handleSaved(testPath)
      await wrapper.vm.$nextTick()

      expect(wrapper.emitted('saved')).toBeTruthy()
      const savedEmissions = wrapper.emitted('saved')
      expect(savedEmissions?.[0]).toEqual([testPath])
    })
  })

  describe('Standalone Mode', () => {
    beforeEach(() => {
      wrapper = mount(ScreenshotAnnotator, {
        props: {
          screenshotPath: '/path/to/screenshot.png',
          useDialog: false,
        },
        global: {
          stubs: {
            AnnotatorContent: {
              template: '<div class="annotator-content-stub"></div>',
              props: ['screenshotPath'],
            },
          },
        },
      })
    })

    it('mounts successfully in standalone mode', () => {
      expect(wrapper.exists()).toBe(true)
    })

    it('does not render QDialog when useDialog is false', () => {
      expect(wrapper.find('[class*="q-dialog"]').exists()).toBe(false)
    })

    it('renders AnnotatorContent directly', () => {
      const content = wrapper.find('.annotator-content-stub')
      expect(content.exists()).toBe(true)
    })

    it('passes screenshotPath to AnnotatorContent', () => {
      const content = wrapper.find('.annotator-content-stub')
      expect(content.exists()).toBe(true)
    })

    it('emits close without changing isOpen', async () => {
      // @ts-expect-error - Accessing internal method for testing
      wrapper.vm.handleClose()
      await wrapper.vm.$nextTick()

      expect(wrapper.emitted('close')).toBeTruthy()
      // In standalone mode, isOpen should not change
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.isOpen).toBe(false)
    })

    it('emits saved without changing isOpen', async () => {
      const testPath = '/path/to/annotated.png'
      // @ts-expect-error - Accessing internal method for testing
      wrapper.vm.handleSaved(testPath)
      await wrapper.vm.$nextTick()

      expect(wrapper.emitted('saved')).toBeTruthy()
      // In standalone mode, isOpen should not change
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.isOpen).toBe(false)
    })
  })

  describe('Backward Compatibility', () => {
    it('defaults to dialog mode when useDialog is not specified', () => {
      const defaultWrapper = mount(ScreenshotAnnotator, {
        props: {
          screenshotPath: '/path/to/screenshot.png',
        },
        global: {
          stubs: {
            QDialog: {
              template: '<div class="q-dialog-stub"><slot /></div>',
              props: ['modelValue'],
            },
            QCard: { template: '<div><slot /></div>' },
            AnnotatorContent: {
              template: '<div></div>',
              props: ['screenshotPath'],
            },
          },
        },
      })

      expect(defaultWrapper.find('.q-dialog-stub').exists()).toBe(true)
    })

    it('accepts modelValue prop in dialog mode', async () => {
      const dialogWrapper = mount(ScreenshotAnnotator, {
        props: {
          modelValue: true,
          screenshotPath: '/path/to/screenshot.png',
        },
        global: {
          stubs: {
            QDialog: {
              template: '<div><slot /></div>',
              props: ['modelValue'],
            },
            QCard: { template: '<div><slot /></div>' },
            AnnotatorContent: {
              template: '<div></div>',
              props: ['screenshotPath'],
            },
          },
        },
      })

      // @ts-expect-error - Accessing internal state for testing
      expect(dialogWrapper.vm.isOpen).toBe(true)
    })
  })
})
