use crossbeam_channel::{unbounded, Receiver};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;
use std::{
    collections::HashSet, fs::File, io::{BufReader, Read}, path::Path, sync::{Arc, Mutex}, thread, time::{Duration, Instant, SystemTime, UNIX_EPOCH}
};
use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;
use std::sync::atomic::{AtomicUsize, Ordering};
use uuid::Uuid;

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

pub fn start_scan_with_id(path: String, app: AppHandle, token: CancellationToken, scan_id: String) {
     thread::spawn(move || {
        let _ = app.emit(
            "scan_started",
            serde_json::json!({ "scan_id": scan_id.clone(), "path": path.clone() }),
        );

        let files = get_media_files(&path);
        let total = files.len().max(1);
        let parallelism = std::thread::available_parallelism()
            .map(|n| n.get().saturating_sub(1).max(1))
            .unwrap_or(1);

        let (sender, receiver) = unbounded::<String>();
        for f in files.iter().cloned() {
            let _ = sender.send(f);
        }
        drop(sender); // close so workers eventually exit

        let processed = Arc::new(AtomicUsize::new(0));
        let records: Arc<Mutex<Vec<FileRecordTemp>>> =
            Arc::new(Mutex::new(Vec::with_capacity(total)));

        // Spawn worker threads
        let mut handles = Vec::with_capacity(parallelism);
        for _ in 0..parallelism {
            let rx: Receiver<String> = receiver.clone();
            let processed = Arc::clone(&processed);
            let records = Arc::clone(&records);
            let cancel = token.clone();

            let handle = thread::spawn(move || {
                while let Ok(file_path) = rx.recv() {
                    if cancel.is_cancelled() {
                        break;
                    }

                    let mut maybe_record: Option<FileRecordTemp> = None;
                    let path_obj = Path::new(&file_path);

                    if let Ok((size, mtime)) = stat_file(path_obj) {
                        match compute_sha256_with_cancel(path_obj, &cancel) {
                            Ok(sha256) => {
                                maybe_record = Some(FileRecordTemp {
                                    path: file_path.clone(),
                                    size,
                                    mtime,
                                    sha256,
                                });
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {
                                break;
                            }
                            Err(e) => {
                                eprintln!("hash error {}: {}", file_path, e);
                            }
                        }
                    } else {
                        eprintln!("stat error {}", file_path);
                    }

                    if let Some(rec) = maybe_record {
                        let mut guard = records.lock().unwrap();
                        guard.push(rec);
                    }

                    processed.fetch_add(1, Ordering::SeqCst);
                }
            });
            handles.push(handle);
        }

        // Centralized progress emitter
        let mut last_percent = -1i64;
        let mut last_emit = Instant::now();
        loop {
            if token.is_cancelled() {
                let _ = app.emit(
                    "scan_cancelled",
                    serde_json::json!({ "scan_id": scan_id.clone() }),
                );
                return;
            }

            let done = processed.load(Ordering::SeqCst);
            let percent = ((done * 100) / total).min(100) as i64;
            let now = Instant::now();
            if percent != last_percent || now.duration_since(last_emit) >= Duration::from_millis(500) {
                let _ = app.emit(
                    "scan_progress",
                    serde_json::json!({
                        "scan_id": scan_id.clone(),
                        "current": done,
                        "total": total,
                        "percent": percent
                    }),
                );
                last_percent = percent;
                last_emit = now;
            }

            if done >= total {
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }

        // Wait for workers
        for h in handles {
            let _ = h.join();
        }

        if token.is_cancelled() {
            let _ = app.emit(
                "scan_cancelled",
                serde_json::json!({ "scan_id": scan_id.clone() }),
            );
            return;
        }

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