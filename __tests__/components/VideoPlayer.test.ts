import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { Quasar } from 'quasar'
import VideoPlayer from '@/components/VideoPlayer.vue'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  convertFileSrc: vi.fn((path: string) => `asset://localhost/${path}`),
}))

describe('VideoPlayer', () => {
  const mountComponent = (filePath: string) =>
    mount(VideoPlayer, {
      props: { filePath },
      global: {
        plugins: [Quasar],
        stubs: {
          QIcon: { template: '<span />' }
        }
      }
    })

  it('renders a video element with the correct src converted to asset URL', () => {
    const wrapper = mountComponent('/path/to/video.mp4')
    const video = wrapper.find('video')
    expect(video.exists()).toBe(true)
    expect(video.attributes('src')).toBe('asset://localhost//path/to/video.mp4')
  })

  it('has controls and preload="metadata" on the video element', () => {
    const wrapper = mountComponent('/path/to/video.mp4')
    const video = wrapper.find('video')
    expect(video.attributes('controls')).toBeDefined()
    expect(video.attributes('preload')).toBe('metadata')
  })

  it('does not show error overlay initially', () => {
    const wrapper = mountComponent('/path/to/video.mp4')
    // The error div is only rendered when hasError is true
    expect(wrapper.find('.video-error').exists()).toBe(false)
  })

  it('shows error overlay when video fails to load', async () => {
    const wrapper = mountComponent('/invalid/path/video.mp4')
    const video = wrapper.find('video')
    await video.trigger('error')
    expect(wrapper.find('.video-error').exists()).toBe(true)
  })

  it('emits "loaded" event with duration when metadata loads', async () => {
    const wrapper = mountComponent('/path/to/video.mp4')
    const video = wrapper.find('video')

    // Simulate the video element having a duration
    Object.defineProperty(video.element, 'duration', { value: 125, configurable: true })
    await video.trigger('loadedmetadata')

    expect(wrapper.emitted('loaded')).toBeTruthy()
    expect(wrapper.emitted('loaded')![0]).toEqual([125])
  })

  it('does not show duration badge before metadata loads', () => {
    const wrapper = mountComponent('/path/to/video.mp4')
    expect(wrapper.find('.video-duration-badge').exists()).toBe(false)
  })

  it('shows duration badge after metadata loads', async () => {
    const wrapper = mountComponent('/path/to/video.mp4')
    const video = wrapper.find('video')

    Object.defineProperty(video.element, 'duration', { value: 90, configurable: true })
    await video.trigger('loadedmetadata')

    const badge = wrapper.find('.video-duration-badge')
    expect(badge.exists()).toBe(true)
    expect(badge.text()).toBe('1:30')
  })

  describe('formatDuration', () => {
    it('formats seconds under a minute correctly', async () => {
      const wrapper = mountComponent('/v.mp4')
      const video = wrapper.find('video')
      Object.defineProperty(video.element, 'duration', { value: 45, configurable: true })
      await video.trigger('loadedmetadata')
      expect(wrapper.find('.video-duration-badge').text()).toBe('0:45')
    })

    it('formats minutes and seconds correctly', async () => {
      const wrapper = mountComponent('/v.mp4')
      const video = wrapper.find('video')
      Object.defineProperty(video.element, 'duration', { value: 185, configurable: true })
      await video.trigger('loadedmetadata')
      expect(wrapper.find('.video-duration-badge').text()).toBe('3:05')
    })

    it('formats hours, minutes and seconds correctly', async () => {
      const wrapper = mountComponent('/v.mp4')
      const video = wrapper.find('video')
      Object.defineProperty(video.element, 'duration', { value: 3661, configurable: true })
      await video.trigger('loadedmetadata')
      expect(wrapper.find('.video-duration-badge').text()).toBe('1:01:01')
    })
  })
})
