use std::error::Error;
use std::thread;
use std::time::Duration;
use enigo::{Enigo, Settings};
#[cfg(target_os = "windows")]
use uni_window::activate_window;
#[cfg(target_os = "macos")]
use uni_window::activate_window_by_pid;
use uni_window::WindowInfo;

pub mod mouse;
pub mod keyboard;

pub use mouse::SmoothMouse;
pub use keyboard::SmartKeyboard;

pub struct InputController {
    pub enigo: Enigo,
}

impl InputController {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let enigo = Enigo::new(&Settings::default())?;
        Ok(Self { enigo })
    }

    pub fn send_key_to_window(&mut self, target: &WindowInfo, key: char) -> Result<(), Box<dyn Error>> {
        // 1. Activate Window
        self.activate_target(target)?;

        // 2. Wait a bit for focus switch
        thread::sleep(Duration::from_millis(50));

        // 3. Send Key
        self.enigo.simulate_keypress_smart(&key.to_string())?;
        
        Ok(())
    }

    pub fn send_text_to_window(&mut self, target: &WindowInfo, text: &str) -> Result<(), Box<dyn Error>> {
        println!("Activating window: {}", target.title);
        self.activate_target(target)?;
        
        println!("Waiting 500ms for window focus...");
        thread::sleep(Duration::from_millis(500));
        
        println!("Sending text: {}", text);
        for c in text.chars() {
            self.enigo.simulate_keypress_smart(&c.to_string())?;
            thread::sleep(Duration::from_millis(50)); // Increased from 10 to 50 for reliability
        }
        println!("Text sent successfully.");
        Ok(())
    }

    fn activate_target(&self, target: &WindowInfo) -> Result<(), Box<dyn Error>> {
        #[cfg(target_os = "macos")]
        activate_window_by_pid(target.pid)?;
        
        #[cfg(target_os = "windows")]
        activate_window(target.id)?;
        Ok(())
    }
}
