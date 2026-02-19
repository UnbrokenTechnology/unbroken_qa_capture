<template>
  <div class="video-player-container">
    <video
      ref="videoEl"
      :src="filePath"
      controls
      preload="metadata"
      class="video-player"
      @loadedmetadata="onLoadedMetadata"
      @error="onError"
    />
    <div
      v-if="hasError"
      class="absolute-full flex flex-center bg-grey-3 video-error"
    >
      <div class="text-center">
        <q-icon
          name="videocam_off"
          size="48px"
          color="grey-6"
        />
        <div class="text-grey-7 q-mt-sm text-caption">
          Failed to load video
        </div>
      </div>
    </div>
    <div
      v-if="duration !== null"
      class="video-duration-badge"
    >
      {{ formatDuration(duration) }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

defineProps<{
  filePath: string
}>()

const emit = defineEmits<{
  (e: 'loaded', duration: number): void
}>()

const videoEl = ref<HTMLVideoElement | null>(null)
const hasError = ref(false)
const duration = ref<number | null>(null)

function onLoadedMetadata() {
  if (videoEl.value && isFinite(videoEl.value.duration)) {
    duration.value = videoEl.value.duration
    emit('loaded', duration.value)
  }
}

function onError() {
  hasError.value = true
}

function formatDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = Math.floor(seconds % 60)
  if (h > 0) {
    return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
  }
  return `${m}:${String(s).padStart(2, '0')}`
}
</script>

<style scoped>
.video-player-container {
  position: relative;
  width: 100%;
  background: #000;
  border-radius: 4px;
  overflow: hidden;
}

.video-player {
  width: 100%;
  display: block;
  max-height: 480px;
}

.video-error {
  min-height: 160px;
}

.video-duration-badge {
  position: absolute;
  bottom: 8px;
  right: 8px;
  background: rgba(0, 0, 0, 0.7);
  color: #fff;
  font-size: 12px;
  padding: 2px 6px;
  border-radius: 3px;
  pointer-events: none;
}
</style>
