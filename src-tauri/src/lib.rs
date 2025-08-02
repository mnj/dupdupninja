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
