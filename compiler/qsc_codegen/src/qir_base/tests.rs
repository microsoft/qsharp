// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines)]
#![allow(clippy::needless_raw_string_hashes)]

use std::sync::Arc;

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_data_structures::index_map::IndexMap;
use qsc_eval::{
    debug::{map_hir_package_to_fir, Frame},
    eval_expr,
    output::GenericReceiver,
    val::GlobalId,
    Env, Error, Global, NodeLookup, State,
};
use qsc_fir::fir::{BlockId, ExprId, ItemKind, PackageId, PatId, StmtId};
use qsc_frontend::compile::{self, compile, PackageStore, SourceMap, TargetProfile};
use qsc_hir::hir;
use qsc_passes::{run_core_passes, run_default_passes, PackageType};

use super::BaseProfSim;

struct Lookup<'a> {
    fir_store: &'a IndexMap<PackageId, qsc_fir::fir::Package>,
}

impl<'a> Lookup<'a> {
    fn get_package(&self, package: PackageId) -> &qsc_fir::fir::Package {
        self.fir_store
            .get(package)
            .expect("Package should be in FIR store")
    }
}

impl<'a> NodeLookup for Lookup<'a> {
    fn get(&self, id: GlobalId) -> Option<Global<'a>> {
        get_global(self.fir_store, id)
    }
    fn get_block(&self, package: PackageId, id: BlockId) -> &qsc_fir::fir::Block {
        self.get_package(package)
            .blocks
            .get(id)
            .expect("BlockId should have been lowered")
    }
    fn get_expr(&self, package: PackageId, id: ExprId) -> &qsc_fir::fir::Expr {
        self.get_package(package)
            .exprs
            .get(id)
            .expect("ExprId should have been lowered")
    }
    fn get_pat(&self, package: PackageId, id: PatId) -> &qsc_fir::fir::Pat {
        self.get_package(package)
            .pats
            .get(id)
            .expect("PatId should have been lowered")
    }
    fn get_stmt(&self, package: PackageId, id: StmtId) -> &qsc_fir::fir::Stmt {
        self.get_package(package)
            .stmts
            .get(id)
            .expect("StmtId should have been lowered")
    }
}

pub(super) fn get_global(
    fir_store: &IndexMap<PackageId, qsc_fir::fir::Package>,
    id: GlobalId,
) -> Option<Global> {
    fir_store
        .get(id.package)
        .and_then(|package| match &package.items.get(id.item)?.kind {
            ItemKind::Callable(callable) => Some(Global::Callable(callable)),
            ItemKind::Namespace(..) => None,
            ItemKind::Ty(..) => Some(Global::Udt),
        })
}

fn generate_qir(
    store: &PackageStore,
    package: hir::PackageId,
) -> std::result::Result<String, (Error, Vec<Frame>)> {
    let mut fir_lowerer = qsc_eval::lower::Lowerer::new();
    let mut fir_store = IndexMap::new();
    let package = map_hir_package_to_fir(package);
    let mut sim = BaseProfSim::default();

    for (id, unit) in store.iter() {
        fir_store.insert(
            map_hir_package_to_fir(id),
            fir_lowerer.lower_package(&unit.package),
        );
    }

    let unit = fir_store.get(package).expect("store should have package");
    let entry_expr = unit.entry.expect("package should have entry");

    let mut stdout = std::io::sink();
    let mut out = GenericReceiver::new(&mut stdout);
    let result = eval_expr(
        &mut State::new(package),
        entry_expr,
        &Lookup {
            fir_store: &fir_store,
        },
        &mut Env::with_empty_scope(),
        &mut sim,
        &mut out,
    );
    match result {
        Ok(val) => Ok(sim.finish(&val)),
        Err((err, stack)) => Err((err, stack)),
    }
}

