<template>
  <q-page class="flex flex-center">
    <div
      class="q-pa-md"
      style="max-width: 800px; width: 100%;"
    >
      <h1 class="text-h3 q-mb-md text-center">
        Unbroken QA Capture
      </h1>

      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            Bug List
          </div>

          <div
            v-if="bugStore.backendBugs.length === 0"
            class="text-center q-pa-md"
          >
            <p class="text-grey-7">
              No bugs available
            </p>
          </div>

          <q-list
            v-else
            separator
          >
            <q-item
              v-for="bug in bugStore.backendBugs"
              :key="bug.id"
              clickable
              class="q-pa-md"
              @click="viewBug(bug.id)"
            >
              <q-item-section>
                <q-item-label class="text-h6">
                  {{ bug.title ?? bug.display_id }}
                </q-item-label>
                <q-item-label caption>
                  Type: {{ bug.type }}
                </q-item-label>
              </q-item-section>
              <q-item-section side>
                <q-icon name="chevron_right" />
              </q-item-section>
            </q-item>
          </q-list>
        </q-card-section>
      </q-card>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { useBugStore } from '@/stores/bug'
import { useRouter } from 'vue-router'

const bugStore = useBugStore()
const router = useRouter()

function viewBug(id: string) {
  router.push({ name: 'bug-detail', params: { id } })
}
</script>

<style scoped>
.flex-center {
  display: flex;
  justify-content: center;
  align-items: center;
}
</style>
