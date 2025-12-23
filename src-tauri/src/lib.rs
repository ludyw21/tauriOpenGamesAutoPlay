mod keypress_simulator;
mod midi_analyzer;
mod mouse_simulator;

use std::sync::Mutex;
use uni_window::WindowInfo;

// Define Global Locked Window State
lazy_static::lazy_static! {
    static ref LOCKED_WINDOW: Mutex<Option<WindowInfo>> = Mutex::new(None);
}

#[tauri::command]
fn get_windows() -> Result<Vec<WindowInfo>, String> {
    uni_window::enumerate_windows().map_err(|e| e.to_string())
}

#[tauri::command]
fn lock_window(window: WindowInfo) {
    let mut locked = LOCKED_WINDOW.lock().unwrap();
    *locked = Some(window);
}

#[tauri::command]
fn unlock_window() {
    let mut locked = LOCKED_WINDOW.lock().unwrap();
    *locked = None;
}

#[tauri::command]
fn get_locked_window() -> Option<WindowInfo> {
    let locked = LOCKED_WINDOW.lock().unwrap();
    locked.clone()
}

fn try_activate_locked_window() -> Result<(), String> {
    let locked = LOCKED_WINDOW.lock().unwrap();
    if let Some(ref window) = *locked {
        #[cfg(target_os = "windows")]
        uni_window::activate_window(window.id).map_err(|e| e.to_string())?;
        
        #[cfg(target_os = "macos")]
        uni_window::activate_window_by_pid(window.pid).map_err(|e| e.to_string())?;
        
        // Wait a bit for window to actually activate
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    Ok(())
}

#[tauri::command]
fn parse_midi(
    file_path: &str,
    min_note: u8,
    max_note: u8,
    black_key_mode: &str,
    trim_long_notes: bool,
) -> Result<midi_analyzer::MidiAnalysis, String> {
    midi_analyzer::analyze_midi_file(
        file_path,
        min_note,
        max_note,
        black_key_mode,
        trim_long_notes,
    )
}

#[tauri::command]
fn start_playback(events: Vec<keypress_simulator::KeyEvent>) -> Result<(), String> {
    try_activate_locked_window()?;
    keypress_simulator::start_playback(events)
}

#[tauri::command]
fn stop_playback() -> Result<(), String> {
    keypress_simulator::stop_playback()
}

#[tauri::command]
fn start_mouse_playback(events: Vec<mouse_simulator::MouseEvent>) -> Result<(), String> {
    try_activate_locked_window()?;
    mouse_simulator::start_mouse_playback(events)
}

#[tauri::command]
fn stop_mouse_playback() -> Result<(), String> {
    mouse_simulator::stop_mouse_playback()
}

#[tauri::command]
async fn pick_mouse_coordinate() -> Result<(i32, i32), String> {
    mouse_simulator::pick_coordinate().await
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_window_state::Builder::default().build()) // Add this line
        .plugin(tauri_plugin_dialog::init()) // Add this line
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            greet,
            parse_midi,
            start_playback,
            stop_playback,
            start_mouse_playback,
            stop_mouse_playback,
            pick_mouse_coordinate,
            get_windows,
            lock_window,
            unlock_window,
            get_locked_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