fn check(program: &str, expr: Option<&str>, expect: &Expect) {
    let mut core = compile::core();
    assert!(run_core_passes(&mut core).is_empty());
    let mut store = PackageStore::new(core);
    let mut std = compile::std(&store, TargetProfile::Base);
    assert!(run_default_passes(
        store.core(),
        &mut std,
        PackageType::Lib,
        TargetProfile::Base
    )
    .is_empty());
    let std = store.insert(std);

    let expr_as_arc: Option<Arc<str>> = expr.map(|s| Arc::from(s.to_string()));
    let sources = SourceMap::new([("test".into(), program.into())], expr_as_arc);

    let mut unit = compile(&store, &[std], sources, TargetProfile::Base);
    assert!(unit.errors.is_empty(), "{:?}", unit.errors);
    assert!(run_default_passes(
        store.core(),
        &mut unit,
        PackageType::Exe,
        TargetProfile::Base
    )
    .is_empty());
    let package = store.insert(unit);

    let qir = generate_qir(&store, package);
    match qir {
        Ok(qir) => expect.assert_eq(&qir),
        Err((err, _)) => expect.assert_debug_eq(&err),
    }
}

#[test]
fn simple_entry_program_is_valid() {
    check(
        indoc! {r#"
    namespace Sample {
        @EntryPoint()
        operation Entry() : Result
        {
            use q = Qubit();
            H(q);
            M(q)
        }
    }
        "#},
        None,
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__qis__ccx__body(%Qubit*, %Qubit*, %Qubit*)
            declare void @__quantum__qis__cx__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__cy__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__rx__body(double, %Qubit*)
            declare void @__quantum__qis__rxx__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__ry__body(double, %Qubit*)
            declare void @__quantum__qis__ryy__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__rz__body(double, %Qubit*)
            declare void @__quantum__qis__rzz__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__h__body(%Qubit*)
            declare void @__quantum__qis__s__body(%Qubit*)
            declare void @__quantum__qis__s__adj(%Qubit*)
            declare void @__quantum__qis__t__body(%Qubit*)
            declare void @__quantum__qis__t__adj(%Qubit*)
            declare void @__quantum__qis__x__body(%Qubit*)
            declare void @__quantum__qis__y__body(%Qubit*)
            declare void @__quantum__qis__z__body(%Qubit*)
            declare void @__quantum__qis__swap__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__mz__body(%Qubit*, %Result* writeonly) #1
            declare void @__quantum__rt__result_record_output(%Result*, i8*)
            declare void @__quantum__rt__array_record_output(i64, i8*)
            declare void @__quantum__rt__tuple_record_output(i64, i8*)

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="2" "required_num_results"="1" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]],
    );
}

#[test]
fn simple_program_is_valid() {
    check(
        "",
        Some(indoc! {r#"
        {
            use q = Qubit();
            H(q);
            M(q)
        }
        "#}),
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__qis__ccx__body(%Qubit*, %Qubit*, %Qubit*)
            declare void @__quantum__qis__cx__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__cy__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__rx__body(double, %Qubit*)
            declare void @__quantum__qis__rxx__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__ry__body(double, %Qubit*)
            declare void @__quantum__qis__ryy__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__rz__body(double, %Qubit*)
            declare void @__quantum__qis__rzz__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__h__body(%Qubit*)
            declare void @__quantum__qis__s__body(%Qubit*)
            declare void @__quantum__qis__s__adj(%Qubit*)
            declare void @__quantum__qis__t__body(%Qubit*)
            declare void @__quantum__qis__t__adj(%Qubit*)
            declare void @__quantum__qis__x__body(%Qubit*)
            declare void @__quantum__qis__y__body(%Qubit*)
            declare void @__quantum__qis__z__body(%Qubit*)
            declare void @__quantum__qis__swap__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__mz__body(%Qubit*, %Result* writeonly) #1
            declare void @__quantum__rt__result_record_output(%Result*, i8*)
            declare void @__quantum__rt__array_record_output(i64, i8*)
            declare void @__quantum__rt__tuple_record_output(i64, i8*)

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="2" "required_num_results"="1" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]],
    );
}

