use crossbeam_channel::{unbounded, Receiver};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;
use std::{
    collections::HashSet, fs::File, io::{BufReader, Read}, path::Path, sync::{atomic::AtomicBool, Arc, Mutex}, thread, time::{Duration, Instant, SystemTime, UNIX_EPOCH}
};
use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;
use std::sync::atomic::{AtomicUsize, Ordering};
use uuid::Uuid;

use crate::scan;

#[derive(Debug, Clone)]
pub struct FileRecordTemp {
    pub path: String,
    pub size: u64,
    pub mtime: i64,
    pub sha256: String,
}

fn compute_sha256_with_cancel(path: &Path, token: &CancellationToken) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();

    let mut buffer = [0u8; 8 * 1024];

    loop {
        if token.is_cancelled() {
            return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "Scan cancelled"));
        }

        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

fn stat_file(path: &Path) -> std::io::Result<(u64, i64)> {
    let metadata = path.metadata()?;
    let size = metadata.len();
    let mtime = metadata
        .modified()?
        .duration_since(UNIX_EPOCH)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
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

/// Starts the scan. Emits:
/// - "scan_started"
/// - repeated "scan_progress" with phase "discover" and `{ discovered }`
/// - repeated "scan_progress" with phase "processing" and `{ current, total, percent }`
/// - "scan_finished" / "scan_cancelled" / "scan_error"
// src-tauri/src/scan.rs (inside the same file with helpers like compute_sha256_with_cancel)

pub fn start_scan_with_id(
    path: String,
    app: AppHandle,
    cancel_token: CancellationToken,
    scan_id: String,
) {
    thread::spawn(move || {
        // --- scan_started ---
        let _ = app.emit(
            "scan_started",
            serde_json::json!({ "scan_id": scan_id.clone(), "path": path.clone() }),
        );

        // Prepare extension set
        let media_exts = [
            "mp4", "mkv", "avi", "mov", "flv", "webm",
            "mp3", "wav", "flac", "aac", "ogg",
            "jpg", "jpeg", "png", "gif", "bmp", "tiff",
        ];
        let exts: HashSet<String> = media_exts.iter().map(|s| s.to_string()).collect();

        // Shared discovery state
        let mut discovered_files = Vec::new();
        let discovered_count = Arc::new(AtomicUsize::new(0));
        let mut last_discover_emit = Instant::now();

        // ---------- DISCOVERY PHASE ----------
        for entry in WalkDir::new(&path).into_iter() {
            if cancel_token.is_cancelled() {
                let _ = app.emit(
                    "scan_cancelled",
                    serde_json::json!({ "scan_id": scan_id.clone() }),
                );
                return;
            }
            if let Ok(e) = entry {
                if e.file_type().is_file() {
                    if let Some(ext_os) = e.path().extension().and_then(|x| x.to_str()) {
                        if exts.contains(&ext_os.to_lowercase()) {
                            let fp = e.path().display().to_string();
                            discovered_files.push(fp);
                            discovered_count.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                }
            }
            // emit every 300ms
            if last_discover_emit.elapsed() >= Duration::from_millis(300) {
                let _ = app.emit(
                    "scan_progress",
                    serde_json::json!({
                        "scan_id": scan_id.clone(),
                        "phase": "discover",
                        "discovered": discovered_count.load(Ordering::SeqCst),
                    }),
                );
                last_discover_emit = Instant::now();
            }
        }
        // Final discovery emit
        let total_discovered = discovered_count.load(Ordering::SeqCst);
        let _ = app.emit(
            "scan_progress",
            serde_json::json!({
                "scan_id": scan_id.clone(),
                "phase": "discover",
                "discovered": total_discovered,
            }),
        );

        // ---------- PROCESSING PHASE ----------
        let total = total_discovered.max(1);
        let processed = Arc::new(AtomicUsize::new(0));
        let records: Arc<Mutex<Vec<FileRecordTemp>>> =
            Arc::new(Mutex::new(Vec::with_capacity(total)));
        let mut last_processing_emit = Instant::now();
        let mut last_percent = -1i64;

        // Feed channel
        let (tx, rx) = crossbeam_channel::bounded::<String>(total);
        for fp in discovered_files {
            if cancel_token.is_cancelled() {
                let _ = app.emit(
                    "scan_cancelled",
                    serde_json::json!({ "scan_id": scan_id.clone() }),
                );
                return;
            }
            let _ = tx.send(fp);
        }
        drop(tx);

        // Worker threads
        let parallelism = std::thread::available_parallelism()
            .map(|n| n.get().saturating_sub(1).max(1))
            .unwrap_or(1);
        let mut handles = Vec::new();
        for _ in 0..parallelism {
            let rx = rx.clone();
            let proc = Arc::clone(&processed);
            let recs = Arc::clone(&records);
            let cancel = cancel_token.clone();
            let scan_id_clone = scan_id.clone();
            let app_evt = app.clone();

            let h = thread::spawn(move || {
                while let Ok(file_path) = rx.recv() {
                    if cancel.is_cancelled() {
                        break;
                    }
                    // stat + hash
                    if let Ok((size, mtime)) = stat_file(Path::new(&file_path)) {
                        if let Ok(sha256) = compute_sha256_with_cancel(Path::new(&file_path), &cancel)
                        {
                            let mut g = recs.lock().unwrap();
                            g.push(FileRecordTemp {
                                path: file_path.clone(),
                                size,
                                mtime,
                                sha256,
                            });
                        }
                    }
                    proc.fetch_add(1, Ordering::SeqCst);
                }
            });
            handles.push(h);
        }

        // Progress loop
        loop {
            if cancel_token.is_cancelled() {
                let _ = app.emit(
                    "scan_cancelled",
                    serde_json::json!({ "scan_id": scan_id.clone() }),
                );
                return;
            }
            let done = processed.load(Ordering::SeqCst);
            let pct = ((done * 100) / total).min(100) as i64;
            if pct != last_percent || last_processing_emit.elapsed() >= Duration::from_millis(500) {
                let _ = app.emit(
                    "scan_progress",
                    serde_json::json!({
                        "scan_id": scan_id.clone(),
                        "phase": "processing",
                        "current": done,
                        "total": total,
                        "percent": pct,
                    }),
                );
                last_percent = pct;
                last_processing_emit = Instant::now();
            }
            if done >= total {
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }

        // Join workers
        for h in handles {
            let _ = h.join();
        }

        if cancel_token.is_cancelled() {
            let _ = app.emit(
                "scan_cancelled",
                serde_json::json!({ "scan_id": scan_id.clone() }),
            );
            return;
        }

        // Final records
        let final_records = { records.lock().unwrap().clone() };
        // TODO: persist `final_records` here

        let _ = app.emit(
            "scan_finished",
            serde_json::json!({ "scan_id": scan_id.clone(), "count": final_records.len() }),
        );
    });
}
