use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};

static LAST_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub(crate) struct OperationId(u64);

impl Clone for OperationId {
    fn clone(&self) -> Self {
        OperationId::default()
    }
}

impl Default for OperationId {
    fn default() -> Self {
        Self(LAST_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl Deref for OperationId {
    type Target = u64;

    fn deref(&self) -> &u64 {
        &self.0
    }
}
