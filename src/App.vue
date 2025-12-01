<script setup lang="ts">
import { ref, onMounted, provide } from "vue";
import themeConfig from "./config/theme.json";
import LeftPanel from "./components/LeftPanel.vue";
import RightPanel from "./components/RightPanel.vue";
import settingsManager from "./utils/settingsManager";
import { error } from '@tauri-apps/plugin-log';

// 主题管理
const currentTheme = ref("default");
const themes = themeConfig.theme;
const colors = ref(themes.find(t => t.name === currentTheme.value) || themes[0]);

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
          error(`保存主题失败: ${err}`);
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

// 初始化主题
onMounted(async () => {
  try {
    // 从设置文件加载主题
    const savedTheme = await settingsManager.loadTheme();
    currentTheme.value = savedTheme;
    const theme = themes.find(t => t.name === savedTheme) || themes[0];
    colors.value = theme;
    updateCSSVariables(theme);
    } catch (err) {
      error(`加载主题失败: ${err}`);
    // 使用默认主题
    updateCSSVariables(colors.value);
  }
});

// 提供主题更新方法给子组件
provide('updateTheme', changeTheme);
provide('currentTheme', currentTheme);
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
      <LeftPanel />

      <!-- 右侧面板 -->
      <RightPanel />
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
</style>
