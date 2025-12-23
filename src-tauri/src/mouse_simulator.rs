use enigo::{Enigo, Settings};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use uni_input::SmoothMouse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseEvent {
    pub time: f64,     // 时间（秒）
    pub x: i32,        // X坐标
    pub y: i32,        // Y坐标
    pub duration: f64, // 持续时间（秒）
}

// 播放状态管理
lazy_static::lazy_static! {
    static ref MOUSE_PLAYBACK_HANDLE: Arc<Mutex<Option<thread::JoinHandle<()>>>> = Arc::new(Mutex::new(None));
    static ref MOUSE_SHOULD_STOP: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

/// 开始播放鼠标事件序列
pub fn start_mouse_playback(events: Vec<MouseEvent>) -> Result<(), String> {
    // 检查是否已有播放在进行
    {
        let handle = MOUSE_PLAYBACK_HANDLE.lock().unwrap();
        if handle.is_some() {
            return Err("Mouse playback already in progress".to_string());
        }
    }

    // 重置停止标志
    {
        let mut should_stop = MOUSE_SHOULD_STOP.lock().unwrap();
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
                let should_stop = MOUSE_SHOULD_STOP.lock().unwrap();
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
                let should_stop = MOUSE_SHOULD_STOP.lock().unwrap();
                if *should_stop {
                    break;
                }
            }

            // 模拟鼠标点击 (调用 uni-input 的 SmoothMouse trait)
            if let Err(e) = enigo.mouse_click_smooth(event.x, event.y) {
                 eprintln!("Failed to simulate mouse click: {}", e);
            }
        }

        // 播放完成，清理句柄
        let mut handle = MOUSE_PLAYBACK_HANDLE.lock().unwrap();
        *handle = None;
    });

    // 保存线程句柄
    {
        let mut playback_handle = MOUSE_PLAYBACK_HANDLE.lock().unwrap();
        *playback_handle = Some(handle);
    }

    Ok(())
}

/// 停止鼠标播放
pub fn stop_mouse_playback() -> Result<(), String> {
    // 设置停止标志
    {
        let mut should_stop = MOUSE_SHOULD_STOP.lock().unwrap();
        *should_stop = true;
    }

    // 等待线程结束
    let handle = {
        let mut playback_handle = MOUSE_PLAYBACK_HANDLE.lock().unwrap();
        playback_handle.take()
    };

    if let Some(handle) = handle {
        let _ = handle.join();
    }

    Ok(())
}

/// 选择鼠标坐标
/// 监听全局鼠标点击事件,返回点击位置的坐标
pub async fn pick_coordinate() -> Result<(i32, i32), String> {
    use rdev::{grab, Button, Event, EventType};
    use std::sync::mpsc::channel;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    };
    use std::time::Duration;

    // 创建通道用于传递坐标
    let (tx, rx) = channel::<(i32, i32)>();

    // 创建停止标志
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = stop_flag.clone();

    // 用于跟踪最后的鼠标位置
    let last_position = Arc::new(Mutex::new((0, 0)));
    let last_position_clone = last_position.clone();

    // 启动监听线程
    let _listen_thread = thread::spawn(move || {
        // Windows: 在监听线程中加载十字准星光标
        #[cfg(target_os = "windows")]
        let cross_cursor = unsafe {
            use windows::Win32::UI::WindowsAndMessaging::{LoadCursorW, IDC_CROSS};
            LoadCursorW(None, IDC_CROSS).ok()
        };

        // 监听回调函数,返回 Option<Event> 来控制事件传播
        let callback = move |event: Event| -> Option<Event> {
            // 检查是否需要停止
            if stop_flag_clone.load(Ordering::Relaxed) {
                return Some(event); // 停止后不再拦截事件
            }

            match event.event_type {
                EventType::MouseMove { x, y } => {
                    // Windows: 每次鼠标移动时设置十字准星光标
                    #[cfg(target_os = "windows")]
                    if let Some(cursor) = cross_cursor {
                        unsafe {
                            use windows::Win32::UI::WindowsAndMessaging::SetCursor;
                            SetCursor(cursor);
                        }
                    }

                    // 更新最后的鼠标位置
                    if let Ok(mut pos) = last_position_clone.lock() {
                        *pos = (x as i32, y as i32);
                    }
                    Some(event) // 允许鼠标移动事件传播
                }
                EventType::ButtonPress(Button::Left) => {
                    // 捕获鼠标左键点击
                    if let Ok(pos) = last_position_clone.lock() {
                        // 如果 last_position 还是初始值 (0, 0),说明点击太快
                        // 这种情况下我们无法获取准确坐标,但至少不返回 (0, 0)
                        if *pos != (0, 0) {
                            let _ = tx.send(*pos);
                            // 设置停止标志
                            stop_flag_clone.store(true, Ordering::Relaxed);
                            // 拦截这个点击事件,不让它传播
                            return None;
                        }
                    }
                    Some(event) // 如果获取失败,允许事件传播
                }
                _ => Some(event), // 其他事件正常传播
            }
        };

        // 使用 grab 来拦截事件
        if let Err(e) = grab(callback) {
            eprintln!("监听鼠标事件失败: {:?}", e);
        }
    });

    // 等待接收坐标(30秒超时)
    match rx.recv_timeout(Duration::from_secs(30)) {
        Ok((x, y)) => {
            // 设置停止标志
            stop_flag.store(true, Ordering::Relaxed);
            // 给监听线程一点时间停止
            thread::sleep(Duration::from_millis(100));

            // Windows: 恢复鼠标样式为箭头
            #[cfg(target_os = "windows")]
            unsafe {
                use windows::Win32::UI::WindowsAndMessaging::{LoadCursorW, SetCursor, IDC_ARROW};
                let arrow_cursor = LoadCursorW(None, IDC_ARROW).ok();
                if let Some(cursor) = arrow_cursor {
                    SetCursor(cursor);
                }
            }

            Ok((x, y))
        }
        Err(_) => {
            // 超时,设置停止标志
            stop_flag.store(true, Ordering::Relaxed);

            // Windows: 恢复鼠标样式为箭头
            #[cfg(target_os = "windows")]
            unsafe {
                use windows::Win32::UI::WindowsAndMessaging::{LoadCursorW, SetCursor, IDC_ARROW};
                let arrow_cursor = LoadCursorW(None, IDC_ARROW).ok();
                if let Some(cursor) = arrow_cursor {
                    SetCursor(cursor);
                }
            }

            Err("等待鼠标点击超时(30秒)".to_string())
        }
    }
}
