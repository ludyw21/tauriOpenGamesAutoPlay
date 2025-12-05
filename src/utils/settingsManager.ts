import { appDataDir, configDir } from '@tauri-apps/api/path';
import { exists, readTextFile, writeTextFile, mkdir } from '@tauri-apps/plugin-fs';
import { info, error, warn } from '@tauri-apps/plugin-log';
import { platform } from '@tauri-apps/plugin-os';
import { CONTROL_KEYS } from '../config/keyboard_mapping';

let osPlatform = 'windows';
// 获取当前操作系统平台
async function initializePlatformModule() {
  try {
    osPlatform = platform();
    info('[settingsManager.ts:11] platform模块加载成功，操作系统: ' + osPlatform);
  } catch (err) {
    warn('[settingsManager.ts:14] 无法加载Tauri OS插件，将使用默认配置路径: ' + err);
    osPlatform = 'windows';
  }
}

interface AppSettings {
  themeSettings: {
    currentTheme: string;
  };
  analyzerSetting?: {
    minNote: number;
    maxNote: number;
    blackKeyMode: string;
    trimLongNotes: boolean;
  };
  simulationSettings?: {
    simulationType: 'keyboard' | 'mouse';
    noteToKey: Record<number, string>;
    noteToMouse?: Record<number, { x: number; y: number }>;
  };
  shortcuts?: Record<string, string>;
  midiFolderPath?: string;
}

class SettingsManager {
  private static instance: SettingsManager;

  private configPath: string | null = null;
  private settings: AppSettings; // 全局配置变量，始终在内存中
  private isInitialized: boolean = false; // 标记是否已初始化
  private initPromise: Promise<void> | null = null; // 初始化锁

  private defaultSettings: AppSettings = {
    themeSettings: {
      currentTheme: 'default'
    },
    analyzerSetting: {
      minNote: 48,
      maxNote: 83,
      blackKeyMode: 'support_black_key',
      trimLongNotes: false
    },
    simulationSettings: {
      simulationType: 'keyboard',
      noteToKey: {}
    },
    shortcuts: { ...CONTROL_KEYS },
    midiFolderPath: undefined,
  };

  private constructor() {
    // 初始化时使用默认设置
    this.settings = { ...this.defaultSettings };
  }

  public static getInstance(): SettingsManager {
    if (!SettingsManager.instance) {
      SettingsManager.instance = new SettingsManager();
    }
    return SettingsManager.instance;
  }

  // 公开的初始化方法，只在 App.vue 中调用一次
  public async initialize(): Promise<void> {
    if (this.isInitialized) {
      info('[settingsManager.ts] 已经初始化过，跳过');
      return;
    }

    if (this.initPromise !== null) {
      info('[settingsManager.ts] 等待现有初始化完成...');
      return this.initPromise;
    }

    info('[settingsManager.ts] 开始初始化配置管理器...');
    this.initPromise = this.doInitialize();

    try {
      await this.initPromise;
      this.isInitialized = true;
      info('[settingsManager.ts] 配置管理器初始化完成');
    } finally {
      this.initPromise = null;
    }
  }

  private async doInitialize(): Promise<void> {
    try {
      // 1. 初始化配置路径
      await this.initializeConfigPath();

      // 2. 从文件加载配置（只读取一次）
      await this.loadFromFile();

    } catch (err) {
      error(`[settingsManager.ts] 初始化失败: ${err}`);
      // 即使失败，也使用默认配置
      this.settings = { ...this.defaultSettings };
    }
  }

