import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useCounterStore } from '@/stores/counter'

describe('Counter Store', () => {
  beforeEach(() => {
    // creates a fresh pinia and makes it active
    // so it's automatically picked up by any useStore() call
    // without having to pass it to it: `useStore(pinia)`
    setActivePinia(createPinia())
  })

  it('initializes with counter at 0', () => {
    const counter = useCounterStore()
    expect(counter.counter).toBe(0)
  })

  it('increments counter', () => {
    const counter = useCounterStore()
    counter.increment()
    expect(counter.counter).toBe(1)
    counter.increment()
    expect(counter.counter).toBe(2)
  })

  it('decrements counter', () => {
    const counter = useCounterStore()
    counter.counter = 5
    counter.decrement()
    expect(counter.counter).toBe(4)
  })

  it('resets counter', () => {
    const counter = useCounterStore()
    counter.counter = 10
    counter.reset()
    expect(counter.counter).toBe(0)
  })
})
