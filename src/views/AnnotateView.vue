<template>
  <div class="annotate-view">
    <ScreenshotAnnotator
      v-if="imagePath"
      :screenshot-path="imagePath"
      :capture-id="captureId"
      @saved="handleSaved"
      @close="handleClose"
    />
    <div
      v-else
      class="error-state"
    >
      <q-card>
        <q-card-section>
          <div class="text-h6">
            Error
          </div>
          <div class="text-body2">
            No image path provided
          </div>
        </q-card-section>
        <q-card-actions>
          <q-btn
            color="primary"
            label="Close"
            @click="handleClose"
          />
        </q-card-actions>
      </q-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { getCurrentWindow } from '@tauri-apps/api/window'
import ScreenshotAnnotator from '../components/ScreenshotAnnotator.vue'

const route = useRoute()
const imagePath = ref<string>('')
const captureId = ref<string | undefined>(undefined)

onMounted(() => {
  // Extract image path and optional capture ID from query parameters
  const imageParam = route.query.image
  if (typeof imageParam === 'string') {
    imagePath.value = decodeURIComponent(imageParam)
  }
  const captureIdParam = route.query.captureId
  if (typeof captureIdParam === 'string') {
    captureId.value = decodeURIComponent(captureIdParam)
  }
})

async function handleSaved(annotatedPath: string) {
  console.log('Annotation saved to:', annotatedPath)
  await handleClose()
}

async function handleClose() {
  const appWindow = getCurrentWindow()
  await appWindow.close()
}
</script>

<style scoped>
.annotate-view {
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  background-color: #1e1e1e;
}

.error-state {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
}
</style>
