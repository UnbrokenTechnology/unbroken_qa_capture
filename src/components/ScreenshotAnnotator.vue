<template>
  <q-dialog
    v-if="useDialog"
    v-model="isOpen"
    maximized
    transition-show="slide-up"
    transition-hide="slide-down"
  >
    <q-card class="bg-grey-9 text-white">
      <AnnotatorContent
        :screenshot-path="screenshotPath"
        :capture-id="captureId"
        @saved="handleSaved"
        @close="handleClose"
      />
    </q-card>
  </q-dialog>
  <AnnotatorContent
    v-else
    :screenshot-path="screenshotPath"
    :capture-id="captureId"
    @saved="handleSaved"
    @close="handleClose"
  />
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import AnnotatorContent from './AnnotatorContent.vue'

interface Props {
  modelValue?: boolean
  screenshotPath: string
  captureId?: string
  useDialog?: boolean  // New prop to control dialog vs standalone mode
}

type Emits = {
  'update:modelValue': [value: boolean]
  saved: [annotatedPath: string]
  close: []
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: false,
  useDialog: true
})

const emit = defineEmits<Emits>()

const isOpen = ref(props.modelValue)

// Watch for model value changes
watch(() => props.modelValue, (newVal) => {
  isOpen.value = newVal
})

watch(isOpen, (newVal) => {
  emit('update:modelValue', newVal)
})

function handleSaved(annotatedPath: string) {
  emit('saved', annotatedPath)
  if (props.useDialog) {
    isOpen.value = false
  }
}

function handleClose() {
  emit('close')
  if (props.useDialog) {
    isOpen.value = false
  }
}
</script>
