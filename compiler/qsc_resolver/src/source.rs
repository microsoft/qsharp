use std::sync::Arc;

pub struct Source {
    pub source: Arc<str>,
    /// whether or not this module has already had its dependencies inspected
    pub inspected: bool,
}

impl Source {
    pub fn new((_, file_contents): (Arc<str>, Arc<str>)) -> Self {
        Self {
            source: file_contents,
            inspected: false,
        }
    }
}
