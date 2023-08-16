// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use num_bigint::BigUint;
use num_complex::Complex;
use qsc_data_structures::index_map::IndexMap;
use qsc_eval::{
    backend::Backend,
    debug::{map_hir_package_to_fir, Frame},
    eval_expr, eval_stmt,
    output::GenericReceiver,
    val::{GlobalId, Value},
    Env, Error, Global, NodeLookup, State,
};
use qsc_fir::fir::{BlockId, ExprId, ItemKind, PackageId, PatId, StmtId};
use qsc_frontend::compile::PackageStore;
use qsc_hir::hir::{self};
use quantum_sparse_sim::QuantumSim;
use std::fmt::{Display, Write};

const PREFIX: &str = include_str!("./qir_base/prefix.ll");
const POSTFIX: &str = include_str!("./qir_base/postfix.ll");

/// # Errors
///
/// This function will return an error if execution was unable to complete.
pub fn generate_qir(
    store: &PackageStore,
    package: hir::PackageId,
) -> std::result::Result<String, (Error, Vec<Frame>)> {
    let mut fir_lowerer = qsc_eval::lower::Lowerer::new();
    let mut fir_store = IndexMap::new();
    let package = map_hir_package_to_fir(package);
    let mut sim = BaseProfSim::default();
    sim.instrs.push_str(PREFIX);

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
        Ok(val) => {
            sim.write_output_recording(&val)
                .expect("writing to string should succeed");
            sim.instrs.push_str(POSTFIX);
            Ok(sim.instrs)
        }
        Err((err, stack)) => Err((err, stack)),
    }
}

/// # Errors
/// This function will return an error if execution was unable to complete.
pub fn generate_qir_for_stmt(
    stmt: StmtId,
    globals: &impl NodeLookup,
    env: &mut Env,
    package: PackageId,
) -> std::result::Result<String, (Error, Vec<Frame>)> {
    let mut sim = BaseProfSim::default();
    sim.instrs.push_str(PREFIX);
    let mut stdout = std::io::sink();
    let mut out = GenericReceiver::new(&mut stdout);
    match eval_stmt(stmt, globals, env, &mut sim, package, &mut out) {
        Ok(val) => {
            sim.write_output_recording(&val)
                .expect("writing to string should succeed");
            sim.instrs.push_str(POSTFIX);
            Ok(sim.instrs)
        }
        Err((err, stack)) => Err((err, stack)),
    }
}

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

#[derive(Default)]
struct BaseProfSim {
    next_meas_id: usize,
    sim: QuantumSim,
    instrs: String,
}

impl BaseProfSim {
    #[must_use]
    fn get_meas_id(&mut self) -> usize {
        let id = self.next_meas_id;
        self.next_meas_id += 1;
        id
    }

    fn write_output_recording(&mut self, val: &Value) -> std::fmt::Result {
        match val {
            Value::Array(arr) => {
                self.write_array_recording(arr.len())?;
                for val in arr.iter() {
                    self.write_output_recording(val)?;
                }
            }
            Value::Result(r) => {
                self.write_result_recording(r.unwrap_id());
            }
            Value::Tuple(tup) => {
                self.write_tuple_recording(tup.len())?;
                for val in tup.iter() {
                    self.write_output_recording(val)?;
                }
            }
            _ => panic!("unexpected value type: {val:?}"),
        }
        Ok(())
    }

    fn write_result_recording(&mut self, res: usize) {
        writeln!(
            self.instrs,
            "  call void @__quantum__rt__result_record_output({}, i8* null)",
            Result(res),
        )
        .expect("writing to string should succeed");
    }

    fn write_tuple_recording(&mut self, size: usize) -> std::fmt::Result {
        writeln!(
            self.instrs,
            "  call void @__quantum__rt__tuple_record_output(i64 {size}, i8* null)"
        )
    }

