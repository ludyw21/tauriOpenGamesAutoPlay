<script setup lang="ts">
import { ref, onMounted, defineEmits, inject } from "vue";
import { Window } from '@tauri-apps/api/window';
import { info } from '@tauri-apps/plugin-log';
import { open } from '@tauri-apps/plugin-dialog'; // 引入 dialog API
import { readDir } from '@tauri-apps/plugin-fs'; // 引入 fs API

import { join } from '@tauri-apps/api/path';

// 从 App.vue 注入 settingsManager
const settingsManager = inject('settingsManager') as any;

// 左侧面板组件
const stayOnTop = ref(false);
info(`[LeftPanel.vue:55] stayOnTop变量初始化: ${stayOnTop.value}`);
const searchText = ref("搜索歌曲...");
const midiFiles = ref<string[]>(["测试歌曲-忘记时间-胡歌.mid"]);
const allMidiFiles = ref<string[]>(["测试歌曲-忘记时间-胡歌.mid"]); // 用于存储所有MIDI文件
const selectedFile = ref<string>("");
const currentFolderPath = ref<string>("");

const emit = defineEmits(['update:selectedSong']);

// 窗口置顶切换
const toggleStayOnTop = async () => {
  try {
    const currentWindow = Window.getCurrent();
    info('[LeftPanel.vue:66] 获取当前窗口实例成功');

    // 首先获取当前的实际状态
    const currentState = await currentWindow.isAlwaysOnTop();
    info(`[LeftPanel.vue:77] 获取到的当前实际窗口状态: ${currentState}`);

    // 计算目标状态（与当前状态相反）
    const targetState = !currentState;
    info(`[LeftPanel.vue:88] 准备切换窗口置顶状态到: ${targetState}`);

    // 设置新状态
    await currentWindow.setAlwaysOnTop(targetState);
    info(`[LeftPanel.vue:99] 窗口置顶状态已设置为: ${targetState}`);

    // 验证设置是否生效
    const verificationState = await currentWindow.isAlwaysOnTop();
    info(`[LeftPanel.vue:1010] 验证后的窗口置顶状态: ${verificationState}`);

    // 更新本地状态变量
    stayOnTop.value = verificationState;
    info(`[LeftPanel.vue:1111] 本地stayOnTop变量已更新为: ${stayOnTop.value}`);
  } catch (error) {
    info(`[LeftPanel.vue:1212] 切换窗口置顶状态失败: ${error}`);
    // 出错时重新获取实际状态以保持同步
    try {
      const currentWindow = Window.getCurrent();
      const actualState = await currentWindow.isAlwaysOnTop();
      stayOnTop.value = actualState;
      info(`[LeftPanel.vue:1313] 出错后同步的实际窗口状态: ${actualState}`);
    } catch (syncError) {
      info(`[LeftPanel.vue:1414] 同步实际状态失败: ${syncError}`);
    }
  }
};

// 组件挂载时检查窗口当前置顶状态
// 组件挂载时检查窗口当前置顶状态
onMounted(async () => {
  info('[LeftPanel.vue:1515] 组件挂载，开始检查窗口置顶状态和加载MIDI文件夹');
  try {
    const currentWindow = Window.getCurrent();
    info('[LeftPanel.vue:1616] 获取当前窗口实例成功');
    const currentState = await currentWindow.isAlwaysOnTop();
    info(`[LeftPanel.vue:1717] 获取到的初始窗口置顶状态: ${currentState}`);
    stayOnTop.value = currentState;

    // 等待 settingsManager 初始化完成
    await settingsManager.initialize();

    // 从内存加载保存的MIDI文件夹路径（同步调用）
    const savedFolderPath = settingsManager.loadMidiFolderPath();
    if (savedFolderPath) {
      info(`[LeftPanel.vue:1818] 加载到保存的MIDI文件夹路径: ${savedFolderPath}`);
      await loadMidiFiles(savedFolderPath);
    } else {
      info('[LeftPanel.vue:1919] 未找到保存的MIDI文件夹路径');
    }
  } catch (error) {
    info(`[LeftPanel.vue:2020] 组件挂载时操作失败: ${error}`);
  }
});

// 选择MIDI文件夹
const selectDirectory = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "选择MIDI文件夹",
    });

    if (selected) {
      const folderPath = selected as string;
      info(`[LeftPanel.vue:2121] 选择了文件夹: ${folderPath}`);
      // 保存文件夹路径到配置
      await settingsManager.saveMidiFolderPath(folderPath);
      await loadMidiFiles(folderPath);
    }
  } catch (error) {
    info(`[LeftPanel.vue:2222] 选择文件夹或读取文件失败: ${error}`);
  }
};

