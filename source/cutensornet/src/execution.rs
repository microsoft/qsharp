//! Runs Base programs with the ④ MPS method, for the Python API; `Sample`
//! uses the batch strategy.
//!
//! Every query starts from one MPS, ψ, the state before the terminal
//! measurements:
//!
//! ```text
//! Base program ─1─► Circuit     ③ walk on the host; rejects feedforward, reset, a 2nd region
//!              ─2─► one MPS ψ   GPU (MpsSession)
//!                     ├─ Expectation ─► read from ψ;  Cost ─► report the call's resources
//!                     └─ Sample ─► draw N rows from ψ
//!                                  ─3─► replay each row through ③ (shot results)
//! ```
//!
//! - The batch strategy is how `Sample` works: one state gives every shot.
//! - Base only applies to every query: step 1 rejects any Adaptive feature.
//!   `Expectation` stays Base-only by design, as in CUDA-Q's `observe`.
//!   Adaptive `Sample` is M1's `mps-a`, with the incremental strategy (below).
//!
//! ⑤ calls this entry ([`run_mps_shots`] for `Sample`,
//! [`evaluate_state_queries`] for `Expectation` and `Cost`), and the entry
//! calls ③ as a library, twice; the GPU work happens in between, outside ③:
//!
//! 1. ③ walks the program with `CircuitPreparationConsumer`, on the host,
//!    once. The walk records the program's gates into a `Circuit` and rejects a
//!    second region, feedforward and reset, before device discovery. A failure
//!    there is a program error.
//! 2. One `MpsSession` evolves ψ, and then samples every shot from it or
//!    evaluates the queries on it. Discovery, session and native failures are
//!    environment errors
//!    ([`MpsExecutionError::is_environment_error`]). The session is closed on
//!    every path, and a cleanup failure is reported beside the execution's.
//! 3. `Sample` only: ③ walks the program again per shot, with
//!    `CuTensorNetMpsConsumer`, which answers each measurement from that
//!    shot's row, so shot results and output records come from the execution
//!    core, not from this crate.
//!
//! Neither consumer touches the GPU, so ③ does not drive cuTensorNet. The
//! incremental strategy for Adaptive programs (M1's `mps-a`) changes that:
//! shots run one at a time, and step 3's consumer holds the live MPS. It
//! applies each region to the MPS on the GPU, and for each measurement
//! computes P(r | earlier outcomes) from the state, draws r from the shot's
//! random stream, projects the MPS onto r and returns r, which ③ feeds
//! forward; a reset is a measurement followed by X on outcome 1. Steps 1 and 3
//! then become one walk, and step 1 stops rejecting feedforward and reset for
//! `Sample`. The session is reused across shots, one execution at a time.
//!
//! The functions are `#[doc(hidden)]`: their only caller is ⑤'s native module,
//! which must change with them.
//!
//! Adding a state method or query: (1) add the variant to [`StateMethod`], or
//! to [`StateQuery`] and [`StateQueryValue`]; (2) validate it in
//! [`evaluate_state_queries`] on the host, before device discovery, so bad
//! input is a program error (`ValueError` in Python), not an environment
//! error; (3) implement it on `MpsSession`, closing the execution on every
//! path as the existing queries do; (4) encode it in ⑤'s `_tensor_network.py`
//! and `tensor_network_state.rs`. Two routes are already implemented below
//! this entry and kept test-only: exact contraction of the state
//! (`MpsSession::evaluate_observables` with `mps = false`) and the probability
//! of given outcomes (`MpsSession::evaluate_probability`, with a
//! `ConditionedCircuit` converted by `ProjectedCircuit::from_conditioned_circuit`).
//! Exposing either takes steps 1, 2 and 4, and lifting its `#[cfg(test)]`.

