import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, VueWrapper } from '@vue/test-utils'
import AnnotatorContent from '../../src/components/AnnotatorContent.vue'
import { Canvas } from 'fabric'
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

describe('AnnotatorContent.vue (Standalone Window Mode)', () => {
  let wrapper: VueWrapper

  beforeEach(() => {
    // Set up Pinia for settings store
    setActivePinia(createPinia())

    wrapper = mount(AnnotatorContent, {
      props: {
        screenshotPath: '/path/to/screenshot.png',
      },
      global: {
        stubs: {
          QBar: { template: '<div><slot /></div>' },
          QSpace: { template: '<div />' },
          QBtn: {
            template: '<button @click="$emit(\'click\')"><slot /></button>',
            props: ['color', 'icon', 'label', 'loading', 'disable', 'dense', 'flat', 'round', 'size'],
          },
          QBtnGroup: { template: '<div><slot /></div>' },
          QBtnToggle: {
            template: '<div />',
            props: ['modelValue', 'options', 'color', 'toggleColor'],
          },
          QSeparator: { template: '<div />' },
          QMenu: {
            template: '<div><slot /></div>',
            props: ['modelValue'],
          },
          QColor: {
            template: '<div />',
            props: ['modelValue'],
          },
          QTooltip: { template: '<div><slot /></div>' },
        },
      },
    })
  })

  describe('Component Mounting', () => {
    it('mounts successfully', () => {
      expect(wrapper.exists()).toBe(true)
    })

    it('has a canvas element', () => {
      expect(wrapper.find('canvas').exists()).toBe(true)
    })

    it('initializes canvas on mount', () => {
      expect(Canvas).toHaveBeenCalled()
    })
  })

  describe('Tool Selection', () => {
    it('starts with select tool', () => {
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.currentTool).toBe('select')
    })

    it('can switch to text tool', async () => {
      // @ts-expect-error - Accessing internal method for testing
      wrapper.vm.setTool('text')
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.currentTool).toBe('text')
    })

    it('can switch to rectangle tool', () => {
      // @ts-expect-error - Accessing internal method for testing
      wrapper.vm.setTool('rectangle')
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.currentTool).toBe('rectangle')
    })

    it('can switch to circle tool', () => {
      // @ts-expect-error - Accessing internal method for testing
      wrapper.vm.setTool('circle')
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.currentTool).toBe('circle')
    })

    it('can switch to freehand tool', () => {
      // @ts-expect-error - Accessing internal method for testing
      wrapper.vm.setTool('freehand')
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.currentTool).toBe('freehand')
    })
  })

  describe('Color and Stroke', () => {
    it('starts with PRD default color #FF3B30 (red)', () => {
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.currentColor).toBe('#FF3B30')
    })

    it('starts with PRD default stroke width 4 (medium)', () => {
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.strokeWidth).toBe(4)
    })

    it('has 6 PRD-compliant preset colors', () => {
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.presetColors).toEqual([
        '#FF3B30', // red
        '#FFCC00', // yellow
        '#007AFF', // blue
        '#34C759', // green
        '#FFFFFF', // white
        '#000000', // black
      ])
    })
  })

  describe('Undo/Redo', () => {
    it('initializes history on mount', () => {
      // Canvas saves initial state on mount, so history should have at least one entry
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.history.length).toBeGreaterThan(0)
    })

    it('starts with redo disabled', () => {
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.canRedo).toBe(false)
    })
  })

  describe('Keyboard Shortcuts', () => {
    it('switches to text tool with T key', async () => {
      const event = new KeyboardEvent('keydown', { key: 'T' })
      window.dispatchEvent(event)
      await wrapper.vm.$nextTick()

      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.currentTool).toBe('text')
    })

    it('switches to rectangle tool with R key', async () => {
      const event = new KeyboardEvent('keydown', { key: 'R' })
      window.dispatchEvent(event)
      await wrapper.vm.$nextTick()

      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.currentTool).toBe('rectangle')
    })

    it('closes on Escape key', async () => {
      const event = new KeyboardEvent('keydown', { key: 'Escape' })
      window.dispatchEvent(event)
      await wrapper.vm.$nextTick()

      expect(wrapper.emitted('close')).toBeTruthy()
    })
  })

  describe('Save Functionality', () => {
    it('emits saved event with path', async () => {
      const mockCanvas = {
        toDataURL: vi.fn(() => 'data:image/png;base64,mockdata'),
        getActiveObject: vi.fn(() => null),
      }
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.canvas = mockCanvas as any

      // @ts-expect-error - Accessing internal method for testing
      await wrapper.vm.saveAnnotatedScreenshot()

      expect(wrapper.emitted('saved')).toBeTruthy()
    })
  })

  describe('Resolution Preservation', () => {
    it('tracks original image dimensions', async () => {
      // @ts-expect-error - Accessing internal method for testing
      await wrapper.vm.loadScreenshot()
      await wrapper.vm.$nextTick()

      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.originalImageWidth).toBe(800)
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.originalImageHeight).toBe(600)
    })

    it('exports at original resolution using correct multiplier', async () => {
      const mockCanvas = {
        toDataURL: vi.fn(() => 'data:image/png;base64,mockdata'),
        getActiveObject: vi.fn(() => null),
      }
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.canvas = mockCanvas as any
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.displayScale = 0.5

      // @ts-expect-error - Accessing internal method for testing
      await wrapper.vm.saveAnnotatedScreenshot()

      expect(mockCanvas.toDataURL).toHaveBeenCalledWith({
        format: 'png',
        quality: 1,
        multiplier: 2,
      })
    })
  })
})
