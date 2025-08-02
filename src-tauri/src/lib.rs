use tauri::{AppHandle, Emitter};
use walkdir::WalkDir;
use std::{collections::HashSet, path::Path, sync::Arc};

use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::time::{SystemTime, UNIX_EPOCH};

use tokio_util::sync::CancellationToken;
use tokio::sync::Semaphore;


#[derive(Debug)]
struct FileRecordTemp {
    path: String,
    size: u64,
    mtime: i64,
    sha256: String,
}

fn compute_sha256(path: &Path) -> io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();

    let mut buffer = [0u8; 8 * 1024];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

fn stat_file(path: &Path) -> io::Result<(u64, i64)> {
    let metadata = path.metadata()?;
    let size = metadata.len();
    let mtime = metadata
        .modified()?
        .duration_since(UNIX_EPOCH)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
        .as_secs() as i64;
    Ok((size, mtime))
}

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

#[tauri::command]
async fn scan_folder(path: String, app: AppHandle) -> Result<(), String> {

    println!("Scanning folder: {}", path);

    let app_clone = app.clone();
    tauri::async_runtime::spawn_blocking(move || {

        // Let frontend know we are starting the scan
        let _ = app_clone.emit("scan_started", serde_json::json!({ }));

        let files = get_media_files(&path);
        let total = files.len().max(1);

        let mut last_percent_sent = -1i64;
        let mut last_emit_instant = std::time::Instant::now();

        let mut records: Vec<FileRecordTemp> = Vec::with_capacity(files.len());

        for (idx, file_path) in files.iter().enumerate() {

            let path = Path::new(file_path);
            let (size, mtime) = match stat_file(path) {
                Ok((size, mtime)) => (size, mtime),
                Err(e) => {
                    eprintln!("Error reading file {}: {}", file_path, e);
                    continue; // Skip this file
                }
            };

            let sha256 = match compute_sha256(path) {
                Ok(sha256) => sha256,
                Err(e) => {
                    eprintln!("Error computing SHA256 for {}: {}", file_path, e);
                    continue; // Skip this file
                }
            };

            records.push(FileRecordTemp {
                path: file_path.clone(),
                size,
                mtime,
                sha256: sha256.clone(),
            });

            // print debug
            println!("Processed file: {} ({} bytes, {} mtime, {})", file_path, size, mtime, sha256);

            let percent = ((idx + 1) * 100 / total).min(100);
            
            // Rate limit: send if integer percent changed OR if >500ms passed (to cover very slow progress)
            let percent_int = percent as i64;
            if percent_int != last_percent_sent || last_emit_instant.elapsed() >= std::time::Duration::from_millis(500)
            {
                let _ = app_clone.emit(
                    "scan_progress",
                    serde_json::json!({
                        "current": idx + 1,
                        "total": total,
                        "percent": percent,
                    }),
                );
            }

            last_percent_sent = percent_int;
            last_emit_instant = std::time::Instant::now();
        }

        // Let frontend know we are done scanning
        let _ = app_clone.emit("scan_finished", serde_json::json!({ }));
    });


    // Placeholder for folder scanning logic
    // This should be replaced with actual scanning code
    
    Ok(())
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
        .invoke_handler(tauri::generate_handler![scan_folder])        
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