#[cfg(any(test, all(target_os = "linux", target_arch = "x86_64")))]
use crate::simulation::ExecutionPolicy;
use crate::{
    AvailabilityError, discover,
    simulation::{
        Circuit, CuTensorNetMpsConsumerError, CuTensorNetSampleMatrix, Gate, SamplingRequest,
        SimulationError, StateQueryResult, UnitaryOperationConversionError,
    },
};
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
use crate::{library::MpsSession, simulation::collect_sampled_shots};
use num_complex::Complex64;
use qdk_simulators::{
    MeasurementResult, OutputRecord, QubitID,
    execution::{
        MeasurementRequest, PauliSum, PreparedAdaptiveProgram, QuantumEvolutionRegion,
        RegionConsumer, drive_prepared_shot,
    },
};
use rand::{RngExt, SeedableRng, rngs::StdRng};
use std::fmt;
use thiserror::Error;

const SAMPLER_HYPER_SAMPLES: i32 = 8;

#[derive(Debug, Error)]
#[error("{message}")]
pub struct MpsExecutionError {
    message: String,
    environment: bool,
}

impl MpsExecutionError {
    #[must_use]
    pub const fn is_environment_error(&self) -> bool {
        self.environment
    }

    fn program(error: impl fmt::Display) -> Self {
        Self {
            message: error.to_string(),
            environment: false,
        }
    }

    fn environment(error: impl fmt::Display) -> Self {
        Self {
            message: error.to_string(),
            environment: true,
        }
    }
}

#[cfg_attr(
    not(all(target_os = "linux", target_arch = "x86_64")),
    allow(
        dead_code,
        reason = "host-independent preflight prepares native inputs before reporting an unsupported target"
    )
)]
#[derive(Debug)]
struct PreparedMpsRun {
    circuit: Circuit,
    sampled_qubits: Box<[QubitID]>,
    shot_count: usize,
    sampling_request: SamplingRequest,
    max_bond_dimension: Option<u32>,
}