  private async initializeConfigPath(): Promise<void> {
    try {
      info('[settingsManager.ts] 初始化配置路径...');

      // 获取操作系统信息
      try {
        await initializePlatformModule();
      } catch (platformError) {
        error(`[settingsManager.ts] 加载platform模块失败: ${platformError}`);
      }

      // 获取应用数据目录
      try {
        let baseDir: string;

        try {
          info('[settingsManager.ts] 尝试使用appDataDir...');
          baseDir = await appDataDir();
        } catch (appDataError) {
          warn('[settingsManager.ts] 无法获取appDataDir，尝试使用configDir');
          try {
            baseDir = await configDir();
          } catch (configDirError) {
            warn('[settingsManager.ts] 无法获取configDir，使用备用路径');
            baseDir = '/tmp';
          }
        }

        info(`[settingsManager.ts] 获取到的基础目录: ${baseDir}`);
        this.configPath = `${baseDir}/config.json`;
        info(`[settingsManager.ts] 配置路径: ${this.configPath}`);

      } catch (dirError) {
        error(`[settingsManager.ts] 获取目录失败: ${dirError}`);
        this.configPath = '/tmp/tauriOpenGamesAutoPlay/config.json';
        warn(`[settingsManager.ts] 使用备用配置路径: ${this.configPath}`);
      }

    } catch (err) {
      error(`[settingsManager.ts] 初始化配置路径失败: ${err}`);
      this.configPath = '/tmp/tauriOpenGamesAutoPlay/config.json';
      info(`[settingsManager.ts] 设置备用配置路径: ${this.configPath}`);
    }
  }

  // 从文件加载配置（只在初始化时调用一次）
  private async loadFromFile(): Promise<void> {
    try {
      info('[settingsManager.ts] 从文件加载配置...');

      if (!this.configPath) {
        warn('[settingsManager.ts] 配置路径未设置，使用默认配置');
        return;
      }

      const fileExists = await exists(this.configPath);
      info(`[settingsManager.ts] 配置文件是否存在: ${fileExists}`);

      if (fileExists) {
        try {
          info(`[settingsManager.ts] 正在读取配置文件: ${this.configPath}`);
          const fileContent = await readTextFile(this.configPath);
          info(`[settingsManager.ts] 读取的配置文件内容: ${fileContent}`);

          const parsedSettings = JSON.parse(fileContent) as AppSettings;

          // 合并到全局配置变量
          this.settings = {
            themeSettings: { ...this.defaultSettings.themeSettings, ...(parsedSettings.themeSettings || {}) },
            analyzerSetting: parsedSettings.analyzerSetting ? {
              ...this.defaultSettings.analyzerSetting,
              ...parsedSettings.analyzerSetting
            } : this.defaultSettings.analyzerSetting,
            simulationSettings: parsedSettings.simulationSettings ? {
              ...this.defaultSettings.simulationSettings,
              ...parsedSettings.simulationSettings
            } : this.defaultSettings.simulationSettings,
            shortcuts: { ...(this.defaultSettings.shortcuts || {}), ...(parsedSettings.shortcuts || {}) },
            midiFolderPath: parsedSettings.midiFolderPath
          };

          info(`[settingsManager.ts] 配置加载成功: ${JSON.stringify(this.settings)}`);
        } catch (parseError) {
          error(`[settingsManager.ts] 解析配置文件失败: ${parseError}`);
          // 保持默认配置
        }
      } else {
        info('[settingsManager.ts] 配置文件不存在，使用默认配置');
      }
    } catch (err) {
      error(`[settingsManager.ts] 从文件加载配置失败: ${err}`);
    }
  }

  async saveSettings(settings: Partial<AppSettings>): Promise<void> {
    try {
      info(`[settingsManager.ts] 开始保存设置: ${JSON.stringify(settings)}`);

      // 1. 更新内存中的配置
      this.settings = {
        ...this.settings,
        ...settings,
        // 深度合并嵌套对象
        themeSettings: {
          ...this.settings.themeSettings,
          ...(settings.themeSettings || {})
        },
        analyzerSetting: settings.analyzerSetting ? {
          ...this.settings.analyzerSetting,
          ...settings.analyzerSetting
        } : this.settings.analyzerSetting,
        simulationSettings: settings.simulationSettings ? {
          ...this.settings.simulationSettings,
          ...settings.simulationSettings
        } : this.settings.simulationSettings,
        shortcuts: {
          ...this.settings.shortcuts,
          ...(settings.shortcuts || {})
        },
        midiFolderPath: settings.midiFolderPath !== undefined ? settings.midiFolderPath : this.settings.midiFolderPath,
      };
      info(`[settingsManager.ts] 内存配置已更新: ${JSON.stringify(this.settings)}`);

      // 2. 写入文件
      await this.saveToFile();

    } catch (err) {
      error(`[settingsManager.ts] 保存设置失败: ${err}`);
      info('[settingsManager.ts] 配置保存过程中出现错误，但内存配置已更新');
    }
  }

