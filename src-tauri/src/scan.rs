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
pub fn start_scan_with_id(path: String, app: AppHandle, cancel_token: CancellationToken, scan_id: String) {
     thread::spawn(move || {
        let _ = app.emit(
            "scan_started",
            serde_json::json!({ "scan_id": scan_id.clone(), "path": path.clone() }),
        );

        // Define media file extensions we want to scan
        let media_exts = [
            "mp4", "mkv", "avi", "mov", "flv", "webm", // video
            "mp3", "wav", "flac", "aac", "ogg",        // audio
            "jpg", "jpeg", "png", "gif", "bmp", "tiff", // image
        ];
        let exts: HashSet<String> = media_exts.iter().map(|s| s.to_string()).collect();

        // Channel for discovered file paths
        let (tx_paths, rx_paths) = unbounded::<String>();

        // Shared state
        let discovered_count = Arc::new(AtomicUsize::new(0));
        let processed = Arc::new(AtomicUsize::new(0));
        let records: Arc<Mutex<Vec<FileRecordTemp>>> = Arc::new(Mutex::new(Vec::new()));
        let discovery_done = Arc::new(AtomicBool::new(false));
        
        let mut last_discover_emit = Instant::now();
        let mut last_processing_emit = Instant::now();
        let mut last_percent_sent = -1i64;
        let mut processing_phase_active = false;

        // Discovery thread
        {
            let tx_paths = tx_paths.clone();
            let discovered_count = Arc::clone(&discovered_count);
            let cancel = cancel_token.clone();
            let app_clone = app.clone();
            let scan_id_clone = scan_id.clone();
            let discovery_done_clone = Arc::clone(&discovery_done);
            
            thread::spawn(move || {
                for entry in WalkDir::new(&path).into_iter() {
                    if cancel.is_cancelled() {
                        break;
                    }

                    match entry {
                        Ok(e) => {
                            if e.file_type().is_file() {
                                if let Some(ext_os) = e.path().extension() {
                                    if let Some(ext) = ext_os.to_str() {
                                        if exts.contains(&ext.to_lowercase()) {
                                            let file_str = e.path().display().to_string();
                                            let _ = tx_paths.send(file_str);
                                            discovered_count.fetch_add(1, Ordering::SeqCst);
                                        }
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            // ignore problematic entries
                        }
                    }

                    // Throttled discovery progress emit (every ~300ms)
                    if last_discover_emit.elapsed() >= Duration::from_millis(300) {
                        let _ = app_clone.emit(
                            "scan_progress",
                            serde_json::json!({
                                "scan_id": scan_id_clone,
                                "phase": "discover",
                                "discovered": discovered_count.load(Ordering::SeqCst),
                            }),
                        );
                        last_discover_emit = Instant::now();                        
                    }
                }
                // signal completion of discovery
                discovery_done_clone.store(true, Ordering::SeqCst);
                drop(tx_paths);
            })
        };

        // Processing phase
        let parallelism = std::thread::available_parallelism()
            .map(|n| n.get().saturating_sub(1).max(1))
            .unwrap_or(1);

        let mut worker_handles = Vec::with_capacity(parallelism);
        for _ in 0..parallelism {
            let rx = rx_paths.clone();
            let processed = Arc::clone(&processed);
            let records = Arc::clone(&records);
            let cancel = cancel_token.clone();

            let handle = thread::spawn(move || {
                while let Ok(file_path) = rx.recv() {
                    if cancel.is_cancelled() {
                        break;
                    }

                    let mut maybe_rec: Option<FileRecordTemp> = None;
                    let path_obj = Path::new(&file_path);
                    match stat_file(path_obj) {
                        Ok((size, mtime)) => match compute_sha256_with_cancel(path_obj, &cancel) {
                            Ok(sha256) => {
                                maybe_rec = Some(FileRecordTemp {
                                    path: file_path.clone(),
                                    size,
                                    mtime,
                                    sha256,
                                });
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {
                                break; // cancelled during hashing
                            }
                            Err(e) => {
                                eprintln!("Failed to process {}: {}", file_path, e);
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to stat {}: {}", file_path, e);
                        }
                    }

                    if let Some(rec) = maybe_rec {
                        let mut guard = records.lock().unwrap();
                        guard.push(rec);
                    }

                    processed.fetch_add(1, Ordering::SeqCst);
                }
            });
            worker_handles.push(handle);
        }

        // --- Unified progress emitter ---
        loop {
            if cancel_token.is_cancelled() {
                let _ = app.emit(
                    "scan_cancelled",
                    serde_json::json!({ "scan_id": scan_id.clone() }),
                );
                return;
            }

            // Always emit latest discover progress while discovery is still ongoing
            if !discovery_done.load(Ordering::SeqCst) {
                let discovered = discovered_count.load(Ordering::SeqCst);
                if last_discover_emit.elapsed() >= Duration::from_millis(300) {
                    let _ = app.emit(
                        "scan_progress",
                        serde_json::json!({
                            "scan_id": scan_id.clone(),
                            "phase": "discover",
                            "discovered": discovered,
                        }),
                    );
                    last_discover_emit = Instant::now();
                }
            }

            // Once any processing has begun (processed > 0), switch to processing phase
            let done = processed.load(Ordering::SeqCst);
            let current_total = discovered_count.load(Ordering::SeqCst).max(1); // dynamic total
            if done > 0 {
                processing_phase_active = true;
            }

            if processing_phase_active {
                let percent = ((done * 100) / current_total).min(100) as i64;
                if percent != last_percent_sent || last_processing_emit.elapsed() >= Duration::from_millis(500) {
                    // emit processing progress
                    let _ = app.emit(
                        "scan_progress",
                        serde_json::json!({
                            "scan_id": scan_id.clone(),
                            "phase": "processing",
                            "current": done,
                            "total": current_total,
                            "percent": percent
                        }),
                    );
                    last_percent_sent = percent;
                    last_processing_emit = Instant::now();
                }
            }

            // Termination: discovery done AND all discovered items processed
            if discovery_done.load(Ordering::SeqCst) && done >= discovered_count.load(Ordering::SeqCst) {
                break;
            }

            thread::sleep(Duration::from_millis(100));
        }

         // join workers
        for h in worker_handles {
            let _ = h.join();
        }

        if cancel_token.is_cancelled() {
            let _ = app.emit(
                "scan_cancelled",
                serde_json::json!({ "scan_id": scan_id.clone() }),
            );
            return;
        }

        // collect final records
        let final_records = {
            let guard = records.lock().unwrap();
            guard.clone()
        };

        // TODO: persist `final_records` into your rusqlite DB here.

        let _ = app.emit(
            "scan_finished",
            serde_json::json!({ "scan_id": scan_id.clone(), "count": final_records.len() }),
        );
    });
}

trait ThreadExt {
    fn is_running(&self) -> bool;
}
impl ThreadExt for thread::JoinHandle<()> {
    fn is_running(&self) -> bool {
        // There's no stable way to check without additional signaling. For simplicity,
        // assume discovery is fast; if you need precise coordination, replace with a shared flag.
        false
    }
}