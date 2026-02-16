import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export interface Environment {
  os: string
  display_resolution: string
  dpi_scaling: string
  ram: string
  cpu: string
  foreground_app: string
}

export interface BugMetadata {
  meeting_id?: string
  software_version?: string
  environment: Environment
  console_captures: string[]
  custom_fields: Record<string, string>
}

export interface Bug {
  id: string
  title: string
  bug_type: string
  description_steps: string
  description_expected: string
  description_actual: string
  metadata: BugMetadata
  folder_path: string
  captures: string[]
  console_output?: string
  created_at?: string
}

export const useBugStore = defineStore('bug', () => {
  // State
  const bugs = ref<Bug[]>([])
  const currentBugId = ref<string | null>(null)

  // Getters
  const allBugs = computed(() => bugs.value)

  const currentBug = computed(() => {
    if (!currentBugId.value) return null
    return bugs.value.find(bug => bug.id === currentBugId.value) || null
  })

  const getBugById = (id: string) => {
    return bugs.value.find(bug => bug.id === id)
  }

  const bugCount = computed(() => bugs.value.length)

  // Actions
  function addBug(bug: Bug) {
    // Check if bug with same ID already exists
    const existingIndex = bugs.value.findIndex(b => b.id === bug.id)
    if (existingIndex >= 0) {
      // Update existing bug
      bugs.value[existingIndex] = bug
    } else {
      // Add new bug
      bugs.value.push(bug)
    }
  }

  function removeBug(id: string) {
    const index = bugs.value.findIndex(bug => bug.id === id)
    if (index >= 0) {
      bugs.value.splice(index, 1)
      if (currentBugId.value === id) {
        currentBugId.value = null
      }
    }
  }

  function updateBug(id: string, updates: Partial<Bug>) {
    const index = bugs.value.findIndex(bug => bug.id === id)
    if (index >= 0) {
      bugs.value[index] = { ...bugs.value[index], ...updates } as Bug
    }
  }

  function setCurrentBug(id: string | null) {
    currentBugId.value = id
  }

  function clearBugs() {
    bugs.value = []
    currentBugId.value = null
  }

  function loadBugs(bugList: Bug[]) {
    bugs.value = bugList
  }

  // For development/testing: Add sample bug data
  function loadSampleData() {
    const sampleBugs: Bug[] = [
      {
        id: '1',
        title: 'Login button not responding',
        bug_type: 'UI',
        description_steps: '1. Open the application\n2. Navigate to login page\n3. Click the login button\n4. Observe that nothing happens',
        description_expected: 'Login button should be clickable and trigger login process',
        description_actual: 'Login button appears disabled and does not respond to clicks',
        metadata: {
          meeting_id: 'MTG-2024-001',
          software_version: '1.2.3',
          environment: {
            os: 'Windows 11 Pro',
            display_resolution: '1920x1080',
            dpi_scaling: '100%',
            ram: '16GB',
            cpu: 'Intel Core i7-10700K',
            foreground_app: 'QA Capture Test App'
          },
          console_captures: [],
          custom_fields: {}
        },
        folder_path: 'C:\\QA\\Session_2024-01-15\\Bug_001',
        captures: [
          'https://via.placeholder.com/800x600/FF6B6B/FFFFFF?text=Login+Screen',
          'https://via.placeholder.com/800x600/4ECDC4/FFFFFF?text=Button+Disabled'
        ],
        console_output: 'Error: Cannot read property "onClick" of undefined\n    at LoginButton.handleClick (login.js:45)\n    at HTMLButtonElement.<anonymous> (dom.js:123)',
        created_at: '2024-01-15T10:30:00Z'
      },
      {
        id: '2',
        title: 'Data not saving in form',
        bug_type: 'Data',
        description_steps: '1. Fill out the user profile form\n2. Enter name, email, and preferences\n3. Click Save button\n4. Navigate away and return\n5. Observe that data was not saved',
        description_expected: 'Form data should be persisted to the database and retained after page navigation',
        description_actual: 'Form appears to save but data is lost after navigating away',
        metadata: {
          software_version: '1.2.3',
          environment: {
            os: 'Windows 11 Pro',
            display_resolution: '2560x1440',
            dpi_scaling: '125%',
            ram: '32GB',
            cpu: 'AMD Ryzen 9 5900X',
            foreground_app: 'QA Capture Test App'
          },
          console_captures: [],
          custom_fields: {
            severity: 'High',
            priority: 'P1'
          }
        },
        folder_path: 'C:\\QA\\Session_2024-01-15\\Bug_002',
        captures: [
          'https://via.placeholder.com/800x600/95E1D3/FFFFFF?text=Form+Filled',
          'https://via.placeholder.com/800x600/F38181/FFFFFF?text=Data+Lost'
        ],
        console_output: 'POST /api/user/profile 500 Internal Server Error\nError: Database connection timeout',
        created_at: '2024-01-15T11:45:00Z'
      }
    ]

    loadBugs(sampleBugs)
  }

  return {
    // State
    bugs: allBugs,
    currentBug,
    bugCount,

    // Actions
    addBug,
    removeBug,
    updateBug,
    setCurrentBug,
    clearBugs,
    loadBugs,
    getBugById,
    loadSampleData
  }
})
