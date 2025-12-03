<script setup lang="ts">
import { ref, reactive, computed, inject, watch } from "vue";
import Dialog from "../common/Dialog.vue";
import themeConfig from "../../config/theme.json";
import { info } from '@tauri-apps/plugin-log';
import { GROUPS, NOTE_NAMES } from "../../config/groups";

// 从 App.vue 注入 settingsManager
const settingsManager = inject('settingsManager') as any;

// 从父组件注入主题相关方法和状态
const updateTheme = inject<(themeName: string) => Promise<void>>('updateTheme');
const currentTheme = inject<import('vue').Ref<string>>('currentTheme');

interface SettingsDialogProps {
  visible: boolean;
}

const props = defineProps<SettingsDialogProps>();
const emit = defineEmits<{
  (e: "update:visible", value: boolean): void;
  (e: "settingsSaved", settings: any): void;
}>();

// 当前激活的标签页
const activeTab = ref("keys");

// 本地状态，用于编辑
const localKeySettings = reactive({
  minNote: 48,
  maxNote: 83,
  blackKeyMode: "support_black_key",
  noteToKey: {} as Record<number, string>
});

const localShortcuts = reactive({
  START_PAUSE: "alt+-",
  STOP: "alt+=",
  PREV_SONG: "alt+up",
  NEXT_SONG: "alt+down"
});

const localThemeSettings = reactive({
  currentTheme: "default"
});

// 初始化本地状态
const initLocalState = () => {
  const settings = settingsManager.getSettings();

  localKeySettings.minNote = settings.keySettings?.minNote || 48;
  localKeySettings.maxNote = settings.keySettings?.maxNote || 83;
  localKeySettings.blackKeyMode = settings.keySettings?.blackKeyMode || "support_black_key";
  localKeySettings.noteToKey = { ...(settings.keySettings?.noteToKey || {}) };

  localShortcuts.START_PAUSE = settings.shortcuts?.START_PAUSE || "alt+-";
  localShortcuts.STOP = settings.shortcuts?.STOP || "alt+=";
  localShortcuts.PREV_SONG = settings.shortcuts?.PREV_SONG || "alt+up";
  localShortcuts.NEXT_SONG = settings.shortcuts?.NEXT_SONG || "alt+down";

  localThemeSettings.currentTheme = settings.themeSettings?.currentTheme || "default";

  // 初始化下拉框选择
  updateDropdownsFromNote('min', localKeySettings.minNote);
  updateDropdownsFromNote('max', localKeySettings.maxNote);
};

// 监听可见性变化，初始化数据
watch(() => props.visible, (newVal) => {
  if (newVal) {
    initLocalState();
    info("[SettingsDialog.vue:74] 加载设置");
  }
});

// 主题选项
const themeOptions = computed(() => {
  return themeConfig.theme.map(theme => ({
    value: theme.name,
    label: theme.name === 'default' ? '默认' : theme.name === 'dark' ? '黑夜' : '少女粉'
  }));
});

// 预设配置选项
const presetConfigs = [
  { id: "default", name: "默认配置", minNote: 48, maxNote: 83 },
  { id: "basic", name: "基础模式", minNote: 40, maxNote: 87 },
  { id: "advanced", name: "高级模式", minNote: 21, maxNote: 108 }
];

// 黑键模式选项
const blackKeyModes = [
  { value: "support_black_key", label: "支持黑键" },
  { value: "auto_sharp", label: "黑键降音" }
];

// 音符分组数据
const groupOptions = computed(() => {
  return Object.keys(GROUPS).map(name => ({
    label: name,
    value: name,
    range: GROUPS[name]
  }));
});

// 最低音选择状态
const minNoteGroup = ref("");
const minNoteValue = ref(0);

// 最高音选择状态
const maxNoteGroup = ref("");
const maxNoteValue = ref(0);

// 根据音符值更新下拉框
const updateDropdownsFromNote = (type: 'min' | 'max', note: number) => {
  for (const [name, [lo, hi]] of Object.entries(GROUPS)) {
    if (note >= lo && note <= hi) {
      if (type === 'min') {
        minNoteGroup.value = name;
        minNoteValue.value = note;
      } else {
        maxNoteGroup.value = name;
        maxNoteValue.value = note;
      }
      break;
    }
  }
};

