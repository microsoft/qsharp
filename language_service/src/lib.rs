// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

mod compilation;
pub mod completion;
pub mod definition;
mod display;
pub mod hover;
mod name_locator;
mod project_system;
pub mod protocol;
mod qsc_utils;
pub mod references;
pub mod rename;
pub mod signature_help;
mod state;
#[cfg(test)]
mod test_utils;
#[cfg(test)]
mod tests;

use compilation::Compilation;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures_util::StreamExt;
use log::{trace, warn};
pub use project_system::JSFileEntry;
use protocol::{
    CompletionList, DiagnosticUpdate, Hover, Location, NotebookMetadata, SignatureHelp,
    WorkspaceConfigurationUpdate,
};
use state::{CompilationState, CompilationStateUpdater};
use std::{cell::RefCell, fmt::Debug, future::Future, pin::Pin, rc::Rc, sync::Arc};

#[derive(Default)]
pub struct LanguageService {
    state: Rc<RefCell<CompilationState>>,
    state_updater: Option<UnboundedSender<Update>>,
}

impl LanguageService {
    pub fn create_update_worker<'a>(
        &mut self,
        diagnostics_receiver: impl Fn(DiagnosticUpdate) + 'a,
        read_file: impl Fn(String) -> Pin<Box<dyn Future<Output = (Arc<str>, Arc<str>)>>> + 'a,
        list_directory: impl Fn(String) -> Pin<Box<dyn Future<Output = Vec<JSFileEntry>>>> + 'a,
        get_manifest: impl Fn(String) -> Pin<Box<dyn Future<Output = Option<qsc_project::ManifestDescriptor>>>>
            + 'a,
    ) -> UpdateWorker<'a> {
        assert!(self.state_updater.is_none());
        let (send, recv) = unbounded();
        let worker = UpdateWorker {
            updater: CompilationStateUpdater::new(
                self.state.clone(),
                diagnostics_receiver,
                read_file,
                list_directory,
                get_manifest,
            ),
            recv,
        };
        self.state_updater = Some(send);
        worker
    }

    pub fn stop_updates(&mut self) {
        // Dropping the sender will cause the
        // worker created in [`create_update_worker()`] to stop.
        self.state_updater = None;
    }

    /// Updates the workspace configuration. If any compiler settings are updated,
    /// a recompilation may be triggered, which will result in a new set of diagnostics
    /// being published.
    ///
    /// LSP: workspace/didChangeConfiguration
    pub fn update_configuration(&mut self, configuration: &WorkspaceConfigurationUpdate) {
        trace!("update_configuration: {configuration:?}");
        self.queue_update(Update::Configuration {
            changed: configuration.clone(),
        });
    }

    /// Indicates that the document has been opened or the source has been updated.
    /// This should be called before any language service requests have been made
    /// for the document, typically when the document is first opened in the editor.
    /// It should also be called whenever the source code is updated.
    ///
    /// This is the "entry point" for the language service's logic, after its constructor.
    ///
    /// LSP: textDocument/didOpen, textDocument/didChange
    pub fn update_document(&mut self, uri: &str, version: u32, text: &str) {
        trace!("update_document: {uri} {version}");
        self.queue_update(Update::Document {
            uri: uri.into(),
            version,
            text: text.into(),
        });
    }

    /// Indicates that the client is no longer interested in the document,
    /// typically occurs when the document is closed in the editor.
    ///
    /// LSP: textDocument/didClose
    pub fn close_document(&mut self, uri: &str) {
        trace!("close_document: {uri}");
        self.queue_update(Update::CloseDocument { uri: uri.into() });
    }

    /// The uri refers to the notebook itself, not any of the individual cells.
    ///
    /// This function expects all Q# content in the notebook every time
    /// it is called, not just the changed cells.
    ///
    /// At this layer we expect the client to have stripped
    /// off all non-Q# content, including Python cells and lines
    /// containing the "%%qsharp" cell magic.
    ///
    /// LSP: notebookDocument/didOpen, notebookDocument/didChange
    pub fn update_notebook_document<'b, I>(
        &mut self,
        notebook_uri: &str,
        notebook_metadata: &NotebookMetadata,
        cells: I,
    ) where
        I: Iterator<Item = (&'b str, u32, &'b str)>, // uri, version, text - basically DidChangeTextDocumentParams in LSP
    {
        trace!("update_notebook_document: {notebook_uri}");
        self.queue_update(Update::NotebookDocument {
            notebook_uri: notebook_uri.into(),
            notebook_metadata: notebook_metadata.clone(),
            cells: cells
                .map(|(uri, version, contents)| (uri.into(), version, contents.into()))
                .collect(),
        });
    }

    /// Indicates that the client is no longer interested in the notebook.
    ///
    /// # Panics
    ///
    /// Panics if `cell_uris` does not contain all the cells associated with
    /// the notebook in the previous `update_notebook_document` call.
    ///
    /// LSP: notebookDocument/didClose
    pub fn close_notebook_document<'b>(
        &mut self,
        notebook_uri: &str,
        cell_uris: impl Iterator<Item = &'b str>,
    ) {
        trace!("close_notebook_document: {notebook_uri}");
        self.queue_update(Update::CloseNotebookDocument {
            notebook_uri: notebook_uri.into(),
            cell_uris: cell_uris.map(Into::into).collect(),
        });
    }

    /// LSP: textDocument/completion
    #[must_use]
    pub fn get_completions(&self, uri: &str, offset: u32) -> CompletionList {
        self.document_op(completion::get_completions, "get_completions", uri, offset)
    }

    /// LSP: textDocument/definition
    #[must_use]
    pub fn get_definition(&self, uri: &str, offset: u32) -> Option<Location> {
        self.document_op(definition::get_definition, "get_definition", uri, offset)
    }

    /// LSP: textDocument/references
    #[must_use]
    pub fn get_references(
        &self,
        uri: &str,
        offset: u32,
        include_declaration: bool,
    ) -> Vec<Location> {
        self.document_op(
            |compilation, uri, offset| {
                references::get_references(compilation, uri, offset, include_declaration)
            },
            "get_references",
            uri,
            offset,
        )
    }

    /// LSP: textDocument/hover
    #[must_use]
    pub fn get_hover(&self, uri: &str, offset: u32) -> Option<Hover> {
        self.document_op(hover::get_hover, "get_hover", uri, offset)
    }

    /// LSP textDocument/signatureHelp
    #[must_use]
    pub fn get_signature_help(&self, uri: &str, offset: u32) -> Option<SignatureHelp> {
        self.document_op(
            signature_help::get_signature_help,
            "get_signature_help",
            uri,
            offset,
        )
    }

    /// LSP: textDocument/rename
    #[must_use]
    pub fn get_rename(&self, uri: &str, offset: u32) -> Vec<Location> {
        self.document_op(rename::get_rename, "get_rename", uri, offset)
    }

    /// LSP: textDocument/prepareRename
    #[must_use]
    pub fn prepare_rename(&self, uri: &str, offset: u32) -> Option<(protocol::Span, String)> {
        self.document_op(rename::prepare_rename, "prepare_rename", uri, offset)
    }

    /// Executes an operation that takes a document uri and offset, using the current compilation for that document
    fn document_op<F, T>(&self, op: F, op_name: &str, uri: &str, offset: u32) -> T
    where
        F: Fn(&Compilation, &str, u32) -> T,
        T: Debug + Default,
    {
        trace!("{op_name}: uri: {uri}, offset: {offset}");

        // Borrow must succeed here. If it doesn't succeed, a writer
        // (i.e. [`state::CompilationStateUpdater`]) must be holding a mutable reference across
        // an `await` point. Which it shouldn't be doing.
        let compilation_state = self.state.borrow();
        if let Some(compilation) = compilation_state.get_compilation(uri) {
            let res = op(compilation, uri, offset);
            trace!("{op_name} result: {res:?}");
            res
        } else {
            // The current state doesn't yet contain the document. Updates must be pending.
            trace!("Skipping {op_name} for {uri} since compilation is in progress");
            T::default()
        }
    }

    fn queue_update(&mut self, update: Update) {
        if let Some(updater) = self.state_updater.as_mut() {
            updater
                .unbounded_send(update)
                .expect("send error in queue_update");
        } else {
            warn!("Ignoring update, no worker is listening");
        }
    }
}

