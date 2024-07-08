// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::sync::Arc;

use crate::compilation::{Compilation, CompilationKind};
use qsc::{
    compile,
    hir::PackageId,
    incremental::Compiler,
    line_column::{Encoding, Position, Range},
    location::Location,
    target::Profile,
    LanguageFeatures, PackageStore, PackageType, SourceMap, Span,
};
use qsc_project::{PackageGraphSources, PackageInfo};
use rustc_hash::FxHashMap;

pub(crate) fn compile_with_fake_stdlib_and_markers(
    source_with_markers: &str,
) -> (Compilation, Position, Vec<Range>) {
    let (compilation, _, cursor_offset, target_spans) =
        compile_project_with_fake_stdlib_and_markers(&[("<source>", source_with_markers)]);
    (
        compilation,
        cursor_offset,
        target_spans.iter().map(|l| l.range).collect(),
    )
}

pub(crate) fn compile_with_fake_stdlib_and_markers_no_cursor(
    source_with_markers: &str,
) -> (Compilation, Vec<Range>) {
    let (compilation, target_spans) = compile_project_with_fake_stdlib_and_markers_no_cursor(&[(
        "<source>",
        source_with_markers,
    )]);
    (compilation, target_spans.iter().map(|l| l.range).collect())
}

pub(crate) fn compile_project_with_fake_stdlib_and_markers(
    sources_with_markers: &[(&str, &str)],
) -> (Compilation, String, Position, Vec<Location>) {
    let (compilation, cursor_location, target_spans) =
        compile_project_with_fake_stdlib_and_markers_cursor_optional(sources_with_markers);

    let (cursor_uri, cursor_offset) =
        cursor_location.expect("input string should have a cursor marker");

    (compilation, cursor_uri, cursor_offset, target_spans)
}

pub(crate) fn compile_project_with_fake_stdlib_and_markers_no_cursor(
    sources_with_markers: &[(&str, &str)],
) -> (Compilation, Vec<Location>) {
    let (compilation, cursor_location, target_spans) =
        compile_project_with_fake_stdlib_and_markers_cursor_optional(sources_with_markers);

    assert!(
        cursor_location.is_none(),
        "did not expect cursor marker in input string"
    );

    (compilation, target_spans)
}

fn compile_project_with_fake_stdlib_and_markers_cursor_optional(
    sources_with_markers: &[(&str, &str)],
) -> (Compilation, Option<(String, Position)>, Vec<Location>) {
    let (sources, cursor_location, target_spans) = get_sources_and_markers(sources_with_markers);

    let source_map = SourceMap::new(sources.clone(), None);
    let (mut package_store, std_package_id) = compile_fake_stdlib();
    let (unit, errors) = compile::compile(
        &package_store,
        &[(std_package_id, None)],
        source_map,
        PackageType::Exe,
        Profile::Unrestricted.into(),
        LanguageFeatures::default(),
    );

    let package_id = package_store.insert(unit);

    (
        Compilation {
            package_store,
            user_package_id: package_id,
            kind: CompilationKind::OpenProject {
                package_graph_sources: PackageGraphSources {
                    root: PackageInfo {
                        sources,
                        language_features: LanguageFeatures::default(),
                        dependencies: FxHashMap::default(),
                        package_type: None,
                    },
                    packages: FxHashMap::default(),
                },
            },
            compile_errors: errors,
            project_errors: Vec::new(),
        },
        cursor_location,
        target_spans,
    )
}

pub(crate) fn compile_notebook_with_fake_stdlib_and_markers(
    cells_with_markers: &[(&str, &str)],
) -> (Compilation, String, Position, Vec<Location>) {
    let (cells, cursor_location, target_spans) = get_sources_and_markers(cells_with_markers);
    let (cell_uri, offset) = cursor_location.expect("input string should have a cursor marker");

    let compilation =
        compile_notebook_with_fake_stdlib(cells.iter().map(|c| (c.0.as_ref(), c.1.as_ref())));
    (compilation, cell_uri, offset, target_spans)
}

pub(crate) fn compile_notebook_with_fake_stdlib<'a, I>(cells: I) -> Compilation
where
    I: Iterator<Item = (&'a str, &'a str)>,
{
    let std_source_map = SourceMap::new(
        [(
            "qsharp-library-source:<std>".into(),
            "namespace FakeStdLib {
                operation Fake() : Unit {}
                operation FakeWithParam(x: Int) : Unit {}
                operation FakeCtlAdj() : Unit is Ctl + Adj {}
                newtype Complex = (Real: Double, Imag: Double);
                function TakesComplex(input : Complex) : Unit {}
                export Fake, FakeWithParam, FakeCtlAdj, Complex, TakesComplex;
            }"
            .into(),
        )],
        None,
    );

    let store = qsc::PackageStore::new(qsc::compile::core());
    let mut compiler = Compiler::new(
        std_source_map,
        PackageType::Lib,
        Profile::Unrestricted.into(),
        LanguageFeatures::default(),
        store,
        &[],
    )
    .expect("expected incremental compiler creation to succeed");

    let mut errors = Vec::new();
    for (name, contents) in cells {
        let increment = compiler
            .compile_fragments(name, contents, |cell_errors| {
                errors.extend(cell_errors);
                Ok(()) // accumulate errors without failing
            })
            .expect("compile_fragments_acc_errors should not fail");

        compiler.update(increment);
    }

    let (package_store, package_id) = compiler.into_package_store();

    Compilation {
        package_store,
        user_package_id: package_id,
        compile_errors: errors,
        kind: CompilationKind::Notebook,
        project_errors: Vec::new(),
    }
}