#[test]
fn output_recording_array() {
    check(
        "",
        Some(indoc! {"{use q = Qubit(); [M(q), M(q)]}"}),
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Result* inttoptr (i64 1 to %Result*)) #1
              call void @__quantum__rt__array_record_output(i64 2, i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__qis__ccx__body(%Qubit*, %Qubit*, %Qubit*)
            declare void @__quantum__qis__cx__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__cy__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__rx__body(double, %Qubit*)
            declare void @__quantum__qis__rxx__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__ry__body(double, %Qubit*)
            declare void @__quantum__qis__ryy__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__rz__body(double, %Qubit*)
            declare void @__quantum__qis__rzz__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__h__body(%Qubit*)
            declare void @__quantum__qis__s__body(%Qubit*)
            declare void @__quantum__qis__s__adj(%Qubit*)
            declare void @__quantum__qis__t__body(%Qubit*)
            declare void @__quantum__qis__t__adj(%Qubit*)
            declare void @__quantum__qis__x__body(%Qubit*)
            declare void @__quantum__qis__y__body(%Qubit*)
            declare void @__quantum__qis__z__body(%Qubit*)
            declare void @__quantum__qis__swap__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__mz__body(%Qubit*, %Result* writeonly) #1
            declare void @__quantum__rt__result_record_output(%Result*, i8*)
            declare void @__quantum__rt__array_record_output(i64, i8*)
            declare void @__quantum__rt__tuple_record_output(i64, i8*)

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="3" "required_num_results"="2" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]],
    );
}

#[test]
fn output_recording_tuple() {
    check(
        "",
        Some(indoc! {"{use q = Qubit(); (M(q), M(q))}"}),
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Result* inttoptr (i64 1 to %Result*)) #1
              call void @__quantum__rt__tuple_record_output(i64 2, i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__qis__ccx__body(%Qubit*, %Qubit*, %Qubit*)
            declare void @__quantum__qis__cx__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__cy__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__rx__body(double, %Qubit*)
            declare void @__quantum__qis__rxx__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__ry__body(double, %Qubit*)
            declare void @__quantum__qis__ryy__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__rz__body(double, %Qubit*)
            declare void @__quantum__qis__rzz__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__h__body(%Qubit*)
            declare void @__quantum__qis__s__body(%Qubit*)
            declare void @__quantum__qis__s__adj(%Qubit*)
            declare void @__quantum__qis__t__body(%Qubit*)
            declare void @__quantum__qis__t__adj(%Qubit*)
            declare void @__quantum__qis__x__body(%Qubit*)
            declare void @__quantum__qis__y__body(%Qubit*)
            declare void @__quantum__qis__z__body(%Qubit*)
            declare void @__quantum__qis__swap__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__mz__body(%Qubit*, %Result* writeonly) #1
            declare void @__quantum__rt__result_record_output(%Result*, i8*)
            declare void @__quantum__rt__array_record_output(i64, i8*)
            declare void @__quantum__rt__tuple_record_output(i64, i8*)

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="3" "required_num_results"="2" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]],
    );
}

#[test]
fn reset_allocates_new_qubit_id() {
    check(
        "",
        Some(indoc! {"{use q = Qubit(); H(q); Reset(q); H(q); M(q)}"}),
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__qis__ccx__body(%Qubit*, %Qubit*, %Qubit*)
            declare void @__quantum__qis__cx__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__cy__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__rx__body(double, %Qubit*)
            declare void @__quantum__qis__rxx__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__ry__body(double, %Qubit*)
            declare void @__quantum__qis__ryy__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__rz__body(double, %Qubit*)
            declare void @__quantum__qis__rzz__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__h__body(%Qubit*)
            declare void @__quantum__qis__s__body(%Qubit*)
            declare void @__quantum__qis__s__adj(%Qubit*)
            declare void @__quantum__qis__t__body(%Qubit*)
            declare void @__quantum__qis__t__adj(%Qubit*)
            declare void @__quantum__qis__x__body(%Qubit*)
            declare void @__quantum__qis__y__body(%Qubit*)
            declare void @__quantum__qis__z__body(%Qubit*)
            declare void @__quantum__qis__swap__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__mz__body(%Qubit*, %Result* writeonly) #1
            declare void @__quantum__rt__result_record_output(%Result*, i8*)
            declare void @__quantum__rt__array_record_output(i64, i8*)
            declare void @__quantum__rt__tuple_record_output(i64, i8*)

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="3" "required_num_results"="1" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]],
    );
}

