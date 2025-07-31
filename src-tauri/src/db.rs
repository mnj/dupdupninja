use rusqlite::{params, Connection, Result};
use chrono::{DateTime, Utc};
use std::path::Path;
use std::path::PathBuf;
use tauri::api::path::app_data_dir;
use std::fs;

const DB_FILE_NAME: &str = "dupdupninja.sqlite3";

#[derive(Debug)]
pub struct FileSet {
    pub id: i64,
    pub set_name: String,
    pub set_description: String,
    pub last_scan_time: DateTime<Utc>,
}

#[derive(Debug)]
pub struct FileRecord {
    pub id: i64,
    pub set_id: i64,
    pub path: String,
    pub size: u64,
    pub mtime: i64,
    pub sha256: String,    
}

pub fn get_db_path() -> PathBuf {
    let mut path = app_data_dir(&tauri::Config::default())
        .expect("Failed to get app data directory");

    fs::create_dir_all(&path).expect("Failed to create app data directory");
    
    path.push(DB_FILE_NAME);
    path
}

pub fn open_connection() -> rusqlite::Result<rusqlite::Connection> {
    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;

    // Ensure tables exist
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS file_sets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            set_name TEXT NOT NULL,
            set_description TEXT NOT NULL,
            last_scan_time DATETIME NOT NULL
        );
        CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            set_id INTEGER NOT NULL,
            path TEXT NOT NULL,
            size INTEGER NOT NULL,
            mtime INTEGER NOT NULL,
            sha256 TEXT NOT NULL,        
            FOREIGN KEY(set_id) REFERENCES file_sets(id)
        );
        "
    )?;

    Ok(conn)
}