<template>
  <q-dialog
    v-model="isOpen"
    maximized
    transition-show="slide-up"
    transition-hide="slide-down"
  >
    <q-card class="bg-grey-9 text-white">
      <q-bar class="bg-primary">
        <div class="text-h6">
          Annotate Screenshot
        </div>
        <q-space />
        <q-btn
          dense
          flat
          icon="close"
          @click="close"
        >
          <q-tooltip>Close without saving</q-tooltip>
        </q-btn>
      </q-bar>

      <q-card-section class="q-pa-md">
        <!-- Toolbar -->
        <div class="row q-gutter-sm q-mb-md items-center">
          <!-- Tool Selection -->
          <q-btn-group>
            <q-btn
              :color="currentTool === 'select' ? 'primary' : 'grey-7'"
              icon="near_me"
              @click="setTool('select')"
            >
              <q-tooltip>Select</q-tooltip>
            </q-btn>
            <q-btn
              :color="currentTool === 'text' ? 'primary' : 'grey-7'"
              icon="text_fields"
              @click="setTool('text')"
            >
              <q-tooltip>Add Text</q-tooltip>
            </q-btn>
            <q-btn
              :color="currentTool === 'rectangle' ? 'primary' : 'grey-7'"
              icon="crop_square"
              @click="setTool('rectangle')"
            >
              <q-tooltip>Draw Rectangle</q-tooltip>
            </q-btn>
            <q-btn
              :color="currentTool === 'circle' ? 'primary' : 'grey-7'"
              icon="panorama_fish_eye"
              @click="setTool('circle')"
            >
              <q-tooltip>Draw Circle</q-tooltip>
            </q-btn>
            <q-btn
              :color="currentTool === 'freehand' ? 'primary' : 'grey-7'"
              icon="brush"
              @click="setTool('freehand')"
            >
              <q-tooltip>Freehand Drawing</q-tooltip>
            </q-btn>
          </q-btn-group>

          <q-separator
            vertical
            inset
          />

          <!-- Color Picker -->
          <div class="row items-center q-gutter-xs">
            <span class="text-caption">Color:</span>
            <q-btn
              round
              size="sm"
              :style="{ backgroundColor: currentColor }"
              @click="showColorPicker = true"
            >
              <q-menu v-model="showColorPicker">
                <q-color
                  v-model="currentColor"
                  @change="updateColor"
                />
              </q-menu>
            </q-btn>
          </div>

          <q-separator
            vertical
            inset
          />

          <!-- Stroke Width -->
          <div class="row items-center q-gutter-xs">
            <span class="text-caption">Width:</span>
            <q-slider
              v-model="strokeWidth"
              :min="1"
              :max="20"
              :step="1"
              style="width: 100px"
              color="primary"
              @update:model-value="updateStrokeWidth"
            />
            <span class="text-caption">{{ strokeWidth }}px</span>
          </div>

          <q-separator
            vertical
            inset
          />

          <!-- Undo/Redo -->
          <q-btn-group>
            <q-btn
              :disable="!canUndo"
              color="grey-7"
              icon="undo"
              @click="undo"
            >
              <q-tooltip>Undo</q-tooltip>
            </q-btn>
            <q-btn
              :disable="!canRedo"
              color="grey-7"
              icon="redo"
              @click="redo"
            >
              <q-tooltip>Redo</q-tooltip>
            </q-btn>
          </q-btn-group>

          <q-space />

          <!-- Save Button -->
          <q-btn
            color="positive"
            icon="save"
            label="Save Annotated Screenshot"
            :loading="saving"
            @click="saveAnnotatedScreenshot"
          />
        </div>

        <!-- Canvas Container -->
        <div class="canvas-container bg-grey-8 rounded-borders">
          <canvas ref="canvasElement" />
        </div>
      </q-card-section>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch, nextTick } from 'vue'
import { Canvas, FabricImage, Rect, Circle, IText, PencilBrush } from 'fabric'

interface Props {
  modelValue: boolean
  screenshotPath: string
}

