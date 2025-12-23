<template>
  <input 
    ref="inputRef"
    type="text" 
    :value="displayValue"
    @keydown="handleKeyDown"
    @focus="handleFocus"
    @blur="handleBlur"
    @click="handleClick"
    :placeholder="placeholder"
    readonly
    class="hotkey-input"
    :class="{ capturing: isCapturing }"
  />
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { info } from '@tauri-apps/plugin-log'

// Props
interface Props {
  modelValue: string
  placeholder?: string
  capturingPlaceholder?: string
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: '点击设置快捷键',
  capturingPlaceholder: '按下快捷键...'
})

// Emits
const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

// State
const inputRef = ref<HTMLInputElement>()
const isCapturing = ref(false)

// Computed
const displayValue = computed(() => {
  if (isCapturing.value) {
    return props.capturingPlaceholder
  }
  return props.modelValue || props.placeholder
})

// Methods
function handleFocus() {
  info('[HotkeyInput] Focus triggered')
  isCapturing.value = true
}

function handleBlur() {
  info('[HotkeyInput] Blur triggered')
  isCapturing.value = false
}

function handleClick() {
    info('[HotkeyInput] Click triggered')
    inputRef.value?.focus()
}

function handleKeyDown(event: KeyboardEvent) {
  info(`[HotkeyInput] Key pressed: ${event.key}, code: ${event.code}, ctrl: ${event.ctrlKey}, shift: ${event.shiftKey}, alt: ${event.altKey}, meta: ${event.metaKey}`)
  
  event.preventDefault()
  event.stopPropagation()
  
  const keys: string[] = []
  
  // 修饰键
  if (event.metaKey) keys.push('Cmd')
  if (event.ctrlKey) keys.push('Ctrl')
  if (event.altKey) keys.push('Alt')
  if (event.shiftKey) keys.push('Shift')
  
  // 主键
  const key = event.key
  
  // 只有当按下的不是修饰键本身时才处理
  if (key && !['Control', 'Shift', 'Alt', 'Meta'].includes(key)) {
    // 特殊键映射
    const keyMap: Record<string, string> = {
      ' ': 'Space',
      'ArrowUp': 'Up',
      'ArrowDown': 'Down',
      'ArrowLeft': 'Left',
      'ArrowRight': 'Right',
      'Escape': 'Esc',
    }
    
    const mappedKey = keyMap[key] || key.toUpperCase()
    keys.push(mappedKey)
    
    // 只有在有主键时才更新快捷键并失去焦点
    if (keys.length > 0) {
      const hotkeyValue = keys.join('+')
      info(`[HotkeyInput] Setting hotkey: ${hotkeyValue}`)
      emit('update:modelValue', hotkeyValue)
      
      // 立即失去焦点以触发界面更新
      if (inputRef.value) {
        inputRef.value.blur()
      }
    }
  } else {
    info('[HotkeyInput] Modifier key only, waiting for main key...')
  }
}

// 暴露方法供父组件调用
defineExpose({
  focus: () => inputRef.value?.focus(),
  blur: () => inputRef.value?.blur()
})
</script>

<style scoped>
.hotkey-input {
  /* 继承原有的 key-input 样式和增强 */
  padding: 0.3rem;
  border: 1px solid var(--border);
  border-radius: 4px;
  background-color: var(--inputbg);
  color: var(--inputfg);
  font-size: 0.85rem;
  max-width: 120px;
  cursor: pointer;
  text-align: center;
  transition: all 0.2s ease;
}

.hotkey-input:hover {
  border-color: var(--primary);
}

.hotkey-input:focus,
.hotkey-input.capturing {
  outline: none;
  border-color: var(--primary);
  background-color: var(--active);
  color: var(--selectfg);
  box-shadow: 0 0 0 2px rgba(var(--primary-rgb), 0.2);
}
</style>
