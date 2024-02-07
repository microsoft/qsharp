// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::sync::Arc;

use qsc_data_structures::{
    line_column::{Encoding, Range},
    span::Span,
};
use qsc_frontend::compile::PackageStore;
use qsc_hir::hir::PackageId;

pub const QSHARP_LIBRARY_URI_SCHEME: &str = "qsharp-library-source";

/// Describes a location in source code in terms of a source name and [`Range`].
#[derive(Debug, PartialEq, Clone)]
pub struct Location {
    pub source: Arc<str>,
    pub range: Range,
}

impl Location {
    /// Creates a [`Location`] from a package ID and a SourceMap-relative span.
    ///
    /// To differentiate user sources from library sources, this function takes
    /// a `user_package_id` parameter which denotes the user package.
    /// All other packages in the package store are assumed to be library packages.
    /// Source names from library packages are prepended with a unique URI scheme.
    #[must_use]
    pub fn from(
        span: Span,
        package_id: PackageId,
        package_store: &PackageStore,
        user_package_id: PackageId,
        position_encoding: Encoding,
    ) -> Self {
        let source = package_store
            .get(package_id)
            .expect("package id must exist in store")
            .sources
            .find_by_offset(span.lo)
            .expect("source should exist for offset");

        let source_name = if package_id == user_package_id {
            source.name.clone()
        } else {
            // Currently the only supported external packages are our library packages,
            // URI's to which need to include our custom library scheme.
            format!("{}:{}", QSHARP_LIBRARY_URI_SCHEME, source.name).into()
        };

        Location {
            source: source_name,
            range: Range::from_span(position_encoding, &source.contents, &(span - source.offset)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::compile;
    use expect_test::expect;
    use qsc_data_structures::{line_column::Encoding, span::Span};
    use qsc_frontend::compile::{PackageStore, RuntimeCapabilityFlags, SourceMap};
    use qsc_hir::hir::PackageId;
    use qsc_passes::PackageType;

    use super::Location;

    #[test]
    fn from_std_span() {
        let (store, std_package_id, user_package_id) = compile_package();

        let location = Location::from(
            Span { lo: 0, hi: 1 },
            std_package_id,
            &store,
            user_package_id,
            Encoding::Utf8,
        );

        expect![[r#"
            Location {
                source: "qsharp-library-source:arrays.qs",
                range: Range {
                    start: Position {
                        line: 0,
                        column: 0,
                    },
                    end: Position {
                        line: 0,
                        column: 1,
                    },
                },
            }
        "#]]
        .assert_debug_eq(&location);
    }

    #[test]
    fn from_core_span() {
        let (store, _, user_package_id) = compile_package();

        let location = Location::from(
            Span { lo: 0, hi: 1 },
            PackageId::CORE,
            &store,
            user_package_id,
            Encoding::Utf8,
        );

        expect![[r#"
            Location {
                source: "qsharp-library-source:core/core.qs",
                range: Range {
                    start: Position {
                        line: 0,
                        column: 0,
                    },
                    end: Position {
                        line: 0,
                        column: 1,
                    },
                },
            }
        "#]]
        .assert_debug_eq(&location);
    }

    #[test]
    fn from_user_span() {
        let (store, _, user_package_id) = compile_package();

        let bar_start_offset = store
            .get(user_package_id)
            .expect("expected to find user package")
            .sources
            .find_by_name("bar.qs")
            .expect("expected to find source")
            .offset;

        let location = Location::from(
            Span {
                lo: bar_start_offset,
                hi: bar_start_offset + 1,
            },
            user_package_id,
            &store,
            user_package_id,
            Encoding::Utf8,
        );

        expect![[r#"
            Location {
                source: "bar.qs",
                range: Range {
                    start: Position {
                        line: 0,
                        column: 0,
                    },
                    end: Position {
                        line: 0,
                        column: 1,
                    },
                },
            }
        "#]]
        .assert_debug_eq(&location);
    }

    #[test]
    fn from_out_of_bounds_lo() {
        let (store, _, user_package_id) = compile_package();

        let location = Location::from(
            Span { lo: 1000, hi: 2000 },
            user_package_id,
            &store,
            user_package_id,
            Encoding::Utf8,
        );

        // Per [`Range`] spec, out of bounds positions map to EOF
        expect![[r#"
            Location {
                source: "bar.qs",
                range: Range {
                    start: Position {
                        line: 0,
                        column: 17,
                    },
                    end: Position {
                        line: 0,
                        column: 17,
                    },
                },
            }
        "#]]
        .assert_debug_eq(&location);
    }

    #[test]
    fn from_out_of_bounds_hi() {
        let (store, _, user_package_id) = compile_package();

        let location = Location::from(
            Span { lo: 0, hi: 2000 },
            user_package_id,
            &store,
            user_package_id,
            Encoding::Utf8,
        );

        // Per [`Range`] spec, out of bounds positions map to EOF
        expect![[r#"
            Location {
                source: "foo.qs",
                range: Range {
                    start: Position {
                        line: 0,
                        column: 0,
                    },
                    end: Position {
                        line: 0,
                        column: 17,
                    },
                },
            }
        "#]]
        .assert_debug_eq(&location);
    }

    fn compile_package() -> (PackageStore, PackageId, PackageId) {
        let mut store = PackageStore::new(compile::core());
        let mut dependencies = Vec::new();

        let (package_type, capabilities) = (PackageType::Lib, RuntimeCapabilityFlags::all());

        let std = compile::std(&store, capabilities);
        let std_package_id = store.insert(std);

        dependencies.push(std_package_id);
        let sources = SourceMap::new(
            [
                ("foo.qs".into(), "namespace Foo { }".into()),
                ("bar.qs".into(), "namespace Bar { }".into()),
            ],
            None,
        );
        let (unit, _) =
            compile::compile(&store, &dependencies, sources, package_type, capabilities);
        let user_package_id = store.insert(unit);

        (store, std_package_id, user_package_id)
    }
}
