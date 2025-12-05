use enigo::{Button, Coordinate, Direction, Enigo, Mouse, Settings};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

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

/// 生成贝塞尔曲线路径
/// 使用二次贝塞尔曲线在起点和终点之间生成平滑路径
fn generate_bezier_path(
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
    steps: usize,
) -> Vec<(i32, i32)> {
    let mut path = Vec::new();

    // 计算控制点（在起点和终点之间的随机位置）
    let mut rng = rand::thread_rng();

    // 控制点偏移范围：距离的 20%-40%
    let dx = end_x - start_x;
    let dy = end_y - start_y;
    let distance = ((dx * dx + dy * dy) as f64).sqrt();
    let offset_range = ((distance * 0.2) as i32).max(10);

    // 生成随机控制点
    let control_x = (start_x + end_x) / 2 + rng.gen_range(-offset_range..=offset_range);
    let control_y = (start_y + end_y) / 2 + rng.gen_range(-offset_range..=offset_range);

    // 生成贝塞尔曲线上的点
    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let t_inv = 1.0 - t;

        // 二次贝塞尔曲线公式: B(t) = (1-t)²P0 + 2(1-t)tP1 + t²P2
        let x = (t_inv * t_inv * start_x as f64
            + 2.0 * t_inv * t * control_x as f64
            + t * t * end_x as f64) as i32;
        let y = (t_inv * t_inv * start_y as f64
            + 2.0 * t_inv * t * control_y as f64
            + t * t * end_y as f64) as i32;

        path.push((x, y));
    }

    path
}

/// 为坐标添加随机偏移（±5像素）
fn add_coordinate_offset(x: i32, y: i32) -> (i32, i32) {
    let mut rng = rand::thread_rng();
    let offset_x = rng.gen_range(-5..=5);
    let offset_y = rng.gen_range(-5..=5);
    (x + offset_x, y + offset_y)
}

/// 模拟鼠标点击
/// 移动鼠标到指定坐标（带随机偏移）并执行左键点击
/// time_to_next: 到下一个事件的时间间隔（秒），用于控制速度
fn simulate_mouse_click(
    enigo: &mut Enigo,
    x: i32,
    y: i32,
    time_to_next: f64,
) -> Result<(), String> {
    // 获取当前鼠标位置
    let (current_x, current_y) = enigo
        .location()
        .map_err(|e| format!("Failed to get mouse location: {:?}", e))?;

    // 应用坐标偏移
    let (target_x, target_y) = add_coordinate_offset(x, y);

    // 根据到下一个事件的时间间隔，动态调整移动参数
    let (steps, step_delay_range, reaction_delay_ms) = if time_to_next < 0.05 {
        // 极快（间隔 < 50ms）：瞬间移动，无延迟
        (1, 0..=0, 0)
    } else if time_to_next < 0.15 {
        // 快速（间隔 < 150ms）：少步骤，极短延迟
        (5, 0..=1, 5)
    } else {
        // 正常：计算步数，自然延迟
        let dx = target_x - current_x;
        let dy = target_y - current_y;
        let distance = ((dx * dx + dy * dy) as f64).sqrt();
        let steps = ((distance / 20.0) as usize).clamp(5, 30);
        (steps, 1..=3, 20)
    };

    // 生成贝塞尔曲线路径
    let path = generate_bezier_path(current_x, current_y, target_x, target_y, steps);

    // 沿路径移动鼠标
    for (px, py) in path {
        enigo
            .move_mouse(px, py, Coordinate::Abs)
            .map_err(|e| format!("Failed to move mouse: {:?}", e))?;

        // 动态延迟
        if *step_delay_range.end() > 0 {
            let mut rng = rand::thread_rng();
            let delay_ms = rng.gen_range(step_delay_range.clone());
            if delay_ms > 0 {
                thread::sleep(Duration::from_millis(delay_ms));
            }
        }
    }

    // 到达目标位置后，动态停顿
    if reaction_delay_ms > 0 {
        thread::sleep(Duration::from_millis(reaction_delay_ms));
    }

    // 执行左键点击（按下并释放）
    enigo
        .button(Button::Left, Direction::Click)
        .map_err(|e| format!("Failed to click mouse: {:?}", e))?;

    Ok(())
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
        let events_len = events.len();

        for (i, event) in events.iter().enumerate() {
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

            // 计算到下一个事件的时间间隔
            let time_to_next = if i + 1 < events_len {
                events[i + 1].time - event.time
            } else {
                1.0 // 最后一个事件，给予充足时间
            };

            // 模拟鼠标点击，传入时间间隔以控制速度
            if let Err(e) = simulate_mouse_click(&mut enigo, event.x, event.y, time_to_next) {
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
/// 等待用户点击鼠标，返回点击位置的坐标
pub async fn pick_coordinate() -> Result<(i32, i32), String> {
    // 创建 Enigo 实例获取当前鼠标位置
    let enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Failed to create Enigo instance: {:?}", e))?;

    // 获取当前鼠标位置
    let (x, y) = enigo
        .location()
        .map_err(|e| format!("Failed to get mouse location: {:?}", e))?;

    // 注意：这是一个简化实现
    // 实际应该监听鼠标点击事件，但 enigo 不直接支持事件监听
    // 这里返回当前位置作为示例
    // 在实际使用中，前端应该通过其他方式（如浏览器API）获取点击坐标
    Ok((x, y))
}
