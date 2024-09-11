// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    circuit::{Circuit, Operation, Register},
    Config,
};
use num_bigint::BigUint;
use num_complex::Complex;
use qsc_data_structures::index_map::IndexMap;
use qsc_eval::{backend::Backend, val::Value};
use std::{fmt::Write, mem::take, rc::Rc};

/// Backend implementation that builds a circuit representation.
pub struct Builder {
    circuit: Circuit,
    config: Config,
    remapper: Remapper,
}

impl Backend for Builder {
    type ResultType = usize;

    fn ccx(&mut self, ctl0: usize, ctl1: usize, q: usize) {
        let ctl0 = self.map(ctl0);
        let ctl1 = self.map(ctl1);
        let q = self.map(q);
        self.push_gate(controlled_gate("X", [ctl0, ctl1], [q]));
    }

    fn cx(&mut self, ctl: usize, q: usize) {
        let ctl = self.map(ctl);
        let q = self.map(q);
        self.push_gate(controlled_gate("X", [ctl], [q]));
    }

    fn cy(&mut self, ctl: usize, q: usize) {
        let ctl = self.map(ctl);
        let q = self.map(q);
        self.push_gate(controlled_gate("Y", [ctl], [q]));
    }

    fn cz(&mut self, ctl: usize, q: usize) {
        let ctl = self.map(ctl);
        let q = self.map(q);
        self.push_gate(controlled_gate("Z", [ctl], [q]));
    }

    fn h(&mut self, q: usize) {
        let q = self.map(q);
        self.push_gate(gate("H", [q]));
    }

    fn m(&mut self, q: usize) -> Self::ResultType {
        if self.config.base_profile {
            // defer the measurement and reset the qubit
            self.remapper.mreset(q)
        } else {
            let mapped_q = self.map(q);
            // In the Circuit schema, result id is per-qubit
            let res_id = self.num_measurements_for_qubit(mapped_q);
            // We don't actually need the Remapper since we're not
            // remapping any qubits, but it's handy for keeping track of measurements
            let id = self.remapper.m(q);

            self.push_gate(measurement_gate(mapped_q.0, res_id));
            id
        }
    }

    fn mresetz(&mut self, q: usize) -> Self::ResultType {
        if self.config.base_profile {
            // defer the measurement
            self.remapper.mreset(q)
        } else {
            let mapped_q = self.map(q);
            // In the Circuit schema, result id is per-qubit
            let res_id = self.num_measurements_for_qubit(mapped_q);
            // We don't actually need the Remapper since we're not
            // remapping any qubits, but it's handy for keeping track of measurements
            let id = self.remapper.m(q);

            // Ideally MResetZ would be atomic but we don't currently have
            // a way to visually represent that. So decompose it into
            // a measurement and a reset gate.
            self.push_gate(measurement_gate(mapped_q.0, res_id));
            self.push_gate(gate(KET_ZERO, [mapped_q]));
            id
        }
    }

    fn reset(&mut self, q: usize) {
        if self.config.base_profile {
            self.remapper.reset(q);
        } else {
            let mapped_q = self.map(q);
            self.push_gate(gate(KET_ZERO, [mapped_q]));
        }
    }

    fn rx(&mut self, theta: f64, q: usize) {
        let q = self.map(q);
        self.push_gate(rotation_gate("rx", theta, [q]));
    }

    fn rxx(&mut self, theta: f64, q0: usize, q1: usize) {
        let q0 = self.map(q0);
        let q1 = self.map(q1);
        self.push_gate(rotation_gate("rxx", theta, [q0, q1]));
    }

    fn ry(&mut self, theta: f64, q: usize) {
        let q = self.map(q);
        self.push_gate(rotation_gate("ry", theta, [q]));
    }

    fn ryy(&mut self, theta: f64, q0: usize, q1: usize) {
        let q0 = self.map(q0);
        let q1 = self.map(q1);
        self.push_gate(rotation_gate("ryy", theta, [q0, q1]));
    }

    fn rz(&mut self, theta: f64, q: usize) {
        let q = self.map(q);
        self.push_gate(rotation_gate("rz", theta, [q]));
    }

