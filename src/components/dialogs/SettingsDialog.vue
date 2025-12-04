<script setup lang="ts">
import { ref, reactive, computed, inject, watch } from "vue";
import Dialog from "../common/Dialog.vue";
import themeConfig from "../../config/theme.json";
import { info } from '@tauri-apps/plugin-log';
import { GROUPS, NOTE_NAMES, getNoteName } from "../../config/groups";
import { NOTE_TO_KEY } from "../../config/keyboard_mapping";

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
  trimLongNotes: false,
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
  localKeySettings.trimLongNotes = settings.keySettings?.trimLongNotes || false;
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
  {
    id: "36key",
    name: "燕云十六声(36键)",
    minNote: 48,
    maxNote: 83,
    blackKeyMode: "support_black_key",
    trimLongNotes: true,
    noteToKey: { ...NOTE_TO_KEY }
  },
  {
    id: "21key",
    name: "燕云十六声(21键)",
    minNote: 48,
    maxNote: 83,
    blackKeyMode: "auto_sharp",
    trimLongNotes: true,
    // 21键模式下，只保留不含组合键的映射（即单键）
    noteToKey: Object.fromEntries(
      Object.entries(NOTE_TO_KEY).filter(([_, key]) => !key.includes('+'))
    )
  }
];

// 黑键模式选项
// const blackKeyModes = [
//   { value: "support_black_key", label: "支持黑键" },
//   { value: "auto_sharp", label: "黑键降音" }
// ];

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

// 计算需要显示的音符列表（按分组）
const displayNoteGroups = computed(() => {
  const groups: { name: string; notes: { note: number; name: string; key: string }[] }[] = [];
  const min = localKeySettings.minNote;
  const max = localKeySettings.maxNote;
  const blackKeyMode = localKeySettings.blackKeyMode;

  // 遍历所有分组
  for (const [groupName, [lo, hi]] of Object.entries(GROUPS)) {
    // 检查分组是否在范围内
    if (hi < min || lo > max) continue;

    const notesInGroup: { note: number; name: string; key: string }[] = [];
    const start = Math.max(lo, min);
    const end = Math.min(hi, max);

    for (let i = start; i <= end; i++) {
      // 如果是自动降音模式，且是黑键，则跳过
      if (blackKeyMode === 'auto_sharp') {
        const noteIndex = i % 12;
        const isBlackKey = [1, 3, 6, 8, 10].includes(noteIndex);
        if (isBlackKey) continue;
      }

      const noteName = getNoteName(i);
      // 获取按键映射：优先使用本地设置，如果没有则使用默认映射，再没有则为空
      const key = localKeySettings.noteToKey[i] !== undefined
        ? localKeySettings.noteToKey[i]
        : (NOTE_TO_KEY[i] || '');

      notesInGroup.push({
        note: i,
        name: `${noteName} (${i})`,
        key: key
      });
    }

    if (notesInGroup.length > 0) {
      groups.push({
        name: groupName,
        notes: notesInGroup
      });
    }
  }

  // 按音高排序分组 (GROUPS对象的键序可能不保证顺序，这里简单按第一个音符排序)
  groups.sort((a, b) => a.notes[0].note - b.notes[0].note);

  return groups;
});

// 更新按键映射
const updateNoteKey = (note: number, key: string) => {
  localKeySettings.noteToKey[note] = key;
};

