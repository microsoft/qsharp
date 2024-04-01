// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{fs::read_to_string, path::Path};

use num_bigint::BigUint;
use num_complex::Complex;
use num_traits::ToPrimitive;
use qsc::{
    interpret::{GenericReceiver, Interpreter},
    Backend, LanguageFeatures, RuntimeCapabilityFlags, SourceMap,
};
use resource_estimator::estimates::{ErrorBudget, Overhead};

#[allow(clippy::struct_field_names)]
#[derive(Clone, Default)]
pub struct LogicalCounts {
    pub(crate) qubit_count: u64,
    pub(crate) cx_count: u64,
    pub(crate) ccx_count: u64,

    free_list: Vec<usize>,
}

impl LogicalCounts {
    #[allow(clippy::similar_names)]
    pub fn new(qubit_count: u64, cx_count: u64, ccx_count: u64) -> Self {
        Self {
            qubit_count,
            cx_count,
            ccx_count,
            free_list: vec![],
        }
    }

    #[allow(clippy::similar_names)]
    #[must_use]
    pub fn from_elliptic_curve_crypto(bit_size: u64, window_size: u64) -> Self {
        let qubit_count = 9 * bit_size + window_size + 4;
        let cx_count = (448 * bit_size.pow(3)).div_ceil(window_size);
        let ccx_count = (348 * bit_size.pow(3)).div_ceil(window_size);

        Self::new(qubit_count, cx_count, ccx_count)
    }

    pub fn from_qsharp(filename: impl AsRef<Path>) -> Result<Self, String> {
        let content = read_to_string(filename).map_err(|_| String::from("Cannot read filename"))?;

        let sources = SourceMap::new([("source".into(), content.into())], None);

        let mut interpreter = Interpreter::new(
            true,
            sources,
            qsc::PackageType::Exe,
            RuntimeCapabilityFlags::all(),
            LanguageFeatures::default(),
        )
        .map_err(|_| String::from("Cannot create interpreter"))?;

        let mut counter = Self::default();
        let mut stdout = std::io::stdout();
        let mut out = GenericReceiver::new(&mut stdout);

        interpreter
            .eval_entry_with_sim(&mut counter, &mut out)
            .map_err(|_| String::from("Cannot estimate Q# code"))?;

        Ok(counter)
    }
}

impl Overhead for LogicalCounts {
    fn logical_qubits(&self) -> u64 {
        let horizontal_routing_qubits = self.qubit_count.div_ceil(2) + 1;

        self.qubit_count + horizontal_routing_qubits
    }

    #[allow(clippy::similar_names)]
    fn logical_depth(&self, _: &ErrorBudget) -> u64 {
        let cx_f = self.cx_count.to_f64().expect("#CX is convertible to f64");
        let ccx_f = self.ccx_count.to_f64().expect("#CCX is convertible to f64");

        // arXiv:2302.06639 (p. 30, Fig. 27); measurement is countes as 0.2
        // cycles according to open source code
        let cx_cycles = 2.2;

        // arXiv:2302.06639 (p. 36, Fig. 33); the cost is approximates as 3
        // CNOT (3 * 2.2), then 1.5 CNOT subject to measurement outcome (1.5
        // * 2.2), and measurement (0.2)
        let ccx_cycles = 10.1;

        ((cx_f * cx_cycles) + (ccx_f * ccx_cycles))
            .ceil()
            .to_u64()
            .expect("logical depth is not too large")
    }

    fn num_magic_states(&self, _: &ErrorBudget, _: usize) -> u64 {
        self.ccx_count
    }
}

impl Backend for LogicalCounts {
    type ResultType = bool;

    fn ccx(&mut self, _ctl0: usize, _ctl1: usize, _q: usize) {
        self.ccx_count += 1;
    }

    fn cx(&mut self, _ctl: usize, _q: usize) {
        self.cx_count += 1;
    }

    fn cy(&mut self, _ctl: usize, _q: usize) {
        self.cx_count += 1;
    }

    fn cz(&mut self, _ctl: usize, _q: usize) {
        self.cx_count += 1;
    }

    fn h(&mut self, _q: usize) {}

    fn m(&mut self, _q: usize) -> Self::ResultType {
        false
    }

    fn mresetz(&mut self, _q: usize) -> Self::ResultType {
        false
    }

    fn reset(&mut self, _q: usize) {}

    fn sadj(&mut self, _q: usize) {}

    fn s(&mut self, _q: usize) {}

    fn swap(&mut self, _q0: usize, _q1: usize) {
        self.cx_count += 3;
    }

    fn x(&mut self, _q: usize) {}

    fn y(&mut self, _q: usize) {}

    fn z(&mut self, _q: usize) {}

    fn qubit_allocate(&mut self) -> usize {
        if let Some(qubit) = self.free_list.pop() {
            qubit
        } else {
            let qubit = self.qubit_count;
            self.qubit_count += 1;
            qubit.to_usize().expect("qubit is not too large")
        }
    }

    fn qubit_release(&mut self, q: usize) {
        self.free_list.push(q);
    }

    fn capture_quantum_state(&mut self) -> (Vec<(BigUint, Complex<f64>)>, usize) {
        (vec![], 0)
    }

    fn qubit_is_zero(&mut self, _q: usize) -> bool {
        true
    }
}