// 获取指定分组内的音符选项
const getNotesInGroup = (groupName: string) => {
  if (!groupName || !GROUPS[groupName]) return [];
  const [lo, hi] = GROUPS[groupName];
  const options = [];
  for (let i = lo; i <= hi; i++) {
    const noteIndex = i % 12;
    const noteName = NOTE_NAMES[noteIndex].toUpperCase();
    options.push({
      value: i,
      label: `${noteName} (${i})`
    });
  }
  return options;
};

// 监听下拉框变化更新设置
watch([minNoteGroup, minNoteValue], ([newGroup, newNote]) => {
  if (newGroup && newNote) {
    // 确保选择的音符在当前分组内，如果不在则默认选分组第一个
    const [lo, hi] = GROUPS[newGroup];
    if (newNote < lo || newNote > hi) {
      minNoteValue.value = lo;
      localKeySettings.minNote = lo;
    } else {
      localKeySettings.minNote = newNote;
    }
  }
});

watch([maxNoteGroup, maxNoteValue], ([newGroup, newNote]) => {
  if (newGroup && newNote) {
    const [lo, hi] = GROUPS[newGroup];
    if (newNote < lo || newNote > hi) {
      maxNoteValue.value = hi; // 最高音默认选分组最后一个
      localKeySettings.maxNote = hi;
    } else {
      localKeySettings.maxNote = newNote;
    }
  }
});

// 保存设置
const saveSettings = async () => {
  // 保存前获取旧的 keySettings
  const oldSettings = settingsManager.getSettings();
  const oldKeySettings = oldSettings.keySettings || {};

  const settings = {
    keySettings: { ...localKeySettings },
    shortcuts: { ...localShortcuts },
    themeSettings: { ...localThemeSettings }
  };

  // 保存到 settingsManager
  await settingsManager.saveSettings(settings);

  // 更新主题
  if (updateTheme && localThemeSettings.currentTheme !== currentTheme?.value) {
    await updateTheme(localThemeSettings.currentTheme);
  }

  // 检测 keySettings 是否变更
  const keySettingsChanged =
    oldKeySettings.minNote !== localKeySettings.minNote ||
    oldKeySettings.maxNote !== localKeySettings.maxNote ||
    oldKeySettings.blackKeyMode !== localKeySettings.blackKeyMode;

  emit("settingsSaved", {
    settings,
    keySettingsChanged
  });
  emit("update:visible", false);
};

// 取消设置
const cancelSettings = () => {
  emit("update:visible", false);
};

// 应用预设配置
const applyPresetConfig = (presetId: string) => {
  info("[SettingsDialog.vue:201] 应用预设配置: " + presetId);
  const preset = presetConfigs.find(p => p.id === presetId);
  if (preset) {
    localKeySettings.minNote = preset.minNote;
    localKeySettings.maxNote = preset.maxNote;
    updateDropdownsFromNote('min', preset.minNote);
    updateDropdownsFromNote('max', preset.maxNote);
  }
};

// 恢复默认快捷键
const restoreDefaultShortcuts = () => {
  localShortcuts.START_PAUSE = "alt+-";
  localShortcuts.STOP = "alt+=";
  localShortcuts.PREV_SONG = "alt+up";
  localShortcuts.NEXT_SONG = "alt+down";
};

// 监听当前主题变化，同步到设置中
watch(() => currentTheme?.value, (newVal) => {
  if (newVal && newVal !== localThemeSettings.currentTheme) {
    localThemeSettings.currentTheme = newVal;
  }
});

</script>

