<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch } from "vue";

interface DialogProps {
  visible: boolean;
  title: string;
  width?: string;
  height?: string;
  modal?: boolean;
}

const props = withDefaults(defineProps<DialogProps>(), {
  width: "500px",
  height: "400px",
  modal: true
});

const emit = defineEmits<{
  (e: "update:visible", value: boolean): void;
  (e: "close"): void;
}>();

const dialogRef = ref<HTMLElement>();
const maskRef = ref<HTMLElement>();

// 居中对话框
const centerDialog = () => {
  if (!dialogRef.value) return;
  
  const dialog = dialogRef.value;
  const rect = dialog.getBoundingClientRect();
  const left = (window.innerWidth - rect.width) / 2;
  const top = (window.innerHeight - rect.height) / 2;
  
  dialog.style.left = `${left}px`;
  dialog.style.top = `${top}px`;
};

// 关闭对话框
const closeDialog = () => {
  emit("update:visible", false);
  emit("close");
};

// 处理点击遮罩层
const handleMaskClick = (e: MouseEvent) => {
  if (e.target === maskRef.value) {
    closeDialog();
  }
};

// 处理ESC键
const handleEscKey = (e: KeyboardEvent) => {
  if (e.key === "Escape" && props.visible) {
    closeDialog();
  }
};

// 监听可见性变化
watch(() => props.visible, (newVal) => {
  if (newVal) {
    setTimeout(() => {
      centerDialog();
    }, 0);
  }
});

// 监听窗口大小变化
const handleResize = () => {
  if (props.visible) {
    centerDialog();
  }
};

onMounted(() => {
  window.addEventListener("resize", handleResize);
  document.addEventListener("keydown", handleEscKey);
});

onBeforeUnmount(() => {
  window.removeEventListener("resize", handleResize);
  document.removeEventListener("keydown", handleEscKey);
});
</script>

<template>
  <Teleport to="body">
    <div 
      v-if="visible" 
      class="dialog-overlay"
      ref="maskRef"
      @click="handleMaskClick"
      :class="{ 'dialog-overlay-modal': modal }"
    >
      <div 
        class="dialog-container"
        ref="dialogRef"
        :style="{ width, height }"
      >
        <!-- 对话框头部 -->
        <div class="dialog-header">
          <h3 class="dialog-title">{{ title }}</h3>
          <button class="dialog-close-btn" @click="closeDialog">×</button>
        </div>
        
        <!-- 对话框内容 -->
        <div class="dialog-body">
          <slot></slot>
        </div>
        
        <!-- 对话框底部 -->
        <div class="dialog-footer" v-if="$slots.footer">
          <slot name="footer"></slot>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.dialog-overlay-modal {
  background-color: rgba(0, 0, 0, 0.5);
}

.dialog-container {
  background-color: var(--bg);
  border: 1px solid var(--border);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  display: flex;
  flex-direction: column;
  position: absolute;
  overflow: hidden;
}

.dialog-header {
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--border);
  display: flex;
  justify-content: space-between;
  align-items: center;
  background-color: var(--active);
}

.dialog-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--fg);
  margin: 0;
}

.dialog-close-btn {
  background: none;
  border: none;
  font-size: 1.25rem;
  color: var(--secondary);
  cursor: pointer;
  padding: 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s ease;
}

.dialog-close-btn:hover {
  background-color: var(--primary);
  color: var(--selectfg);
}

.dialog-body {
  flex: 1;
  padding: 1rem;
  overflow-y: auto;
}

.dialog-footer {
  padding: 0.75rem 1rem;
  border-top: 1px solid var(--border);
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
}
</style>