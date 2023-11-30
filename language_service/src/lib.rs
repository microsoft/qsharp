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
#[cfg(test)]
mod test_utils;
#[cfg(test)]
mod tests;

use compilation::Compilation;
use log::{error, trace};
use miette::Diagnostic;
pub use project_system::JSFileEntry;
use protocol::{
    CompletionList, DiagnosticUpdate, Hover, Location, SignatureHelp, WorkspaceConfigurationUpdate,
};
use qsc::{compile::Error, PackageType, TargetProfile};
use qsc_project::FileSystemAsync;
use rustc_hash::{FxHashMap, FxHashSet};
use std::{future::Future, mem::take, path::PathBuf, pin::Pin, sync::Arc};

type CompilationUri = Arc<str>;
type DocumentUri = Arc<str>;

/// the desugared return type of an "async fn"
type PinnedFuture<T> = Pin<Box<dyn Future<Output = T>>>;

/// represents a unary async fn where `Arg` is the input
/// parameter and `Return` is the return type. The lifetime
/// `'a` represents the lifetime of the contained `dyn Fn`.
type AsyncFunction<'a, Arg, Return> = Box<dyn Fn(Arg) -> PinnedFuture<Return> + 'a>;

pub struct LanguageService<'a> {
    /// Workspace configuration can include compiler settings
    /// that affect error checking and other language server behavior.
    /// Currently these settings apply to all documents in the
    /// workspace. Per-document configurations are not supported.
    configuration: WorkspaceConfiguration,
    /// A `CompilationUri` is an identifier for a unique compilation.
    /// It is NOT required to be a uri that represents an actual document.
    ///
    /// For single Q# documents, the `CompilationUri` is the same as the
    /// document uri.
    ///
    /// For notebooks, the `CompilationUri` is the notebook uri.
    ///
    /// The `CompilatinUri` is used when compilation-level errors get reported
    /// to the client. Compilation-level errors are defined as errors without
    /// an associated source document.
    compilations: FxHashMap<CompilationUri, Compilation>,
    /// All the documents that we were told about by the client.
    ///
    /// This map doesn't necessarily contain ALL the documents that
    /// make up a compilation - only the ones that are currently open.
    open_documents: FxHashMap<DocumentUri, OpenDocument>,
    /// Documents that we have previously published errors about. We need to
    /// keep track of this so we can clear errors from them when documents are removed
    /// from a compilation or when a recompilation occurs.
    documents_with_errors: FxHashSet<DocumentUri>,
    /// Callback which will receive diagnostics (compilation errors)
    /// whenever a (re-)compilation occurs.
    diagnostics_receiver: AsyncFunction<'a, DiagnosticUpdate, ()>,
    /// Callback which lets the service read a file from the target filesystem
    read_file_callback: AsyncFunction<'a, PathBuf, (Arc<str>, Arc<str>)>,
    /// Callback which lets the service list directory contents
    /// on the target file system
    list_directory: AsyncFunction<'a, PathBuf, Vec<JSFileEntry>>,
    /// Fetch the manifest file for a specific path
    get_manifest: AsyncFunction<'a, String, Option<qsc_project::ManifestDescriptor>>,
}

#[derive(Debug)]
struct WorkspaceConfiguration {
    pub target_profile: TargetProfile,
    pub package_type: PackageType,
}

impl Default for WorkspaceConfiguration {
    fn default() -> Self {
        Self {
            target_profile: TargetProfile::Full,
            package_type: PackageType::Exe,
        }
    }
}

#[derive(Debug)]
struct OpenDocument {
    /// This version is the document version provided by the client.
    /// It increases strictly with each text change, though this knowledge should
    /// not be important. The version is only ever used when publishing
    /// diagnostics to help the client associate the list of diagnostics
    /// with a snapshot of the document.
    pub version: u32,
    pub compilation: CompilationUri,
}

impl<'a> LanguageService<'a> {
    pub fn new(
        diagnostics_receiver: impl Fn(DiagnosticUpdate) -> Pin<Box<dyn Future<Output = ()>>> + 'a,
        read_file: impl Fn(PathBuf) -> Pin<Box<dyn Future<Output = (Arc<str>, Arc<str>)>>> + 'a,
        list_directory: impl Fn(PathBuf) -> Pin<Box<dyn Future<Output = Vec<JSFileEntry>>>> + 'a,
        get_manifest: impl Fn(String) -> Pin<Box<dyn Future<Output = Option<qsc_project::ManifestDescriptor>>>>
            + 'a,
    ) -> Self {
        LanguageService {
            configuration: WorkspaceConfiguration::default(),
            compilations: FxHashMap::default(),
            open_documents: FxHashMap::default(),
            documents_with_errors: FxHashSet::default(),
            diagnostics_receiver: Box::new(diagnostics_receiver),
            read_file_callback: Box::new(read_file),
            list_directory: Box::new(list_directory),
            get_manifest: Box::new(get_manifest),
        }
    }