  // 将配置写入文件
  private async saveToFile(): Promise<void> {
    try {
      if (!this.configPath) {
        warn('[settingsManager.ts] 配置路径未设置，无法保存到文件');
        return;
      }

      const directory = this.configPath.substring(0, this.configPath.lastIndexOf('/'));
      info(`[settingsManager.ts] 配置目录路径: ${directory}`);

      // 确保目录存在
      try {
        const dirExists = await exists(directory);
        info(`[settingsManager.ts] 目录是否存在: ${dirExists}`);
        if (!dirExists) {
          info('[settingsManager.ts] 创建目录...');
          await mkdir(directory, { recursive: true });
          info('[settingsManager.ts] 目录创建成功');
        }
      } catch (dirError) {
        warn(`[settingsManager.ts] 创建目录失败: ${dirError}`);
      }

      // 写入文件
      info(`[settingsManager.ts] 准备写入配置文件: ${this.configPath}`);
      await writeTextFile(this.configPath, JSON.stringify(this.settings, null, 2));
      info('[settingsManager.ts] 配置文件写入成功!');

    } catch (err) {
      error(`[settingsManager.ts] 写入配置文件失败: ${err}`);
      info('[settingsManager.ts] 配置已在内存中更新，但无法保存到文件系统');
    }
  }

  // 获取当前配置（从内存读取）
  getSettings(): AppSettings {
    return { ...this.settings };
  }

  // 为了向后兼容，保留 loadSettings 方法，但直接从内存返回
  async loadSettings(): Promise<AppSettings> {
    info('[settingsManager.ts] 从内存加载设置');
    return this.getSettings();
  }

  async saveTheme(theme: string): Promise<void> {
    try {
      info(`[settingsManager.ts] 保存主题: ${theme}`);

      // 验证主题值
      if (!theme || typeof theme !== 'string') {
        warn('[settingsManager.ts] 无效的主题值，使用默认值');
        theme = this.defaultSettings.themeSettings.currentTheme;
      }

      // 调用saveSettings保存主题
      await this.saveSettings({ themeSettings: { currentTheme: theme } });
      info(`[settingsManager.ts] 主题保存成功: ${theme}`);
    } catch (err) {
      error(`[settingsManager.ts] 保存主题过程中发生错误: ${err}`);
      info('[settingsManager.ts] 主题保存失败，但应用将继续运行');
    }
  }

  // 从内存读取主题
  loadTheme(): string {
    info('[settingsManager.ts] 从内存加载主题');
    const theme = this.settings?.themeSettings?.currentTheme;

    if (!theme || typeof theme !== 'string') {
      warn('[settingsManager.ts] 无效的主题值，使用默认主题');
      return this.defaultSettings.themeSettings.currentTheme;
    }

    info(`[settingsManager.ts] 加载的主题: ${theme}`);
    return theme;
  }

  async saveMidiFolderPath(path: string): Promise<void> {
    await this.saveSettings({ midiFolderPath: path });
  }

  // 从内存读取 MIDI 文件夹路径
  loadMidiFolderPath(): string | undefined {
    info('[settingsManager.ts] 从内存加载MIDI文件夹路径');
    return this.settings.midiFolderPath;
  }
}

// 导出单例
export default SettingsManager.getInstance();
