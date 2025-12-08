use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

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

/// 将字符映射到 macOS 虚拟键码
/// 使用 kVK_ANSI_* 键码，这些是位置相关的，不需要访问输入源
#[cfg(target_os = "macos")]
fn char_to_macos_keycode(ch: char) -> Option<u16> {
    match ch.to_ascii_lowercase() {
        'a' => Some(0x00), // kVK_ANSI_A
        'b' => Some(0x0B), // kVK_ANSI_B
        'c' => Some(0x08), // kVK_ANSI_C
        'd' => Some(0x02), // kVK_ANSI_D
        'e' => Some(0x0E), // kVK_ANSI_E
        'f' => Some(0x03), // kVK_ANSI_F
        'g' => Some(0x05), // kVK_ANSI_G
        'h' => Some(0x04), // kVK_ANSI_H
        'i' => Some(0x22), // kVK_ANSI_I
        'j' => Some(0x26), // kVK_ANSI_J
        'k' => Some(0x28), // kVK_ANSI_K
        'l' => Some(0x25), // kVK_ANSI_L
        'm' => Some(0x2E), // kVK_ANSI_M
        'n' => Some(0x2D), // kVK_ANSI_N
        'o' => Some(0x1F), // kVK_ANSI_O
        'p' => Some(0x23), // kVK_ANSI_P
        'q' => Some(0x0C), // kVK_ANSI_Q
        'r' => Some(0x0F), // kVK_ANSI_R
        's' => Some(0x01), // kVK_ANSI_S
        't' => Some(0x11), // kVK_ANSI_T
        'u' => Some(0x20), // kVK_ANSI_U
        'v' => Some(0x09), // kVK_ANSI_V
        'w' => Some(0x0D), // kVK_ANSI_W
        'x' => Some(0x07), // kVK_ANSI_X
        'y' => Some(0x10), // kVK_ANSI_Y
        'z' => Some(0x06), // kVK_ANSI_Z
        '0' => Some(0x1D), // kVK_ANSI_0
        '1' => Some(0x12), // kVK_ANSI_1
        '2' => Some(0x13), // kVK_ANSI_2
        '3' => Some(0x14), // kVK_ANSI_3
        '4' => Some(0x15), // kVK_ANSI_4
        '5' => Some(0x17), // kVK_ANSI_5
        '6' => Some(0x16), // kVK_ANSI_6
        '7' => Some(0x1A), // kVK_ANSI_7
        '8' => Some(0x1C), // kVK_ANSI_8
        '9' => Some(0x19), // kVK_ANSI_9
        _ => None,
    }
}

/// 将字符映射到 Windows 扫描码
/// 使用硬件扫描码,游戏的 DirectInput 可以识别
#[cfg(target_os = "windows")]
fn char_to_windows_scancode(ch: char) -> Option<u16> {
    match ch.to_ascii_lowercase() {
        'a' => Some(0x1E), // Scan code for A
        'b' => Some(0x30), // Scan code for B
        'c' => Some(0x2E), // Scan code for C
        'd' => Some(0x20), // Scan code for D
        'e' => Some(0x12), // Scan code for E
        'f' => Some(0x21), // Scan code for F
        'g' => Some(0x22), // Scan code for G
        'h' => Some(0x23), // Scan code for H
        'i' => Some(0x17), // Scan code for I
        'j' => Some(0x24), // Scan code for J
        'k' => Some(0x25), // Scan code for K
        'l' => Some(0x26), // Scan code for L
        'm' => Some(0x32), // Scan code for M
        'n' => Some(0x31), // Scan code for N
        'o' => Some(0x18), // Scan code for O
        'p' => Some(0x19), // Scan code for P
        'q' => Some(0x10), // Scan code for Q
        'r' => Some(0x13), // Scan code for R
        's' => Some(0x1F), // Scan code for S
        't' => Some(0x14), // Scan code for T
        'u' => Some(0x16), // Scan code for U
        'v' => Some(0x2F), // Scan code for V
        'w' => Some(0x11), // Scan code for W
        'x' => Some(0x2D), // Scan code for X
        'y' => Some(0x15), // Scan code for Y
        'z' => Some(0x2C), // Scan code for Z
        '0' => Some(0x0B), // Scan code for 0
        '1' => Some(0x02), // Scan code for 1
        '2' => Some(0x03), // Scan code for 2
        '3' => Some(0x04), // Scan code for 3
        '4' => Some(0x05), // Scan code for 4
        '5' => Some(0x06), // Scan code for 5
        '6' => Some(0x07), // Scan code for 6
        '7' => Some(0x08), // Scan code for 7
        '8' => Some(0x09), // Scan code for 8
        '9' => Some(0x0A), // Scan code for 9
        _ => None,
    }
}

/// 将修饰键映射到 Windows 扫描码
/// 用于游戏的 DirectInput 识别
#[cfg(target_os = "windows")]
fn modifier_to_windows_scancode(modifier: Key) -> Option<u16> {
    match modifier {
        Key::Shift => Some(0x2A),   // Left Shift scan code
        Key::Control => Some(0x1D), // Left Ctrl scan code
        Key::Alt => Some(0x38),     // Left Alt scan code
        Key::Meta => Some(0x5B),    // Left Windows key scan code (extended)
        _ => None,
    }
}

