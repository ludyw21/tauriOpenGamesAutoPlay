import { appDataDir, configDir } from '@tauri-apps/api/path';
import { exists, readTextFile, writeTextFile, mkdir } from '@tauri-apps/plugin-fs';
import { info, error, warn } from '@tauri-apps/plugin-log';
import { platform } from '@tauri-apps/plugin-os';

let osPlatform = 'windows';
// 获取当前操作系统平台
async function initializePlatformModule() {
  try {
    osPlatform = platform();
    info(`platform模块加载成功，操作系统: ${osPlatform}`);
  } catch (err) {
      warn(`无法加载Tauri OS插件，将使用默认配置路径: ${err}`);
    osPlatform = 'windows';
  }
}

interface AppSettings {
  themeSettings: {
    currentTheme: string;
  };
  keySettings?: {
    minNote: number;
    maxNote: number;
    blackKeyMode: string;
    noteToKey: Record<number, string>;
  };
  shortcuts?: Record<string, string>;
  midiFolderPath?: string; // Add this line
}

class SettingsManager {
  private configPath: string | null = null;
  private defaultSettings: AppSettings = {
    themeSettings: {
      currentTheme: 'default'
    },
    shortcuts: {},
    midiFolderPath: undefined, // Initialize the new property
  };

  constructor() {
    // 异步初始化，但不在构造函数中等待
    this.initializeConfigPath().catch(err => {
        error(`构造函数中初始化配置路径失败: ${err}`);
    });
  }

  private async initializeConfigPath(): Promise<void> {
    try {
        info('初始化配置路径...');
      
      // 首先尝试获取操作系统信息
      try {
        await initializePlatformModule();
      } catch (platformError) {
          error(`加载platform模块失败: ${platformError}`);
        // 继续执行，不中断初始化过程
      }
      
      // 尝试获取应用数据目录
      try {
        let baseDir: string;
        
        // 尝试使用appDataDir作为首选
        try {
            info('尝试使用appDataDir...');
          baseDir = await appDataDir();
        } catch (appDataError) {
            warn('无法获取appDataDir，尝试使用configDir');
          try {
            baseDir = await configDir();
          } catch (configDirError) {
              warn('无法获取configDir，使用备用路径');
            // 使用备用路径
            baseDir = '/tmp';
          }
        }
        
        info(`获取到的基础目录: ${baseDir}`);
        
        // 构造完整的配置文件路径
        try {
          const appDir = `${baseDir}/tauriOpenGamesAutoPlay`;
          this.configPath = `${appDir}/config.json`;
          info(`配置路径初始化完成: ${this.configPath}`);
        } catch (pathError) {
            error(`构造配置路径失败: ${pathError}`);
          // 使用备用路径
          this.configPath = '/tmp/tauriOpenGamesAutoPlay/config.json';
          warn(`使用备用配置路径: ${this.configPath}`);
        }
        
      } catch (dirError) {
          error(`获取目录失败: ${dirError}`);
        // 使用备用路径
        warn('使用备用配置路径');
        this.configPath = '/tmp/tauriOpenGamesAutoPlay/config.json';
        info(`备用配置路径: ${this.configPath}`);
      }
      
    } catch (err) {
        error(`初始化配置路径失败: ${err}`);
      // 确保即使出现任何错误，也设置一个可用的配置路径
      this.configPath = '/tmp/tauriOpenGamesAutoPlay/config.json';
      info(`设置备用配置路径: ${this.configPath}`);
    }
  }

  async saveSettings(settings: Partial<AppSettings>): Promise<void> {
    try {
        info(`开始保存设置: ${JSON.stringify(settings)}`);
      
      // 确保配置路径已初始化
      if (!this.configPath) {
          info('配置路径未初始化，正在初始化...');
        await this.initializeConfigPath();
        info(`初始化后的配置路径: ${this.configPath}`);
      }

      // 如果配置路径仍然为null，使用备用路径
      if (!this.configPath) {
          warn('无法初始化配置路径，使用备用路径...');
        this.configPath = '/tmp/tauriOpenGamesAutoPlay/config.json';
        info(`使用备用配置路径: ${this.configPath}`);
      }

      // 读取现有配置（如果存在）
        info('读取现有配置...');
      let currentSettings = await this.loadSettings();
        info(`当前配置: ${JSON.stringify(currentSettings)}`);
      
      // 合并新配置
      const updatedSettings = {
        ...currentSettings,
        ...settings,
        // 深度合并嵌套对象
        themeSettings: {
          ...currentSettings.themeSettings,
          ...(settings.themeSettings || {})
        },
        // 确保shortcuts属性存在
        shortcuts: {
          ...currentSettings.shortcuts,
          ...(settings.shortcuts || {})
        },
        // 合并 midiFolderPath
        midiFolderPath: settings.midiFolderPath !== undefined ? settings.midiFolderPath : currentSettings.midiFolderPath,
      };
      info(`更新后的配置: ${JSON.stringify(updatedSettings)}`);

      // 确保目录存在并写入文件
      try {
        if (this.configPath) {
          const directory = this.configPath.substring(0, this.configPath.lastIndexOf('/'));
            info(`配置目录路径: ${directory}`);
          
          // 尝试创建目录
          try {
            const dirExists = await exists(directory);
              info(`目录是否存在: ${dirExists}`);
            if (!dirExists) {
                info('创建目录...');
                // 使用mkdir而不是create，确保支持递归创建目录
                await mkdir(directory, { recursive: true });
                info('目录创建成功');
              }
          } catch (dirError) {
              warn(`创建目录失败: ${dirError}`);
            // 继续尝试写入文件，可能目录已经存在或有其他原因
          }
          
          // 保存配置
          info(`准备写入配置文件: ${this.configPath}`);
          try {
            await writeTextFile(this.configPath!, JSON.stringify(updatedSettings, null, 2));
            info('配置文件写入成功!');
          } catch (writeError) {
            error(`写入文件失败，详细错误: ${JSON.stringify(writeError)}`);
            throw writeError; // 重新抛出错误，以便外部catch块可以捕获
          }
          
          // 验证文件是否成功写入
          try {
            const fileExists = await exists(this.configPath!);
              info(`配置文件是否存在: ${fileExists}`);
            if (fileExists) {
              const savedContent = await readTextFile(this.configPath!);
                info(`保存的文件内容: ${savedContent}`);
            }
          } catch (verifyError) {
              warn(`验证文件写入失败: ${verifyError}`);
          }
        }
      } catch (writeError) {
          error(`写入配置文件失败: ${writeError}`);
        // 不抛出错误，允许应用继续运行，但记录详细日志
        info('配置已更新但无法保存到文件系统，将使用内存中的配置');
      }
    } catch (err) {
        error(`保存设置失败: ${err}`);
      // 不再抛出错误，允许应用继续运行
      info('配置保存过程中出现错误，但应用将继续运行');
    }
  }

