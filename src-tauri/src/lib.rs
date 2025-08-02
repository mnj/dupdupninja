mod scan;
mod scan_registry;

use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::scan_registry::SCAN_REGISTRY;
use crate::scan::start_scan_with_id;

use tokio_util::sync::CancellationToken;

#[tauri::command]
async fn scan_folder(path: String, app: AppHandle) -> Result<String /* ScanId */, String> {

    let scan_id = Uuid::new_v4().to_string();
    let token = CancellationToken::new();
    SCAN_REGISTRY.register(scan_id.clone(), token.clone());
    start_scan_with_id(path, app.clone(), token.clone(), scan_id.clone());

    Ok(scan_id)
    // let scan_id = uuid::Uuid::new_v4().to_string();
    // println!("Scanning folder: {}, Scan Id: {}", path, scan_id);
    // let cancel_token = CancellationToken::new();

    // let app_clone = app.clone();
    // tauri::async_runtime::spawn_blocking(move || {

    //     // Let frontend know we are starting the scan
    //     let _ = app_clone.emit("scan_started", serde_json::json!({ }));

    //     let files = get_media_files(&path);
    //     let total = files.len().max(1);

    //     let mut last_percent_sent = -1i64;
    //     let mut last_emit_instant = std::time::Instant::now();

    //     let mut records: Vec<FileRecordTemp> = Vec::with_capacity(files.len());

    //     for (idx, file_path) in files.iter().enumerate() {

    //         let path = Path::new(file_path);
    //         let (size, mtime) = match stat_file(path) {
    //             Ok((size, mtime)) => (size, mtime),
    //             Err(e) => {
    //                 eprintln!("Error reading file {}: {}", file_path, e);
    //                 continue; // Skip this file
    //             }
    //         };

    //         let sha256 = match compute_sha256(path) {
    //             Ok(sha256) => sha256,
    //             Err(e) => {
    //                 eprintln!("Error computing SHA256 for {}: {}", file_path, e);
    //                 continue; // Skip this file
    //             }
    //         };

    //         records.push(FileRecordTemp {
    //             path: file_path.clone(),
    //             size,
    //             mtime,
    //             sha256: sha256.clone(),
    //         });

    //         // print debug
    //         println!("Processed file: {} ({} bytes, {} mtime, {})", file_path, size, mtime, sha256);

    //         let percent = ((idx + 1) * 100 / total).min(100);
            
    //         // Rate limit: send if integer percent changed OR if >500ms passed (to cover very slow progress)
    //         let percent_int = percent as i64;
    //         if percent_int != last_percent_sent || last_emit_instant.elapsed() >= std::time::Duration::from_millis(500)
    //         {
    //             let _ = app_clone.emit(
    //                 "scan_progress",
    //                 serde_json::json!({
    //                     "current": idx + 1,
    //                     "total": total,
    //                     "percent": percent,
    //                 }),
    //             );
    //         }

    //         last_percent_sent = percent_int;
    //         last_emit_instant = std::time::Instant::now();
    //     }

    //     // Let frontend know we are done scanning
    //     let _ = app_clone.emit("scan_finished", serde_json::json!({ }));
    // });


    // Placeholder for folder scanning logic
    // This should be replaced with actual scanning code
    
}

#[tauri::command]
fn cancel_scan(scan_id: String) {
    SCAN_REGISTRY.cancel(&scan_id);
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
        .invoke_handler(tauri::generate_handler![scan_folder, cancel_scan])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