<template>
  <Dialog :visible="visible" @update:visible="emit('update:visible', $event)" title="设置" width="650px" height="600px">
    <!-- 标签页导航 -->
    <div class="settings-tabs">
      <div class="tab-button" :class="{ active: activeTab === 'keys' }" @click="activeTab = 'keys'">
        按键设置
      </div>
      <div class="tab-button" :class="{ active: activeTab === 'shortcuts' }" @click="activeTab = 'shortcuts'">
        快捷键设置
      </div>
      <div class="tab-button" :class="{ active: activeTab === 'theme' }" @click="activeTab = 'theme'">
        主题设置
      </div>
    </div>

    <!-- 标签页内容 -->
    <div class="tab-content">
      <!-- 按键设置标签页 -->
      <div v-if="activeTab === 'keys'" class="keys-tab">
        <!-- 预设配置 -->
        <div class="setting-section">
          <h4 class="section-title">预设配置</h4>
          <div class="preset-buttons">
            <button v-for="preset in presetConfigs" :key="preset.id" class="btn btn-small"
              @click="applyPresetConfig(preset.id)">
              {{ preset.name }}
            </button>
          </div>
        </div>

        <!-- 音符范围设置 -->
        <div class="setting-section">
          <h4 class="section-title">音符范围设置</h4>
          <div class="range-controls-row">
            <!-- 最低音 -->
            <div class="range-control-group">
              <label>最低音：</label>
              <div class="cascade-select">
                <select v-model="minNoteGroup" class="setting-select group-select">
                  <option v-for="group in groupOptions" :key="group.value" :value="group.value">
                    {{ group.label }}
                  </option>
                </select>
                <select v-model="minNoteValue" class="setting-select note-select">
                  <option v-for="note in getNotesInGroup(minNoteGroup)" :key="note.value" :value="note.value">
                    {{ note.label }}
                  </option>
                </select>
              </div>
            </div>

            <!-- 最高音 -->
            <div class="range-control-group">
              <label>最高音：</label>
              <div class="cascade-select">
                <select v-model="maxNoteGroup" class="setting-select group-select">
                  <option v-for="group in groupOptions" :key="group.value" :value="group.value">
                    {{ group.label }}
                  </option>
                </select>
                <select v-model="maxNoteValue" class="setting-select note-select">
                  <option v-for="note in getNotesInGroup(maxNoteGroup)" :key="note.value" :value="note.value">
                    {{ note.label }}
                  </option>
                </select>
              </div>
            </div>
          </div>
        </div>

        <!-- 黑键模式 -->
        <div class="setting-section">
          <h4 class="section-title">黑键设置</h4>
          <div class="black-key-settings">
            <div class="control-group">
              <label>模式：</label>
              <select v-model="localKeySettings.blackKeyMode" class="setting-select mode-select">
                <option v-for="mode in blackKeyModes" :key="mode.value" :value="mode.value">
                  {{ mode.label }}
                </option>
              </select>
            </div>
          </div>
        </div>
      </div>

      <!-- 快捷键设置标签页 -->
      <div v-if="activeTab === 'shortcuts'" class="shortcuts-tab">
        <div class="setting-section">
          <h4 class="section-title">播放控制快捷键</h4>

          <div class="shortcut-item">
            <label>开始/暂停：</label>
            <div class="shortcut-display">
              {{ localShortcuts.START_PAUSE }}
            </div>
          </div>

          <div class="shortcut-item">
            <label>停止：</label>
            <div class="shortcut-display">
              {{ localShortcuts.STOP }}
            </div>
          </div>

          <div class="shortcut-item">
            <label>上一曲：</label>
            <div class="shortcut-display">
              {{ localShortcuts.PREV_SONG }}
            </div>
          </div>

          <div class="shortcut-item">
            <label>下一曲：</label>
            <div class="shortcut-display">
              {{ localShortcuts.NEXT_SONG }}
            </div>
          </div>

          <button @click="restoreDefaultShortcuts" class="restore-button btn btn-small">
            恢复默认快捷键
          </button>
        </div>
      </div>

      <!-- 主题设置标签页 -->
      <div v-if="activeTab === 'theme'" class="theme-tab">
        <div class="setting-section">
          <h4 class="section-title">主题选择</h4>
          <div class="theme-options">
            <div v-for="option in themeOptions" :key="option.value" class="theme-option"
              :class="{ active: localThemeSettings.currentTheme === option.value }"
              @click="localThemeSettings.currentTheme = option.value">
              <div class="theme-preview"
                :style="{ backgroundColor: themeConfig.theme.find(t => t.name === option.value)?.bg || '#ffffff' }">
                <div class="theme-color"
                  :style="{ backgroundColor: themeConfig.theme.find(t => t.name === option.value)?.primary || '#007bff' }">
                </div>
                <div class="theme-color"
                  :style="{ backgroundColor: themeConfig.theme.find(t => t.name === option.value)?.success || '#28a745' }">
                </div>
                <div class="theme-color"
                  :style="{ backgroundColor: themeConfig.theme.find(t => t.name === option.value)?.info || '#17a2b8' }">
                </div>
              </div>
              <span class="theme-name">
                {{ option.label }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 底部按钮 -->
    <template #footer>
      <div class="dialog-footer">
        <button @click="cancelSettings" class="cancel-button">取消</button>
        <button @click="saveSettings" class="save-button">保存</button>
      </div>
    </template>
  </Dialog>
</template>

<style scoped>
.settings-tabs {
  display: flex;
  border-bottom: 1px solid var(--border);
  margin-bottom: 1rem;
}

.tab-button {
  padding: 0.75rem 1.5rem;
  cursor: pointer;
  border: none;
  background: none;
  font-size: 0.95rem;
  color: var(--secondary);
  border-bottom: 2px solid transparent;
  transition: all 0.2s ease;
}

.tab-button:hover {
  color: var(--primary);
}

.tab-button.active {
  color: var(--primary);
  border-bottom-color: var(--primary);
  font-weight: 500;
}

.tab-content {
  flex: 1;
  overflow-y: auto;
  padding-right: 0.5rem;
}

.setting-section {
  margin-bottom: 1.5rem;
}

.section-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--fg);
  margin-bottom: 0.75rem;
}