fn compile_fake_stdlib() -> (PackageStore, PackageId) {
    let mut package_store = PackageStore::new(compile::core());
    let std_source_map = SourceMap::new(
        [(
            "qsharp-library-source:<std>".into(),
            r#"namespace FakeStdLib {
                operation Fake() : Unit {}
                operation FakeWithParam(x : Int) : Unit {}
                operation FakeCtlAdj() : Unit is Ctl + Adj {}
                newtype Udt = (x : Int, y : Int);
                newtype UdtWrapper = (inner : Udt);
                newtype UdtFn = (Int -> Int);
                newtype UdtFnWithUdtParams = (Udt -> Udt);
                function TakesUdt(input : Udt) : Udt {
                    fail "not implemented"
                }
                operation RefFake() : Unit {
                    Fake();
                }
                operation FakeWithTypeParam<'A>(a : 'A) : 'A { a }
                internal operation Hidden() : Unit {}
                struct FakeStruct { x : Int, y : Int }
                struct StructWrapper { inner : FakeStruct }
                struct StructFn { inner : Int -> Int }
                struct StructFnWithStructParams { inner : FakeStruct -> FakeStruct }
                function TakesStruct(input : FakeStruct) : FakeStruct {
                    fail "not implemented"
                }
                export Fake, FakeWithParam, FakeCtlAdj, Udt, UdtWrapper, UdtFn, UdtFnWithUdtParams, TakesUdt, RefFake, FakeWithTypeParam;
                export FakeStruct, StructWrapper, StructFn, StructFnWithStructParams, TakesStruct;
            }

            namespace Microsoft.Quantum.Unstable {
                operation UnstableFake() : Unit {}
                export UnstableFake;
            }"#
            .into(),
        )],
        None,
    );
    let (std_compile_unit, std_errors) = compile::compile(
        &package_store,
        &[(PackageId::CORE, None)],
        std_source_map,
        PackageType::Lib,
        Profile::Unrestricted.into(),
        LanguageFeatures::default(),
    );
    assert!(std_errors.is_empty());
    let std_package_id = package_store.insert(std_compile_unit);
    (package_store, std_package_id)
}

#[allow(clippy::type_complexity)]
fn get_sources_and_markers(
    sources: &[(&str, &str)],
) -> (
    Vec<(Arc<str>, Arc<str>)>,
    Option<(String, Position)>,
    Vec<Location>,
) {
    let (mut cursor_uri, mut cursor_offset, mut target_spans) = (None, None, Vec::new());
    let sources = sources
        .iter()
        .map(|s| {
            let (source, cursor_offsets, targets) = get_source_and_marker_offsets(s.1);
            if !cursor_offsets.is_empty() {
                assert!(
                    cursor_uri.replace(s.0).is_none(),
                    "only one cell can have a cursor marker"
                );
                assert!(
                    cursor_offset
                        .replace(Position::from_utf8_byte_offset(
                            Encoding::Utf8,
                            &source,
                            cursor_offsets[0]
                        ))
                        .is_none(),
                    "only one cell can have a cursor marker"
                );
            }
            if !targets.is_empty() {
                for span in target_offsets_to_spans(&targets) {
                    target_spans.push(Location {
                        source: s.0.into(),
                        range: Range::from_span(Encoding::Utf8, &source, &span),
                    });
                }
            }
            (Arc::from(s.0), Arc::from(source.as_ref()))
        })
        .collect();
    let cursor_location = cursor_uri.map(|cursor_uri| {
        (
            cursor_uri.into(),
            cursor_offset.expect("cursor offset should be set"),
        )
    });
    (sources, cursor_location, target_spans)
}

fn get_source_and_marker_offsets(source_with_markers: &str) -> (String, Vec<u32>, Vec<u32>) {
    let mut cursor_offsets = Vec::new();
    let mut target_offsets = Vec::new();
    let mut source = source_with_markers.to_string();
    let markers = &['↘', '◉'];

    loop {
        let next_offset = source.find(markers);
        match next_offset {
            #[allow(clippy::cast_possible_truncation)]
            Some(offset) => match source.chars().nth(offset) {
                Some('↘') => cursor_offsets.push(offset as u32),
                Some('◉') => target_offsets.push(offset as u32),
                _ => panic!("Expected to find marker"),
            },
            None => break,
        };
        source = source.replacen(markers, "", 1);
    }
    (source, cursor_offsets, target_offsets)
}

fn target_offsets_to_spans(target_offsets: &[u32]) -> Vec<Span> {
    assert!(target_offsets.len() % 2 == 0);
    let limit = target_offsets.len() / 2;
    let mut spans = vec![];
    for i in 0..limit {
        spans.push(Span {
            lo: target_offsets[i * 2],
            hi: target_offsets[i * 2 + 1],
        });
    }
    spans
}
