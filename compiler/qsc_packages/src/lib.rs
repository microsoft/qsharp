// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.
use std::{str::FromStr, sync::Arc};

use rustc_hash::FxHashMap;

/// A program that is ready to be built -- dependencies have all been built, and the user code is ready.
#[derive(Debug)]
pub struct BuildableProgram {
    pub store: qsc::PackageStore,
    pub user_code: qsc_project::PackageInfo,
    pub user_code_dependencies: Vec<(qsc::hir::PackageId, Option<Arc<str>>)>,
}

impl BuildableProgram {
    pub fn new(program: qsc_project::ProgramConfig) -> Self {
        prepare_package_store(program)
    }
}

/// Given a program config, prepare the package store by compiling all dependencies in the correct order and inserting them.
pub fn prepare_package_store(program: qsc_project::ProgramConfig) -> BuildableProgram {
    // TODO profile is no longer in program config -- why?
    let capabilities =
        qsc::target::Profile::from_str(&program.target_profile) // TODO
            .unwrap_or_else(|()| panic!("Invalid target : {}", program.target_profile))
            .into();

    let mut package_store = qsc::PackageStore::new(qsc::compile::core());
    let std = qsc::compile::std(&package_store, capabilities);
    let std_id = package_store.insert(std);

    let mut canonical_package_identifier_to_package_id_mapping = FxHashMap::default();

    let (ordered_packages, user_code) = program
        .package_graph_sources
        .compilation_order()
        .expect("TODO error handling");
    for (package_name, package_to_compile) in ordered_packages {
        let sources: Vec<(Arc<str>, Arc<str>)> = package_to_compile
            .sources
            .into_iter()
            .map(|(path, contents)| (path.into(), contents.into()))
            .collect::<Vec<_>>();
        let source_map = qsc::SourceMap::new(sources, None);
        let dependencies = package_to_compile
            .dependencies
            .iter()
            .map(|(alias, key)| {
                (
                    alias.clone(),
                    canonical_package_identifier_to_package_id_mapping
                        .get(key)
                        .copied()
                        .expect("TODO handle this err: missing package"),
                )
            })
            .collect::<FxHashMap<_, _>>();
        // TODO use aliases to resolve dependencies
        // for now just use the package key
        let dependencies = dependencies
            .iter()
            .map(|(alias, b)| (*b, Some(alias.clone())))
            .chain(std::iter::once((std_id, None)))
            .collect::<Vec<_>>();
        let (compile_unit, dependency_errors) = qsc::compile::compile(
            &package_store,
            &dependencies[..],
            source_map,
            qsc::PackageType::Lib,
            capabilities,
            qsc::LanguageFeatures::from_iter(package_to_compile.language_features),
        );
        if !dependency_errors.is_empty() {
            todo!("handle errors in dependencies: {dependency_errors:?}");
        }

        let package_id = package_store.insert(compile_unit);
        canonical_package_identifier_to_package_id_mapping.insert(package_name, package_id);
    }

    let user_code_dependencies = user_code
        .dependencies
        .iter()
        .map(|(alias, key)| {
            (
                canonical_package_identifier_to_package_id_mapping
                    .get(key)
                    .copied()
                    .expect("TODO handle this err: missing package"),
                Some(alias.clone()),
            )
        })
        .collect::<Vec<_>>();

    BuildableProgram {
        store: package_store,
        user_code,
        user_code_dependencies,
    }
}

#[cfg(test)]
mod tests {
    // Copyright (c) Microsoft Corporation.
    // Licensed under the MIT License.
    use std::sync::Arc;

    use expect_test::expect;
    use qsc_project::PackageInfo;
    use rustc_hash::FxHashMap;

    fn mock_program() -> qsc_project::ProgramConfig {
        qsc_project::ProgramConfig {
            // Mock data for the ProgramConfig
            package_graph_sources: qsc_project::PackageGraphSources {
                root: qsc_project::PackageInfo {
                    sources: vec![(
                        Arc::from("test"),
                        Arc::from("@EntryPoint() operation Main() : Unit {}"),
                    )],
                    language_features: Default::default(),
                    dependencies: FxHashMap::from_iter(vec![(
                        Arc::from("SomeLibraryAlias"),
                        Arc::from("SomeLibraryKey"),
                    )]),
                },
                packages: FxHashMap::from_iter(vec![(
                    Arc::from("SomeLibraryKey"),
                    PackageInfo {
                        sources: vec![(
                            Arc::from("librarymain"),
                            Arc::from("operation LibraryMain() : Unit {} export LibraryMain;"),
                        )],
                        language_features: Default::default(),
                        dependencies: Default::default(),
                    },
                )]),
            },
            lints: vec![],
            errors: vec![],
            target_profile: "unrestricted".into(),
        }
    }

    fn pretty_print_buildable_program(program: &super::BuildableProgram) -> String {
        let mut buf = String::new();
        buf.push_str("store:\n___________\n");
        for (id, unit) in program.store.iter() {
            buf.push_str(&format!("{id} : {:?}\n", unit));
        }

        buf.push_str("user_code:\n___________\n");
        buf.push_str(&format!("{:?}", program.user_code));
        buf.push_str("user_code_dependencies:\n___________\n");
        buf.push_str(&format!("{:?}\n", program.user_code_dependencies));
        buf
    }

