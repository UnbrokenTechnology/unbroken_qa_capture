<template>
  <q-layout view="lHh Lpr lFf">
    <q-header
      elevated
      class="bg-primary text-white"
    >
      <q-toolbar>
        <q-toolbar-title>
          Unbroken QA Capture
        </q-toolbar-title>
      </q-toolbar>
    </q-header>

    <q-page-container>
      <q-page class="flex flex-center">
        <div class="q-pa-md text-center">
          <h1 class="text-h3 q-mb-md">
            Welcome to Unbroken QA Capture
          </h1>

          <div class="q-gutter-md">
            <q-input
              v-model="name"
              filled
              label="Enter your name"
              style="max-width: 300px"
            />

            <q-btn
              color="primary"
              label="Greet from Rust"
              :loading="loading"
              @click="handleGreet"
            />

            <div
              v-if="greetMsg"
              class="q-mt-md"
            >
              <q-card class="bg-green-1">
                <q-card-section>
                  <div class="text-h6">
                    {{ greetMsg }}
                  </div>
                </q-card-section>
              </q-card>
            </div>
          </div>

          <div class="q-mt-xl text-caption">
            Counter: {{ counter }}
            <q-btn
              size="sm"
              color="secondary"
              label="Increment"
              class="q-ml-sm"
              @click="incrementCounter"
            />
          </div>
        </div>
      </q-page>
    </q-page-container>
  </q-layout>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useCounterStore } from '@/stores/counter'
import { storeToRefs } from 'pinia'

const name = ref('')
const greetMsg = ref('')
const loading = ref(false)

const counterStore = useCounterStore()
const { counter } = storeToRefs(counterStore)
const { increment: incrementCounter } = counterStore

async function handleGreet() {
  if (!name.value) return

  loading.value = true
  try {
    greetMsg.value = await invoke<string>('greet', { name: name.value })
  } catch (error) {
    console.error('Error calling Rust command:', error)
    greetMsg.value = 'Error calling Rust command'
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.flex-center {
  display: flex;
  justify-content: center;
  align-items: center;
}
</style>
