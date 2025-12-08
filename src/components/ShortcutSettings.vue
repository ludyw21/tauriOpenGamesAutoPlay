<script setup lang="ts">
import { ref, reactive, inject, onMounted } from "vue";
import { info } from '@tauri-apps/plugin-log';
import Toast from "./common/Toast.vue";

// 从 App.vue 注入 settingsManager
const settingsManager = inject('settingsManager') as any;

// 本地快捷键状态
const localShortcuts = reactive({
  START_PAUSE: "alt+-",
  STOP: "alt+=",
  PREV_SONG: "alt+up",
  NEXT_SONG: "alt+down"
});

// Toast 通知状态
const toastVisible = ref(false);
const toastMessage = ref('');
const toastType = ref<'success' | 'info' | 'warning' | 'error'>('success');

// 初始化本地状态
const initLocalState = () => {
  const settings = settingsManager.getSettings();
  localShortcuts.START_PAUSE = settings.shortcuts?.START_PAUSE || "alt+-";
  localShortcuts.STOP = settings.shortcuts?.STOP || "alt+=";
  localShortcuts.PREV_SONG = settings.shortcuts?.PREV_SONG || "alt+up";
  localShortcuts.NEXT_SONG = settings.shortcuts?.NEXT_SONG || "alt+down";
};

// 组件挂载时加载设置
onMounted(() => {
  initLocalState();
  info("[ShortcutSettings.vue] 加载快捷键设置");
});

// 恢复默认快捷键
const restoreDefaultShortcuts = () => {
  localShortcuts.START_PAUSE = "alt+-";
  localShortcuts.STOP = "alt+=";
  localShortcuts.PREV_SONG = "alt+up";
  localShortcuts.NEXT_SONG = "alt+down";
  info("[ShortcutSettings.vue] 恢复默认快捷键");
};

// 保存设置
const saveSettings = async () => {
  const settings = {
    shortcuts: { ...localShortcuts }
  };

  await settingsManager.saveSettings(settings);
  info("[ShortcutSettings.vue] 快捷键设置已保存");

  // 显示 Toast 通知
  toastMessage.value = "快捷键设置已保存!";
  toastType.value = 'success';
  toastVisible.value = true;
};
</script>

<template>
  <div class="shortcut-settings-view">
    <!-- Toast 通知 -->
    <Toast :visible="toastVisible" @update:visible="toastVisible = $event" :message="toastMessage" :type="toastType" />


    <div class="settings-content">
      <div class="setting-section">
        <h4 class="section-title">播放控制快捷键</h4>

        <div class="shortcut-item">
          <label>开始/暂停模拟:</label>
          <input type="text" v-model="localShortcuts.START_PAUSE" class="shortcut-input">
        </div>

        <div class="shortcut-item">
          <label>停止模拟:</label>
          <input type="text" v-model="localShortcuts.STOP" class="shortcut-input">
        </div>

        <div class="shortcut-item">
          <label>上一首:</label>
          <input type="text" v-model="localShortcuts.PREV_SONG" class="shortcut-input">
        </div>

        <div class="shortcut-item">
          <label>下一首:</label>
          <input type="text" v-model="localShortcuts.NEXT_SONG" class="shortcut-input">
        </div>

        <div class="button-group">
          <button @click="restoreDefaultShortcuts" class="btn btn-secondary">
            恢复默认快捷键
          </button>
          <button @click="saveSettings" class="btn btn-primary">
            保存设置
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.shortcut-settings-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  padding: 2rem;
  background-color: var(--bg);
  overflow-y: auto;
}

.view-title {
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--fg);
  margin-bottom: 2rem;
  padding-bottom: 1rem;
  border-bottom: 2px solid var(--border);
}

.settings-content {
  flex: 1;
}

.setting-section {
  margin-bottom: 2rem;
}

.section-title {
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--primary);
  margin-bottom: 1.5rem;
}

.shortcut-item {
  display: flex;
  align-items: center;
  margin-bottom: 1rem;
  gap: 1rem;
}

.shortcut-item label {
  min-width: 150px;
  font-size: 0.95rem;
  color: var(--fg);
  font-weight: 500;
}

.shortcut-input {
  flex: 1;
  max-width: 300px;
  padding: 0.5rem;
  border: 1px solid var(--border);
  border-radius: 4px;
  background-color: var(--inputbg);
  color: var(--inputfg);
  font-size: 0.9rem;
}

.shortcut-input:focus {
  outline: none;
  border-color: var(--primary);
}

.button-group {
  display: flex;
  gap: 1rem;
  margin-top: 2rem;
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

.btn-secondary {
  background-color: var(--inputbg);
  color: var(--inputfg);
}

.btn-secondary:hover {
  background-color: var(--active);
}
</style>