    #[test]
    fn test_prepare_package_store() {
        let program = mock_program();
        let buildable_program = super::prepare_package_store(program);

        // compile the user code
        let compiled = qsc::compile::compile(
            &buildable_program.store,
            &buildable_program.user_code_dependencies[..],
            qsc::SourceMap::new(
                buildable_program.user_code.sources,
                None, /* TODO entry */
            ),
            qsc::PackageType::Exe,
            Default::default(),
            Default::default(),
        );
        let res = format!("{:?}", compiled);

        expect![[r#"(CompileUnit { package: Package { items: IndexMap { values: ["0: Item { id: LocalItemId(0), span: Span { lo: 0, hi: 40 }, parent: None, doc: \"\", attrs: [], visibility: Public, kind: Namespace(Idents([Ident { id: NodeId(5), span: Span { lo: 0, hi: 40 }, name: \"test\" }]), [LocalItemId(1)]) }", "1: Item { id: LocalItemId(1), span: Span { lo: 0, hi: 40 }, parent: Some(LocalItemId(0)), doc: \"\", attrs: [EntryPoint], visibility: Internal, kind: Callable(CallableDecl { id: NodeId(0), span: Span { lo: 14, hi: 40 }, kind: Operation, name: Ident { id: NodeId(1), span: Span { lo: 24, hi: 28 }, name: \"Main\" }, generics: [], input: Pat { id: NodeId(2), span: Span { lo: 28, hi: 30 }, ty: Tuple([]), kind: Tuple([]) }, output: Tuple([]), functors: Empty, body: SpecDecl { id: NodeId(3), span: Span { lo: 14, hi: 40 }, body: Impl(None, Block { id: NodeId(4), span: Span { lo: 38, hi: 40 }, ty: Tuple([]), stmts: [] }) }, adj: None, ctl: None, ctl_adj: None }) }"] }, stmts: [], entry: Some(Expr { id: NodeId(8), span: Span { lo: 0, hi: 0 }, ty: Tuple([]), kind: Call(Expr { id: NodeId(7), span: Span { lo: 24, hi: 28 }, ty: Tuple([]), kind: Var(Item(ItemId { package: None, item: LocalItemId(1) }), []) }, Expr { id: NodeId(6), span: Span { lo: 28, hi: 30 }, ty: Tuple([]), kind: Tuple([]) }) }) }, ast: AstPackage { package: Package { id: NodeId(0), nodes: [Namespace(Namespace { id: NodeId(1), span: Span { lo: 0, hi: 40 }, doc: "", name: Idents([Ident { id: NodeId(2), span: Span { lo: 0, hi: 40 }, name: "test" }]), items: [Item { id: NodeId(3), span: Span { lo: 0, hi: 40 }, doc: "", attrs: [Attr { id: NodeId(4), span: Span { lo: 0, hi: 13 }, name: Ident { id: NodeId(5), span: Span { lo: 1, hi: 11 }, name: "EntryPoint" }, arg: Expr { id: NodeId(6), span: Span { lo: 11, hi: 13 }, kind: Tuple([]) } }], kind: Callable(CallableDecl { id: NodeId(7), span: Span { lo: 14, hi: 40 }, kind: Operation, name: Ident { id: NodeId(8), span: Span { lo: 24, hi: 28 }, name: "Main" }, generics: [], input: Pat { id: NodeId(9), span: Span { lo: 28, hi: 30 }, kind: Tuple([]) }, output: Ty { id: NodeId(10), span: Span { lo: 33, hi: 37 }, kind: Path(Path { id: NodeId(11), span: Span { lo: 33, hi: 37 }, namespace: None, name: Ident { id: NodeId(12), span: Span { lo: 33, hi: 37 }, name: "Unit" } }) }, functors: None, body: Block(Block { id: NodeId(13), span: Span { lo: 38, hi: 40 }, stmts: [] }) }) }] })], entry: None }, tys: Table { udts: {}, terms: IndexMap { values: ["9: Tuple([])", "13: Tuple([])"] }, generics: IndexMap { values: [] } }, names: IndexMap { values: ["1: Item(ItemId { package: None, item: LocalItemId(0) }, Available)", "8: Item(ItemId { package: None, item: LocalItemId(1) }, Available)", "11: UnitTy"] }, locals: Locals { scopes: [Scope { span: Span { lo: 0, hi: 40 }, kind: Namespace(NamespaceId(12)), opens: {[]: [Open { namespace: NamespaceId(12), span: Span { lo: 0, hi: 40 } }]}, tys: {}, terms: {}, vars: {}, ty_vars: {} }, Scope { span: Span { lo: 14, hi: 40 }, kind: Callable, opens: {}, tys: {}, terms: {}, vars: {}, ty_vars: {} }, Scope { span: Span { lo: 38, hi: 40 }, kind: Block, opens: {}, tys: {}, terms: {}, vars: {}, ty_vars: {} }] } }, assigner: Assigner { next_node: NodeId(9), next_item: LocalItemId(2) }, sources: SourceMap { sources: [Source { name: "test", contents: "@EntryPoint() operation Main() : Unit {}", offset: 0 }], common_prefix: None, entry: None }, errors: [], dropped_names: [] }, [])"#]].assert_eq(&res);
    }
}