/// 解析按键字符串，返回修饰键和主键
/// 例如: "shift+a" -> (vec![Key::Shift], 'a')
///       "ctrl+c" -> (vec![Key::Control or Key::Meta], 'c')
fn parse_key_string(key_str: &str) -> Result<(Vec<Key>, Option<char>), String> {
    let parts: Vec<&str> = key_str.split('+').collect();
    let mut modifiers = Vec::new();
    let mut main_key: Option<char> = None;

    // 检测操作系统
    let is_macos = cfg!(target_os = "macos");

    for (i, part) in parts.iter().enumerate() {
        let part_lower = part.to_lowercase();

        if i < parts.len() - 1 {
            // 修饰键
            match part_lower.as_str() {
                "shift" => modifiers.push(Key::Shift),
                "ctrl" | "control" => {
                    // 在 macOS 上，ctrl 映射到 Command 键（Meta）
                    // 在 Windows/Linux 上，ctrl 映射到 Control 键
                    if is_macos {
                        modifiers.push(Key::Meta);
                    } else {
                        modifiers.push(Key::Control);
                    }
                }
                "alt" => modifiers.push(Key::Alt),
                "meta" | "cmd" | "command" | "win" | "super" => modifiers.push(Key::Meta),
                _ => return Err(format!("Unknown modifier key: {}", part)),
            }
        } else {
            // 主键（最后一个部分）
            if part.len() == 1 {
                main_key = part.chars().next();
            } else {
                return Err(format!("Invalid main key: {}", part));
            }
        }
    }

    Ok((modifiers, main_key))
}

/// 模拟按键按下
fn simulate_keypress(enigo: &mut Enigo, key_str: &str, _duration: f64) -> Result<(), String> {
    let (modifiers, main_key) = parse_key_string(key_str)?;

    // 按下修饰键,每个修饰键之间添加小延迟
    for modifier in &modifiers {
        #[cfg(target_os = "windows")]
        {
            // Windows: 使用扫描码以支持游戏的 DirectInput
            if let Some(scancode) = modifier_to_windows_scancode(*modifier) {
                enigo
                    .raw(scancode, Direction::Press)
                    .map_err(|e| format!("Failed to press modifier: {:?}", e))?;
            } else {
                // 如果没有扫描码映射,回退到普通方法
                enigo
                    .key(*modifier, Direction::Press)
                    .map_err(|e| format!("Failed to press modifier: {:?}", e))?;
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            // 其他平台:使用标准方法
            enigo
                .key(*modifier, Direction::Press)
                .map_err(|e| format!("Failed to press modifier: {:?}", e))?;
        }

        // 修饰键之间延迟 5ms(减少延迟以提高播放速度)
        thread::sleep(Duration::from_millis(5));
    }

    // 修饰键按下后,等待 10ms 再按主键(减少延迟以提高播放速度)
    if !modifiers.is_empty() {
        thread::sleep(Duration::from_millis(10));
    }

    // 按下主键
    if let Some(ch) = main_key {
        // 在 macOS 上使用原始键码以避免线程安全问题
        #[cfg(target_os = "macos")]
        {
            if let Some(keycode) = char_to_macos_keycode(ch) {
                enigo
                    .raw(keycode, Direction::Press)
                    .map_err(|e| format!("Failed to press key: {:?}", e))?;

                // 最小延迟以确保按键被识别(不使用 duration,让播放时间由播放循环控制)
                thread::sleep(Duration::from_millis(1));

                // 释放主键
                enigo
                    .raw(keycode, Direction::Release)
                    .map_err(|e| format!("Failed to release key: {:?}", e))?;
            } else {
                return Err(format!("Unsupported character: {}", ch));
            }
        }

        // 在 Windows 上使用扫描码以支持游戏的 DirectInput
        #[cfg(target_os = "windows")]
        {
            if let Some(scancode) = char_to_windows_scancode(ch) {
                enigo
                    .raw(scancode, Direction::Press)
                    .map_err(|e| format!("Failed to press key: {:?}", e))?;

                // 最小延迟以确保按键被识别(不使用 duration,让播放时间由播放循环控制)
                thread::sleep(Duration::from_millis(1));

                // 释放主键
                enigo
                    .raw(scancode, Direction::Release)
                    .map_err(|e| format!("Failed to release key: {:?}", e))?;
            } else {
                return Err(format!("Unsupported character: {}", ch));
            }
        }

        // 在其他平台(Linux等)上使用 Unicode
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            enigo
                .key(Key::Unicode(ch), Direction::Press)
                .map_err(|e| format!("Failed to press key: {:?}", e))?;

            // 最小延迟以确保按键被识别(不使用 duration,让播放时间由播放循环控制)
            thread::sleep(Duration::from_millis(1));

            // 释放主键
            enigo
                .key(Key::Unicode(ch), Direction::Release)
                .map_err(|e| format!("Failed to release key: {:?}", e))?;
        }
    }

    // 主键释放后,等待 10ms 再释放修饰键(减少延迟以提高播放速度)
    if !modifiers.is_empty() {
        thread::sleep(Duration::from_millis(10));
    }

    // 释放修饰键（逆序）,每个修饰键之间添加小延迟
    for modifier in modifiers.iter().rev() {
        #[cfg(target_os = "windows")]
        {
            // Windows: 使用扫描码以支持游戏的 DirectInput
            if let Some(scancode) = modifier_to_windows_scancode(*modifier) {
                enigo
                    .raw(scancode, Direction::Release)
                    .map_err(|e| format!("Failed to release modifier: {:?}", e))?;
            } else {
                // 如果没有扫描码映射,回退到普通方法
                enigo
                    .key(*modifier, Direction::Release)
                    .map_err(|e| format!("Failed to release modifier: {:?}", e))?;
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            // 其他平台:使用标准方法
            enigo
                .key(*modifier, Direction::Release)
                .map_err(|e| format!("Failed to release modifier: {:?}", e))?;
        }

        // 修饰键之间延迟 30ms
        thread::sleep(Duration::from_millis(30));
    }

    Ok(())
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

            // 模拟按键
            if let Err(e) = simulate_keypress(&mut enigo, &event.key, event.duration) {
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
        // 使用超时等待
        let _ = handle.join();
    }

    Ok(())
}
