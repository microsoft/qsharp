// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::state::CompilationStateUpdater;
use async_trait::async_trait;
use qsc_project::{FileSystemAsync, JSFileEntry};

#[async_trait(?Send)]
impl qsc_project::FileSystemAsync for CompilationStateUpdater<'_> {
    type Entry = JSFileEntry;
    async fn read_file(
        &self,
        path: &std::path::Path,
    ) -> miette::Result<(std::sync::Arc<str>, std::sync::Arc<str>)> {
        FileSystemAsync::read_file(&self.fs_callbacks, path).await
    }

    async fn list_directory(&self, path: &std::path::Path) -> miette::Result<Vec<Self::Entry>> {
        FileSystemAsync::list_directory(&self.fs_callbacks, path).await
    }

    async fn resolve_path(
        &self,
        base: &std::path::Path,
        path: &std::path::Path,
    ) -> miette::Result<std::path::PathBuf> {
        FileSystemAsync::resolve_path(&self.fs_callbacks, base, path).await
    }

    async fn fetch_github(
        &self,
        owner: &str,
        repo: &str,
        r#ref: &str,
        path: &str,
    ) -> miette::Result<std::sync::Arc<str>> {
        FileSystemAsync::fetch_github(&self.fs_callbacks, owner, repo, r#ref, path).await
    }
}