type Emits = {
  'update:modelValue': [value: boolean]
  saved: [annotatedPath: string]
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const isOpen = ref(props.modelValue)
const canvasElement = ref<HTMLCanvasElement | null>(null)
const canvas = ref<Canvas | null>(null)
const currentTool = ref<'select' | 'text' | 'rectangle' | 'circle' | 'freehand'>('select')
const currentColor = ref('#FF0000')
const strokeWidth = ref(3)
const showColorPicker = ref(false)
const saving = ref(false)

// Undo/Redo state
const history = ref<string[]>([])
const historyStep = ref(0)
const canUndo = ref(false)
const canRedo = ref(false)

// Drawing state
let isDrawing = false
let drawingObject: Rect | Circle | null = null
let startX = 0
let startY = 0

// Watch for model value changes
watch(() => props.modelValue, (newVal) => {
  isOpen.value = newVal
  if (newVal && !canvas.value) {
    nextTick(() => {
      initializeCanvas()
    })
  }
})

watch(isOpen, (newVal) => {
  emit('update:modelValue', newVal)
})

onMounted(() => {
  if (props.modelValue) {
    initializeCanvas()
  }
})

onBeforeUnmount(() => {
  if (canvas.value) {
    canvas.value.dispose()
  }
})

function initializeCanvas() {
  if (!canvasElement.value) return

  // Initialize Fabric canvas
  canvas.value = new Canvas(canvasElement.value, {
    width: 1200,
    height: 800,
    backgroundColor: '#ffffff',
  })

  // Load the screenshot image
  loadScreenshot()

  // Set up event handlers
  setupEventHandlers()

  // Save initial state
  saveHistory()
}

async function loadScreenshot() {
  if (!canvas.value || !props.screenshotPath) return

  try {
    const img = await FabricImage.fromURL(props.screenshotPath)

    // Scale image to fit canvas
    const canvasWidth = canvas.value.width || 1200
    const canvasHeight = canvas.value.height || 800

    const scale = Math.min(
      canvasWidth / (img.width || 1),
      canvasHeight / (img.height || 1)
    )

    img.scale(scale)
    img.set({
      selectable: false,
      evented: false,
    })

    canvas.value.add(img)
    // Send image to back
    canvas.value.remove(img)
    canvas.value.insertAt(0, img)
    canvas.value.renderAll()

    saveHistory()
  } catch (error) {
    console.error('Failed to load screenshot:', error)
  }
}

function setupEventHandlers() {
  if (!canvas.value) return

  canvas.value.on('mouse:down', handleMouseDown)
  canvas.value.on('mouse:move', handleMouseMove)
  canvas.value.on('mouse:up', handleMouseUp)
  canvas.value.on('object:added', () => saveHistory())
  canvas.value.on('object:modified', () => saveHistory())
}

function handleMouseDown(event: any) {
  if (!canvas.value || currentTool.value === 'select') return

  const pointer = canvas.value.getScenePoint(event.e)
  startX = pointer.x
  startY = pointer.y

  if (currentTool.value === 'text') {
    const text = new IText('Text', {
      left: startX,
      top: startY,
      fill: currentColor.value,
      fontSize: 20,
    })
    canvas.value.add(text)
    canvas.value.setActiveObject(text)
    canvas.value.renderAll()
  } else if (currentTool.value === 'rectangle' || currentTool.value === 'circle') {
    isDrawing = true
    canvas.value.selection = false
  }
}

function handleMouseMove(event: any) {
  if (!canvas.value || !isDrawing) return

  const pointer = canvas.value.getScenePoint(event.e)

  if (currentTool.value === 'rectangle') {
    if (!drawingObject) {
      drawingObject = new Rect({
        left: startX,
        top: startY,
        width: 0,
        height: 0,
        fill: 'transparent',
        stroke: currentColor.value,
        strokeWidth: strokeWidth.value,
      })
      canvas.value.add(drawingObject)
    }

    const width = pointer.x - startX
    const height = pointer.y - startY

    drawingObject.set({
      width: Math.abs(width),
      height: Math.abs(height),
      left: width > 0 ? startX : pointer.x,
      top: height > 0 ? startY : pointer.y,
    })

    canvas.value.renderAll()
  } else if (currentTool.value === 'circle') {
    if (!drawingObject) {
      const circle = new Circle({
        left: startX,
        top: startY,
        radius: 0,
        fill: 'transparent',
        stroke: currentColor.value,
        strokeWidth: strokeWidth.value,
      })
      drawingObject = circle as any
      canvas.value.add(circle)
    }

    const radius = Math.sqrt(
      Math.pow(pointer.x - startX, 2) + Math.pow(pointer.y - startY, 2)
    )

    ;(drawingObject as Circle).set({ radius })
    canvas.value.renderAll()
  }
}

function handleMouseUp() {
  if (!canvas.value) return

  isDrawing = false
  drawingObject = null
  canvas.value.selection = true
}

function setTool(tool: typeof currentTool.value) {
  currentTool.value = tool

  if (!canvas.value) return

  if (tool === 'freehand') {
    canvas.value.isDrawingMode = true
    const brush = new PencilBrush(canvas.value as any)
    brush.color = currentColor.value
    brush.width = strokeWidth.value
    canvas.value.freeDrawingBrush = brush as any
  } else {
    canvas.value.isDrawingMode = false
  }

  canvas.value.selection = tool === 'select'
}

function updateColor() {
  if (!canvas.value) return

  if (canvas.value.isDrawingMode && canvas.value.freeDrawingBrush) {
    canvas.value.freeDrawingBrush.color = currentColor.value
  }

  const activeObject = canvas.value.getActiveObject()
  if (activeObject) {
    if (activeObject instanceof IText) {
      activeObject.set({ fill: currentColor.value })
    } else {
      activeObject.set({ stroke: currentColor.value })
    }
    canvas.value.renderAll()
  }
}

function updateStrokeWidth() {
  if (!canvas.value) return

  if (canvas.value.isDrawingMode && canvas.value.freeDrawingBrush) {
    canvas.value.freeDrawingBrush.width = strokeWidth.value
  }

  const activeObject = canvas.value.getActiveObject()
  if (activeObject && !(activeObject instanceof IText)) {
    activeObject.set({ strokeWidth: strokeWidth.value })
    canvas.value.renderAll()
  }
}

function saveHistory() {
  if (!canvas.value) return

  const json = JSON.stringify(canvas.value.toJSON())

  // Remove any redo history
  history.value = history.value.slice(0, historyStep.value + 1)

  // Add current state
  history.value.push(json)
  historyStep.value = history.value.length - 1

  // Limit history size
  if (history.value.length > 50) {
    history.value.shift()
    historyStep.value--
  }

  updateUndoRedoState()
}

function undo() {
  if (!canvas.value || historyStep.value <= 0) return

  historyStep.value--
  loadHistoryState()
  updateUndoRedoState()
}

function redo() {
  if (!canvas.value || historyStep.value >= history.value.length - 1) return

  historyStep.value++
  loadHistoryState()
  updateUndoRedoState()
}

async function loadHistoryState() {
  if (!canvas.value || !history.value[historyStep.value]) return

  const state = history.value[historyStep.value]
  if (!state) return

  try {
    await canvas.value.loadFromJSON(state)
    canvas.value.renderAll()
  } catch (error) {
    console.error('Failed to load history state:', error)
  }
}

function updateUndoRedoState() {
  canUndo.value = historyStep.value > 0
  canRedo.value = historyStep.value < history.value.length - 1
}

async function saveAnnotatedScreenshot() {
  if (!canvas.value) return

  saving.value = true

  try {
    // Export canvas to data URL (we'll use it in the future)
    // const dataUrl = canvas.value.toDataURL({
    //   format: 'png',
    //   quality: 1,
    //   multiplier: 1,
    // })

    // Generate annotated filename
    const originalPath = props.screenshotPath
    const lastDot = originalPath.lastIndexOf('.')
    const annotatedPath = lastDot > 0
      ? `${originalPath.substring(0, lastDot)}_annotated${originalPath.substring(lastDot)}`
      : `${originalPath}_annotated.png`

    // Emit the annotated path so parent can save it
    // TODO: Implement actual save via Tauri command
    emit('saved', annotatedPath)

    close()
  } catch (error) {
    console.error('Failed to save annotated screenshot:', error)
  } finally {
    saving.value = false
  }
}

function close() {
  isOpen.value = false
}
</script>

<style scoped>
.canvas-container {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 16px;
  min-height: 600px;
}

canvas {
  border: 1px solid #ccc;
  cursor: crosshair;
}
</style>