// 保存设置
const saveSettings = async () => {
  // 保存前获取旧的 keySettings
  const oldSettings = settingsManager.getSettings();
  const oldKeySettings = oldSettings.keySettings || {};

  // 确保 noteToKey 包含当前范围内所有的按键配置（包括默认值）
  const fullNoteToKey: Record<number, string> = {};
  const min = localKeySettings.minNote;
  const max = localKeySettings.maxNote;
  const blackKeyMode = localKeySettings.blackKeyMode;

  for (let i = min; i <= max; i++) {
    // 如果是自动降音模式，且是黑键，则跳过保存
    if (blackKeyMode === 'auto_sharp') {
      const noteIndex = i % 12;
      const isBlackKey = [1, 3, 6, 8, 10].includes(noteIndex);
      if (isBlackKey) continue;
    }

    // 获取有效按键：优先使用本地设置，如果没有则使用默认映射
    // 注意：如果用户显式清空了按键（值为""），也会被保留
    const key = localKeySettings.noteToKey[i] !== undefined
      ? localKeySettings.noteToKey[i]
      : (NOTE_TO_KEY[i] || '');

    if (key !== undefined) {
      fullNoteToKey[i] = key;
    }
  }

  // 更新本地状态中的 noteToKey，以便保存
  localKeySettings.noteToKey = fullNoteToKey;

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
    oldKeySettings.blackKeyMode !== localKeySettings.blackKeyMode ||
    oldKeySettings.trimLongNotes !== localKeySettings.trimLongNotes;

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
    localKeySettings.blackKeyMode = preset.blackKeyMode;
    localKeySettings.trimLongNotes = preset.trimLongNotes;
    // 更新映射
    localKeySettings.noteToKey = { ...preset.noteToKey };

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
        <div class="setting-section preset-config-section">
          <h4 class="section-title">预设配置</h4>
          <div class="preset-buttons">
            <button v-for="preset in presetConfigs" :key="preset.id" class="btn btn-small"
              @click="applyPresetConfig(preset.id)">
              {{ preset.name }}
            </button>
          </div>
        </div>

        <!-- 基础配置 -->
        <div class="setting-section">
          <h4 class="section-title">基础配置</h4>
          <div class="basic-config-container">
            <!-- 左侧：音域设置 (2/3) -->
            <div class="range-settings">
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

            <!-- 右侧：黑键设置 (1/3) -->
            <div class="black-key-settings">
              <!-- 黑键模式开关 -->
              <div class="switch-control">
                <label class="switch">
                  <input type="checkbox" :checked="localKeySettings.blackKeyMode === 'support_black_key'"
                    @change="e => localKeySettings.blackKeyMode = (e.target as HTMLInputElement).checked ? 'support_black_key' : 'auto_sharp'">
                  <span class="slider round"></span>
                </label>
                <span class="switch-label">{{ localKeySettings.blackKeyMode === 'support_black_key' ? '支持黑键' : '黑键降音'
                }}</span>
              </div>

              <!-- 长音修剪开关 -->
              <div class="switch-control">
                <label class="switch">
                  <input type="checkbox" v-model="localKeySettings.trimLongNotes">
                  <span class="slider round"></span>
                </label>
                <span class="switch-label">长音修剪</span>
              </div>
            </div>
          </div>
        </div>

        <!-- 按键配置 -->
        <div class="setting-section">
          <h4 class="section-title">模拟配置</h4>
          <div class="key-mapping-container">
            <div v-for="group in displayNoteGroups" :key="group.name" class="note-group">
              <div class="group-header">{{ group.name }}</div>
              <div class="group-notes">
                <div v-for="note in group.notes" :key="note.note" class="note-item">
                  <label>{{ note.name }}:</label>
                  <input type="text" class="key-input" :value="note.key"
                    @input="e => updateNoteKey(note.note, (e.target as HTMLInputElement).value)" placeholder="未设置">
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 快捷键设置标签页 -->
      <div v-if="activeTab === 'shortcuts'" class="shortcuts-tab">
        <div class="setting-section">
          <h4 class="section-title">播放控制快捷键</h4>

          <div class="shortcut-item">
            <label>开始/暂停模拟：</label>
            <input type="text" v-model="localShortcuts.START_PAUSE" class="shortcut-input">
          </div>

          <div class="shortcut-item">
            <label>停止模拟：</label>
            <input type="text" v-model="localShortcuts.STOP" class="shortcut-input">
          </div>

          <div class="shortcut-item">
            <label>上一首：</label>
            <input type="text" v-model="localShortcuts.PREV_SONG" class="shortcut-input">
          </div>

          <div class="shortcut-item">
            <label>下一首：</label>
            <input type="text" v-model="localShortcuts.NEXT_SONG" class="shortcut-input">
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
/* 基础配置布局 */
.basic-config-container {
  display: flex;
  gap: 2rem;
  align-items: flex-start;
  flex-wrap: wrap;
  /* 允许换行 */
}

.range-settings {
  flex: 2;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  min-width: 300px;
  /* 确保在小屏幕下有足够的宽度 */
}

.range-control-group {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 1rem;
}

.range-control-group label {
  font-size: 0.9rem;
  color: var(--fg);
  font-weight: 500;
  min-width: 60px;
}

.black-key-settings {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  /* 与音域设置垂直居中对齐 */
  min-width: 150px;
}

.switch-control {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  margin-bottom: 1rem;
}

.switch-label {
  font-size: 0.9rem;
  color: var(--fg);
}

/* 开关样式 */
.switch {
  position: relative;
  display: inline-block;
  width: 40px;
  height: 20px;
}

.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: #ccc;
  transition: .4s;
}

