<script setup lang="ts">
import { ref } from "vue";

// 左侧面板组件
const stayOnTop = ref(false);
const searchText = ref("搜索歌曲...");
const midiFiles = ref<string[]>(["测试歌曲-忘记时间-胡歌.mid"]);
const selectedFile = ref<string>("");

// 窗口置顶切换
const toggleStayOnTop = () => {
  stayOnTop.value = !stayOnTop.value;
  // 这里需要调用Tauri API来实现窗口置顶
};

// 选择MIDI文件夹
const selectDirectory = () => {
  // 这里需要调用Tauri API来选择文件夹
};

// 搜索歌曲
const filterSongs = () => {
  // 实现搜索逻辑
};

// 歌曲选中
const songSelected = (file: string) => {
  selectedFile.value = file;
  // 通知父组件更新选中的MIDI文件
};
</script>

<template>
  <aside class="left-panel">
    <!-- 置顶复选框 -->
    <div class="top-section">
      <div class="checkbox-item">
        <input 
          type="checkbox" 
          id="stayOnTop" 
          v-model="stayOnTop"
          @change="toggleStayOnTop"
        />
        <label for="stayOnTop">窗口置顶</label>
      </div>
    </div>
    
    <!-- 文件选择按钮 -->
    <div class="file-select-section">
      <button class="btn btn-primary" @click="selectDirectory">
        选择MIDI文件夹
      </button>
    </div>
    
    <!-- 搜索框 -->
    <div class="search-section">
      <input 
        type="text" 
        v-model="searchText"
        @focus="searchText === '搜索歌曲...' && (searchText = '')"
        @keyup="filterSongs"
        placeholder="搜索歌曲..."
      />
    </div>
    
    <!-- 歌曲列表 -->
    <div class="song-list-section">
      <div class="song-list-header">
        <h3>歌曲列表</h3>
      </div>
      <div class="song-list">
        <div 
          v-for="file in midiFiles" 
          :key="file"
          class="song-item"
          :class="{ active: selectedFile === file }"
          @click="songSelected(file)"
        >
          <span class="song-name">{{ file }}</span>
        </div>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.left-panel {
  width: 220px;
  background-color: var(--light);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.top-section {
  padding: 0.5rem 1rem;
  background-color: var(--active);
  border-bottom: 1px solid var(--border);
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

.file-select-section {
  padding: 0.75rem 1rem;
  background-color: var(--bg);
  border-bottom: 1px solid var(--border);
}

.search-section {
  padding: 0.75rem 1rem;
  background-color: var(--bg);
  border-bottom: 1px solid var(--border);
}

.search-section input {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid var(--border);
  border-radius: 4px;
  background-color: var(--inputbg);
  color: var(--inputfg);
  font-size: 0.9rem;
}

.search-section input:focus {
  outline: none;
  border-color: var(--primary);
}

.song-list-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.song-list-header {
  padding: 0.5rem 1rem;
  background-color: var(--active);
  border-bottom: 1px solid var(--border);
  font-weight: 600;
}

.song-list-header h3 {
  font-size: 0.9rem;
  margin: 0;
  color: var(--fg);
}

.song-list {
  flex: 1;
  overflow-y: auto;
  background-color: var(--bg);
}

.song-item {
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--border);
  cursor: pointer;
  transition: background-color 0.2s ease;
  font-size: 0.9rem;
  color: var(--fg);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.song-item:hover {
  background-color: var(--active);
}

.song-item.active {
  background-color: var(--primary);
  color: var(--selectfg);
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
  width: 100%;
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
