<script setup lang="ts">
import { ref } from "vue";
import EventTableDialog from "./dialogs/EventTableDialog.vue";
import SettingsDialog from "./dialogs/SettingsDialog.vue";
import HelpDialog from "./dialogs/HelpDialog.vue";

// 右侧面板组件
const currentMinNote = ref(48);
const currentMaxNote = ref(83);
const selectedMidiFile = ref("");
const remainingTime = ref("00:00");
const allTracksSelected = ref(true);
const tracks = ref([
  {
    id: 0,
    name: "钢琴",
    noteCount: 120,
    selected: true,
    transpose: 0,
    octave: 0,
    analysis: "音轨分析结果：包含C4到A5的音符，适合当前播放范围"
  }
]);

// 获取音符名称
const getNoteName = (note: number): string => {
  const noteNames = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
  const octave = Math.floor(note / 12) - 1;
  const noteIndex = note % 12;
  return `${noteNames[noteIndex]}${octave}`;
};

// 切换全选
const toggleSelectAll = () => {
  allTracksSelected.value = !allTracksSelected.value;
  tracks.value.forEach(track => {
    track.selected = allTracksSelected.value;
  });
};

// 切换音轨选择
const toggleTrackSelection = (trackId: number) => {
  const track = tracks.value.find(t => t.id === trackId);
  if (track) {
    track.selected = !track.selected;
    // 更新全选状态
    allTracksSelected.value = tracks.value.every(t => t.selected);
  }
};

// 调整移调
const adjustTranspose = (trackId: number, direction: number) => {
  const track = tracks.value.find(t => t.id === trackId);
  if (track) {
    track.transpose += direction;
  }
};

// 调整八度
const adjustOctave = (trackId: number, direction: number) => {
  const track = tracks.value.find(t => t.id === trackId);
  if (track) {
    track.octave += direction;
  }
};

// 重置转音设置
const resetTranspose = (trackId: number) => {
  const track = tracks.value.find(t => t.id === trackId);
  if (track) {
    track.transpose = 0;
    track.octave = 0;
  }
};

// 播放控制
const togglePlay = () => {
  // 实现播放/暂停逻辑
};

const stopPlayback = () => {
  // 实现停止逻辑
};

const togglePreview = () => {
  // 实现预览逻辑
};

const toggleMidiPlayback = () => {
  // 实现试听MIDI逻辑
};

// 对话框显示状态
const showEventTableDialog = ref(false);
const showSettingsDialog = ref(false);
const showHelpDialog = ref(false);

// 显示事件表
const showEventTable = () => {
  showEventTableDialog.value = true;
};

// 显示设置
const showSettings = () => {
  showSettingsDialog.value = true;
};

// 显示帮助
const showHelp = () => {
  showHelpDialog.value = true;
};

// 处理设置保存
const handleSettingsSaved = (settings: any) => {
  console.log("设置已保存:", settings);
  // 这里可以根据需要更新组件中的相关设置
};
</script>

<template>
  <section class="right-panel">
    <!-- 音轨详情区域 -->
    <div class="tracks-frame">
      <h3 class="frame-title">
        音轨详情【 当前播放范围：{{ getNoteName(currentMinNote) }}({{ currentMinNote }}) - {{ getNoteName(currentMaxNote) }}({{ currentMaxNote }}) 】
      </h3>
      
      <!-- 所有音轨复选框 -->
      <div class="all-tracks-section">
        <div class="checkbox-item">
          <input 
            type="checkbox" 
            id="allTracks" 
            v-model="allTracksSelected"
            @change="toggleSelectAll"
          />
          <label for="allTracks">所有音轨</label>
        </div>
      </div>
      
      <!-- 当前歌曲名称 -->
      <div class="current-song-section">
        <div class="song-info">
          <span class="label">当前歌曲：</span>
          <span class="value">{{ selectedMidiFile || "未选择" }}</span>
        </div>
      </div>
      
      <!-- 音轨列表 -->
      <div class="tracks-list-section">
        <div class="tracks-list">
          <div 
            v-for="track in tracks" 
            :key="track.id"
            class="track-item"
          >
            <!-- 音轨选择 -->
            <div class="track-selection">
              <input 
                type="checkbox" 
                :id="`track-${track.id}`" 
                v-model="track.selected"
                @change="toggleTrackSelection(track.id)"
              />
            </div>
            
            <!-- 音轨信息和分析 -->
            <div class="track-info">
              <div class="track-header">
                <h4 class="track-title">音轨{{ track.id + 1 }}：{{ track.name }} ({{ track.noteCount }}个音符)</h4>
              </div>
              <div class="track-analysis">
                <p>{{ track.analysis }}</p>
              </div>
            </div>
            
            <!-- 转音设置 -->
            <div class="transpose-settings">
              <div class="setting-group">
                <label>移调:</label>
                <div class="control-buttons">
                  <button class="btn btn-small" @click="adjustTranspose(track.id, -1)">-</button>
                  <input type="number" v-model.number="track.transpose" class="number-input" />
                  <button class="btn btn-small" @click="adjustTranspose(track.id, 1)">+</button>
                </div>
              </div>
              <div class="setting-group">
                <label>八度:</label>
                <div class="control-buttons">
                  <button class="btn btn-small" @click="adjustOctave(track.id, -1)">-</button>
                  <input type="number" v-model.number="track.octave" class="number-input" />
                  <button class="btn btn-small" @click="adjustOctave(track.id, 1)">+</button>
                </div>
              </div>
              <div class="reset-link" @click="resetTranspose(track.id)">
                重置
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
    
    <!-- 操作区域 -->
    <div class="operation-frame">
      <h3 class="frame-title">操作</h3>
      
      <!-- 剩余时间 -->
      <div class="time-section">
        <div class="time-label">剩余时间: {{ remainingTime }}</div>
      </div>
      
      <!-- 控制按钮 -->
      <div class="control-buttons-section">
        <button class="btn btn-success" @click="togglePlay">播放</button>
        <button class="btn btn-danger" @click="stopPlayback">停止</button>
        <button class="btn btn-info" @click="togglePreview">预览</button>
        <button class="btn btn-info" @click="toggleMidiPlayback">试听MIDI</button>
      </div>
    </div>
    
    <!-- 其他功能 -->
    <div class="other-frame">
      <h3 class="frame-title">其他</h3>
      
      <!-- 其他按钮 -->
      <div class="other-buttons-section">
        <button class="btn btn-secondary" @click="showEventTable">事件表</button>
        <button class="btn btn-secondary" @click="showSettings">设置</button>
        <button class="btn btn-secondary" @click="showHelp">帮助</button>
      </div>
    </div>
  </section>
  
  <!-- 对话框组件 -->
  <EventTableDialog 
    v-model:visible="showEventTableDialog"
  />
  <SettingsDialog 
    v-model:visible="showSettingsDialog"
    @settingsSaved="handleSettingsSaved"
  />
  <HelpDialog 
    v-model:visible="showHelpDialog"
  />
