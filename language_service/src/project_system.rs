use crate::LanguageService;
use async_trait::async_trait;
use std::{
    future::{self, Future},
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
};

#[async_trait(?Send)]
impl qsc_project::FileSystem for LanguageService<'_> {
    type Entry = PathBuf;
    async fn read_file(
        &self,
        path: &std::path::Path,
    ) -> miette::Result<(std::sync::Arc<str>, std::sync::Arc<str>)> {
        Ok((self.read_file)(path.into()).await)
    }

    async fn list_directory(&self, path: &std::path::Path) -> miette::Result<Vec<Self::Entry>> {
        Ok((self.list_directory)(path.into()).await)
    }
}
