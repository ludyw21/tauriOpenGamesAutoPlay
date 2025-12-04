import { register, unregister, isRegistered } from '@tauri-apps/plugin-global-shortcut';
import { info, error } from '@tauri-apps/plugin-log';

export interface ShortcutHandlers {
  onStartPause: () => void;
  onStop: () => void;
  onPrevSong: () => void;
  onNextSong: () => void;
}

class ShortcutService {
  private registeredShortcuts: string[] = [];
  private lastTriggerTime: Map<string, number> = new Map();
  private readonly DEBOUNCE_DELAY = 300; // 防抖延迟（毫秒）

  /**
   * 注册全局快捷键
   * @param shortcuts 快捷键配置对象
   * @param handlers 快捷键处理函数
   */
  async registerShortcuts(
    shortcuts: Record<string, string>,
    handlers: ShortcutHandlers
  ): Promise<void> {
    try {
      info('[ShortcutService] 开始注册全局快捷键...');

      // 注册开始/暂停快捷键
      if (shortcuts.START_PAUSE) {
        await this.registerSingleShortcut(
          shortcuts.START_PAUSE,
          'START_PAUSE',
          handlers.onStartPause
        );
      }

      // 注册停止快捷键
      if (shortcuts.STOP) {
        await this.registerSingleShortcut(
          shortcuts.STOP,
          'STOP',
          handlers.onStop
        );
      }

      // 注册上一首快捷键
      if (shortcuts.PREV_SONG) {
        await this.registerSingleShortcut(
          shortcuts.PREV_SONG,
          'PREV_SONG',
          handlers.onPrevSong
        );
      }

      // 注册下一首快捷键
      if (shortcuts.NEXT_SONG) {
        await this.registerSingleShortcut(
          shortcuts.NEXT_SONG,
          'NEXT_SONG',
          handlers.onNextSong
        );
      }

      info(`[ShortcutService] 成功注册 ${this.registeredShortcuts.length} 个全局快捷键`);
    } catch (err) {
      error(`[ShortcutService] 注册快捷键失败: ${err}`);
      throw err;
    }
  }

  /**
   * 注册单个快捷键
   */
  private async registerSingleShortcut(
    shortcut: string,
    name: string,
    handler: () => void
  ): Promise<void> {
    try {
      // 检查是否已注册
      const alreadyRegistered = await isRegistered(shortcut);
      if (alreadyRegistered) {
        info(`[ShortcutService] 快捷键 ${shortcut} (${name}) 已被注册，先注销`);
        await unregister(shortcut);
      }

      // 注册快捷键（带防抖）
      await register(shortcut, () => {
        const now = Date.now();
        const lastTime = this.lastTriggerTime.get(name) || 0;

        // 检查是否在防抖延迟内
        if (now - lastTime < this.DEBOUNCE_DELAY) {
          info(`[ShortcutService] 快捷键 ${name} 触发过快，忽略 (距上次 ${now - lastTime}ms)`);
          return;
        }

        // 更新最后触发时间
        this.lastTriggerTime.set(name, now);
        info(`[ShortcutService] 快捷键触发: ${name} (${shortcut})`);
        handler();
      });

      this.registeredShortcuts.push(shortcut);
      info(`[ShortcutService] 成功注册快捷键: ${name} (${shortcut})`);
    } catch (err) {
      error(`[ShortcutService] 注册快捷键 ${name} (${shortcut}) 失败: ${err}`);
      throw err;
    }
  }

  /**
   * 注销所有已注册的快捷键
   */
  async unregisterAll(): Promise<void> {
    try {
      info('[ShortcutService] 开始注销所有快捷键...');

      for (const shortcut of this.registeredShortcuts) {
        try {
          await unregister(shortcut);
          info(`[ShortcutService] 成功注销快捷键: ${shortcut}`);
        } catch (err) {
          error(`[ShortcutService] 注销快捷键 ${shortcut} 失败: ${err}`);
        }
      }

      this.registeredShortcuts = [];
      this.lastTriggerTime.clear(); // 清除防抖记录
      info('[ShortcutService] 所有快捷键已注销');
    } catch (err) {
      error(`[ShortcutService] 注销快捷键失败: ${err}`);
    }
  }

  /**
   * 重新注册快捷键（用于配置更新后）
   */
  async reregisterShortcuts(
    shortcuts: Record<string, string>,
    handlers: ShortcutHandlers
  ): Promise<void> {
    await this.unregisterAll();
    await this.registerShortcuts(shortcuts, handlers);
  }
}

// 导出单例
export default new ShortcutService();