pub struct UpdateWorker<'a> {
    updater: CompilationStateUpdater<'a>,
    recv: UnboundedReceiver<Update>,
}

impl UpdateWorker<'_> {
    #[allow(clippy::await_holding_refcell_ref)]
    pub async fn run(&mut self) {
        while let Some(update) = self.recv.next().await {
            trace!("start applying updates");
            self.apply_this_and_pending(vec![update]).await;
            trace!("end applying updates updates");
        }
    }

    #[cfg(test)]
    async fn apply_pending(&mut self) {
        self.apply_this_and_pending(vec![]).await;
    }

    async fn apply_this_and_pending(&mut self, mut updates: Vec<Update>) {
        // Consume any backed up messages in the channel as well.
        while let Ok(update) = self.recv.try_next() {
            match update {
                Some(update) => push_update(&mut updates, update),
                None => return, // channel has been closed, don't bother with updates.
            }
        }

        if updates.len() > 100 {
            // This indicates that we're not keeping up with incoming updates.
            // Harmless, but an indicator that we could try intelligently
            // dropping updates or otherwise optimizing.
            warn!(
                "perf: {} pending updates found even after deduping",
                updates.len()
            );
        }

        for update in updates.drain(..) {
            apply_update(&mut self.updater, update).await;
        }
    }
}