#[derive(Debug, Error)]
enum CircuitPreparationError {
    #[error(transparent)]
    Consumer(#[from] CuTensorNetMpsConsumerError),

    #[error(transparent)]
    Conversion(#[from] UnitaryOperationConversionError),

    #[error(transparent)]
    Circuit(#[from] SimulationError),
}

struct CircuitPreparationConsumer {
    circuit: Circuit,
    last_measured_qubit: Option<QubitID>,
}

impl CircuitPreparationConsumer {
    fn new(qubit_count: u32) -> Result<Self, CircuitPreparationError> {
        Ok(Self {
            circuit: Circuit::new(qubit_count)?,
            last_measured_qubit: None,
        })
    }
}

impl RegionConsumer for CircuitPreparationConsumer {
    type PreparedRegion<'region> = &'region QuantumEvolutionRegion;
    type RegionReport = ();
    type ExecutionReport = ();
    type Error = CircuitPreparationError;

    fn prepare_region<'region>(
        &mut self,
        region: &'region QuantumEvolutionRegion,
    ) -> Result<Self::PreparedRegion<'region>, Self::Error> {
        for &operation in region.operations() {
            if let Some(gate) = Gate::from_unitary_operation(operation)? {
                self.circuit.push(gate)?;
            }
        }
        Ok(region)
    }

    fn execute_region(
        &mut self,
        _region: Self::PreparedRegion<'_>,
    ) -> Result<Self::RegionReport, Self::Error> {
        if let Some(qubit) = self.last_measured_qubit {
            return Err(CuTensorNetMpsConsumerError::UnsupportedFeedforward { qubit }.into());
        }
        Ok(())
    }

    fn measure(&mut self, request: MeasurementRequest) -> Result<MeasurementResult, Self::Error> {
        self.last_measured_qubit = Some(request.qubit);
        Ok(MeasurementResult::Zero)
    }

    fn reset(&mut self, qubit: QubitID) -> Result<(), Self::Error> {
        Err(CuTensorNetMpsConsumerError::UnsupportedReset { qubit }.into())
    }

    fn finish_execution(&mut self) -> Result<Self::ExecutionReport, Self::Error> {
        Ok(())
    }

    fn close(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Executes a prepared Base-profile program through one cuTensorNet MPS session.
///
/// `max_bond_dimension` caps the MPS bonds at χ; `None` keeps the backend
/// default, as for [`StateMethod::Mps`].
///
/// This is an internal cross-crate entrypoint for the Python native module.
#[doc(hidden)]
pub fn run_mps_shots(
    prepared_program: &PreparedAdaptiveProgram<u64>,
    shots: u32,
    seed: Option<u32>,
    max_bond_dimension: Option<u32>,
) -> Result<Vec<Vec<OutputRecord>>, MpsExecutionError> {
    let prepared_run = prepare_mps_run(prepared_program, shots, seed, max_bond_dimension)?;
    execute_mps_run(prepared_program, &prepared_run)
}

fn prepare_mps_run(
    prepared_program: &PreparedAdaptiveProgram<u64>,
    shots: u32,
    seed: Option<u32>,
    max_bond_dimension: Option<u32>,
) -> Result<PreparedMpsRun, MpsExecutionError> {
    let circuit = prepare_circuit(prepared_program)?;
    let measured_qubits = prepared_program.measured_qubits().map_err(|error| {
        MpsExecutionError::program(CuTensorNetMpsConsumerError::InvalidMeasurementMetadata {
            error,
        })
    })?;
    let sampled_qubits = CuTensorNetSampleMatrix::sampled_qubits(measured_qubits);
    let shot_count =
        usize::try_from(shots).map_err(|error| MpsExecutionError::program(error.to_string()))?;
    let derived_seed = derive_sampler_seed(seed);
    let sampling_request = SamplingRequest::new(
        shot_count,
        SAMPLER_HYPER_SAMPLES,
        Some(derived_seed),
        derived_seed,
    )
    .map_err(MpsExecutionError::program)?;

    Ok(PreparedMpsRun {
        circuit,
        sampled_qubits,
        shot_count,
        sampling_request,
        max_bond_dimension,
    })
}

/// Resolves the program's single unitary region into a circuit, rejecting
/// more regions, feedforward and reset on the host, before device discovery.
fn prepare_circuit(
    prepared_program: &PreparedAdaptiveProgram<u64>,
) -> Result<Circuit, MpsExecutionError> {
    let region_count = prepared_program.regions().len();
    if region_count > 1 {
        return Err(MpsExecutionError::program(
            CuTensorNetMpsConsumerError::UnsupportedRegionCount {
                actual: region_count,
            },
        ));
    }

    let mut consumer = CircuitPreparationConsumer::new(prepared_program.program().num_qubits)
        .map_err(MpsExecutionError::program)?;
    drive_prepared_shot(prepared_program, &mut consumer).map_err(MpsExecutionError::program)?;
    if region_count != 1 {
        return Err(MpsExecutionError::program(
            CuTensorNetMpsConsumerError::UnsupportedRegionCount {
                actual: region_count,
            },
        ));
    }
    Ok(consumer.circuit)
}

/// How a state query represents ψ.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateMethod {
    /// ψ is a matrix product state with bonds capped at χ; `None` keeps the
    /// backend default.
    Mps { max_bond_dimension: Option<u32> },
}

/// One query on a cuTensorNet state.
///
/// A call reads exactly one state, ψ = U|0…0⟩, the state before the terminal
/// measurements, so [`StateQuery::Cost`] always describes the one MPS it
/// computed.
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq)]
pub enum StateQuery {
    /// `⟨ψ|O|ψ⟩ / ⟨ψ|ψ⟩`.
    Expectation(PauliSum),
    /// The resources of the one MPS this call computed.
    Cost,
}

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq)]
pub enum StateQueryValue {
    Expectation(Complex64),
    Cost(MpsCost),
}

/// Resources of one MPS evaluation.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MpsCost {
    /// The largest bond the truncated state reached, at most χ.
    pub max_bond_dimension: usize,
    /// Device bytes of the realized site tensors.
    pub state_bytes: usize,
    /// The largest device scratch any step of this evaluation requested:
    /// computing the state, or any expectation evaluated with it.
    pub workspace_bytes: usize,
}

