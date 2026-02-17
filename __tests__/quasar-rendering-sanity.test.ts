import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { defineComponent } from 'vue'
import { readFileSync } from 'fs'
import { resolve } from 'path'
import { Quasar, QBtn, QIcon, QCard, QCardSection, QBadge, QChip } from 'quasar'

// Mock Tauri APIs (included for safety even though we only test Quasar primitives)
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => ({
    setAlwaysOnTop: vi.fn().mockResolvedValue(undefined),
  })),
}))
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))
vi.mock('@/api/tauri', () => ({
  createSession: vi.fn(),
  getSession: vi.fn(),
  updateSession: vi.fn(),
  deleteSession: vi.fn(),
  listSessions: vi.fn(),
  getActiveSession: vi.fn(),
  getSessionSummaries: vi.fn(),
  updateSessionStatus: vi.fn(),
  createBug: vi.fn(),
  getBug: vi.fn(),
  updateBug: vi.fn(),
  deleteBug: vi.fn(),
  listBugs: vi.fn(),
  getBugsBySession: vi.fn(),
  getBugNotes: vi.fn(),
  updateBugNotes: vi.fn(),
  updateTrayIcon: vi.fn(),
  updateTrayTooltip: vi.fn(),
}))

describe('Quasar Rendering Sanity', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  describe('Quasar Plugin & Icon Set Verification', () => {
    it('Quasar, Notify, and Dialog are defined objects', async () => {
      const { Notify, Dialog } = await import('quasar')
      expect(Quasar).toBeDefined()
      expect(typeof Quasar).toBe('object')
      expect(Notify).toBeDefined()
      expect(typeof Notify).toBe('object')
      expect(Dialog).toBeDefined()
      expect(typeof Dialog).toBe('object')
    })

    it('material-icons icon set has name and type properties', async () => {
      const iconSetModule = await import('quasar/icon-set/material-icons')
      const iconSet = iconSetModule.default
      expect(iconSet).toBeDefined()
      expect(iconSet).toHaveProperty('name')
      expect(iconSet).toHaveProperty('type')
    })

    it('main.ts imports material-icons icon set', () => {
      const mainTs = readFileSync(resolve(__dirname, '../src/main.ts'), 'utf-8')
      expect(mainTs).toContain("import iconSet from 'quasar/icon-set/material-icons'")
    })

    it('main.ts configures Notify and Dialog plugins', () => {
      const mainTs = readFileSync(resolve(__dirname, '../src/main.ts'), 'utf-8')
      expect(mainTs).toMatch(/plugins:\s*\{\s*Notify,\s*Dialog\s*\}/)
    })

    it('main.ts passes iconSet to Quasar config', () => {
      const mainTs = readFileSync(resolve(__dirname, '../src/main.ts'), 'utf-8')
      expect(mainTs).toContain('iconSet')
      // Verify iconSet appears inside the app.use(Quasar, { ... }) block
      expect(mainTs).toMatch(/app\.use\(Quasar,\s*\{[\s\S]*iconSet/)
    })
  })

  describe('Individual Component Rendering', () => {
    it('QBtn renders a <button> element and does not leak tag name as text', () => {
      const wrapper = mount(QBtn, {
        props: { label: 'Click me' },
        global: { plugins: [Quasar] },
      })
      expect(wrapper.find('button').exists()).toBe(true)
      expect(wrapper.text()).not.toContain('q-btn')
    })

    it('QIcon renders with .q-icon class when given a name prop', () => {
      const wrapper = mount(QIcon, {
        props: { name: 'home' },
        global: { plugins: [Quasar] },
      })
      expect(wrapper.find('.q-icon').exists()).toBe(true)
    })

    it('QCard renders with .q-card class', () => {
      const wrapper = mount(QCard, {
        global: { plugins: [Quasar] },
      })
      expect(wrapper.find('.q-card').exists()).toBe(true)
    })

    it('QBadge renders with .q-badge class', () => {
      const wrapper = mount(QBadge, {
        props: { label: '5' },
        global: { plugins: [Quasar] },
      })
      expect(wrapper.find('.q-badge').exists()).toBe(true)
    })

    it('QChip renders with .q-chip class', () => {
      const wrapper = mount(QChip, {
        global: { plugins: [Quasar] },
        slots: { default: 'Test Chip' },
      })
      expect(wrapper.find('.q-chip').exists()).toBe(true)
    })
  })

  describe('Mixed Component Tree', () => {
    const TestComponent = defineComponent({
      template: `
        <div>
          <q-btn label="Test" />
          <q-icon name="home" />
          <q-card>
            <q-card-section>Content</q-card-section>
          </q-card>
        </div>
      `,
    })

    const mountMixed = () =>
      mount(TestComponent, {
        global: {
          plugins: [Quasar],
          components: { QBtn, QIcon, QCard, QCardSection },
        },
      })

    it('renders a <button> from q-btn in the mixed tree', () => {
      const wrapper = mountMixed()
      expect(wrapper.find('button').exists()).toBe(true)
    })

    it('renders .q-icon from q-icon in the mixed tree', () => {
      const wrapper = mountMixed()
      expect(wrapper.find('.q-icon').exists()).toBe(true)
    })

    it('renders .q-card from q-card in the mixed tree', () => {
      const wrapper = mountMixed()
      expect(wrapper.find('.q-card').exists()).toBe(true)
    })

    it('does not leak unresolved Quasar tag names as raw text', () => {
      const wrapper = mountMixed()
      const text = wrapper.text()
      expect(text).not.toMatch(/\bq-btn\b/)
      expect(text).not.toMatch(/\bq-icon\b/)
      expect(text).not.toMatch(/\bq-card\b/)
    })
  })
})
