mod midi_analyzer;

#[tauri::command]
fn parse_midi(
    file_path: &str,
    min_note: u8,
    max_note: u8,
    black_key_mode: &str,
) -> Result<midi_analyzer::MidiAnalysis, String> {
    midi_analyzer::analyze_midi_file(file_path, min_note, max_note, black_key_mode)
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
        .invoke_handler(tauri::generate_handler![greet, parse_midi])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
