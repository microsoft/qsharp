// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::remapper::{HardwareId, Remapper};
use num_bigint::BigUint;
use num_complex::Complex;
use qsc_eval::{
    backend::Backend,
    debug::{map_hir_package_to_fir, Frame},
    eval,
    output::GenericReceiver,
    val::Value,
    Env, Error,
};
use qsc_fir::fir;
use qsc_frontend::compile::PackageStore;
use qsc_hir::hir::{self};
use rustc_hash::FxHashSet;
use std::fmt::{Display, Write};

/// # Errors
///
/// This function will return an error if execution was unable to complete.
/// # Panics
///
/// This function will panic if compiler state is invalid or in out-of-memory conditions.
pub fn generate_qir(
    store: &PackageStore,
    package: hir::PackageId,
) -> std::result::Result<String, (Error, Vec<Frame>)> {
    let mut fir_lowerer = qsc_eval::lower::Lowerer::new();
    let mut fir_store = fir::PackageStore::new();
    for (id, unit) in store {
        fir_store.insert(
            map_hir_package_to_fir(id),
            fir_lowerer.lower_package(&unit.package),
        );
    }

    let package = map_hir_package_to_fir(package);
    let unit = fir_store.get(package);

    let mut sim = BaseProfSim::default();
    let mut stdout = std::io::sink();
    let mut out = GenericReceiver::new(&mut stdout);
    let result = eval(
        package,
        None,
        unit.entry_exec_graph.clone(),
        &fir_store,
        &mut Env::default(),
        &mut sim,
        &mut out,
    );
    match result {
        Ok(val) => Ok(sim.finish(&val)),
        Err((err, stack)) => Err((err, stack)),
    }
}

pub struct BaseProfSim {
    instrs: String,
    decls: String,
    decl_names: FxHashSet<String>,
    remapper: Remapper,
}

impl Default for BaseProfSim {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseProfSim {
    #[must_use]
    pub fn new() -> Self {
        let mut sim = BaseProfSim {
            instrs: String::new(),
            decls: String::new(),
            decl_names: FxHashSet::default(),
            remapper: Remapper::default(),
        };
        sim.instrs.push_str(include_str!("./qir_base/prefix.ll"));
        sim
    }

    #[must_use]
    pub fn finish(mut self, val: &Value) -> String {
        for (mapped_q, id) in self.remapper.measurements() {
            writeln!(
                self.instrs,
                "  call void @__quantum__qis__mz__body({}, {}) #1",
                Qubit(*mapped_q),
                Result(*id),
            )
            .expect("writing to string should succeed");
        }
        self.write_output_recording(val)
            .expect("writing to string should succeed");

        write!(
            self.instrs,
            include_str!("./qir_base/postfix.ll"),
            self.decls,
            self.remapper.num_qubits(),
            self.remapper.num_measurements()
        )
        .expect("writing to string should succeed");

        self.instrs
    }

    fn map(&mut self, qubit: usize) -> HardwareId {
        self.remapper.map(qubit)
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

    fn write_arg(&mut self, arg: &Value) -> std::result::Result<(), String> {
        match arg {
            Value::Qubit(q) => {
                let q = self.map(q.0);
                write!(self.instrs, "{}", Qubit(q))
            }
            Value::Double(d) => write!(self.instrs, "{}", Double(*d)),
            Value::Bool(b) => write!(self.instrs, "{}", Bool(*b)),
            Value::Int(i) => write!(self.instrs, "{}", Int(*i)),
            _ => return Err(format!("unsupported argument type: {}", arg.type_name())),
        }
        .expect("writing to string should succeed");
        Ok(())
    }

    fn write_decl_type(&mut self, ty: &Value) -> std::result::Result<(), String> {
        match ty {
            Value::Qubit(_) => write!(self.decls, "%Qubit*"),
            Value::Double(_) => write!(self.decls, "double"),
            Value::Bool(_) => write!(self.decls, "i1"),
            Value::Int(_) => write!(self.decls, "i64"),
            _ => return Err(format!("unsupported argument type: {}", ty.type_name())),
        }
        .expect("writing to string should succeed");
        Ok(())
    }

    fn write_decl(&mut self, name: &str, arg: &Value) -> std::result::Result<(), String> {
        if self.decl_names.insert(name.to_string()) {
            write!(self.decls, "declare void @{name}(").expect("writing to string should succeed");
            if let Value::Tuple(args) = arg {
                if let Some((first, rest)) = args.split_first() {
                    self.write_decl_type(first)?;
                    for arg in rest {
                        write!(self.decls, ", ").expect("writing to string should succeed");
                        self.write_decl_type(arg)?;
                    }
                }
            } else {
                self.write_decl_type(arg)?;
            }
            writeln!(self.decls, ")").expect("writing to string should succeed");
        }

        Ok(())
    }
}

impl Backend for BaseProfSim {
    type ResultType = usize;

