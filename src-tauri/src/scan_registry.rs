use once_cell::sync::Lazy;
use dashmap::DashMap;
use tokio_util::sync::CancellationToken;

pub struct ScanRegistry {
    inner: DashMap<String, CancellationToken>,
}

impl ScanRegistry {
    pub fn register(&self, scan_id: String, cancellation_token: CancellationToken) {
        self.inner.insert(scan_id, cancellation_token);
    }

    pub fn cancel(&self, scan_id: &str) {
        if let Some(entry) = self.inner.get(scan_id) {
            entry.cancel();            
        }
    }

    pub fn get(&self, scan_id: &str) -> Option<CancellationToken> {
        self.inner.get(scan_id).map(|entry| entry.clone())
    }

    pub fn unregister(&self, scan_id: &str) {
        self.inner.remove(scan_id);
    }
}

pub static SCAN_REGISTRY: Lazy<ScanRegistry> = Lazy::new(|| ScanRegistry {
    inner: DashMap::new(),
});