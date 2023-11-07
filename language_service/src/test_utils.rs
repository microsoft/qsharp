// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{protocol, qsc_utils::Compilation};
use qsc::{compile, hir::PackageId, PackageStore, PackageType, SourceMap, TargetProfile};

pub(crate) fn get_source_and_marker_offsets(
    source_with_markers: &str,
) -> (String, Vec<u32>, Vec<u32>) {
    let mut cursor_offsets: Vec<u32> = Vec::new();
    let mut target_offsets: Vec<u32> = Vec::new();
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

pub(crate) fn target_offsets_to_spans(target_offsets: &Vec<u32>) -> Vec<protocol::Span> {
    assert!(target_offsets.len() % 2 == 0);
    let limit = target_offsets.len() / 2;
    let mut spans = vec![];
    for i in 0..limit {
        spans.push(protocol::Span {
            start: target_offsets[i * 2],
            end: target_offsets[i * 2 + 1],
        });
    }
    spans
}

pub(crate) fn compile_with_fake_stdlib(source_name: &str, source_contents: &str) -> Compilation {
    let mut package_store = PackageStore::new(compile::core());
    let std_source_map = SourceMap::new(
        [(
            "<std>".into(),
            r#"namespace FakeStdLib {
                operation Fake() : Unit {}
                operation FakeWithParam(x: Int) : Unit {}
                operation FakeCtlAdj() : Unit is Ctl + Adj {}
                newtype Udt = (x: Int, y: Int);
                newtype UdtWrapper = (inner: Udt);
                newtype UdtFn = (Int -> Int);
                newtype UdtFnWithUdtParams = (Udt -> Udt);
                function TakesUdt(input : Udt) : Udt {
                    fail "not implemented"
                }
                operation RefFake() : Unit {
                    Fake();
                }
            }"#
            .into(),
        )],
        None,
    );
    let (std_compile_unit, std_errors) = compile::compile(
        &package_store,
        &[PackageId::CORE],
        std_source_map,
        PackageType::Lib,
        TargetProfile::Full,
    );
    assert!(std_errors.is_empty());
    let std_package_id = package_store.insert(std_compile_unit);
    let source_map = SourceMap::new([(source_name.into(), source_contents.into())], None);
    let (unit, errors) = compile::compile(
        &package_store,
        &[std_package_id],
        source_map,
        PackageType::Exe,
        TargetProfile::Full,
    );
    Compilation {
        package_store,
        std_package_id,
        user_unit: unit,
        errors,
    }
}
