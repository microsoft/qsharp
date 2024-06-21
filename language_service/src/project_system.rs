// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::state::CompilationStateUpdater;
use async_trait::async_trait;
use qsc_project::JSFileEntry;

#[async_trait(?Send)]
impl qsc_project::FileSystemAsync for CompilationStateUpdater<'_> {
    type Entry = JSFileEntry;
    async fn read_file(
        &self,
        path: &std::path::Path,
    ) -> miette::Result<(std::sync::Arc<str>, std::sync::Arc<str>)> {
        Ok((self.fs_callbacks.read_file)(path.to_string_lossy().to_string()).await)
    }

    async fn list_directory(&self, path: &std::path::Path) -> miette::Result<Vec<Self::Entry>> {
        Ok((self.fs_callbacks.list_directory)(path.to_string_lossy().to_string()).await)
    }

    async fn resolve_path(
        &self,
        base: &std::path::Path,
        path: &std::path::Path,
    ) -> miette::Result<std::path::PathBuf> {
        Ok((self.fs_callbacks.resolve_path)((
            base.to_string_lossy().to_string(),
            path.to_string_lossy().to_string(),
        ))
        .await
        .to_string()
        .into())
    }

    async fn fetch_github(
        &self,
        owner: &str,
        repo: &str,
        r#ref: &str,
        path: &str,
    ) -> miette::Result<std::sync::Arc<str>> {
        Ok((self.fs_callbacks.fetch_github)((
            owner.to_string(),
            repo.to_string(),
            r#ref.to_string(),
            path.to_string(),
        ))
        .await)
    }
}