/// Evaluates `queries` through one cuTensorNet session, on ψ, the state a
/// prepared Base-profile program reaches before its terminal measurements.
///
/// Program errors are reported before device discovery: no query, more than
/// one region, feedforward, reset or a query on an absent qubit.
///
/// This is an internal cross-crate entrypoint for the Python native module.
#[doc(hidden)]
pub fn evaluate_state_queries(
    prepared_program: &PreparedAdaptiveProgram<u64>,
    queries: &[StateQuery],
    method: StateMethod,
) -> Result<Vec<StateQueryValue>, MpsExecutionError> {
    if queries.is_empty() {
        return Err(MpsExecutionError::program(
            "queries must contain at least one query",
        ));
    }
    let circuit = prepare_circuit(prepared_program)?;
    let qubit_count = circuit.qubit_count() as QubitID;
    for query in queries {
        if let StateQuery::Expectation(observable) = query
            && let Some(qubit) = observable.max_qubit()
            && qubit >= qubit_count
        {
            return Err(MpsExecutionError::program(format_args!(
                "Expectation acts on qubit {qubit}, but the program has {qubit_count} qubits"
            )));
        }
    }
    let observables = queries
        .iter()
        .filter_map(|query| match query {
            StateQuery::Expectation(observable) => Some(observable.clone()),
            StateQuery::Cost => None,
        })
        .collect::<Vec<_>>();
    let result = evaluate_observables(&circuit, &observables, method)?;
    let mut expectations = result.expectations.into_iter();
    queries
        .iter()
        .map(|query| match query {
            StateQuery::Expectation(_) => Ok(StateQueryValue::Expectation(
                expectations
                    .next()
                    .expect("one expectation per Expectation query"),
            )),
            StateQuery::Cost => result
                .cost
                .map(StateQueryValue::Cost)
                .ok_or_else(|| MpsExecutionError::program("an MPS evaluation reported no Cost")),
        })
        .collect()
}