fn push_update(pending_updates: &mut Vec<Update>, update: Update) {
    // Dedup consecutive updates to the same document.
    match &update {
        Update::Document { uri, .. } => {
            if let Some(last) = pending_updates.last_mut() {
                if let Update::Document { uri: last_uri, .. } = last {
                    if last_uri == uri {
                        // overwrite the last element
                        *last = update;
                        return;
                    }
                }
            }
        }
        Update::NotebookDocument { notebook_uri, .. } => {
            if let Some(last) = pending_updates.last_mut() {
                if let Update::NotebookDocument {
                    notebook_uri: last_uri,
                    ..
                } = last
                {
                    if last_uri == notebook_uri {
                        // overwrite the last element
                        *last = update;
                        return;
                    }
                }
            }
        }
        Update::Configuration { .. }
        | Update::CloseDocument { .. }
        | Update::CloseNotebookDocument { .. } => (), // These events aren't noisy enough to bother deduping.
    }
    pending_updates.push(update);
}

async fn apply_update(updater: &mut CompilationStateUpdater<'_>, update: Update) {
    match update {
        Update::CloseDocument { uri } => {
            updater.close_document(&uri);
        }
        Update::Document { uri, version, text } => {
            updater.update_document(&uri, version, &text).await;
        }
        Update::NotebookDocument {
            notebook_uri,
            notebook_metadata,
            cells,
        } => updater.update_notebook_document(
            &notebook_uri,
            &notebook_metadata,
            cells
                .iter()
                .map(|(uri, version, contents)| (uri.as_ref(), *version, contents.as_ref())),
        ),
        Update::CloseNotebookDocument {
            notebook_uri,
            cell_uris,
        } => updater.close_notebook_document(&notebook_uri, cell_uris.iter().map(AsRef::as_ref)),
        Update::Configuration { changed } => {
            updater.update_configuration(&changed);
        }
    }
}

enum Update {
    Configuration {
        changed: WorkspaceConfigurationUpdate,
    },
    Document {
        uri: String,
        version: u32,
        text: String,
    },
    CloseDocument {
        uri: String,
    },
    NotebookDocument {
        notebook_uri: String,
        notebook_metadata: NotebookMetadata,
        cells: Vec<(String, u32, String)>,
    },
    CloseNotebookDocument {
        notebook_uri: String,
        cell_uris: Vec<String>,
    },
}
