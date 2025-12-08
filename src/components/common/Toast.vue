<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';

interface ToastProps {
  message: string;
  type?: 'success' | 'info' | 'warning' | 'error';
  duration?: number;
  visible: boolean;
}

const props = withDefaults(defineProps<ToastProps>(), {
  type: 'success',
  duration: 3000
});

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void;
}>();

const show = ref(false);

watch(() => props.visible, (newVal) => {
  if (newVal) {
    show.value = true;
    setTimeout(() => {
      show.value = false;
      setTimeout(() => emit('update:visible', false), 300);
    }, props.duration);
  }
});

onMounted(() => {
  if (props.visible) {
    show.value = true;
    setTimeout(() => {
      show.value = false;
      setTimeout(() => emit('update:visible', false), 300);
    }, props.duration);
  }
});
</script>

<template>
  <Transition name="toast">
    <div v-if="show" class="toast" :class="`toast-${type}`">
      <div class="toast-icon">
        <svg v-if="type === 'success'" viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
        <svg v-else-if="type === 'info'" viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <circle cx="12" cy="12" r="10" stroke-width="2"/>
          <path stroke-linecap="round" stroke-width="2" d="M12 16v-4M12 8h.01"/>
        </svg>
        <svg v-else-if="type === 'warning'" viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <svg v-else-if="type === 'error'" viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <circle cx="12" cy="12" r="10" stroke-width="2"/>
          <path stroke-linecap="round" stroke-width="2" d="M15 9l-6 6M9 9l6 6"/>
        </svg>
      </div>
      <div class="toast-message">{{ message }}</div>
    </div>
  </Transition>
</template>

<style scoped>
.toast {
  position: fixed;
  top: 20px;
  right: 20px;
  min-width: 250px;
  max-width: 400px;
  padding: 12px 16px;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  display: flex;
  align-items: center;
  gap: 12px;
  z-index: 9999;
  backdrop-filter: blur(10px);
  font-size: 14px;
}

.toast-success {
  background-color: rgba(34, 197, 94, 0.95);
  color: white;
}

.toast-info {
  background-color: rgba(59, 130, 246, 0.95);
  color: white;
}

.toast-warning {
  background-color: rgba(251, 146, 60, 0.95);
  color: white;
}

.toast-error {
  background-color: rgba(239, 68, 68, 0.95);
  color: white;
}

.toast-icon {
  flex-shrink: 0;
  width: 20px;
  height: 20px;
}

.toast-icon svg {
  width: 100%;
  height: 100%;
}

.toast-message {
  flex: 1;
  line-height: 1.5;
  white-space: pre-line;
}

.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(100%);
}

.toast-leave-to {
  opacity: 0;
  transform: translateY(-20px);
}
</style>
