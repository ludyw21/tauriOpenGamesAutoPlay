use enigo::{Button, Coordinate, Direction, Enigo, Mouse};
use rand::Rng;
use std::thread;
use std::time::Duration;

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

pub trait SmoothMouse {
    fn mouse_move_smooth(&mut self, target_x: i32, target_y: i32, total_duration_ms: u64) -> Result<(), String>;
    fn mouse_click_smooth(&mut self, target_x: i32, target_y: i32) -> Result<(), String>;
}

impl SmoothMouse for Enigo {
    fn mouse_move_smooth(&mut self, x: i32, y: i32, _total_duration_ms: u64) -> Result<(), String> {
         // 获取当前鼠标位置
        let (current_x, current_y) = self
            .location()
            .map_err(|e| format!("Failed to get mouse location: {:?}", e))?;

        // 应用坐标偏移 (Ref Logic)
        // Note: Reference `simulate_mouse_click` combined move + click + duration logic.
        // Here we extract move logic.
        
        let (target_x, target_y) = add_coordinate_offset(x, y);

        let dx = target_x - current_x;
        let dy = target_y - current_y;
        let distance = ((dx * dx + dy * dy) as f64).sqrt();
        
        // Simple step calculation based on distance
        let steps = ((distance / 20.0) as usize).clamp(5, 50);

        // 生成贝塞尔曲线路径
        let path = generate_bezier_path(current_x, current_y, target_x, target_y, steps);

        // 沿路径移动鼠标
        for (px, py) in path {
            self
                .move_mouse(px, py, Coordinate::Abs)
                .map_err(|e| format!("Failed to move mouse: {:?}", e))?;
            
            // Minimal sleep to smooth out
            thread::sleep(Duration::from_millis(5)); 
        }
        
        Ok(())
    }

    fn mouse_click_smooth(&mut self, target_x: i32, target_y: i32) -> Result<(), String> {
        self.mouse_move_smooth(target_x, target_y, 200)?;
        
        // Small delay before click like reference
        thread::sleep(Duration::from_millis(20));
        
        self.button(Button::Left, Direction::Click)
            .map_err(|e| format!("Failed to click mouse: {:?}", e))?;
            
        Ok(())
    }
}