</template>

<style scoped>
.right-panel {
  flex: 1;
  background-color: var(--bg);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 0.5rem;
}

/* 通用框架样式 */
.tracks-frame,
.operation-frame,
.other-frame {
  background-color: var(--bg);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 0.75rem;
  margin-bottom: 0.75rem;
}

.frame-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--fg);
  margin: 0 0 0.75rem 0;
}

/* 音轨详情区域 */
.all-tracks-section {
  margin-bottom: 0.75rem;
}

.checkbox-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.checkbox-item input[type="checkbox"] {
  accent-color: var(--primary);
}

.checkbox-item label {
  font-size: 0.9rem;
  color: var(--fg);
  cursor: pointer;
}

.current-song-section {
  margin-bottom: 0.75rem;
}

.song-info {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.song-info .label {
  font-size: 0.9rem;
  color: var(--secondary);
}

.song-info .value {
  font-size: 0.9rem;
  color: var(--fg);
  font-weight: 500;
}

/* 音轨列表 */
.tracks-list-section {
  max-height: 300px;
  overflow-y: auto;
  background-color: var(--bg);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 0.5rem;
}

.track-item {
  display: flex;
  gap: 0.75rem;
  padding: 0.75rem;
  background-color: var(--active);
  border: 1px solid var(--border);
  border-radius: 4px;
  margin-bottom: 0.5rem;
}

.track-item:last-child {
  margin-bottom: 0;
}

.track-selection {
  display: flex;
  align-items: flex-start;
  padding-top: 0.25rem;
}

.track-info {
  flex: 1;
}

.track-header {
  margin-bottom: 0.5rem;
}

.track-title {
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--fg);
  margin: 0;
}

.track-analysis {
  font-size: 0.8rem;
  color: var(--fg);
  background-color: var(--bg);
  padding: 0.5rem;
  border-radius: 4px;
  border: 1px solid var(--border);
  max-height: 60px;
  overflow-y: auto;
}

.track-analysis p {
  margin: 0;
}

/* 转音设置 */
.transpose-settings {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  min-width: 150px;
}

.setting-group {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.setting-group label {
  font-size: 0.8rem;
  color: var(--secondary);
  font-weight: 500;
}

.control-buttons {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.btn-small {
  padding: 0.25rem 0.5rem;
  font-size: 0.8rem;
  min-width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border);
  border-radius: 4px;
  background-color: var(--inputbg);
  color: var(--inputfg);
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-small:hover {
  background-color: var(--primary);
  border-color: var(--primary);
  color: var(--selectfg);
}

.number-input {
  width: 50px;
  padding: 0.25rem;
  border: 1px solid var(--border);
  border-radius: 4px;
  background-color: var(--inputbg);
  color: var(--inputfg);
  font-size: 0.8rem;
  text-align: center;
}

.number-input:focus {
  outline: none;
  border-color: var(--primary);
}

.reset-link {
  font-size: 0.8rem;
  color: var(--primary);
  text-decoration: underline;
  cursor: pointer;
  margin-top: 0.25rem;
  align-self: flex-start;
}

.reset-link:hover {
  color: var(--light);
}

/* 操作区域 */
.time-section {
  margin-bottom: 0.75rem;
}

.time-label {
  font-size: 0.9rem;
  color: var(--fg);
  text-align: center;
  font-weight: 500;
}

.control-buttons-section {
  display: flex;
  gap: 0.5rem;
  justify-content: center;
}

/* 其他功能区域 */
.other-buttons-section {
  display: flex;
  gap: 0.5rem;
  justify-content: center;
}

/* 按钮样式 */
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0.5rem 1rem;
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 0.9rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  flex: 1;
  max-width: 120px;
}

/* 播放控制按钮 */
.btn-success {
  background-color: var(--success);
  color: var(--selectfg);
  border-color: var(--success);
}

.btn-danger {
  background-color: var(--danger);
  color: var(--selectfg);
  border-color: var(--danger);
}

.btn-info {
  background-color: var(--info);
  color: var(--selectfg);
  border-color: var(--info);
}

/* 其他区域按钮 - 和预览按钮颜色一致 */
.other-buttons-section .btn {
  background-color: var(--info);
  color: var(--selectfg);
  border-color: var(--info);
}

/* 悬停效果 */
.btn:hover {
  background-color: var(--dark);
  border-color: var(--dark);
}
</style>