    fn ccx(&mut self, ctl0: usize, ctl1: usize, q: usize) {
        let ctl0 = self.map(ctl0);
        let ctl1 = self.map(ctl1);
        let q = self.map(q);
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
        let ctl = self.map(ctl);
        let q = self.map(q);
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__cx__body({}, {})",
            Qubit(ctl),
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn cy(&mut self, ctl: usize, q: usize) {
        let ctl = self.map(ctl);
        let q = self.map(q);
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__cy__body({}, {})",
            Qubit(ctl),
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn cz(&mut self, ctl: usize, q: usize) {
        let ctl = self.map(ctl);
        let q = self.map(q);
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__cz__body({}, {})",
            Qubit(ctl),
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn h(&mut self, q: usize) {
        let q = self.map(q);
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__h__body({})",
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn m(&mut self, q: usize) -> Self::ResultType {
        // Measurements are tracked separately from instructions, so that they can be
        // deferred until the end of the program.
        self.remapper.mreset(q)
    }

    fn mresetz(&mut self, q: usize) -> Self::ResultType {
        self.remapper.mreset(q)
    }

    fn reset(&mut self, q: usize) {
        // Reset is a no-op in Base Profile, but does force qubit remapping so that future
        // operations on the given qubit id are performed on a fresh qubit. Clear the entry in the map
        // so it is known to require remapping on next use.
        self.remapper.reset(q);
    }

    fn rx(&mut self, theta: f64, q: usize) {
        let q = self.map(q);
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__rx__body({}, {})",
            Double(theta),
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn rxx(&mut self, theta: f64, q0: usize, q1: usize) {
        let q0 = self.map(q0);
        let q1 = self.map(q1);
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__rxx__body({}, {}, {})",
            Double(theta),
            Qubit(q0),
            Qubit(q1),
        )
        .expect("writing to string should succeed");
    }

    fn ry(&mut self, theta: f64, q: usize) {
        let q = self.map(q);
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__ry__body({}, {})",
            Double(theta),
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn ryy(&mut self, theta: f64, q0: usize, q1: usize) {
        let q0 = self.map(q0);
        let q1 = self.map(q1);
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__ryy__body({}, {}, {})",
            Double(theta),
            Qubit(q0),
            Qubit(q1),
        )
        .expect("writing to string should succeed");
    }

    fn rz(&mut self, theta: f64, q: usize) {
        let q = self.map(q);
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__rz__body({}, {})",
            Double(theta),
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn rzz(&mut self, theta: f64, q0: usize, q1: usize) {
        let q0 = self.map(q0);
        let q1 = self.map(q1);
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__rzz__body({}, {}, {})",
            Double(theta),
            Qubit(q0),
            Qubit(q1),
        )
        .expect("writing to string should succeed");
    }

    fn sadj(&mut self, q: usize) {
        let q = self.map(q);
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__s__adj({})",
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn s(&mut self, q: usize) {
        let q = self.map(q);
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__s__body({})",
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn swap(&mut self, q0: usize, q1: usize) {
        let q0 = self.map(q0);
        let q1 = self.map(q1);
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__swap__body({}, {})",
            Qubit(q0),
            Qubit(q1),
        )
        .expect("writing to string should succeed");
    }

    fn tadj(&mut self, q: usize) {
        let q = self.map(q);
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__t__adj({})",
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn t(&mut self, q: usize) {
        let q = self.map(q);
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__t__body({})",
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn x(&mut self, q: usize) {
        let q = self.map(q);
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__x__body({})",
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn y(&mut self, q: usize) {
        let q = self.map(q);
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__y__body({})",
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn z(&mut self, q: usize) {
        let q = self.map(q);
        writeln!(
            self.instrs,
            "  call void @__quantum__qis__z__body({})",
            Qubit(q),
        )
        .expect("writing to string should succeed");
    }

    fn qubit_allocate(&mut self) -> usize {
        self.remapper.qubit_allocate()
    }

    fn qubit_release(&mut self, q: usize) {
        self.remapper.qubit_release(q);
    }

    fn capture_quantum_state(&mut self) -> (Vec<(BigUint, Complex<f64>)>, usize) {
        (Vec::new(), 0)
    }

    fn qubit_is_zero(&mut self, _q: usize) -> bool {
        // Because `qubit_is_zero` is called on every qubit release, this must return
        // true to avoid a panic.
        true
    }

    fn custom_intrinsic(
        &mut self,
        name: &str,
        arg: Value,
    ) -> Option<std::result::Result<Value, String>> {
        match self.write_decl(name, &arg) {
            Ok(()) => {}
            Err(e) => return Some(Err(e)),
        }
        write!(self.instrs, "  call void @{name}(").expect("writing to string should succeed");

        if let Value::Tuple(args) = arg {
            if let Some((first, rest)) = args.split_first() {
                match self.write_arg(first) {
                    Ok(()) => {}
                    Err(e) => return Some(Err(e)),
                }
                for arg in rest {
                    write!(self.instrs, ", ").expect("writing to string should succeed");
                    match self.write_arg(arg) {
                        Ok(()) => {}
                        Err(e) => return Some(Err(e)),
                    }
                }
            }
        } else {
            match self.write_arg(&arg) {
                Ok(()) => {}
                Err(e) => return Some(Err(e)),
            }
        }

        writeln!(self.instrs, ")").expect("writing to string should succeed");
        Some(Ok(Value::unit()))
    }
}

struct Qubit(HardwareId);

impl Display for Qubit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%Qubit* inttoptr (i64 {} to %Qubit*)", self.0 .0)
    }
}

struct Result(usize);

impl Display for Result {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%Result* inttoptr (i64 {} to %Result*)", self.0)
    }
}

struct Double(f64);

impl Display for Double {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.0;
        if (v.floor() - v.ceil()).abs() < f64::EPSILON {
            // The value is a whole number, which requires at least one decimal point
            // to differentiate it from an integer value.
            write!(f, "double {v:.1}")
        } else {
            write!(f, "double {v}")
        }
    }
}

struct Bool(bool);

impl Display for Bool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 {
            write!(f, "i1 true")
        } else {
            write!(f, "i1 false")
        }
    }
}

struct Int(i64);

impl Display for Int {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "i64 {}", self.0)
    }
}