.slider:before {
  position: absolute;
  content: "";
  height: 16px;
  width: 16px;
  left: 2px;
  bottom: 2px;
  background-color: white;
  transition: .4s;
}

input:checked+.slider {
  background-color: var(--primary);
}

input:focus+.slider {
  box-shadow: 0 0 1px var(--primary);
}

input:checked+.slider:before {
  transform: translateX(20px);
}

/* Rounded sliders */
.slider.round {
  border-radius: 20px;
}

.slider.round:before {
  border-radius: 50%;
}

/* 按键映射样式 */
.key-mapping-container {
  overflow-x: visible;
  overflow-y: visible;
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 1rem;
  background-color: var(--bg);
  display: flex;
  flex-direction: row;
  gap: 1.5rem;
}

.note-group {
  min-width: 150px;
  display: flex;
  flex-direction: column;
}

.group-header {
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--primary);
  margin-bottom: 0.8rem;
  padding-bottom: 0.25rem;
  border-bottom: 1px solid var(--border);
  text-align: center;
}

.group-notes {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  overflow-y: auto;
  padding-right: 0.5rem;
}

.note-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.note-item label {
  font-size: 0.95rem;
  /* 放大字体 */
  color: var(--fg);
  width: 65px;
  /* 缩小标签宽度至适配文本的合理值 */
  white-space: nowrap;
  /* 防止换行 */
}

.key-input {
  flex: none;
  /* 取消 flex 伸缩 */
  width: 75px;
  /* 增加输入框宽度至合理尺寸 */
  padding: 0.25rem 0.5rem;
  border: 1px solid var(--border);
  border-radius: 4px;
  background-color: var(--inputbg);
  color: var(--inputfg);
  font-size: 0.85rem;
}

.key-input:focus {
  border-color: var(--primary);
  outline: none;
}

/* 快捷键输入框样式 */
.shortcut-input {
  padding: 0.35rem 0.75rem;
  background-color: var(--inputbg);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-family: monospace;
  font-size: 0.9rem;
  color: var(--inputfg);
  width: 100px;
}

.shortcut-input:focus {
  border-color: var(--primary);
  outline: none;
}

/* 保留原有样式 */
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
  margin-bottom: 0.7rem;
}

.section-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--fg);
  margin-bottom: 0.7rem;
}

/* 预设配置部分 */
.preset-config-section {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.7rem;
}

.preset-config-section .section-title {
  margin-bottom: 0;
}

.preset-buttons {
  display: flex;
  gap: 0.75rem;
  flex-wrap: wrap;
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

.shortcut-item {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 0.75rem;
}

.shortcut-item label {
  width: 150px;
  font-size: 0.9rem;
  color: var(--fg);
}

.restore-button {
  margin-top: 1rem;
}

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