// 搜索歌曲
const filterSongs = () => {
  const query = searchText.value.toLowerCase();
  if (query === "" || query === "搜索歌曲...") {
    midiFiles.value = allMidiFiles.value;
  } else {
    midiFiles.value = allMidiFiles.value.filter(file =>
      file.toLowerCase().includes(query)
    );
  }
};

// 歌曲选中
const songSelected = async (file: string) => {
  selectedFile.value = file;
  if (currentFolderPath.value) {
    try {
      const fullPath = await join(currentFolderPath.value, file);
      emit('update:selectedSong', fullPath);
    } catch (e) {
      info(`[LeftPanel.vue:2323] 拼接路径失败: ${e}`);
      emit('update:selectedSong', file); // Fallback
    }
  } else {
    emit('update:selectedSong', file);
  }
};

// 加载MIDI文件列表
const loadMidiFiles = async (folderPath: string) => {
  try {
    currentFolderPath.value = folderPath;
    const entries = await readDir(folderPath);
    const midiFilesList = entries
      .filter((entry) => entry.name?.endsWith(".mid") || entry.name?.endsWith(".midi"))
      .map((entry) => entry.name as string)
      .sort(); // 按名称排序

    allMidiFiles.value = midiFilesList; // 更新所有MIDI文件列表
    midiFiles.value = allMidiFiles.value; // 将所有文件显示在列表中
    info(`[LeftPanel.vue:2424] 从 ${folderPath} 找到 ${midiFilesList.length} 个MIDI文件`);
  } catch (error) {
    info(`[LeftPanel.vue:2525] 读取MIDI文件失败: ${error}`);
    allMidiFiles.value = [];
    midiFiles.value = []; // 清空列表
  }
};

// 选择上一首歌曲
const selectPrevSong = async () => {
  if (midiFiles.value.length === 0) {
    info('[LeftPanel.vue] 没有可用的歌曲');
    return;
  }

  const currentIndex = midiFiles.value.indexOf(selectedFile.value);
  let prevIndex: number;

  if (currentIndex <= 0) {
    // 如果是第一首或未选中，则选择最后一首
    prevIndex = midiFiles.value.length - 1;
  } else {
    prevIndex = currentIndex - 1;
  }

  const prevFile = midiFiles.value[prevIndex];
  info(`[LeftPanel.vue] 选择上一首: ${prevFile}`);
  await songSelected(prevFile);
};

// 选择下一首歌曲
const selectNextSong = async () => {
  if (midiFiles.value.length === 0) {
    info('[LeftPanel.vue] 没有可用的歌曲');
    return;
  }

  const currentIndex = midiFiles.value.indexOf(selectedFile.value);
  let nextIndex: number;

  if (currentIndex >= midiFiles.value.length - 1 || currentIndex === -1) {
    // 如果是最后一首或未选中，则选择第一首
    nextIndex = 0;
  } else {
    nextIndex = currentIndex + 1;
  }

  const nextFile = midiFiles.value[nextIndex];
  info(`[LeftPanel.vue] 选择下一首: ${nextFile}`);
  await songSelected(nextFile);
};

// 暴露方法给父组件
defineExpose({
  selectPrevSong,
  selectNextSong
});

</script>

<template>
  <aside class="left-panel">
    <!-- 置顶复选框 -->
    <div class="top-section">
      <div class="checkbox-item">
        <input type="checkbox" id="stayOnTop" v-model="stayOnTop" @change="toggleStayOnTop" />
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
      <input type="text" v-model="searchText" @focus="searchText === '搜索歌曲...' && (searchText = '')"
        @keyup="filterSongs" placeholder="搜索歌曲..." />
    </div>

    <!-- 歌曲列表 -->
    <div class="song-list-section">
      <div class="song-list-header">
        <h3>歌曲列表</h3>
      </div>
      <div class="song-list">
        <div v-for="file in midiFiles" :key="file" class="song-item" :class="{ active: selectedFile === file }"
          @click="songSelected(file)">
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
  padding: 0.5rem 1rem;
  background-color: var(--bg);
  border-bottom: 1px solid var(--border);
}

.search-section {
  padding: 0.5rem 1rem;
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
  padding: 0.5rem 1rem;
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
  padding: 0.4rem 1rem;
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