#[cfg(any(test, all(target_os = "linux", target_arch = "x86_64")))]
fn mps_policy(max_bond_dimension: Option<u32>) -> ExecutionPolicy {
    let base = ExecutionPolicy::base_qualification();
    max_bond_dimension.map_or(base, |chi| base.with_bond_cap(chi.into()))
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn evaluate_observables(
    circuit: &Circuit,
    observables: &[PauliSum],
    method: StateMethod,
) -> Result<StateQueryResult, MpsExecutionError> {
    let StateMethod::Mps { max_bond_dimension } = method;
    let availability = discover().map_err(MpsExecutionError::environment)?;
    let mut session = MpsSession::new(availability.libraries, mps_policy(max_bond_dimension))
        .map_err(MpsExecutionError::environment)?;
    let execution = session
        .evaluate_observables(circuit, observables, true)
        .map_err(MpsExecutionError::environment);
    combine_execution_and_session_cleanup(execution, session.close())
}

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
fn evaluate_observables(
    _circuit: &Circuit,
    _observables: &[PauliSum],
    _method: StateMethod,
) -> Result<StateQueryResult, MpsExecutionError> {
    let error = discover().expect_err("cuTensorNet discovery is unsupported on this target");
    Err(MpsExecutionError::environment(error))
}

fn derive_sampler_seed(seed: Option<u32>) -> i32 {
    let mut rng = if let Some(seed) = seed {
        StdRng::seed_from_u64(seed.into())
    } else {
        StdRng::from_rng(&mut rand::rng())
    };
    rng.random_range(1..=i32::MAX)
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn execute_mps_run(
    prepared_program: &PreparedAdaptiveProgram<u64>,
    prepared_run: &PreparedMpsRun,
) -> Result<Vec<Vec<OutputRecord>>, MpsExecutionError> {
    let availability = discover().map_err(MpsExecutionError::environment)?;
    let mut session = MpsSession::new(
        availability.libraries,
        mps_policy(prepared_run.max_bond_dimension),
    )
    .map_err(MpsExecutionError::environment)?;
    let execution = session
        .sample(
            &prepared_run.circuit,
            &prepared_run.sampled_qubits,
            prepared_run.sampling_request,
        )
        .map_err(MpsExecutionError::environment)
        .and_then(|samples| {
            collect_sampled_shots(prepared_program, prepared_run.shot_count, samples)
                .map_err(MpsExecutionError::program)
        });
    combine_execution_and_session_cleanup(execution, session.close())
}

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
fn execute_mps_run(
    _prepared_program: &PreparedAdaptiveProgram<u64>,
    prepared_run: &PreparedMpsRun,
) -> Result<Vec<Vec<OutputRecord>>, MpsExecutionError> {
    let _ = prepared_run;
    let error = discover().expect_err("cuTensorNet discovery is unsupported on this target");
    Err(MpsExecutionError::environment(error))
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn combine_execution_and_session_cleanup<T>(
    execution: Result<T, MpsExecutionError>,
    cleanup: Result<(), SimulationError>,
) -> Result<T, MpsExecutionError> {
    match (execution, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(cleanup)) => Err(MpsExecutionError::environment(cleanup)),
        (Err(execution), Err(cleanup)) => Err(MpsExecutionError::environment(format_args!(
            "execution failed ({execution}); cleanup also failed ({cleanup})"
        ))),
    }
}

impl From<AvailabilityError> for MpsExecutionError {
    fn from(error: AvailabilityError) -> Self {
        Self::environment(error)
    }
}

#[cfg(test)]
mod tests {
    use super::{StateMethod, StateQuery, evaluate_state_queries, mps_policy, prepare_mps_run};
    use crate::simulation::ExecutionPolicy;
    use num_complex::Complex64;
    use qdk_simulators::{
        bytecode::{AdaptiveProgram, Block, Instruction, Op},
        execution::{PauliSum, PreparedAdaptiveProgram},
    };

    const IMMEDIATE_AUX1: u64 = 1 << 20;
    const IMMEDIATE_AUX2: u64 = 1 << 21;

    fn operation(operation_id: u64) -> Op<u64> {
        Op {
            op_id: operation_id,
            q1: 0,
            q2: 0,
            q3: 0,
            angle: 0,
        }
    }

    fn prepared_program(
        instructions: Vec<Instruction<u64>>,
        quantum_ops: Vec<Op<u64>>,
    ) -> PreparedAdaptiveProgram<u64> {
        PreparedAdaptiveProgram::new(AdaptiveProgram {
            num_qubits: 2,
            num_results: 1,
            num_registers: 0,
            entry_block: 0,
            block_table: vec![Block {
                instr_offset: 0,
                instr_count: instructions.len() as u64,
            }],
            instructions,
            function_table: Vec::new(),
            phi_entries: Vec::new(),
            switch_cases: Vec::new(),
            call_args: Vec::new(),
            constant_data: Vec::new(),
            quantum_ops,
        })
        .expect("test program should prepare")
    }

    fn gate(target: u64) -> Instruction<u64> {
        Instruction {
            opcode: 0x10 | (1 << 16) | IMMEDIATE_AUX1 | IMMEDIATE_AUX2,
            aux1: target,
            ..Instruction::default()
        }
    }

    fn measure(qubit: u64) -> Instruction<u64> {
        Instruction {
            opcode: 0x11 | IMMEDIATE_AUX1 | IMMEDIATE_AUX2,
            aux0: 1,
            aux1: qubit,
            aux2: 0,
            ..Instruction::default()
        }
    }

    fn reset(qubit: u64) -> Instruction<u64> {
        Instruction {
            opcode: 0x12 | IMMEDIATE_AUX1,
            aux0: 2,
            aux1: qubit,
            ..Instruction::default()
        }
    }

    fn ret() -> Instruction<u64> {
        Instruction {
            opcode: 0x02,
            ..Instruction::default()
        }
    }

    #[test]
    fn state_queries_reject_program_errors_before_device_discovery() {
        let program = prepared_program(
            vec![gate(0), measure(0), ret()],
            vec![operation(5), operation(21)],
        );
        let mut absent_qubit = PauliSum::new();
        absent_qubit
            .push_labels(Complex64::new(1.0, 0.0), "Z", &[2])
            .expect("the term is well formed");
        let mut present_qubit = PauliSum::new();
        present_qubit
            .push_labels(Complex64::new(1.0, 0.0), "Z", &[1])
            .expect("the term is well formed");
        let mps = StateMethod::Mps {
            max_bond_dimension: None,
        };
        let cases = [
            (Vec::new(), mps, "queries must contain at least one query"),
            (
                vec![
                    StateQuery::Expectation(present_qubit),
                    StateQuery::Expectation(absent_qubit),
                ],
                mps,
                "Expectation acts on qubit 2, but the program has 2 qubits",
            ),
        ];

        for (queries, method, message) in cases {
            let error = evaluate_state_queries(&program, &queries, method)
                .expect_err("the program error should be rejected");
            assert_eq!(error.to_string(), message);
            assert!(!error.is_environment_error());
        }
    }

    #[test]
    fn state_queries_share_the_circuit_preflight() {
        let program = prepared_program(
            vec![measure(0), gate(1), ret()],
            vec![operation(5), operation(21)],
        );

        let error = evaluate_state_queries(
            &program,
            &[StateQuery::Cost],
            StateMethod::Mps {
                max_bond_dimension: Some(4),
            },
        )
        .expect_err("feedforward should be rejected");

        assert!(error.to_string().contains("after measuring qubit 0"));
        assert!(!error.is_environment_error());
    }

    #[test]
    fn preflight_rejects_multiple_regions_before_device_discovery() {
        let program = prepared_program(
            vec![gate(0), measure(0), gate(1), ret()],
            vec![operation(5), operation(21)],
        );

        let error = prepare_mps_run(&program, 1, Some(42), None)
            .expect_err("multiple regions should be rejected");

        assert_eq!(
            error.to_string(),
            "cuTensorNet batch sampling requires exactly one quantum evolution region, found 2"
        );
        assert!(!error.is_environment_error());
    }

    #[test]
    fn preflight_rejects_feedforward_before_device_discovery() {
        let program = prepared_program(
            vec![measure(0), gate(1), ret()],
            vec![operation(5), operation(21)],
        );

        let error = prepare_mps_run(&program, 1, Some(42), None)
            .expect_err("feedforward should be rejected");

        assert!(
            error.to_string().contains(
                "cuTensorNet batch sampling cannot execute a quantum evolution region after measuring qubit 0"
            )
        );
        assert!(!error.is_environment_error());
    }

    #[test]
    fn preflight_accepts_every_unitary_operation() {
        // I, X, Y, Z, H, S, S†, T, T†, SX, SX†, Rx, Ry, Rz, CX, CZ, Rxx, Ryy,
        // Rzz, SWAP and CY, on qubits (0, 1) where they take two.
        for operation_id in (0..=19).filter(|&id| id != 1).chain([24, 29]) {
            let two_qubit_gate = Instruction { aux2: 1, ..gate(0) };
            let program = prepared_program(
                vec![two_qubit_gate, measure(0), ret()],
                vec![operation(operation_id), operation(21)],
            );

            prepare_mps_run(&program, 1, Some(42), None).unwrap_or_else(|error| {
                panic!("operation {operation_id} should pass the preflight: {error}")
            });
        }
    }

    #[test]
    fn max_bond_dimension_reaches_the_mps_policy() {
        let program = prepared_program(
            vec![gate(0), measure(0), ret()],
            vec![operation(5), operation(21)],
        );
        let base = ExecutionPolicy::base_qualification();

        let uncapped = prepare_mps_run(&program, 1, Some(42), None).expect("preflight passes");
        let capped = prepare_mps_run(&program, 1, Some(42), Some(7)).expect("preflight passes");

        assert_eq!(mps_policy(uncapped.max_bond_dimension), base);
        assert_eq!(mps_policy(capped.max_bond_dimension), base.with_bond_cap(7));
        assert_ne!(base.with_bond_cap(7), base);
    }

    #[test]
    fn preflight_rejects_reset_before_device_discovery() {
        let program = prepared_program(
            vec![gate(0), reset(0), measure(0), ret()],
            vec![operation(5), operation(21), operation(1)],
        );

        let error =
            prepare_mps_run(&program, 1, Some(42), None).expect_err("reset should be rejected");

        assert_eq!(
            error.to_string(),
            "consumer execution failed: reset is not supported by cuTensorNet MPS batch sampling (qubit 0)"
        );
        assert!(!error.is_environment_error());
    }
}
