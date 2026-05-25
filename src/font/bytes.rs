use alloc::sync::Arc;

pub enum Bytes {
    Static(&'static [u8]),
    Arc(Arc<[u8]>),
}

impl From<&'static [u8]> for Bytes {
    fn from(value: &'static [u8]) -> Self {
        Self::Static(value)
    }
}

impl From<Arc<[u8]>> for Bytes {
    fn from(value: Arc<[u8]>) -> Self {
        Self::Arc(value)
    }
}