    fn rzz(&mut self, theta: f64, q0: usize, q1: usize) {
        let q0 = self.map(q0);
        let q1 = self.map(q1);
        self.push_gate(rotation_gate("rzz", theta, [q0, q1]));
    }

    fn sadj(&mut self, q: usize) {
        let q = self.map(q);
        self.push_gate(adjoint_gate("S", [q]));
    }

    fn s(&mut self, q: usize) {
        let q = self.map(q);
        self.push_gate(gate("S", [q]));
    }

    fn swap(&mut self, q0: usize, q1: usize) {
        let q0 = self.map(q0);
        let q1 = self.map(q1);
        self.push_gate(gate("SWAP", [q0, q1]));
    }

    fn tadj(&mut self, q: usize) {
        let q = self.map(q);
        self.push_gate(adjoint_gate("T", [q]));
    }

    fn t(&mut self, q: usize) {
        let q = self.map(q);
        self.push_gate(gate("T", [q]));
    }

    fn x(&mut self, q: usize) {
        let q = self.map(q);
        self.push_gate(gate("X", [q]));
    }

    fn y(&mut self, q: usize) {
        let q = self.map(q);
        self.push_gate(gate("Y", [q]));
    }

    fn z(&mut self, q: usize) {
        let q = self.map(q);
        self.push_gate(gate("Z", [q]));
    }

    fn qubit_allocate(&mut self) -> usize {
        self.remapper.qubit_allocate()
    }

    fn qubit_release(&mut self, q: usize) {
        self.remapper.qubit_release(q);
    }

    fn qubit_swap_id(&mut self, q0: usize, q1: usize) {
        self.remapper.swap(q0, q1);
    }

    fn capture_quantum_state(&mut self) -> (Vec<(BigUint, Complex<f64>)>, usize) {
        (Vec::new(), 0)
    }

    fn qubit_is_zero(&mut self, _q: usize) -> bool {
        // Because `qubit_is_zero` is called on every qubit release, this must return
        // true to avoid a panic.
        true
    }

    fn custom_intrinsic(&mut self, name: &str, arg: Value) -> Option<Result<Value, String>> {
        // The qubit arguments are treated as the targets for custom gates.
        // Any remaining arguments will be kept in the display_args field
        // to be shown as part of the gate label when the circuit is rendered.
        let (qubit_args, classical_args) = self.split_qubit_args(arg);

        self.push_gate(custom_gate(
            name,
            &qubit_args,
            if classical_args.is_empty() {
                None
            } else {
                Some(classical_args)
            },
        ));

        match name {
            // Special case this known intrinsic to match the simulator
            // behavior, so that our samples will work
            "BeginEstimateCaching" => Some(Ok(Value::Bool(true))),
            _ => Some(Ok(Value::unit())),
        }
    }
}

