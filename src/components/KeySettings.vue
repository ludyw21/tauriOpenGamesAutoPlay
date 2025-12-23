<script setup lang="ts">
import { ref, reactive, computed, inject, watch, onMounted } from "vue";
import { info, error } from '@tauri-apps/plugin-log';
import { GROUPS, NOTE_NAMES, getNoteName } from "../config/groups";
import { NOTE_TO_KEY } from "../config/keyboard_mapping";
import Toast from "./common/Toast.vue";
import HotkeyInput from "./HotkeyInput.vue";

// 从 App.vue 注入 settingsManager
const settingsManager = inject('settingsManager') as any;

// 本地状态
const localAnalyzerSettings = reactive({
  minNote: 48,
  maxNote: 83,
  blackKeyMode: "support_black_key",
  trimLongNotes: false
});

const localSimulationSettings = reactive({
  simulationType: "keyboard" as 'keyboard' | 'mouse',
  noteToKey: {} as Record<number, string>,
  noteToMouse: {} as Record<number, { x: number; y: number }>
});

// Toast 通知状态
const toastVisible = ref(false);
const toastMessage = ref('');
const toastType = ref<'success' | 'info' | 'warning' | 'error'>('success');

// 预设配置选项
const presetConfigs = [
  {
    id: "36key",
    name: "燕云十六声(36键)",
    minNote: 48,
    maxNote: 83,
    blackKeyMode: "support_black_key",
    trimLongNotes: true,
    simulationType: "keyboard",
    noteToKey: { ...NOTE_TO_KEY }
  },
  {
    id: "21key",
    name: "燕云十六声(21键)",
    minNote: 48,
    maxNote: 83,
    blackKeyMode: "auto_sharp",
    trimLongNotes: true,
    simulationType: "keyboard",
    noteToKey: Object.fromEntries(
      Object.entries(NOTE_TO_KEY).filter(([_, key]) => !key.includes('+'))
    )
  }
];

// 音符分组数据
const groupOptions = computed(() => {
  return Object.keys(GROUPS).map(name => ({
    label: name,
    value: name,
    range: GROUPS[name]
  }));
});

// 最低音/最高音选择状态
const minNoteGroup = ref("");
const minNoteValue = ref(0);
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

// 初始化本地状态
const initLocalState = () => {
  const settings = settingsManager.getSettings();

  localAnalyzerSettings.minNote = settings.analyzerSetting?.minNote || 48;
  localAnalyzerSettings.maxNote = settings.analyzerSetting?.maxNote || 83;
  localAnalyzerSettings.blackKeyMode = settings.analyzerSetting?.blackKeyMode || "support_black_key";
  localAnalyzerSettings.trimLongNotes = settings.analyzerSetting?.trimLongNotes ?? false;

  localSimulationSettings.simulationType = settings.simulationSettings?.simulationType || "keyboard";
  localSimulationSettings.noteToKey = { ...(settings.simulationSettings?.noteToKey || {}) };
  localSimulationSettings.noteToMouse = { ...(settings.simulationSettings?.noteToMouse || {}) };

  // 调试日志
  info(`[KeySettings.vue] 加载配置 - simulationType: ${localSimulationSettings.simulationType}`);
  info(`[KeySettings.vue] 加载配置 - noteToMouse: ${JSON.stringify(localSimulationSettings.noteToMouse)}`);

  updateDropdownsFromNote('min', localAnalyzerSettings.minNote);
  updateDropdownsFromNote('max', localAnalyzerSettings.maxNote);
};

// 组件挂载时加载设置
onMounted(async () => {
  // 等待settingsManager初始化完成
  await settingsManager.initialize();
  initLocalState();
  info("[KeySettings.vue] 加载按键设置");
});