    /// Updates the workspace configuration. If any compiler settings are updated,
    /// a recompilation may be triggered, which will result in a new set of diagnostics
    /// being published.
    pub async fn update_configuration(&mut self, configuration: &WorkspaceConfigurationUpdate) {
        trace!("update_configuration: {configuration:?}");

        let need_recompile = self.apply_configuration(configuration);

        // Some configuration options require a recompilation as they impact error checking
        if need_recompile {
            self.recompile_all().await;
        }
    }

    /// Indicates that the document has been opened or the source has been updated.
    /// This should be called before any language service requests have been made
    /// for the document, typically when the document is first opened in the editor.
    /// It should also be called whenever the source code is updated.
    ///
    /// This is the "entry point" for the language service's logic, after its constructor.
    ///
    /// LSP: textDocument/didOpen, textDocument/didChange
    pub async fn update_document(&mut self, uri: &str, version: u32, text: &str) {
        trace!("update_document: {uri} {version}");
        let manifest = (self.get_manifest)(uri.to_string()).await;
        let sources = if let Some(ref manifest) = manifest {
            let project = match self.load_project(manifest).await {
                Ok(o) => o,
                Err(e) => {
                    error!("failed to load manifest: {e:?}");
                    return;
                }
            };
            project.sources
        } else {
            trace!("Running in single file mode");
            vec![(Arc::from(uri), Arc::from(text))]
        };
        let compilation = Compilation::new(
            &sources,
            self.configuration.package_type,
            self.configuration.target_profile,
        );
        // If we are in single file mode, use the file's path as the compilation identifier.
        // If we are compiling a project, use the path to the project manifest
        let uri: Arc<str> = if let Some(manifest) = manifest {
            Arc::from(manifest.manifest_dir.to_string_lossy().to_string())
        } else {
            uri.into()
        };
        trace!("Loaded project uri {uri} with {} sources", sources.len());
        self.compilations.insert(uri.clone(), compilation);

        // There may be open buffers with sources in the project.
        // These buffers need to have their diagnostics reloaded,
        // to be in the context of the project.
        // We remove them from the existing compilations and update
        // their compilation URI
        for (path, _contents) in &sources {
            log::trace!("Updating compilation of {path} to {uri}");
            self.open_documents
                .entry(path.clone())
                .and_modify(|x| {
                    // remove any old single-file compilations of this document
                    // if this is a project
                    if x.compilation != uri {
                        self.compilations.remove(&x.compilation);
                    }
                    x.compilation = uri.clone();
                })
                .or_insert(OpenDocument {
                    version,
                    compilation: uri.clone(),
                });
        }

        self.publish_diagnostics().await;
    }

