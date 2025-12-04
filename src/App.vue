<script setup lang="ts">
import { ref, onMounted, onUnmounted, provide } from "vue";
import themeConfig from "./config/theme.json";
import LeftPanel from "./components/LeftPanel.vue";
import RightPanel from "./components/RightPanel.vue";
import settingsManager from "./utils/settingsManager";
import shortcutService from "./services/shortcutService";
import { error, info } from '@tauri-apps/plugin-log';

// 主题管理
const currentTheme = ref("default");
const selectedMidiFile = ref<string | null>(null);
const themes = themeConfig.theme;
const colors = ref(themes.find(t => t.name === currentTheme.value) || themes[0]);

// 组件引用
const leftPanelRef = ref<InstanceType<typeof LeftPanel> | null>(null);
const rightPanelRef = ref<InstanceType<typeof RightPanel> | null>(null);

// 切换主题
const changeTheme = async (themeName: string) => {
  currentTheme.value = themeName;
  const theme = themes.find(t => t.name === themeName);
  if (theme) {
    colors.value = theme;
    updateCSSVariables(theme);

    // 保存主题设置
    try {
      await settingsManager.saveTheme(themeName);
    } catch (err) {
      error(`[App.vue:21] 保存主题失败: ${err}`);
    }
  }
};

// 更新CSS变量
const updateCSSVariables = (theme: any) => {
  const root = document.documentElement;
  Object.entries(theme).forEach(([key, value]) => {
    if (key !== "name") {
      root.style.setProperty(`--${key}`, value as string);
    }
  });
};

// 初始化应用
onMounted(async () => {
  try {
    info('[App.vue:32] 开始初始化应用...');

    // 1. 初始化配置管理器（只调用一次，从文件读取配置到内存）
    await settingsManager.initialize();
    info('[App.vue:43] 配置管理器初始化完成');

    // 2. 从内存加载主题
    const savedTheme = settingsManager.loadTheme();
    info(`[App.vue:54] 加载的主题: ${savedTheme}`);
    currentTheme.value = savedTheme;
    const theme = themes.find(t => t.name === savedTheme) || themes[0];
    colors.value = theme;
    updateCSSVariables(theme);

    // 3. 注册全局快捷键
    const settings = settingsManager.getSettings();
    const shortcuts = settings.shortcuts || {};
    info(`[App.vue] 准备注册全局快捷键: ${JSON.stringify(shortcuts)}`);

    await shortcutService.registerShortcuts(shortcuts, {
      onStartPause: () => {
        info('[App.vue] 快捷键触发: 开始/暂停');
        rightPanelRef.value?.togglePlay();
      },
      onStop: () => {
        info('[App.vue] 快捷键触发: 停止');
        rightPanelRef.value?.stopPlayback();
      },
      onPrevSong: async () => {
        info('[App.vue] 快捷键触发: 上一首');
        // 先停止当前播放
        await rightPanelRef.value?.stopPlayback();
        // 选择上一首
        await leftPanelRef.value?.selectPrevSong();
        // 延迟一下再开始播放，确保歌曲已切换
        setTimeout(() => {
          rightPanelRef.value?.togglePlay();
        }, 100);
      },
      onNextSong: async () => {
        info('[App.vue] 快捷键触发: 下一首');
        // 先停止当前播放
        await rightPanelRef.value?.stopPlayback();
        // 选择下一首
        await leftPanelRef.value?.selectNextSong();
        // 延迟一下再开始播放，确保歌曲已切换
        setTimeout(() => {
          rightPanelRef.value?.togglePlay();
        }, 100);
      }
    });

    info('[App.vue:65] 应用初始化完成');
  } catch (err) {
    error(`[App.vue:76] 初始化失败: ${err}`);
    // 使用默认主题
    updateCSSVariables(colors.value);
  }
});

// 组件卸载时注销快捷键
onUnmounted(async () => {
  try {
    info('[App.vue] 开始注销全局快捷键...');
    await shortcutService.unregisterAll();
    info('[App.vue] 全局快捷键已注销');
  } catch (err) {
    error(`[App.vue] 注销快捷键失败: ${err}`);
  }
});

// 提供主题更新方法和配置管理器给子组件
provide('updateTheme', changeTheme);
provide('currentTheme', currentTheme);
provide('settingsManager', settingsManager);
</script>

<template>
  <div class="app-container">
    <!-- 顶部工具栏
    <header class="app-header">
      <h1>OpenGamesAutoPlay</h1>
      <div class="theme-selector">
        <select v-model="currentTheme" @change="changeTheme(currentTheme)">
          <option v-for="theme in themes" :key="theme.name" :value="theme.name">
            {{ theme.name === 'default' ? '默认' : theme.name === 'dark' ? '黑夜' : '少女粉' }}
          </option>
        </select>
      </div>
    </header>
    -->
    <!-- 主内容区 -->
    <main class="main-content">
      <!-- 左侧面板 -->
      <LeftPanel ref="leftPanelRef" @update:selectedSong="selectedMidiFile = $event" />

      <!-- 右侧面板 -->
      <RightPanel ref="rightPanelRef" :selectedMidiFile="selectedMidiFile" />
    </main>
  </div>
</template>

<style>
:root {
  /* 默认主题变量，将被JavaScript动态替换 */
  --primary: #007bff;
  --secondary: #6c757d;
  --success: #28a745;
  --info: #17a2b8;
  --warning: #ffc107;
  --danger: #dc3545;
  --light: #f8f9fa;
  --dark: #343a40;
  --bg: #ffffff;
  --fg: #343a40;
  --selectbg: #007bff;
  --selectfg: #ffffff;
  --border: #dee2e6;
  --inputfg: #343a40;
  --inputbg: #ffffff;
  --active: #e9ecef;

  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

body {
  background-color: var(--bg);
  color: var(--fg);
  transition: background-color 0.3s, color 0.3s;
}

.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
}

.app-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 1rem;
  background-color: var(--primary);
  color: var(--selectfg);
  border-bottom: 1px solid var(--border);
  height: 60px;
}

.app-header h1 {
  font-size: 1.5rem;
  font-weight: 600;
}

.theme-selector select {
  padding: 0.3rem 0.6rem;
  border: 1px solid var(--border);
  border-radius: 4px;
  background-color: var(--inputbg);
  color: var(--inputfg);
  font-size: 0.9rem;
  cursor: pointer;
}

.main-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.left-panel {
  width: 250px;
  background-color: var(--light);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.right-panel {
  flex: 1;
  background-color: var(--bg);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.panel-header {
  padding: 0.5rem 1rem;
  background-color: var(--active);
  border-bottom: 1px solid var(--border);
  font-weight: 600;
}

.panel-header h2 {
  font-size: 1.1rem;
}

.panel-content {
  padding: 0.5rem 1rem;
  flex: 1;
  overflow-y: auto;
}

/* 滚动条样式 */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: var(--light);
}

::-webkit-scrollbar-thumb {
  background: var(--secondary);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--primary);
}

/* 选择样式 */
::selection {
  background-color: var(--selectbg);
  color: var(--selectfg);
}

/* 禁用按钮样式 */
.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  background-color: var(--secondary) !important;
  color: var(--light) !important;
}
</style>
