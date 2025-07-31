use tauri::{AppHandle, Emitter};
use walkdir::WalkDir;
use std::{collections::HashSet, path::Path};

fn get_media_files<P: AsRef<Path>>(path: P) -> Vec<String> {
    
    let media_exts = [
        "mp4", "mkv", "avi", "mov", "flv", "webm", // video
        "mp3", "wav", "flac", "aac", "ogg",        // audio
        "jpg", "jpeg", "png", "gif", "bmp", "tiff", // image
    ];
    let exts: HashSet<_> = media_exts.iter().cloned().collect();
    
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| exts.contains(&ext.to_lowercase().as_str()))
                .unwrap_or(false)
        })
         .map(|entry| entry.path().display().to_string())
        .collect()

}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn scan_folder(path: String, app: AppHandle) -> Result<(), String> {

    println!("Scanning folder: {}", path);

    let app_clone = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let files = get_media_files(&path);
        let total = files.len().max(1);
        for(idx, file_path) in files.iter().enumerate() {
            let progress = (idx + 1) as f64 / total as f64 * 100.0;
            println!("Processing file: {} ({:.2}%)", file_path, progress);
            
            // Update the Tauri app with the progress
             let _ = app_clone.emit(
                "scan_progress",
                serde_json::json!({
                    "current": idx + 1,
                    "total": total,
                    "percent": ((idx + 1) * 100 / total).min(100),
                    "file": file_path,
                }),
            );

            // For now, we just simulate a delay
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });


    // Placeholder for folder scanning logic
    // This should be replaced with actual scanning code
    
    Ok(())
    //Ok(format!("Scanning folder: {}", path))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, scan_folder])        
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
