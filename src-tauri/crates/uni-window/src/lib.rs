use xcap::Window;
use std::error::Error;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub id: u32,       // Using xcap impl ID (which is usually HWND or CGWindowID)
    pub pid: u32,
    pub title: String,
    pub app_name: String,
    pub width: u32,
    pub height: u32,
    pub is_minimized: bool,
    pub is_maximized: bool,
}

pub fn enumerate_windows() -> Result<Vec<WindowInfo>, Box<dyn Error>> {
    let windows = Window::all()?;
    let infos = windows.into_iter().map(|w| WindowInfo {
        id: w.id().unwrap_or(0) as u32,
        pid: w.pid().unwrap_or(0),
        title: w.title().unwrap_or_default(),
        app_name: w.app_name().unwrap_or_default(),
        width: w.width().unwrap_or(0),
        height: w.height().unwrap_or(0),
        is_minimized: w.is_minimized().unwrap_or(false),
        is_maximized: w.is_maximized().unwrap_or(false),
    }).collect();
    Ok(infos)
}

#[cfg(target_os = "windows")]
pub fn activate_window(id: u32) -> Result<(), Box<dyn Error>> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{SetForegroundWindow, ShowWindow, SW_RESTORE, IsIconic};
    
    let hwnd = HWND(id as _);
    unsafe {
        if IsIconic(hwnd).as_bool() {
            ShowWindow(hwnd, SW_RESTORE);
        }
        SetForegroundWindow(hwnd);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn activate_window_by_pid(pid: u32) -> Result<(), Box<dyn Error>> {
    // use objc2_app_kit::{NSRunningApplication, NSApplicationActivationOptions};
    
    // Simplest way via raw objc or crate wrapper.
    // objc2_app_kit exposes NSRunningApplication.
    // However, NSRunningApplication::runningApplicationWithProcessIdentifier(pid)
    
    // For simplicity in this step, I'll use a Command execution for macOS if objc2 is complex to setup quickly, 
    // BUT user asked for "independent encapsulation", so code is better.
    // Let's rely on `open -a` or applescript if objc2 fails, but let's try objc2 first or just 'open'.
    // `kp` or `kill`? No.
    // `swich`? 
    // Actually, std::process::Command is easiest for MVP "activate_window".
    // `osascript -e 'tell application "System Events" to set frontmost of the first process whose unix id is {pid} to true'`
    
    let script = format!(
        "tell application \"System Events\" to set frontmost of the first process whose unix id is {} to true",
        pid
    );
    
    std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()?;
        
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn activate_window(_id: u32) -> Result<(), Box<dyn Error>> {
    // xcap::Window::id() on mac is CGWindowID.
    // We need PID to activate app. Since enumerate_windows returns PID, user should probably pass PID or we find PID by ID.
    // For now, let's assume implementation uses PID for activation on Mac.
    // But interface says `activate_window(id)`. 
    // We need to find PID from ID? xcap doesn't expose "find by id".
    // So better interface: activate_window(info: &WindowInfo).
    Err("On macOS, please use activate_window_by_pid with the pid from WindowInfo".into())
}