impl Builder {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Builder {
            circuit: Circuit::default(),
            config,
            remapper: Remapper::default(),
        }
    }

    #[must_use]
    pub fn snapshot(&self) -> Circuit {
        let circuit = self.circuit.clone();
        self.finish_circuit(circuit)
    }

    #[must_use]
    pub fn finish(mut self) -> Circuit {
        let circuit = take(&mut self.circuit);
        self.finish_circuit(circuit)
    }

    fn map(&mut self, qubit: usize) -> WireId {
        self.remapper.map(qubit)
    }

    fn push_gate(&mut self, gate: Operation) {
        self.circuit.operations.push(gate);
    }

    fn num_measurements_by_qubit(&self) -> IndexMap<usize, usize> {
        self.remapper.qubit_measurement_counts.iter().fold(
            IndexMap::default(),
            |mut map: IndexMap<usize, usize>, (q, _)| {
                match map.get_mut(q.0) {
                    Some(rs) => *rs += 1,
                    None => {
                        map.insert(q.0, 1);
                    }
                }
                map
            },
        )
    }

    fn num_measurements_for_qubit(&self, qubit: WireId) -> usize {
        self.remapper
            .qubit_measurement_counts
            .get(qubit)
            .copied()
            .unwrap_or_default()
    }

    fn finish_circuit(&self, mut circuit: Circuit) -> Circuit {
        let by_qubit = self.num_measurements_by_qubit();

        // add deferred measurements
        if self.config.base_profile {
            for (qubit, _) in &by_qubit {
                // guaranteed one measurement per qubit, so result is always 0
                circuit.operations.push(measurement_gate(qubit, 0));
            }
        }

        // add qubit declarations
        for i in 0..self.remapper.num_qubits() {
            let num_measurements = by_qubit.get(i).map_or(0, |c| *c);
            circuit.qubits.push(crate::circuit::Qubit {
                id: i,
                num_children: num_measurements,
            });
        }

        circuit
    }

    /// Splits the qubit arguments from classical arguments so that the qubits
    /// can be treated as the targets for custom gates.
    /// The classical arguments get formatted into a comma-separated list.
    fn split_qubit_args(&mut self, arg: Value) -> (Vec<WireId>, String) {
        let arg = if let Value::Tuple(vals) = arg {
            vals
        } else {
            // Single arguments are not passed as tuples, wrap in an array
            Rc::new([arg])
        };
        let mut qubits = vec![];
        let mut classical_args = String::new();
        self.push_vals(&arg, &mut qubits, &mut classical_args);
        (qubits, classical_args)
    }

    /// Pushes all qubit values into `qubits`, and formats all classical values into `classical_args`.
    fn push_val(&mut self, arg: &Value, qubits: &mut Vec<WireId>, classical_args: &mut String) {
        match arg {
            Value::Array(vals) => {
                self.push_list::<'[', ']'>(vals, qubits, classical_args);
            }
            Value::Tuple(vals) => {
                self.push_list::<'(', ')'>(vals, qubits, classical_args);
            }
            Value::Qubit(q) => {
                qubits.push(self.map(q.0));
            }
            v => {
                let _ = write!(classical_args, "{v}");
            }
        }
        qubits.sort_unstable_by_key(|q| q.0);
        qubits.dedup_by_key(|q| q.0);
    }

    /// Pushes all qubit values into `qubits`, and formats all
    /// classical values into `classical_args` as a list.
    fn push_list<const OPEN: char, const CLOSE: char>(
        &mut self,
        vals: &[Value],
        qubits: &mut Vec<WireId>,
        classical_args: &mut String,
    ) {
        classical_args.push(OPEN);
        let start = classical_args.len();
        self.push_vals(vals, qubits, classical_args);
        if classical_args.len() > start {
            classical_args.push(CLOSE);
        } else {
            classical_args.pop();
        }
    }

    /// Pushes all qubit values into `qubits`, and formats all
    /// classical values into `classical_args` as comma-separated values.
    fn push_vals(&mut self, vals: &[Value], qubits: &mut Vec<WireId>, classical_args: &mut String) {
        let mut any = false;
        for v in vals.iter() {
            let start = classical_args.len();
            self.push_val(v, qubits, classical_args);
            if classical_args.len() > start {
                any = true;
                classical_args.push_str(", ");
            }
        }
        if any {
            // remove trailing comma
            classical_args.pop();
            classical_args.pop();
        }
    }
}

/// Provides support for qubit id allocation, measurement and
/// reset operations for Base Profile targets.
///
/// Since qubit reuse is disallowed, a mapping is maintained
/// from allocated qubit ids to hardware qubit ids. Each time
/// a qubit is reset, it is remapped to a fresh hardware qubit.
///
/// Note that even though qubit reset & reuse is disallowed,
/// qubit ids are still reused for new allocations.
/// Measurements are tracked and deferred.
#[derive(Default)]
struct Remapper {
    next_meas_id: usize,
    next_qubit_id: usize,
    next_qubit_wire_id: WireId,
    qubit_map: IndexMap<usize, WireId>,
    qubit_measurement_counts: IndexMap<WireId, usize>,
}

impl Remapper {
    fn map(&mut self, qubit: usize) -> WireId {
        if let Some(mapped) = self.qubit_map.get(qubit) {
            *mapped
        } else {
            let mapped = self.next_qubit_wire_id;
            self.next_qubit_wire_id.0 += 1;
            self.qubit_map.insert(qubit, mapped);
            mapped
        }
    }