#[test]
fn reuse_after_measurement_uses_fresh_aux_qubit_id() {
    check(
        "",
        Some(indoc! {"{use q = Qubit(); H(q); M(q); H(q); M(q)}"}),
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Result* inttoptr (i64 1 to %Result*)) #1
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__qis__ccx__body(%Qubit*, %Qubit*, %Qubit*)
            declare void @__quantum__qis__cx__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__cy__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__rx__body(double, %Qubit*)
            declare void @__quantum__qis__rxx__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__ry__body(double, %Qubit*)
            declare void @__quantum__qis__ryy__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__rz__body(double, %Qubit*)
            declare void @__quantum__qis__rzz__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__h__body(%Qubit*)
            declare void @__quantum__qis__s__body(%Qubit*)
            declare void @__quantum__qis__s__adj(%Qubit*)
            declare void @__quantum__qis__t__body(%Qubit*)
            declare void @__quantum__qis__t__adj(%Qubit*)
            declare void @__quantum__qis__x__body(%Qubit*)
            declare void @__quantum__qis__y__body(%Qubit*)
            declare void @__quantum__qis__z__body(%Qubit*)
            declare void @__quantum__qis__swap__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__mz__body(%Qubit*, %Result* writeonly) #1
            declare void @__quantum__rt__result_record_output(%Result*, i8*)
            declare void @__quantum__rt__array_record_output(i64, i8*)
            declare void @__quantum__rt__tuple_record_output(i64, i8*)

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="3" "required_num_results"="2" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]],
    );
}

#[test]
fn qubit_allocation_allows_reuse_of_unmeasured_qubits() {
    check(
        "",
        Some(indoc! {"{
            open Microsoft.Quantum.Measurement;
            { use (c, q) = (Qubit(), Qubit()); CNOT(c, q); MResetZ(q); }
            { use (c, q) = (Qubit(), Qubit()); CNOT(c, q); MResetZ(q) }
        }"}),
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Result* inttoptr (i64 1 to %Result*)) #1
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__qis__ccx__body(%Qubit*, %Qubit*, %Qubit*)
            declare void @__quantum__qis__cx__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__cy__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__rx__body(double, %Qubit*)
            declare void @__quantum__qis__rxx__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__ry__body(double, %Qubit*)
            declare void @__quantum__qis__ryy__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__rz__body(double, %Qubit*)
            declare void @__quantum__qis__rzz__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__h__body(%Qubit*)
            declare void @__quantum__qis__s__body(%Qubit*)
            declare void @__quantum__qis__s__adj(%Qubit*)
            declare void @__quantum__qis__t__body(%Qubit*)
            declare void @__quantum__qis__t__adj(%Qubit*)
            declare void @__quantum__qis__x__body(%Qubit*)
            declare void @__quantum__qis__y__body(%Qubit*)
            declare void @__quantum__qis__z__body(%Qubit*)
            declare void @__quantum__qis__swap__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__mz__body(%Qubit*, %Result* writeonly) #1
            declare void @__quantum__rt__result_record_output(%Result*, i8*)
            declare void @__quantum__rt__array_record_output(i64, i8*)
            declare void @__quantum__rt__tuple_record_output(i64, i8*)

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="3" "required_num_results"="2" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]],
    );
}