    /// Indicates that the client is no longer interested in the document,
    /// typically occurs when the document is closed in the editor.
    ///
    /// LSP: textDocument/didClose
    pub async fn close_document(&mut self, uri: &str) {
        trace!("close_document: {uri}");

        self.compilations.remove(uri);
        self.open_documents.remove(uri);

        self.publish_diagnostics().await;
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
    pub async fn update_notebook_document<'b, I>(&mut self, notebook_uri: &str, cells: I)
    where
        I: Iterator<Item = (&'b str, u32, &'b str)>, // uri, version, text - basically DidChangeTextDocumentParams in LSP
    {
        trace!("update_notebook_document: {notebook_uri}");

        let compilation_uri: Arc<str> = notebook_uri.into();

        // First remove all previously known cells for this notebook
        self.open_documents
            .retain(|_, open_doc| notebook_uri != open_doc.compilation.as_ref());

        // Compile the notebook and add each cell into the document map
        let compilation =
            Compilation::new_notebook(cells.map(|(cell_uri, version, cell_contents)| {
                trace!("update_notebook_document: cell: {cell_uri} {version}");
                self.open_documents.insert(
                    (*cell_uri).into(),
                    OpenDocument {
                        version,
                        compilation: compilation_uri.clone(),
                    },
                );
                (Arc::from(cell_uri), Arc::from(cell_contents))
            }));

        self.compilations
            .insert(compilation_uri.clone(), compilation);

        self.publish_diagnostics().await;
    }

    /// Indicates that the client is no longer interested in the notebook.
    ///
    /// # Panics
    ///
    /// Panics if `cell_uris` does not contain all the cells associated with
    /// the notebook in the previous `update_notebook_document` call.
    ///
    /// LSP: notebookDocument/didClose
    pub async fn close_notebook_document<'b>(
        &mut self,
        notebook_uri: &str,
        cell_uris: impl Iterator<Item = &'b str>,
    ) {
        trace!("close_notebook_document: {notebook_uri}");

        for cell_uri in cell_uris {
            trace!("close_notebook_document: cell: {cell_uri}");
            self.open_documents.remove(cell_uri);
        }

        // The client should have sent all cell uris along with
        // the notebook. Validate our assumptions about the client
        // here, by checking that all the cells for this notebook
        // have been removed from the open documents map.
        for open_doc in self.open_documents.values() {
            assert!(
                notebook_uri != open_doc.compilation.as_ref(),
                "all cells should have been closed along with the notebook"
            );
        }

        self.compilations.remove(notebook_uri);

        self.publish_diagnostics().await;
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

    /// LSP: textDocument/hover
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
        T: std::fmt::Debug,
    {
        trace!("{op_name}: uri: {uri}, offset: {offset}");
        let compilation_uri = &self
            .open_documents
            .get(uri)
            .unwrap_or_else(|| {
                panic!("{op_name} (called on {uri}) should not be called for a document that has not been opened",)
            })
            .compilation;

        trace!("{op_name}: compilation_uri: {compilation_uri}");
        let compilation = self.compilations.get(compilation_uri).unwrap_or_else(|| {
            panic!("{op_name} should not be called before compilation has been initialized",)
        });

        let res = op(compilation, uri, offset);
        trace!("{op_name} result: {res:?}");
        res
    }

    // It gets really messy knowing when to clear diagnostics
    // when the document changes ownership between compilations, etc.
    // So let's do it the simplest way possible. Republish all the diagnostics every time.
    async fn publish_diagnostics(&mut self) {
        let last_docs_with_errors = take(&mut self.documents_with_errors);

        for (compilation_uri, compilation) in &self.compilations {
            trace!("publishing diagnostics for {compilation_uri}");
            for (uri, errors) in map_errors_to_docs(compilation_uri, &compilation.errors) {
                if !self.documents_with_errors.insert(uri.clone()) {
                    // We already published diagnostics for this document for
                    // a different compilation.
                    // When the same document is included in multiple compilations,
                    // only report the errors for one of them, the goal being
                    // a less confusing user experience.
                    continue;
                }

                self.publish_diagnostics_for_doc(&uri, errors).await;
            }
        }

        // Clear errors from any documents that previously had errors
        for uri in last_docs_with_errors.difference(&self.documents_with_errors) {
            self.publish_diagnostics_for_doc(uri, vec![]).await;
        }
    }

    async fn publish_diagnostics_for_doc(&self, uri: &str, errors: Vec<Error>) {
        let version = self.open_documents.get(uri).map(|d| d.version);
        trace!("publishing diagnostics for {uri} {version:?}): {errors:?}");
        (self.diagnostics_receiver)(DiagnosticUpdate {
            uri: uri.into(),
            version,
            errors,
        })
        .await;
    }

    fn apply_configuration(&mut self, configuration: &WorkspaceConfigurationUpdate) -> bool {
        let mut need_recompile = false;

        if let Some(package_type) = configuration.package_type {
            need_recompile |= self.configuration.package_type != package_type;
            self.configuration.package_type = package_type;
        }

        if let Some(target_profile) = configuration.target_profile {
            need_recompile |= self.configuration.target_profile != target_profile;
            self.configuration.target_profile = target_profile;
        }

        trace!("need_recompile after configuration update: {need_recompile}");
        need_recompile
    }

    /// Recompiles the currently known documents with
    /// the current configuration. Publishes updated
    /// diagnostics for all documents.
    async fn recompile_all(&mut self) {
        for compilation in self.compilations.values_mut() {
            compilation.recompile(
                self.configuration.package_type,
                self.configuration.target_profile,
            );
        }

        self.publish_diagnostics().await;
    }
}

fn map_errors_to_docs(
    compilation_uri: &Arc<str>,
    errors: &Vec<Error>,
) -> FxHashMap<Arc<str>, Vec<Error>> {
    let mut map = FxHashMap::default();

    for err in errors {
        // Use the compilation_uri as a location for span-less errors
        let doc = err
            .labels()
            .into_iter()
            .flatten()
            .next()
            .map_or(compilation_uri, |l| {
                let (source, _) = err.resolve_span(l.inner());
                &source.name
            });

        map.entry(doc.clone())
            .or_insert_with(Vec::new)
            .push(err.clone());
    }

    map
}