// 监听模拟类型切换,重新加载 noteToMouse
watch(() => localSimulationSettings.simulationType, (newType) => {
  info(`[KeySettings.vue] 切换模拟类型到: ${newType}`);
  if (newType === 'mouse') {
    // 切换到鼠标模拟时,重新从配置加载 noteToMouse
    const settings = settingsManager.getSettings();
    localSimulationSettings.noteToMouse = { ...(settings.simulationSettings?.noteToMouse || {}) };
    info(`[KeySettings.vue] 重新加载 noteToMouse: ${JSON.stringify(localSimulationSettings.noteToMouse)}`);
  }
});


// 监听下拉框变化更新设置
watch([minNoteGroup, minNoteValue], ([newGroup, newNote]) => {
  if (newGroup && newNote) {
    const [lo, hi] = GROUPS[newGroup];
    if (newNote < lo || newNote > hi) {
      minNoteValue.value = lo;
      localAnalyzerSettings.minNote = lo;
    } else {
      localAnalyzerSettings.minNote = newNote;
    }
  }
});

watch([maxNoteGroup, maxNoteValue], ([newGroup, newNote]) => {
  if (newGroup && newNote) {
    const [lo, hi] = GROUPS[newGroup];
    if (newNote < lo || newNote > hi) {
      maxNoteValue.value = hi;
      localAnalyzerSettings.maxNote = hi;
    } else {
      localAnalyzerSettings.maxNote = newNote;
    }
  }
});

