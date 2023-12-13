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
        Ok((self.read_file_callback)(path.to_string_lossy().to_string()).await)
    }

    async fn list_directory(&self, path: &std::path::Path) -> miette::Result<Vec<Self::Entry>> {
        Ok((self.list_directory)(path.to_string_lossy().to_string()).await)
    }
}
