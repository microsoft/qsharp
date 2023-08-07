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
    eval_expr,
    output::GenericReceiver,
    val::{self, GlobalId, Value},
    Env, Error, Global, NodeLookup, State,
};
use qsc_fir::fir::{BlockId, ExprId, ItemKind, PackageId, PatId, StmtId};
use qsc_frontend::compile::PackageStore;
use qsc_hir::hir::{self};
use quantum_sparse_sim::QuantumSim;
use std::fmt::Write;

const PREFIX: &str = include_str!("./qir_base/prefix.ll");
const POSTFIX: &str = include_str!("./qir_base/postfix.ll");

/// # Errors
///
/// This function will return an error if execution was unable to complete.
pub fn generate_qir(
    store: &PackageStore,
    package: hir::PackageId,
) -> Result<String, (Error, Vec<Frame>)> {
    let mut fir_lowerer = qsc_eval::lower::Lowerer::new();
    let mut fir_store = IndexMap::new();
    let package = map_hir_package_to_fir(package);
    let mut sim = BaseProfSim::default();
    write!(&mut sim.instrs, "{PREFIX}").expect("writing to string should succeed");

    for (id, unit) in store.iter() {
        fir_store.insert(
            map_hir_package_to_fir(id),
            fir_lowerer.lower_package(&unit.package),
        );
    }

    let unit = fir_store.get(package).expect("store should have package");
    let entry_expr = unit.entry.expect("package should have entry");

    let mut stdout = vec![];
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
            write_output_recording(&val, &mut sim.instrs);
            write!(sim.instrs, "{POSTFIX}").expect("writing to string should succeed");
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

#[derive(Debug, PartialEq, Copy, Clone)]
struct StaticResultId(usize);

impl From<val::Result> for StaticResultId {
    fn from(r: val::Result) -> Self {
        Self(r.into())
    }
}

impl From<StaticResultId> for val::Result {
    fn from(r: StaticResultId) -> Self {
        r.0.into()
    }
}

impl BaseProfSim {
    #[must_use]
    fn get_meas_id(&mut self) -> usize {
        let id = self.next_meas_id;
        self.next_meas_id += 1;
        id
    }
}

impl Backend for BaseProfSim {
    type ResultType = StaticResultId;

    fn ccx(&mut self, ctl0: usize, ctl1: usize, q: usize) {
        write!(&mut self.instrs, "  call void @__quantum__qis__ccx__body(")
            .expect("writing to string should succeed");
        write_qubit(ctl0, &mut self.instrs);
        write!(self.instrs, ", ").expect("writing to string should succeed");
        write_qubit(ctl1, &mut self.instrs);
        write!(self.instrs, ", ").expect("writing to string should succeed");
        write_qubit(q, &mut self.instrs);
        writeln!(&mut self.instrs, ")").expect("writing to string should succeed");
    }

    fn cx(&mut self, ctl: usize, q: usize) {
        write!(&mut self.instrs, "  call void @__quantum__qis__cx__body(")
            .expect("writing to string should succeed");
        write_qubit(ctl, &mut self.instrs);
        write!(self.instrs, ", ").expect("writing to string should succeed");
        write_qubit(q, &mut self.instrs);
        writeln!(&mut self.instrs, ")").expect("writing to string should succeed");
    }

    fn cy(&mut self, ctl: usize, q: usize) {
        write!(&mut self.instrs, "  call void @__quantum__qis__cy__body(")
            .expect("writing to string should succeed");
        write_qubit(ctl, &mut self.instrs);
        write!(self.instrs, ", ").expect("writing to string should succeed");
        write_qubit(q, &mut self.instrs);
        writeln!(&mut self.instrs, ")").expect("writing to string should succeed");
    }

    fn cz(&mut self, ctl: usize, q: usize) {
        write!(&mut self.instrs, "  call void @__quantum__qis__cz__body(")
            .expect("writing to string should succeed");
        write_qubit(ctl, &mut self.instrs);
        write!(self.instrs, ", ").expect("writing to string should succeed");
        write_qubit(q, &mut self.instrs);
        writeln!(&mut self.instrs, ")").expect("writing to string should succeed");
    }

    fn h(&mut self, q: usize) {
        write!(&mut self.instrs, "  call void @__quantum__qis__h__body(")
            .expect("writing to string should succeed");
        write_qubit(q, &mut self.instrs);
        writeln!(&mut self.instrs, ")").expect("writing to string should succeed");
    }

    fn m(&mut self, q: usize) -> Self::ResultType {
        let id = self.get_meas_id();
        write!(self.instrs, "  call void @__quantum__qis__mz__body(")
            .expect("writing to string should succeed");
        write_qubit(q, &mut self.instrs);
        write!(self.instrs, ", ").expect("writing to string should succeed");
        write_result(id, &mut self.instrs);
        writeln!(&mut self.instrs, ")").expect("writing to string should succeed");
        StaticResultId(id)
    }

    fn mresetz(&mut self, q: usize) -> Self::ResultType {
        let id = self.get_meas_id();
        write!(self.instrs, "  call void @__quantum__qis__mz__body(")
            .expect("writing to string should succeed");
        write_qubit(q, &mut self.instrs);
        write!(self.instrs, ", ").expect("writing to string should succeed");
        write_result(id, &mut self.instrs);
        writeln!(&mut self.instrs, ")").expect("writing to string should succeed");
        self.reset(q);
        StaticResultId(id)
    }

    fn reset(&mut self, q: usize) {
        write!(self.instrs, "  call void @__quantum__qis__reset__body(")
            .expect("writing to string should succeed");
        write_qubit(q, &mut self.instrs);
        writeln!(&mut self.instrs, ")").expect("writing to string should succeed");
    }

    fn rx(&mut self, theta: f64, q: usize) {
        write!(self.instrs, "  call void @__quantum__qis__rx__body(")
            .expect("writing to string should succeed");
        write!(self.instrs, "double {theta}").expect("writing to string should succeed");
        write!(self.instrs, ", ").expect("writing to string should succeed");
        write_qubit(q, &mut self.instrs);
        writeln!(&mut self.instrs, ")").expect("writing to string should succeed");
    }

    fn rxx(&mut self, theta: f64, q0: usize, q1: usize) {
        write!(self.instrs, "  call void @__quantum__qis__rxx__body(")
            .expect("writing to string should succeed");
        write!(self.instrs, "double {theta}").expect("writing to string should succeed");
        write!(self.instrs, ", ").expect("writing to string should succeed");
        write_qubit(q0, &mut self.instrs);
        write!(self.instrs, ", ").expect("writing to string should succeed");
        write_qubit(q1, &mut self.instrs);
        writeln!(&mut self.instrs, ")").expect("writing to string should succeed");
    }

    fn ry(&mut self, theta: f64, q: usize) {
        write!(self.instrs, "  call void @__quantum__qis__ry__body(")
            .expect("writing to string should succeed");
        write!(self.instrs, "double {theta}").expect("writing to string should succeed");
        write!(self.instrs, ", ").expect("writing to string should succeed");
        write_qubit(q, &mut self.instrs);
        writeln!(&mut self.instrs, ")").expect("writing to string should succeed");
    }

    fn ryy(&mut self, theta: f64, q0: usize, q1: usize) {
        write!(self.instrs, "  call void @__quantum__qis__ryy__body(")
            .expect("writing to string should succeed");
        write!(self.instrs, "double {theta}").expect("writing to string should succeed");
        write!(self.instrs, ", ").expect("writing to string should succeed");
        write_qubit(q0, &mut self.instrs);
        write!(self.instrs, ", ").expect("writing to string should succeed");
        write_qubit(q1, &mut self.instrs);
        writeln!(&mut self.instrs, ")").expect("writing to string should succeed");
    }

    fn rz(&mut self, theta: f64, q: usize) {
        write!(self.instrs, "  call void @__quantum__qis__rz__body(")
            .expect("writing to string should succeed");
        write!(self.instrs, "double {theta}").expect("writing to string should succeed");
        write!(self.instrs, ", ").expect("writing to string should succeed");
        write_qubit(q, &mut self.instrs);
        writeln!(&mut self.instrs, ")").expect("writing to string should succeed");
    }

    fn rzz(&mut self, theta: f64, q0: usize, q1: usize) {
        write!(self.instrs, "  call void @__quantum__qis__rzz__body(")
            .expect("writing to string should succeed");
        write!(self.instrs, "double {theta}").expect("writing to string should succeed");
        write!(self.instrs, ", ").expect("writing to string should succeed");
        write_qubit(q0, &mut self.instrs);
        write!(self.instrs, ", ").expect("writing to string should succeed");
        write_qubit(q1, &mut self.instrs);
        writeln!(&mut self.instrs, ")").expect("writing to string should succeed");
    }

    fn sadj(&mut self, q: usize) {
        write!(self.instrs, "  call void @__quantum__qis__s__adj(")
            .expect("writing to string should succeed");
        write_qubit(q, &mut self.instrs);
        writeln!(&mut self.instrs, ")").expect("writing to string should succeed");
    }

    fn s(&mut self, q: usize) {
        write!(self.instrs, "  call void @__quantum__qis__s__body(")
            .expect("writing to string should succeed");
        write_qubit(q, &mut self.instrs);
        writeln!(&mut self.instrs, ")").expect("writing to string should succeed");
    }

    fn swap(&mut self, q0: usize, q1: usize) {
        write!(self.instrs, "  call void @__quantum__qis__swap__body(")
            .expect("writing to string should succeed");
        write_qubit(q0, &mut self.instrs);
        write!(self.instrs, ", ").expect("writing to string should succeed");
        write_qubit(q1, &mut self.instrs);
        writeln!(&mut self.instrs, ")").expect("writing to string should succeed");
    }

    fn tadj(&mut self, q: usize) {
        // writeln!(&mut self.instrs, "tadj {q}",).expect("writing to string should succeed");
        write!(self.instrs, "  call void @__quantum__qis__t__adj(")
            .expect("writing to string should succeed");
        write_qubit(q, &mut self.instrs);
        writeln!(&mut self.instrs, ")").expect("writing to string should succeed");
    }

    fn t(&mut self, q: usize) {
        write!(self.instrs, "  call void @__quantum__qis__t__body(")
            .expect("writing to string should succeed");
        write_qubit(q, &mut self.instrs);
        writeln!(&mut self.instrs, ")").expect("writing to string should succeed");
    }

    fn x(&mut self, q: usize) {
        write!(self.instrs, "  call void @__quantum__qis__x__body(")
            .expect("writing to string should succeed");
        write_qubit(q, &mut self.instrs);
        writeln!(&mut self.instrs, ")").expect("writing to string should succeed");
    }

    fn y(&mut self, q: usize) {
        write!(self.instrs, "  call void @__quantum__qis__y__body(")
            .expect("writing to string should succeed");
        write_qubit(q, &mut self.instrs);
        writeln!(&mut self.instrs, ")").expect("writing to string should succeed");
    }

    fn z(&mut self, q: usize) {
        // writeln!(&mut self.instrs, "z {q}",).expect("writing to string should succeed");
        write!(self.instrs, "  call void @__quantum__qis__z__body(")
            .expect("writing to string should succeed");
        write_qubit(q, &mut self.instrs);
        writeln!(&mut self.instrs, ")").expect("writing to string should succeed");
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
        panic!("calling qubit_is_zero during base profile simulation is not supported")
    }
}

fn write_qubit(q: usize, f: &mut impl Write) {
    write!(f, "%Qubit* inttoptr (i64 {q} to %Qubit*)").expect("writing to string should succeed");
}

fn write_result(r: usize, f: &mut impl Write) {
    write!(f, "%Result* inttoptr (i64 {r} to %Result*)").expect("writing to string should succeed");
}

fn write_output_recording(val: &Value, f: &mut impl Write) {
    match val {
        Value::Array(arr) => {
            write_array_recording(arr.len(), f);
            for v in arr.iter() {
                write_output_recording(v, f);
            }
        }
        Value::Result(r) => {
            let r = StaticResultId::from(*r);
            write_result_recording(r.0, f);
        }
        Value::Tuple(tup) => {
            write_tuple_recording(tup.len(), f);
            for v in tup.iter() {
                write_output_recording(v, f);
            }
        }
        _ => panic!("unexpected value type: {val:?}"),
    }
}

fn write_result_recording(r: usize, f: &mut impl Write) {
    write!(f, "  call void @__quantum__rt__result_record_output(")
        .expect("writing to string should succeed");
    write_result(r, f);
    writeln!(f, ", i8* null)").expect("writing to string should succeed");
}

fn write_tuple_recording(s: usize, f: &mut impl Write) {
    writeln!(
        f,
        "  call void @__quantum__rt__tuple_record_output(i64 {s}, i8* null)"
    )
    .expect("writing to string should succeed");
}

fn write_array_recording(s: usize, f: &mut impl Write) {
    writeln!(
        f,
        "  call void @__quantum__rt__array_record_output(i64 {s}, i8* null)"
    )
    .expect("writing to string should succeed");
}