// 计算需要显示的音符列表
const displayNoteGroups = computed(() => {
  const groups: { name: string; notes: { note: number; name: string; key: string }[] }[] = [];
  const min = localAnalyzerSettings.minNote;
  const max = localAnalyzerSettings.maxNote;
  const blackKeyMode = localAnalyzerSettings.blackKeyMode;

  for (const [groupName, [lo, hi]] of Object.entries(GROUPS)) {
    if (hi < min || lo > max) continue;

    const notesInGroup: { note: number; name: string; key: string }[] = [];
    const start = Math.max(lo, min);
    const end = Math.min(hi, max);

    for (let i = start; i <= end; i++) {
      if (blackKeyMode === 'auto_sharp') {
        const noteIndex = i % 12;
        const isBlackKey = [1, 3, 6, 8, 10].includes(noteIndex);
        if (isBlackKey) continue;
      }

      const noteName = getNoteName(i);
      const key = localSimulationSettings.noteToKey[i] !== undefined
        ? localSimulationSettings.noteToKey[i]
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

  groups.sort((a, b) => a.notes[0].note - b.notes[0].note);
  return groups;
});

// 更新按键映射
const updateNoteKey = (note: number, key: string) => {
  localSimulationSettings.noteToKey[note] = key;
};

// 应用预设配置
const applyPresetConfig = (presetId: string) => {
  info("[KeySettings.vue] 应用预设配置: " + presetId);
  const preset = presetConfigs.find(p => p.id === presetId);
  if (preset) {
    localAnalyzerSettings.minNote = preset.minNote;
    localAnalyzerSettings.maxNote = preset.maxNote;
    localAnalyzerSettings.blackKeyMode = preset.blackKeyMode;
    localAnalyzerSettings.trimLongNotes = preset.trimLongNotes;

    if (preset.simulationType) {
      localSimulationSettings.simulationType = preset.simulationType as 'keyboard' | 'mouse';
    }

    localSimulationSettings.noteToKey = { ...preset.noteToKey };

    updateDropdownsFromNote('min', preset.minNote);
    updateDropdownsFromNote('max', preset.maxNote);
  }
};

// 格式化鼠标坐标显示
const formatMouseCoordinate = (note: number): string => {
  const coord = localSimulationSettings.noteToMouse[note];
  if (coord) {
    return `(${coord.x}, ${coord.y})`;
  }
  return '';
};

// 选择鼠标坐标
const pickingNote = ref<number | null>(null);

const pickCoordinate = async (note: number) => {
  try {
    info(`[KeySettings.vue] 开始选择音符${note}的坐标`);
    pickingNote.value = note;

    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    const { invoke } = await import('@tauri-apps/api/core');

    const appWindow = getCurrentWindow();
    await appWindow.minimize();
    await new Promise(resolve => setTimeout(resolve, 1000));

    const result = await invoke<[number, number]>('pick_mouse_coordinate');
    const [x, y] = result;

    info(`[KeySettings.vue] 获取到鼠标位置: (${x}, ${y})`);
    localSimulationSettings.noteToMouse[note] = { x, y };

    await appWindow.unminimize();
    pickingNote.value = null;

    info(`[KeySettings.vue] 音符${note}的坐标已设置为: (${x}, ${y})`);
  } catch (e) {
    error(`[KeySettings.vue] 选择坐标失败: ${e}`);
    pickingNote.value = null;

    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().unminimize();
    } catch { }
  }
};

// 保存设置
const saveSettings = async () => {
  const oldSettings = settingsManager.getSettings();
  const oldAnalyzerSetting = oldSettings.analyzerSetting || {};

  const fullNoteToKey: Record<number, string> = {};
  const min = localAnalyzerSettings.minNote;
  const max = localAnalyzerSettings.maxNote;
  const blackKeyMode = localAnalyzerSettings.blackKeyMode;

  for (let i = min; i <= max; i++) {
    if (blackKeyMode === 'auto_sharp') {
      const noteIndex = i % 12;
      const isBlackKey = [1, 3, 6, 8, 10].includes(noteIndex);
      if (isBlackKey) continue;
    }

    const key = localSimulationSettings.noteToKey[i] !== undefined
      ? localSimulationSettings.noteToKey[i]
      : (NOTE_TO_KEY[i] || '');

    if (key !== undefined) {
      fullNoteToKey[i] = key;
    }
  }

  localSimulationSettings.noteToKey = fullNoteToKey;

  // 保留原有的 noteToMouse 配置(不清空鼠标坐标设置)
  const currentSettings = settingsManager.getSettings();

  // 如果当前是鼠标模拟,使用 localSimulationSettings 的 noteToMouse
  // 否则从配置文件读取以保留原有配置
  const noteToMouseToSave = localSimulationSettings.simulationType === 'mouse'
    ? localSimulationSettings.noteToMouse
    : (currentSettings.simulationSettings?.noteToMouse || {});

  const settings = {
    analyzerSetting: { ...localAnalyzerSettings },
    simulationSettings: {
      ...localSimulationSettings,
      noteToMouse: noteToMouseToSave
    }
  };

  // 调试日志
  info(`[KeySettings.vue] 保存配置 - simulationType: ${localSimulationSettings.simulationType}`);
  info(`[KeySettings.vue] 保存配置 - noteToMouse: ${JSON.stringify(noteToMouseToSave)}`);

  await settingsManager.saveSettings(settings);

  const analyzerSettingChanged =
    oldAnalyzerSetting.minNote !== localAnalyzerSettings.minNote ||
    oldAnalyzerSetting.maxNote !== localAnalyzerSettings.maxNote ||
    oldAnalyzerSetting.blackKeyMode !== localAnalyzerSettings.blackKeyMode ||
    oldAnalyzerSetting.trimLongNotes !== localAnalyzerSettings.trimLongNotes;

  info("[KeySettings.vue] 按键设置已保存");

  // 显示 Toast 通知
  toastMessage.value = "按键设置已保存!" + (analyzerSettingChanged ? "\n音域设置已变更,请重新选择MIDI文件以应用新设置。" : "");
  toastType.value = analyzerSettingChanged ? 'warning' : 'success';
  toastVisible.value = true;
};
</script>

<template>
  <div class="key-settings-view">
    <!-- Toast 通知 -->
    <Toast :visible="toastVisible" @update:visible="toastVisible = $event" :message="toastMessage" :type="toastType" />

    <!-- 保存按钮 -->
    <div class="top-actions">
      <button @click="saveSettings" class="btn btn-primary">
        保存设置
      </button>
    </div>

    <div class="settings-content">
      <!-- 预设配置 -->
      <div class="setting-section">
        <div class="preset-header">
          <h4 class="section-title">预设配置</h4>
          <div class="preset-buttons">
            <button v-for="preset in presetConfigs" :key="preset.id" class="btn btn-small"
              @click="applyPresetConfig(preset.id)">
              {{ preset.name }}
            </button>
          </div>
        </div>
      </div>

      <!-- 基础配置 -->
      <div class="setting-section">
        <h4 class="section-title">基础配置</h4>
        <div class="basic-config-container">
          <!-- 左侧:音域设置 -->
          <div class="range-settings">
            <!-- 最低音 -->
            <div class="range-control-group">
              <label>最低音:</label>
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
              <label>最高音:</label>
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

          <!-- 右侧:黑键设置 -->
          <div class="black-key-settings">
            <!-- 黑键模式开关 -->
            <div class="switch-control">
              <label class="switch">
                <input type="checkbox" :checked="localAnalyzerSettings.blackKeyMode === 'support_black_key'"
                  @change="e => localAnalyzerSettings.blackKeyMode = (e.target as HTMLInputElement).checked ? 'support_black_key' : 'auto_sharp'">
                <span class="slider round"></span>
              </label>
              <span class="switch-label">{{ localAnalyzerSettings.blackKeyMode === 'support_black_key' ? '支持黑键' : '黑键降音'
                }}</span>
            </div>

            <!-- 长音修剪开关 -->
            <div class="switch-control">
              <label class="switch">
                <input type="checkbox" v-model="localAnalyzerSettings.trimLongNotes">
                <span class="slider round"></span>
              </label>
              <span class="switch-label">长音修剪</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 模拟配置 -->
      <div class="setting-section">
        <h4 class="section-title">模拟配置</h4>

        <!-- 模拟类型页签 -->
        <div class="simulation-tabs">
          <div class="simulation-tab" :class="{ active: localSimulationSettings.simulationType === 'keyboard' }"
            @click="localSimulationSettings.simulationType = 'keyboard'">
            按键模拟
          </div>
          <div class="simulation-tab" :class="{ active: localSimulationSettings.simulationType === 'mouse' }"
            @click="localSimulationSettings.simulationType = 'mouse'">
            鼠标模拟
          </div>
        </div>

        <!-- 按键模拟内容 -->
        <div v-if="localSimulationSettings.simulationType === 'keyboard'" class="key-mapping-container">
          <div v-for="group in displayNoteGroups" :key="group.name" class="note-group">
            <div class="group-header">{{ group.name }}</div>
            <div class="group-notes">
              <div v-for="note in group.notes" :key="note.note" class="note-item-horizontal">
                <label>{{ note.name }}</label>
                <HotkeyInput 
                  :model-value="note.key"
                  @update:model-value="(val: string) => updateNoteKey(note.note, val)" 
                  placeholder="点击设置" 
                />
              </div>
            </div>
          </div>
        </div>

        <!-- 鼠标模拟内容 -->
        <div v-if="localSimulationSettings.simulationType === 'mouse'" class="key-mapping-container">
          <div v-for="group in displayNoteGroups" :key="group.name" class="note-group">
            <div class="group-header">{{ group.name }}</div>
            <div class="group-notes">
              <div v-for="note in group.notes" :key="note.note" class="note-item-horizontal"
                :class="{ 'picking': pickingNote === note.note }">
                <label>{{ note.name }}</label>
                <input type="text" class="key-input" :value="formatMouseCoordinate(note.note)" disabled
                  placeholder="未设置">
                <button class="btn-pick-coordinate" @click="pickCoordinate(note.note)"
                  :class="{ 'active': pickingNote === note.note }"
                  :title="pickingNote === note.note ? '点击任意位置设置坐标' : '选择坐标'">
                  <svg viewBox="0 0 1024 1024" xmlns="http://www.w3.org/2000/svg" class="coordinate-icon">
                    <path
                      d="M544 0v65.152A448.064 448.064 0 0 1 958.848 480L1024 480v64h-65.152A448.064 448.064 0 0 1 544 958.848V1024h-64v-65.152A448.064 448.064 0 0 1 65.216 544H0v-64h65.152A448.064 448.064 0 0 1 480 65.216L480 0h64z m0 129.28V192h-64v-62.72A384.064 384.064 0 0 0 129.28 480H192v64h-62.72a384.128 384.128 0 0 0 350.72 350.72V832h64v62.72a384.128 384.128 0 0 0 350.72-350.72H832v-64h62.72a384.128 384.128 0 0 0-335.232-349.12l-15.424-1.536zM512 320a192 192 0 1 1 0 384 192 192 0 0 1 0-384z m0 64a128 128 0 1 0 0 256 128 128 0 0 0 0-256z m0 64a64 64 0 1 1 0 128 64 64 0 0 1 0-128z">
                    </path>
                  </svg>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.key-settings-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  padding: 2rem;
  background-color: var(--bg);
  overflow-y: auto;
}

