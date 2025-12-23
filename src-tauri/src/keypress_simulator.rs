use enigo::{Enigo, Settings};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use uni_input::SmartKeyboard;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEvent {
    pub time: f64,     // 时间（秒）
    pub key: String,   // 按键字符串，如 "a", "shift+a", "ctrl+c"
    pub duration: f64, // 按键持续时间（秒）
}

// 播放状态管理
lazy_static::lazy_static! {
    static ref PLAYBACK_HANDLE: Arc<Mutex<Option<thread::JoinHandle<()>>>> = Arc::new(Mutex::new(None));
    static ref SHOULD_STOP: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

/// 开始播放按键序列
pub fn start_playback(events: Vec<KeyEvent>) -> Result<(), String> {
    // 检查是否已有播放在进行
    {
        let handle = PLAYBACK_HANDLE.lock().unwrap();
        if handle.is_some() {
            return Err("Playback already in progress".to_string());
        }
    }

    // 重置停止标志
    {
        let mut should_stop = SHOULD_STOP.lock().unwrap();
        *should_stop = false;
    }

    // 在新线程中执行播放
    let handle = thread::spawn(move || {
        // 创建 Enigo 实例
        let mut enigo = match Enigo::new(&Settings::default()) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Failed to create Enigo instance: {:?}", e);
                return;
            }
        };

        let start_time = std::time::Instant::now();

        for event in events {
            // 检查是否需要停止
            {
                let should_stop = SHOULD_STOP.lock().unwrap();
                if *should_stop {
                    break;
                }
            }

            // 等待到事件时间
            let target_time = Duration::from_secs_f64(event.time);
            let elapsed = start_time.elapsed();

            if target_time > elapsed {
                let wait_time = target_time - elapsed;
                thread::sleep(wait_time);
            }

            // 再次检查是否需要停止
            {
                let should_stop = SHOULD_STOP.lock().unwrap();
                if *should_stop {
                    break;
                }
            }

            // 模拟按键 (调用 uni-input 的 SmartKeyboard trait)
            if let Err(e) = enigo.simulate_keypress_smart(&event.key) {
                eprintln!("Failed to simulate keypress: {}", e);
            }
        }

        // 播放完成，清理句柄
        let mut handle = PLAYBACK_HANDLE.lock().unwrap();
        *handle = None;
    });

    // 保存线程句柄
    {
        let mut playback_handle = PLAYBACK_HANDLE.lock().unwrap();
        *playback_handle = Some(handle);
    }

    Ok(())
}

/// 停止播放
pub fn stop_playback() -> Result<(), String> {
    // 设置停止标志
    {
        let mut should_stop = SHOULD_STOP.lock().unwrap();
        *should_stop = true;
    }

    // 等待线程结束（最多等待1秒）
    let handle = {
        let mut playback_handle = PLAYBACK_HANDLE.lock().unwrap();
        playback_handle.take()
    };

    if let Some(handle) = handle {
        let _ = handle.join();
    }

    Ok(())
}
