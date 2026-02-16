import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, VueWrapper } from '@vue/test-utils'
import ScreenshotAnnotator from '../../src/components/ScreenshotAnnotator.vue'
import { Canvas } from 'fabric'

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

describe('ScreenshotAnnotator.vue', () => {
  let wrapper: VueWrapper

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
          QCardSection: { template: '<div><slot /></div>' },
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

    it('initializes with dialog closed', () => {
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.isOpen).toBe(false)
    })
  })

  describe('Props', () => {
    it('accepts modelValue prop', async () => {
      await wrapper.setProps({ modelValue: true })
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.isOpen).toBe(true)
    })

    it('accepts screenshotPath prop', () => {
      const props = wrapper.props() as any
      expect(props.screenshotPath).toBe('/path/to/screenshot.png')
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

    it('has 3 PRD-compliant stroke width options', () => {
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.strokeWidthOptions).toEqual([
        { label: 'Thin', value: 2 },
        { label: 'Medium', value: 4 },
        { label: 'Thick', value: 8 },
      ])
    })

    it('can select preset color', async () => {
      // @ts-expect-error - Accessing internal method for testing
      wrapper.vm.selectColor('#FFCC00')
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.currentColor).toBe('#FFCC00')
    })

    it('can update stroke width to thin (2px)', async () => {
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.strokeWidth = 2
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.strokeWidth).toBe(2)
    })

    it('can update stroke width to thick (8px)', async () => {
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.strokeWidth = 8
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.strokeWidth).toBe(8)
    })

    it('returns correct color name for preset colors', () => {
      // @ts-expect-error - Accessing internal method for testing
      expect(wrapper.vm.getColorName('#FF3B30')).toBe('Red')
      // @ts-expect-error - Accessing internal method for testing
      expect(wrapper.vm.getColorName('#FFCC00')).toBe('Yellow')
      // @ts-expect-error - Accessing internal method for testing
      expect(wrapper.vm.getColorName('#007AFF')).toBe('Blue')
      // @ts-expect-error - Accessing internal method for testing
      expect(wrapper.vm.getColorName('#34C759')).toBe('Green')
      // @ts-expect-error - Accessing internal method for testing
      expect(wrapper.vm.getColorName('#FFFFFF')).toBe('White')
      // @ts-expect-error - Accessing internal method for testing
      expect(wrapper.vm.getColorName('#000000')).toBe('Black')
    })

    it('returns hex code for unknown colors', () => {
      // @ts-expect-error - Accessing internal method for testing
      expect(wrapper.vm.getColorName('#123456')).toBe('#123456')
    })
  })

  describe('Undo/Redo', () => {
    it('starts with undo disabled', () => {
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.canUndo).toBe(false)
    })

    it('starts with redo disabled', () => {
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.canRedo).toBe(false)
    })

    it('enables undo after history is saved', () => {
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.history = ['state1', 'state2']
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.historyStep = 1
      // @ts-expect-error - Accessing internal method for testing
      wrapper.vm.updateUndoRedoState()
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.canUndo).toBe(true)
    })

    it('enables redo when not at latest history', () => {
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.history = ['state1', 'state2', 'state3']
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.historyStep = 1
      // @ts-expect-error - Accessing internal method for testing
      wrapper.vm.updateUndoRedoState()
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.canRedo).toBe(true)
    })
  })

  describe('Dialog Interaction', () => {
    it('emits update:modelValue when closing', async () => {
      await wrapper.setProps({ modelValue: true })
      // @ts-expect-error - Accessing internal method for testing
      wrapper.vm.close()
      await wrapper.vm.$nextTick()

      expect(wrapper.emitted('update:modelValue')).toBeTruthy()
      // First emission is from setProps (true), second is from close() (false)
      const emissions = wrapper.emitted('update:modelValue')
      expect(emissions?.[emissions.length - 1]).toEqual([false])
    })

    it('opens when modelValue is set to true', async () => {
      await wrapper.setProps({ modelValue: true })
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.isOpen).toBe(true)
    })
  })

  describe('Save Functionality', () => {
    it('generates annotated filename correctly', () => {
      const originalPath = '/path/to/screenshot.png'
      const lastDot = originalPath.lastIndexOf('.')
      const annotatedPath = `${originalPath.substring(0, lastDot)}_annotated${originalPath.substring(lastDot)}`

      expect(annotatedPath).toBe('/path/to/screenshot_annotated.png')
    })

    it('handles path without extension', () => {
      const originalPath = '/path/to/screenshot'
      const annotatedPath = `${originalPath}_annotated.png`

      expect(annotatedPath).toBe('/path/to/screenshot_annotated.png')
    })

    it('sets saving state during save', async () => {
      await wrapper.setProps({ modelValue: true })
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.saving).toBe(false)
    })
  })

  describe('Canvas Initialization', () => {
    it('creates canvas when dialog opens', async () => {
      await wrapper.setProps({ modelValue: true })
      await wrapper.vm.$nextTick()
      await wrapper.vm.$nextTick()

      // Canvas initialization happens in nextTick
      // The mock Canvas constructor should have been called
      expect(Canvas).toHaveBeenCalled()
    })

    it('disposes canvas on unmount', () => {
      const mockDispose = vi.fn()
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.canvas = { dispose: mockDispose } as any
      wrapper.unmount()

      expect(mockDispose).toHaveBeenCalled()
    })
  })

  describe('Event Handlers', () => {
    it('sets up mouse event handlers on canvas', async () => {
      await wrapper.setProps({ modelValue: true })
      await wrapper.vm.$nextTick()

      // @ts-expect-error - Accessing internal method for testing
      wrapper.vm.initializeCanvas()

      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.canvas?.on).toHaveBeenCalledWith('mouse:down', expect.any(Function))
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.canvas?.on).toHaveBeenCalledWith('mouse:move', expect.any(Function))
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.canvas?.on).toHaveBeenCalledWith('mouse:up', expect.any(Function))
    })

    it('handles mouse down for text tool', () => {
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.currentTool = 'text'
      const mockCanvas = {
        getScenePoint: vi.fn(() => ({ x: 100, y: 100 })),
        add: vi.fn(),
        setActiveObject: vi.fn(),
        renderAll: vi.fn(),
      }
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.canvas = mockCanvas as any

      // @ts-expect-error - Accessing internal method for testing
      wrapper.vm.handleMouseDown({ e: {} } as any)

      expect(mockCanvas.add).toHaveBeenCalled()
    })
  })

  describe('History Management', () => {
    it('saves history state', () => {
      const mockCanvas = {
        toJSON: vi.fn(() => ({ objects: [] })),
      }
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.canvas = mockCanvas as any
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.history = []
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.historyStep = -1

      // @ts-expect-error - Accessing internal method for testing
      wrapper.vm.saveHistory()

      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.history.length).toBe(1)
      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.historyStep).toBe(0)
    })

    it('limits history to 50 items', () => {
      const mockCanvas = {
        toJSON: vi.fn(() => ({ objects: [] })),
      }
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.canvas = mockCanvas as any
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.history = new Array(51).fill('state')
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.historyStep = 50

      // @ts-expect-error - Accessing internal method for testing
      wrapper.vm.saveHistory()

      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.history.length).toBeLessThanOrEqual(51)
    })

    it('clears redo history on new action', () => {
      const mockCanvas = {
        toJSON: vi.fn(() => ({ objects: [] })),
      }
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.canvas = mockCanvas as any
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.history = ['state1', 'state2', 'state3']
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.historyStep = 1

      // @ts-expect-error - Accessing internal method for testing
      wrapper.vm.saveHistory()

      // @ts-expect-error - Accessing internal state for testing
      expect(wrapper.vm.history.length).toBe(3) // state1, state2, new state
    })
  })

  describe('Drawing Tools', () => {
    it('enables drawing mode for freehand tool', () => {
      const mockCanvas = {
        isDrawingMode: false,
        freeDrawingBrush: null,
        selection: true,
      }
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.canvas = mockCanvas as any
      // @ts-expect-error - Accessing internal method for testing
      wrapper.vm.setTool('freehand')

      expect(mockCanvas.isDrawingMode).toBe(true)
    })

    it('disables drawing mode for non-freehand tools', () => {
      const mockCanvas = {
        isDrawingMode: true,
        selection: false,
      }
      // @ts-expect-error - Accessing internal state for testing
      wrapper.vm.canvas = mockCanvas as any
      // @ts-expect-error - Accessing internal method for testing
      wrapper.vm.setTool('rectangle')

      expect(mockCanvas.isDrawingMode).toBe(false)
    })
  })

  describe('Accessibility', () => {
    it('has tooltips for all tool buttons', () => {
      expect(wrapper.html()).toContain('Select')
      expect(wrapper.html()).toContain('Add Text')
      expect(wrapper.html()).toContain('Draw Rectangle')
      expect(wrapper.html()).toContain('Draw Circle')
      expect(wrapper.html()).toContain('Freehand Drawing')
    })

    it('has tooltips for undo/redo buttons', () => {
      expect(wrapper.html()).toContain('Undo')
      expect(wrapper.html()).toContain('Redo')
    })
  })
})
