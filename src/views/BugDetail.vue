<template>
  <q-page padding>
    <div
      v-if="!bug"
      class="flex flex-center q-pa-xl"
    >
      <div class="text-h6 text-grey">
        Bug not found
      </div>
    </div>

    <div
      v-else
      class="bug-detail"
    >
      <!-- Header with back button and title -->
      <div class="row items-center q-mb-md">
        <q-btn
          flat
          round
          dense
          icon="arrow_back"
          class="q-mr-md"
          @click="goBack"
        />
        <div class="text-h4 flex-1">
          {{ bug.title }}
        </div>
        <q-btn
          color="primary"
          icon="content_copy"
          label="Copy to Clipboard"
          :loading="copying"
          @click="copyToClipboard"
        />
      </div>

      <!-- Bug Metadata Card -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            Bug Information
          </div>
          <div class="row q-col-gutter-md">
            <div class="col-12 col-md-6">
              <div class="text-caption text-grey-7">
                Type
              </div>
              <div class="text-body1">
                {{ bug.bug_type }}
              </div>
            </div>
            <div class="col-12 col-md-6">
              <div class="text-caption text-grey-7">
                Folder
              </div>
              <div class="text-body1">
                {{ bug.folder_path }}
              </div>
            </div>
            <div
              v-if="bug.metadata.software_version"
              class="col-12 col-md-6"
            >
              <div class="text-caption text-grey-7">
                Software Version
              </div>
              <div class="text-body1">
                {{ bug.metadata.software_version }}
              </div>
            </div>
            <div
              v-if="bug.metadata.meeting_id"
              class="col-12 col-md-6"
            >
              <div class="text-caption text-grey-7">
                Meeting ID
              </div>
              <div class="text-body1">
                {{ bug.metadata.meeting_id }}
              </div>
            </div>
          </div>
        </q-card-section>
      </q-card>

      <!-- Description Card -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            Description
          </div>

          <div class="q-mb-md">
            <div class="text-subtitle2 text-primary q-mb-sm">
              Steps to Reproduce
            </div>
            <div class="text-body2 whitespace-pre-wrap">
              {{ bug.description_steps }}
            </div>
          </div>

          <div class="q-mb-md">
            <div class="text-subtitle2 text-positive q-mb-sm">
              Expected Result
            </div>
            <div class="text-body2 whitespace-pre-wrap">
              {{ bug.description_expected }}
            </div>
          </div>

          <div>
            <div class="text-subtitle2 text-negative q-mb-sm">
              Actual Result
            </div>
            <div class="text-body2 whitespace-pre-wrap">
              {{ bug.description_actual }}
            </div>
          </div>
        </q-card-section>
      </q-card>

      <!-- Screenshots Carousel Card -->
      <q-card
        v-if="bug.captures.length > 0"
        class="q-mb-md"
      >
        <q-card-section>
          <div class="text-h6 q-mb-md">
            Screenshots ({{ bug.captures.length }})
          </div>
          <q-carousel
            v-model="currentSlide"
            swipeable
            animated
            navigation
            infinite
            arrows
            control-color="primary"
            height="500px"
            class="bg-grey-2 rounded-borders"
          >
            <q-carousel-slide
              v-for="(capture, index) in bug.captures"
              :key="index"
              :name="index"
              class="q-pa-none"
            >
              <div class="full-width full-height flex flex-center">
                <q-img
                  :src="capture"
                  :alt="`Screenshot ${index + 1}`"
                  fit="contain"
                  class="full-width full-height"
                  :ratio="16/9"
                >
                  <template #error>
                    <div class="absolute-full flex flex-center bg-grey-3">
                      <div class="text-center">
                        <q-icon
                          name="broken_image"
                          size="64px"
                          color="grey-6"
                        />
                        <div class="text-grey-7 q-mt-sm">
                          Failed to load image
                        </div>
                      </div>
                    </div>
                  </template>
                  <template #loading>
                    <div class="absolute-full flex flex-center">
                      <q-spinner
                        color="primary"
                        size="50px"
                      />
                    </div>
                  </template>
                </q-img>
              </div>
            </q-carousel-slide>
          </q-carousel>
        </q-card-section>
      </q-card>

      <!-- Console Output Card -->
      <q-card
        v-if="bug.console_output"
        class="q-mb-md"
      >
        <q-card-section>
          <div class="text-h6 q-mb-md">
            Console Output
          </div>
          <q-scroll-area
            style="height: 300px"
            class="bg-grey-10 text-white rounded-borders q-pa-md"
          >
            <pre class="text-body2 q-ma-none">{{ bug.console_output }}</pre>
          </q-scroll-area>
        </q-card-section>
      </q-card>

      <!-- Environment Information Card -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            Environment
          </div>
          <div class="row q-col-gutter-md">
            <div class="col-12 col-md-6">
              <div class="text-caption text-grey-7">
                Operating System
              </div>
              <div class="text-body1">
                {{ bug.metadata.environment.os }}
              </div>
            </div>
            <div class="col-12 col-md-6">
              <div class="text-caption text-grey-7">
                Display Resolution
              </div>
              <div class="text-body1">
                {{ bug.metadata.environment.display_resolution }}
              </div>
            </div>
            <div class="col-12 col-md-6">
              <div class="text-caption text-grey-7">
                DPI Scaling
              </div>
              <div class="text-body1">
                {{ bug.metadata.environment.dpi_scaling }}
              </div>
            </div>
            <div class="col-12 col-md-6">
              <div class="text-caption text-grey-7">
                RAM
              </div>
              <div class="text-body1">
                {{ bug.metadata.environment.ram }}
              </div>
            </div>
            <div class="col-12 col-md-6">
              <div class="text-caption text-grey-7">
                CPU
              </div>
              <div class="text-body1">
                {{ bug.metadata.environment.cpu }}
              </div>
            </div>
            <div class="col-12 col-md-6">
              <div class="text-caption text-grey-7">
                Foreground App
              </div>
              <div class="text-body1">
                {{ bug.metadata.environment.foreground_app }}
              </div>
            </div>
          </div>
        </q-card-section>
      </q-card>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useBugStore } from '@/stores/bug'
import { invoke } from '@tauri-apps/api/core'
import { useQuasar } from 'quasar'

const route = useRoute()
const router = useRouter()
const bugStore = useBugStore()
const $q = useQuasar()

const currentSlide = ref(0)
const copying = ref(false)

// Get bug ID from route params
const bugId = computed(() => route.params.id as string)

// Get the bug data from the store
const bug = computed(() => bugStore.getBugById(bugId.value))

// Navigate back to bug list
function goBack() {
  router.back()
}

// Copy bug to clipboard
async function copyToClipboard() {
  if (!bug.value) return

  copying.value = true
  try {
    await invoke('copy_bug_to_clipboard', {
      folderPath: bug.value.folder_path
    })

    $q.notify({
      type: 'positive',
      message: 'Bug report copied to clipboard',
      position: 'top',
      timeout: 2000
    })
  } catch (error) {
    $q.notify({
      type: 'negative',
      message: `Failed to copy bug report: ${error}`,
      position: 'top',
      timeout: 3000
    })
  } finally {
    copying.value = false
  }
}

// Load bug data on mount if needed
onMounted(() => {
  // If bug store is empty, you might want to fetch data here
  // For now, we assume the store is populated elsewhere
})
</script>

<style scoped>
.bug-detail {
  max-width: 1200px;
  margin: 0 auto;
}

.whitespace-pre-wrap {
  white-space: pre-wrap;
}

.flex-center {
  display: flex;
  justify-content: center;
  align-items: center;
}

.flex-1 {
  flex: 1;
}
</style>