    fn m(&mut self, q: usize) -> usize {
        let mapped_q = self.map(q);
        let id = self.get_meas_id();
        match self.qubit_measurement_counts.get_mut(mapped_q) {
            Some(count) => *count += 1,
            None => {
                self.qubit_measurement_counts.insert(mapped_q, 1);
            }
        }
        id
    }

    fn mreset(&mut self, q: usize) -> usize {
        let id = self.m(q);
        self.reset(q);
        id
    }

    fn reset(&mut self, q: usize) {
        self.qubit_map.remove(q);
    }

    fn qubit_allocate(&mut self) -> usize {
        let id = self.next_qubit_id;
        self.next_qubit_id += 1;
        let _ = self.map(id);
        id
    }

    fn qubit_release(&mut self, _q: usize) {
        self.next_qubit_id -= 1;
    }

    fn swap(&mut self, q0: usize, q1: usize) {
        let q0_mapped = self.map(q0);
        let q1_mapped = self.map(q1);
        self.qubit_map.insert(q0, q1_mapped);
        self.qubit_map.insert(q1, q0_mapped);
    }

    #[must_use]
    fn num_qubits(&self) -> usize {
        self.next_qubit_wire_id.0
    }

    #[must_use]
    fn get_meas_id(&mut self) -> usize {
        let id = self.next_meas_id;
        self.next_meas_id += 1;
        id
    }
}

#[derive(Copy, Clone, Default)]
struct WireId(pub usize);

impl From<usize> for WireId {
    fn from(id: usize) -> Self {
        WireId(id)
    }
}

impl From<WireId> for usize {
    fn from(id: WireId) -> Self {
        id.0
    }
}

#[allow(clippy::unicode_not_nfc)]
static KET_ZERO: &str = "|0〉";

fn gate<const N: usize>(name: &str, targets: [WireId; N]) -> Operation {
    Operation {
        gate: name.into(),
        display_args: None,
        is_controlled: false,
        is_adjoint: false,
        is_measurement: false,
        controls: vec![],
        targets: targets.iter().map(|q| Register::quantum(q.0)).collect(),
        children: vec![],
    }
}

fn adjoint_gate<const N: usize>(name: &str, targets: [WireId; N]) -> Operation {
    Operation {
        gate: name.into(),
        display_args: None,
        is_controlled: false,
        is_adjoint: true,
        is_measurement: false,
        controls: vec![],
        targets: targets.iter().map(|q| Register::quantum(q.0)).collect(),
        children: vec![],
    }
}

fn controlled_gate<const M: usize, const N: usize>(
    name: &str,
    controls: [WireId; M],
    targets: [WireId; N],
) -> Operation {
    Operation {
        gate: name.into(),
        display_args: None,
        is_controlled: true,
        is_adjoint: false,
        is_measurement: false,
        controls: controls.iter().map(|q| Register::quantum(q.0)).collect(),
        targets: targets.iter().map(|q| Register::quantum(q.0)).collect(),
        children: vec![],
    }
}

fn measurement_gate(qubit: usize, result: usize) -> Operation {
    Operation {
        gate: "Measure".into(),
        display_args: None,
        is_controlled: false,
        is_adjoint: false,
        is_measurement: true,
        controls: vec![Register::quantum(qubit)],
        targets: vec![Register::classical(qubit, result)],
        children: vec![],
    }
}

fn rotation_gate<const N: usize>(name: &str, theta: f64, targets: [WireId; N]) -> Operation {
    Operation {
        gate: name.into(),
        display_args: Some(format!("{theta:.4}")),
        is_controlled: false,
        is_adjoint: false,
        is_measurement: false,
        controls: vec![],
        targets: targets.iter().map(|q| Register::quantum(q.0)).collect(),
        children: vec![],
    }
}

fn custom_gate(name: &str, targets: &[WireId], display_args: Option<String>) -> Operation {
    Operation {
        gate: name.into(),
        display_args,
        is_controlled: false,
        is_adjoint: false,
        is_measurement: false,
        controls: vec![],
        targets: targets.iter().map(|q| Register::quantum(q.0)).collect(),
        children: vec![],
    }
}