#[test]
fn verify_all_intrinsics() {
    check(
        "",
        Some(indoc! {"{
            use (q1, q2, q3) = (Qubit(), Qubit(), Qubit());
            CCNOT(q1, q2, q3);
            CX(q1, q2);
            CY(q1, q2);
            CZ(q1, q2);
            Rx(0.0, q1);
            Rxx(0.0, q1, q2);
            Ry(0.0, q1);
            Ryy(0.0, q1, q2);
            Rz(0.0, q1);
            Rzz(0.0, q1, q2);
            H(q1);
            S(q1);
            Adjoint S(q1);
            T(q1);
            Adjoint T(q1);
            X(q1);
            Y(q1);
            Z(q1);
            SWAP(q1, q2);
            Reset(q1);
            (M(q1),
            Microsoft.Quantum.Measurement.MResetZ(q1))
        }"}),
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cy__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__rx__body(double 0.0, %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__rxx__body(double 0.0, %Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__ry__body(double 0.0, %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__ryy__body(double 0.0, %Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__rz__body(double 0.0, %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__rzz__body(double 0.0, %Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__s__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__s__adj(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__y__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__z__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__swap__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 4 to %Qubit*), %Result* inttoptr (i64 1 to %Result*)) #1
              call void @__quantum__rt__tuple_record_output(i64 2, i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__qis__ccx__body(%Qubit*, %Qubit*, %Qubit*)
            declare void @__quantum__qis__cx__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__cy__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__rx__body(double, %Qubit*)
            declare void @__quantum__qis__rxx__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__ry__body(double, %Qubit*)
            declare void @__quantum__qis__ryy__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__rz__body(double, %Qubit*)
            declare void @__quantum__qis__rzz__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__h__body(%Qubit*)
            declare void @__quantum__qis__s__body(%Qubit*)
            declare void @__quantum__qis__s__adj(%Qubit*)
            declare void @__quantum__qis__t__body(%Qubit*)
            declare void @__quantum__qis__t__adj(%Qubit*)
            declare void @__quantum__qis__x__body(%Qubit*)
            declare void @__quantum__qis__y__body(%Qubit*)
            declare void @__quantum__qis__z__body(%Qubit*)
            declare void @__quantum__qis__swap__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__mz__body(%Qubit*, %Result* writeonly) #1
            declare void @__quantum__rt__result_record_output(%Result*, i8*)
            declare void @__quantum__rt__array_record_output(i64, i8*)
            declare void @__quantum__rt__tuple_record_output(i64, i8*)

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="5" "required_num_results"="2" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]],
    );
}

#[test]
fn complex_program_is_valid() {
    check(
        "",
        Some(indoc! {"{
            open Microsoft.Quantum.Measurement;
            open Microsoft.Quantum.Math;

            operation SWAPfromExp(q1 : Qubit, q2 : Qubit) : Unit is Ctl + Adj {
                let theta  = PI() / 4.0;
                Exp([PauliX, PauliX], theta, [q1, q2]);
                Exp([PauliY, PauliY], theta, [q1, q2]);
                Exp([PauliZ, PauliZ], theta, [q1, q2]);
            }

            use (aux, ctls, qs) = (Qubit(), Qubit[3], Qubit[2]);
            within {
                H(aux);
                ApplyToEachA(CNOT(aux, _), ctls + qs);
            }
            apply {
                Controlled SWAPfromExp(ctls, (qs[0], qs[1]));

                Controlled Adjoint SWAP(ctls, (qs[0], qs[1]));
            }

            MResetEachZ([aux] + ctls + qs)
        }"}),
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 5 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__rz__body(double -0.7853981633974483, %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__rz__body(double 0.7853981633974483, %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 5 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 5 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__s__body(%Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__rz__body(double -0.7853981633974483, %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__rz__body(double 0.7853981633974483, %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__s__adj(%Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 5 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 5 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__rz__body(double -0.7853981633974483, %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__rz__body(double 0.7853981633974483, %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 5 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 4 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 4 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Result* inttoptr (i64 2 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Result* inttoptr (i64 3 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 4 to %Qubit*), %Result* inttoptr (i64 4 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 5 to %Qubit*), %Result* inttoptr (i64 5 to %Result*)) #1
              call void @__quantum__rt__array_record_output(i64 6, i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 2 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 3 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 4 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 5 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__qis__ccx__body(%Qubit*, %Qubit*, %Qubit*)
            declare void @__quantum__qis__cx__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__cy__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__rx__body(double, %Qubit*)
            declare void @__quantum__qis__rxx__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__ry__body(double, %Qubit*)
            declare void @__quantum__qis__ryy__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__rz__body(double, %Qubit*)
            declare void @__quantum__qis__rzz__body(double, %Qubit*, %Qubit*)
            declare void @__quantum__qis__h__body(%Qubit*)
            declare void @__quantum__qis__s__body(%Qubit*)
            declare void @__quantum__qis__s__adj(%Qubit*)
            declare void @__quantum__qis__t__body(%Qubit*)
            declare void @__quantum__qis__t__adj(%Qubit*)
            declare void @__quantum__qis__x__body(%Qubit*)
            declare void @__quantum__qis__y__body(%Qubit*)
            declare void @__quantum__qis__z__body(%Qubit*)
            declare void @__quantum__qis__swap__body(%Qubit*, %Qubit*)
            declare void @__quantum__qis__mz__body(%Qubit*, %Result* writeonly) #1
            declare void @__quantum__rt__result_record_output(%Result*, i8*)
            declare void @__quantum__rt__array_record_output(i64, i8*)
            declare void @__quantum__rt__tuple_record_output(i64, i8*)

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="8" "required_num_results"="6" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]],
    );
}
