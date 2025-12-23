use enigo::{Direction, Enigo, Key, Keyboard};
use std::thread;
use std::time::Duration;

/// 将字符映射到 macOS 虚拟键码
/// 使用 kVK_ANSI_* 键码
#[cfg(target_os = "macos")]
fn char_to_macos_keycode(ch: char) -> Option<u16> {
    match ch.to_ascii_lowercase() {
        'a' => Some(0x00), 'b' => Some(0x0B), 'c' => Some(0x08), 'd' => Some(0x02), 'e' => Some(0x0E),
        'f' => Some(0x03), 'g' => Some(0x05), 'h' => Some(0x04), 'i' => Some(0x22), 'j' => Some(0x26),
        'k' => Some(0x28), 'l' => Some(0x25), 'm' => Some(0x2E), 'n' => Some(0x2D), 'o' => Some(0x1F),
        'p' => Some(0x23), 'q' => Some(0x0C), 'r' => Some(0x0F), 's' => Some(0x01), 't' => Some(0x11),
        'u' => Some(0x20), 'v' => Some(0x09), 'w' => Some(0x0D), 'x' => Some(0x07), 'y' => Some(0x10),
        'z' => Some(0x06), '0' => Some(0x1D), '1' => Some(0x12), '2' => Some(0x13), '3' => Some(0x14),
        '4' => Some(0x15), '5' => Some(0x17), '6' => Some(0x16), '7' => Some(0x1A), '8' => Some(0x1C),
        '9' => Some(0x19),
        _ => None,
    }
}

/// 将字符映射到 Windows 扫描码
#[cfg(target_os = "windows")]
fn char_to_windows_scancode(ch: char) -> Option<u16> {
    match ch.to_ascii_lowercase() {
        'a' => Some(0x1E), 'b' => Some(0x30), 'c' => Some(0x2E), 'd' => Some(0x20), 'e' => Some(0x12),
        'f' => Some(0x21), 'g' => Some(0x22), 'h' => Some(0x23), 'i' => Some(0x17), 'j' => Some(0x24),
        'k' => Some(0x25), 'l' => Some(0x26), 'm' => Some(0x32), 'n' => Some(0x31), 'o' => Some(0x18),
        'p' => Some(0x19), 'q' => Some(0x10), 'r' => Some(0x13), 's' => Some(0x1F), 't' => Some(0x14),
        'u' => Some(0x16), 'v' => Some(0x2F), 'w' => Some(0x11), 'x' => Some(0x2D), 'y' => Some(0x15),
        'z' => Some(0x2C), '0' => Some(0x0B), '1' => Some(0x02), '2' => Some(0x03), '3' => Some(0x04),
        '4' => Some(0x05), '5' => Some(0x06), '6' => Some(0x07), '7' => Some(0x08), '8' => Some(0x09),
        '9' => Some(0x0A),
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
fn parse_key_string(key_str: &str) -> Result<(Vec<Key>, Option<char>), String> {
    let parts: Vec<&str> = key_str.split('+').collect();
    let mut modifiers = Vec::new();
    let mut main_key: Option<char> = None;

    let is_macos = cfg!(target_os = "macos");

    for (i, part) in parts.iter().enumerate() {
        let part_lower = part.to_lowercase();
        if i < parts.len() - 1 {
            match part_lower.as_str() {
                "shift" => modifiers.push(Key::Shift),
                "ctrl" | "control" => if is_macos { modifiers.push(Key::Meta) } else { modifiers.push(Key::Control) },
                "alt" => modifiers.push(Key::Alt),
                "meta" | "cmd" | "win" => modifiers.push(Key::Meta),
                _ => return Err(format!("Unknown modifier: {}", part)),
            }
        } else {
            if part.len() == 1 { main_key = part.chars().next(); }
            else { return Err(format!("Invalid main key: {}", part)); }
        }
    }
    Ok((modifiers, main_key))
}

pub trait SmartKeyboard {
    fn simulate_keypress_smart(&mut self, key_str: &str) -> Result<(), String>;
}

impl SmartKeyboard for Enigo {
    fn simulate_keypress_smart(&mut self, key_str: &str) -> Result<(), String> {
        let (modifiers, main_key) = parse_key_string(key_str)?;

        // Press modifiers
        for modifier in &modifiers {
             #[cfg(target_os = "windows")]
             {
                 if let Some(scancode) = modifier_to_windows_scancode(*modifier) {
                     self.raw(scancode, Direction::Press).map_err(|e| format!("{:?}",e))?;
                 } else {
                     self.key(*modifier, Direction::Press).map_err(|e| format!("{:?}",e))?;
                 }
             }

             #[cfg(not(target_os = "windows"))]
             {
                 self.key(*modifier, Direction::Press).map_err(|e| format!("{:?}",e))?;
             }
             
             thread::sleep(Duration::from_millis(5));
        }

        if !modifiers.is_empty() { thread::sleep(Duration::from_millis(10)); }

        if let Some(ch) = main_key {
            #[cfg(target_os = "macos")]
            if let Some(code) = char_to_macos_keycode(ch) {
                self.raw(code, Direction::Press).map_err(|e| format!("{:?}",e))?;
                thread::sleep(Duration::from_millis(20)); // Short hold
                self.raw(code, Direction::Release).map_err(|e| format!("{:?}",e))?;
            } else {
                 self.key(Key::Unicode(ch), Direction::Click).map_err(|e| format!("{:?}",e))?;
            }

            #[cfg(target_os = "windows")]
            if let Some(code) = char_to_windows_scancode(ch) {
                self.raw(code, Direction::Press).map_err(|e| format!("{:?}",e))?;
                thread::sleep(Duration::from_millis(20));
                self.raw(code, Direction::Release).map_err(|e| format!("{:?}",e))?;
            } else {
                 self.key(Key::Unicode(ch), Direction::Click).map_err(|e| format!("{:?}",e))?;
            }
            
            #[cfg(not(any(target_os = "macos", target_os = "windows")))]
            self.key(Key::Unicode(ch), Direction::Click).map_err(|e| format!("{:?}",e))?;
        }

        if !modifiers.is_empty() { thread::sleep(Duration::from_millis(10)); }

        // Release modifiers
        for modifier in modifiers.iter().rev() {
             #[cfg(target_os = "windows")]
             {
                 if let Some(scancode) = modifier_to_windows_scancode(*modifier) {
                     self.raw(scancode, Direction::Release).map_err(|e| format!("{:?}",e))?;
                 } else {
                     self.key(*modifier, Direction::Release).map_err(|e| format!("{:?}",e))?;
                 }
             }

             #[cfg(not(target_os = "windows"))]
             {
                 self.key(*modifier, Direction::Release).map_err(|e| format!("{:?}",e))?;
             }

             thread::sleep(Duration::from_millis(30));
        }

        Ok(())
    }
}
