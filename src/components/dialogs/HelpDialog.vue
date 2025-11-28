<script setup lang="ts">
import Dialog from "../common/Dialog.vue";
import { ref } from "vue";

interface HelpDialogProps {
  visible: boolean;
}

const props = defineProps<HelpDialogProps>();
const emit = defineEmits<{
  (e: "update:visible", value: boolean): void;
}>();

// 使用说明文本
const usageText = ref([
  "1. 使用管理员权限启动",
  "2. 选择MIDI文件和音轨",
  "3. 点击播放按钮开始演奏",
  "4. 支持36键模式"
]);
</script>

<template>
  <Dialog
    :visible="visible"
    @update:visible="emit('update:visible', $event)"
    title="帮助"
    width="400px"
    height="300px"
  >
    <div class="help-content">
      <div class="usage-section">
        <h4 class="section-title">使用说明</h4>
        <ul class="usage-list">
          <li v-for="(item, index) in usageText" :key="index">
            {{ item }}
          </li>
        </ul>
      </div>
      
      <div class="notice-section">
        <h4 class="section-title">注意事项</h4>
        <ul class="notice-list">
          <li>请确保游戏窗口处于前台</li>
          <li>使用前请先测试音符范围是否合适</li>
          <li>如有问题请检查配置设置</li>
        </ul>
      </div>
    </div>
    
    <template #footer>
      <button class="btn btn-primary" @click="emit('update:visible', false)">
        确定
      </button>
    </template>
  </Dialog>
</template>

<style scoped>
.help-content {
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.section-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--fg);
  margin-bottom: 0.75rem;
}

.usage-list,
.notice-list {
  padding-left: 1.5rem;
  margin: 0;
}

.usage-list li,
.notice-list li {
  margin-bottom: 0.5rem;
  font-size: 0.9rem;
  color: var(--fg);
  line-height: 1.5;
}

.usage-list li:last-child,
.notice-list li:last-child {
  margin-bottom: 0;
}

.btn {
  padding: 0.5rem 1.5rem;
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 0.9rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-primary {
  background-color: var(--primary);
  color: var(--selectfg);
  border-color: var(--primary);
}

.btn-primary:hover {
  background-color: var(--dark);
  border-color: var(--dark);
}
</style>