  async loadSettings(): Promise<AppSettings> {
    try {
        info('开始加载设置...');
      
      // 确保配置路径已初始化
      if (!this.configPath) {
          info('配置路径未初始化，正在初始化...');
        await this.initializeConfigPath();
        info(`初始化后的配置路径: ${this.configPath}`);
      }

      // 如果配置路径仍然为null，使用备用路径
      if (!this.configPath) {
          warn('无法初始化配置路径，使用备用路径...');
        this.configPath = '/tmp/tauriOpenGamesAutoPlay/config.json';
        info(`使用备用配置路径: ${this.configPath}`);
      }

      // 尝试从文件加载配置
      if (this.configPath) {
        try {
            const fileExists = await exists(this.configPath);
            info(`配置文件是否存在: ${fileExists}`);
          
          if (fileExists) {
            try {
              const fileContent = await readTextFile(this.configPath);
                info(`读取的配置文件内容: ${fileContent}`);
              
              // 尝试解析JSON
              const parsedSettings = JSON.parse(fileContent) as AppSettings;
              
              // 确保所有必需的字段都存在，并进行安全合并
              const settings: AppSettings = {
                themeSettings: { ...this.defaultSettings.themeSettings, ...(parsedSettings.themeSettings || {}) },
                shortcuts: { ...(this.defaultSettings.shortcuts || {}), ...(parsedSettings.shortcuts || {}) }
              };
              
              info(`加载的设置: ${JSON.stringify(settings)}`);
              return settings;
            } catch (parseError) {
                error(`解析配置文件失败: ${parseError}`);
              // 解析失败时，返回默认设置
              info('配置文件解析失败，返回默认设置');
              return this.defaultSettings;
            }
          } else {
              info('配置文件不存在，返回默认设置');
            return this.defaultSettings;
          }
        } catch (fileError) {
            error(`访问配置文件失败: ${fileError}`);
          // 文件访问失败时，返回默认设置
          info('文件访问失败，返回默认设置');
          return this.defaultSettings;
        }
      }
      
      // 如果所有尝试都失败，返回默认设置
      info('无法访问配置路径，返回默认设置');
      return this.defaultSettings;
    } catch (err) {
        error(`加载设置过程中发生未知错误: ${err}`);
      // 捕获所有错误，确保返回默认设置，不中断应用
      info('返回默认设置以确保应用正常运行');
      return this.defaultSettings;
    }
  }

  async saveTheme(theme: string): Promise<void> {
    try {
        info(`保存主题: ${theme}`);
      
      // 验证主题值
      if (!theme || typeof theme !== 'string') {
          warn('无效的主题值，使用默认值');
        theme = this.defaultSettings.themeSettings.currentTheme;
      }
      
      // 调用saveSettings保存主题，不再抛出错误
      await this.saveSettings({ themeSettings: { currentTheme: theme } });
      info(`主题保存成功: ${theme}`);
    } catch (err) {
        error(`保存主题过程中发生错误: ${err}`);
      // 不再抛出错误，允许应用继续运行
      info('主题保存失败，但应用将继续运行');
    }
  }

  async loadTheme(): Promise<string> {
    try {
        info('开始加载主题...');
      
      // 调用loadSettings加载设置，它已经有完整的错误处理
      const settings = await this.loadSettings();
      
      // 安全获取主题值
      const theme = settings?.themeSettings?.currentTheme;
      
      // 验证主题值
        if (!theme || typeof theme !== 'string') {
          warn('无效的主题值，使用默认主题');
        return this.defaultSettings.themeSettings.currentTheme;
      }
      
      info(`加载的主题: ${theme}`);
      return theme;
    } catch (err) {
        error(`加载主题过程中发生未知错误: ${err}`);
      // 返回默认主题，确保应用不会中断
      info('返回默认主题以确保应用正常运行');
      return this.defaultSettings.themeSettings.currentTheme;
    }
  }

  async saveMidiFolderPath(path: string): Promise<void> {
    await this.saveSettings({ midiFolderPath: path });
  }

  async loadMidiFolderPath(): Promise<string | undefined> {
    const settings = await this.loadSettings();
    return settings.midiFolderPath;
  }
}

// 导出单例
export default new SettingsManager();