/* 按键设置样式 */
.preset-buttons {
  display: flex;
  gap: 0.75rem;
  flex-wrap: wrap;
}

.range-controls-row {
  display: flex;
  gap: 2rem;
  align-items: flex-start;
}

.range-control-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.range-control-group label {
  font-size: 0.9rem;
  color: var(--fg);
  font-weight: 500;
}

.cascade-select {
  display: flex;
  gap: 0.5rem;
}

.setting-select {
  padding: 0.35rem;
  border: 1px solid var(--border);
  border-radius: 4px;
  background-color: var(--inputbg);
  color: var(--inputfg);
  font-size: 0.9rem;
}

.group-select {
  width: 140px;
}

.note-select {
  width: 80px;
}

.mode-select {
  width: 150px;
}

.control-group {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.control-group label {
  font-size: 0.9rem;
  color: var(--fg);
}

/* 快捷键设置样式 */
.shortcut-item {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 0.75rem;
}

.shortcut-item label {
  width: 100px;
  font-size: 0.9rem;
  color: var(--fg);
}

.shortcut-display {
  padding: 0.35rem 0.75rem;
  background-color: var(--active);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-family: monospace;
  font-size: 0.9rem;
  color: var(--inputfg);
}

.restore-button {
  margin-top: 1rem;
}

/* 主题设置样式 */
.theme-options {
  display: flex;
  gap: 1.5rem;
  flex-wrap: wrap;
}

.theme-option {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  padding: 0.5rem;
  border-radius: 4px;
  transition: all 0.2s ease;
}

.theme-option:hover {
  background-color: var(--active);
}

.theme-option.active {
  background-color: var(--active);
  border: 1px solid var(--primary);
}

.theme-preview {
  width: 100px;
  height: 60px;
  border-radius: 4px;
  padding: 0.5rem;
  display: flex;
  gap: 0.5rem;
  align-items: center;
  justify-content: center;
}

.theme-color {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 1px solid var(--border);
}

.theme-name {
  font-size: 0.9rem;
  color: var(--fg);
}

/* 按钮样式 */
.btn {
  padding: 0.5rem 1rem;
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 0.9rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-small {
  padding: 0.35rem 0.75rem;
  font-size: 0.85rem;
  background-color: var(--inputbg);
  color: var(--inputfg);
}

.btn-small:hover {
  background-color: var(--active);
}

/* 对话框底部按钮样式 */
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.75rem;
  padding: 0.5rem 1rem;
}

.cancel-button,
.save-button {
  padding: 0.4rem 0.6rem;
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 0.9rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.cancel-button {
  background-color: var(--inputbg);
  color: var(--inputfg);
  border-color: var(--border);
}

.cancel-button:hover {
  background-color: var(--active);
}

.save-button {
  background-color: var(--primary);
  color: var(--selectfg);
  border-color: var(--primary);
}

.save-button:hover {
  background-color: var(--dark);
  border-color: var(--dark);
}
</style>