/* 顶部操作区 */
.top-actions {
  display: flex;
  justify-content: flex-end;
  margin-bottom: 1.5rem;
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
  margin: 0;
}

/* 预设配置 */
.preset-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}

.preset-buttons {
  display: flex;
  gap: 1rem;
  flex-wrap: wrap;
}

/* 基础配置布局 */
.basic-config-container {
  display: flex;
  gap: 2rem;
  align-items: flex-start;
  flex-wrap: wrap;
}

.range-settings {
  flex: 2;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  min-width: 300px;
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

.cascade-select {
  display: flex;
  gap: 0.5rem;
}

.setting-select {
  padding: 0.4rem;
  border: 1px solid var(--border);
  border-radius: 4px;
  background-color: var(--inputbg);
  color: var(--inputfg);
  font-size: 0.9rem;
}

.group-select {
  min-width: 120px;
}

.note-select {
  min-width: 100px;
}

.setting-select:focus {
  outline: none;
  border-color: var(--primary);
}

.black-key-settings {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
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

.slider.round {
  border-radius: 20px;
}

.slider.round:before {
  border-radius: 50%;
}

/* 模拟配置 */
.simulation-tabs {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.simulation-tab {
  padding: 0.5rem 1rem;
  border: 1px solid var(--border);
  border-radius: 4px;
  background-color: var(--inputbg);
  color: var(--inputfg);
  cursor: pointer;
  transition: all 0.2s ease;
}

.simulation-tab:hover {
  background-color: var(--active);
}

.simulation-tab.active {
  background-color: var(--primary);
  color: var(--selectfg);
  border-color: var(--primary);
}

/* 按键映射样式 */
.key-mapping-container {
  overflow-x: auto;
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
  flex-shrink: 0;
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
  padding-right: 0.5rem;
}

.note-item {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.note-item label {
  font-size: 0.8rem;
  color: var(--fg);
}

/* 水平排列的音符项 */
.note-item-horizontal {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 0.5rem;
}

.note-item-horizontal label {
  font-size: 0.8rem;
  color: var(--fg);
  min-width: 60px;
  flex-shrink: 0;
}

.key-input {
  padding: 0.3rem;
  border: 1px solid var(--border);
  border-radius: 4px;
  background-color: var(--inputbg);
  color: var(--inputfg);
  font-size: 0.85rem;
  max-width: 120px;
}

.key-input:focus {
  outline: none;
  border-color: var(--primary);
}

.key-input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-pick-coordinate {
  padding: 0.3rem;
  border: 1px solid var(--border);
  border-radius: 4px;
  background-color: var(--inputbg);
  color: var(--inputfg);
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
}

.btn-pick-coordinate:hover {
  background-color: var(--active);
}

.btn-pick-coordinate.active {
  background-color: var(--primary);
  border-color: var(--primary);
}

.coordinate-icon {
  width: 16px;
  height: 16px;
  fill: currentColor;
}

.note-item.picking,
.note-item-horizontal.picking {
  background-color: var(--active);
  padding: 0.25rem;
  border-radius: 4px;
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

.btn-small {
  padding: 0.35rem 0.75rem;
  font-size: 0.85rem;
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