    fn write_array_recording(&mut self, size: usize) -> std::fmt::Result {
        writeln!(
            self.instrs,
            "  call void @__quantum__rt__array_record_output(i64 {size}, i8* null)"
        )
    }
}

impl Backend for BaseProfSim {
    type ResultType = usize;

    fn ccx(&mut self, ctl0: usize, ctl1: usize, q: usize) {
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__ccx__body({}, {}, {})",
            Qubit(ctl0),
            Qubit(ctl1),
            Qubit(q)
        )
        .expect("writing to string should succeed");
    }

    fn cx(&mut self, ctl: usize, q: usize) {
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__cx__body({}, {})",
            Qubit(ctl),
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn cy(&mut self, ctl: usize, q: usize) {
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__cy__body({}, {})",
            Qubit(ctl),
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn cz(&mut self, ctl: usize, q: usize) {
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__cz__body({}, {})",
            Qubit(ctl),
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn h(&mut self, q: usize) {
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__h__body({})",
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn m(&mut self, q: usize) -> Self::ResultType {
        let id = self.get_meas_id();
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__mz__body({}, {}) #1",
            Qubit(q),
            Result(id),
        )
        .expect("writing to string should succeed");
        id
    }

    fn mresetz(&mut self, q: usize) -> Self::ResultType {
        let id = self.get_meas_id();
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__mz__body({}, {}) #1",
            Qubit(q),
            Result(id),
        )
        .expect("writing to string should succeed");
        self.reset(q);
        id
    }

    fn reset(&mut self, q: usize) {
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__reset__body({})",
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn rx(&mut self, theta: f64, q: usize) {
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__rx__body(double {theta}, {})",
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn rxx(&mut self, theta: f64, q0: usize, q1: usize) {
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__rxx__body(double {theta}, {}, {})",
            Qubit(q0),
            Qubit(q1),
        )
        .expect("writing to string should succeed");
    }

    fn ry(&mut self, theta: f64, q: usize) {
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__ry__body(double {theta}, {})",
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn ryy(&mut self, theta: f64, q0: usize, q1: usize) {
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__ryy__body(double {theta}, {}, {})",
            Qubit(q0),
            Qubit(q1),
        )
        .expect("writing to string should succeed");
    }

    fn rz(&mut self, theta: f64, q: usize) {
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__rz__body(double {theta}, {})",
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn rzz(&mut self, theta: f64, q0: usize, q1: usize) {
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__rzz__body(double {theta}, {}, {})",
            Qubit(q0),
            Qubit(q1),
        )
        .expect("writing to string should succeed");
    }

    fn sadj(&mut self, q: usize) {
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__s__adj({})",
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn s(&mut self, q: usize) {
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__s__body({})",
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn swap(&mut self, q0: usize, q1: usize) {
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__swap__body({}, {})",
            Qubit(q0),
            Qubit(q1),
        )
        .expect("writing to string should succeed");
    }

    fn tadj(&mut self, q: usize) {
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__t__adj({})",
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn t(&mut self, q: usize) {
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__t__body({})",
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn x(&mut self, q: usize) {
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__x__body({})",
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn y(&mut self, q: usize) {
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__y__body({})",
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn z(&mut self, q: usize) {
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__z__body({})",
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn qubit_allocate(&mut self) -> usize {
        self.sim.allocate()
    }

    fn qubit_release(&mut self, q: usize) {
        self.sim.release(q);
    }

    fn capture_quantum_state(&mut self) -> (Vec<(BigUint, Complex<f64>)>, usize) {
        (Vec::new(), 0)
    }

    fn qubit_is_zero(&mut self, _q: usize) -> bool {
        // Because `qubit_is_zero` is called on every qubit release, this must return
        // true to avoid a panic.
        true
    }
}

struct Qubit(usize);

impl Display for Qubit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%Qubit* inttoptr (i64 {} to %Qubit*)", self.0)
    }
}

struct Result(usize);

impl Display for Result {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%Result* inttoptr (i64 {} to %Result*)", self.0)
    }
}
