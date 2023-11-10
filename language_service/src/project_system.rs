use crate::LanguageService;
use async_trait::async_trait;
use std::{
    future::{self, Future},
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
};

#[async_trait]
impl qsc_project::FileSystem for LanguageService<'_> {
    type Entry = PathBuf;
    async fn read_file(&self, path: &Path) -> miette::Result<(Arc<str>, Arc<str>)> {
        todo!()
    }

    async fn list_directory(&self, path: &Path) -> miette::Result<Vec<Self::Entry>> {
        todo!()
    }
    // async fn read_file(
    //     &self,
    //     path: &std::path::Path,
    // ) -> miette::Result<(std::sync::Arc<str>, std::sync::Arc<str>)> {
    //     Ok((self.read_file)(path.into()))
    // }

    // async fn list_directory(
    //     &self,
    //     path: &std::path::Path,
    // ) -> Pin<Box<dyn Future<Output = miette::Result<Vec<Self::Entry>>>>> {
    //     Box::pin(future::ready(Ok((self.list_directory)(path.into()))))
    // }
}
