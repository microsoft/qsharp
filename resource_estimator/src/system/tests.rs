// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde_json::Value;

use crate::estimates::{
    ErrorBudget, ErrorCorrection, Factory, FactoryBuilder, Overhead, PhysicalResourceEstimation,
    PhysicalResourceEstimationResult,
};
use crate::LogicalResources;

use super::estimate_physical_resources;

use crate::system::{
    data::{ErrorBudgetSpecification, JobParams, LogicalResourceCounts},
    error::IO,
    modeling::GateBasedPhysicalQubit,
    modeling::{PhysicalQubit, Protocol, TFactory},
    optimization::TFactoryBuilder,
    Result,
};

use std::rc::Rc;

#[test]
fn estimate_single() {
    let logical_resources = LogicalResources {
        num_qubits: 100,
        t_count: 0,
        rotation_count: 112_110,
        rotation_depth: 2001,
        ccz_count: 0,
        measurement_count: 0,
    };

    let params: &str = "[{}]";
    let result = estimate_physical_resources(&logical_resources, params);

    let json_value: Vec<Value> =
        serde_json::from_str(&result.expect("result is err")).expect("Failed to parse JSON");
    assert_eq!(json_value.len(), 1);

    let map = json_value[0].as_object().expect("Failed build map");
    assert!(!map.contains_key("frontierEntries"));
    assert!(map.contains_key("logicalQubit"));
    assert!(map.contains_key("physicalCounts"));
    assert!(map.contains_key("physicalCountsFormatted"));
}

#[test]
fn estimate_frontier() {
    let logical_resources = LogicalResources {
        num_qubits: 100,
        t_count: 0,
        rotation_count: 112_110,
        rotation_depth: 2001,
        ccz_count: 0,
        measurement_count: 0,
    };

    let params: &str = r#"[{
        "estimateType": "frontier"
    }]"#;

    let result = estimate_physical_resources(&logical_resources, params);

    let json_value: Vec<Value> =
        serde_json::from_str(&result.expect("result is err")).expect("Failed to parse JSON");
    assert_eq!(json_value.len(), 1);

    let map = json_value[0].as_object().expect("Failed build map");
    assert!(map.contains_key("frontierEntries"));
    assert!(!map.contains_key("logicalQubit"));
    assert!(!map.contains_key("physicalCounts"));
    assert!(!map.contains_key("physicalCountsFormatted"));
}

#[test]
fn physical_estimates_crash() {
    let result = estimate_physical_resources(
        &LogicalResources {
            num_qubits: 9,
            t_count: 160,
            rotation_count: 0,
            rotation_depth: 0,
            ccz_count: 8,
            measurement_count: 5,
        },
        r#"[{"qubitParams": {"name": "qubit_maj_ns_e6"},
            "qecScheme": {"name": "floquet_code"},
            "errorBudget": 0.075}]"#,
    );

    assert!(result
        .expect("estimation should succeed")
        .contains(r#""status":"success"#));
}

#[derive(Clone)]
struct TestLayoutOverhead {
    num_qubits: u64,
    logical_depth: u64,
    num_tstates: u64,
}

impl TestLayoutOverhead {
    pub fn new(num_qubits: u64, logical_depth: u64, num_tstates: u64) -> Self {
        Self {
            num_qubits,
            logical_depth,
            num_tstates,
        }
    }
}

impl Overhead for TestLayoutOverhead {
    fn logical_qubits(&self) -> u64 {
        self.num_qubits
    }

    fn logical_qubits_without_padding(&self) -> u64 {
        self.num_qubits
    }

    fn logical_depth(&self, _: u64) -> u64 {
        self.logical_depth
    }

    fn num_magic_states(&self, _: u64) -> u64 {
        self.num_tstates
    }

    fn num_magic_states_per_rotation(&self, _: f64) -> Option<u64> {
        None
    }
}

#[test]
pub fn test_no_tstates() {
    let ftp = Protocol::default();
    let qubit = Rc::new(PhysicalQubit::default());

    let partitioning = ErrorBudget::new(1e-3, 0.0, 0.0);
    let layout_overhead = TestLayoutOverhead::new(12, 0, 0);

    let estimation = PhysicalResourceEstimation::new(
        ftp,
        qubit,
        TFactoryBuilder::default(),
        layout_overhead,
        partitioning,
    );

    assert!(estimation.estimate().is_err());
}

#[test]
pub fn single_tstate() -> Result<()> {
    let ftp = Protocol::default();
    let qubit = Rc::new(PhysicalQubit::default());

    let partitioning = ErrorBudget::new(0.5e-4, 0.5e-4, 0.0);
    let layout_overhead = TestLayoutOverhead::new(4, 1, 1);

    let estimation = PhysicalResourceEstimation::new(
        ftp,
        qubit,
        TFactoryBuilder::default(),
        layout_overhead,
        partitioning,
    );

    estimation.estimate()?;

    Ok(())
}

#[test]
pub fn perfect_tstate() -> Result<()> {
    let ftp = Protocol::default();
    let qubit = Rc::new(PhysicalQubit::GateBased(GateBasedPhysicalQubit {
        t_gate_error_rate: 0.5e-4,
        ..GateBasedPhysicalQubit::default()
    }));

    let partitioning = ErrorBudget::new(0.5e-4, 0.5e-4, 0.0);
    let layout_overhead = TestLayoutOverhead::new(4, 1, 1);

    let estimation = PhysicalResourceEstimation::new(
        ftp,
        qubit,
        TFactoryBuilder::default(),
        layout_overhead,
        partitioning,
    );

    estimation.estimate()?;

    Ok(())
}

fn hubbard_overhead_and_partitioning() -> Result<(LogicalResourceCounts, ErrorBudget)> {
    let logical_counts =
        serde_json::from_str(include_str!("counts.json")).map_err(IO::CannotParseJSON)?;
    let partitioning = ErrorBudgetSpecification::Total(1e-3)
        .partitioning(&logical_counts)
        .expect("partitioning should succeed");

    Ok((logical_counts, partitioning))
}

fn validate_result_invariants<L: Overhead + Clone>(
    result: &PhysicalResourceEstimationResult<PhysicalQubit, TFactory, L>,
) {
    assert_eq!(
        result.physical_qubits(),
        result.physical_qubits_for_factories() + result.physical_qubits_for_algorithm()
    );
    assert_eq!(
        result.physical_qubits_for_factories(),
        result
            .factory()
            .expect("tfactory should be valid")
            .physical_qubits()
            * result.num_factories()
    );

    assert!(
        result.logical_qubit().logical_error_rate() <= result.required_logical_qubit_error_rate()
    );

    assert!(
        (result
            .factory()
            .expect("tfactory should be valid")
            .duration()
            * result.num_factory_runs())
            <= result.runtime()
    );
}

#[allow(clippy::too_many_lines)]
#[test]
pub fn test_hubbard_e2e() -> Result<()> {
    let ftp = Protocol::default();
    let qubit = Rc::new(PhysicalQubit::default());
    let (layout_overhead, partitioning) = hubbard_overhead_and_partitioning()?;
    let estimation = PhysicalResourceEstimation::new(
        ftp,
        qubit.clone(),
        TFactoryBuilder::default(),
        layout_overhead,
        partitioning,
    );

    let result = estimation.estimate()?;

    let logical_qubit = result.logical_qubit();
    let tfactory = result.factory().expect("tfactory should be valid");

    assert_eq!(logical_qubit.code_distance(), 17);
    assert_eq!(logical_qubit.logical_cycle_time(), 6800);

    assert_eq!(result.layout_overhead().logical_qubits(), 72);
    assert_eq!(result.algorithmic_logical_depth(), 22623);
    assert_eq!(result.num_factories(), 14);
    assert_eq!(result.num_factory_runs(), 1667);
    assert_eq!(result.physical_qubits_for_factories(), 252_000);
    assert_eq!(result.physical_qubits_for_algorithm(), 41616);
    assert_eq!(result.physical_qubits(), 293_616);
    assert_eq!(result.runtime(), 153_836_400);

    assert_eq!(tfactory.physical_qubits(), 18000);
    assert_eq!(tfactory.duration(), 92000);
    assert_eq!(tfactory.num_rounds(), 2);
    assert_eq!(tfactory.num_units_per_round(), vec![18, 1]);
    assert_eq!(
        tfactory.unit_names(),
        vec![
            String::from("15-to-1 space efficient"),
            String::from("15-to-1 RM prep")
        ]
    );

    validate_result_invariants(&result);

    let same_ftp = Protocol::default();
    let output_t_error_rate = result
        .required_logical_magic_state_error_rate()
        .expect("required_logical_tstate_error_rate should be valid");
    let tfactories = TFactoryBuilder::default().find_factories(
        &same_ftp,
        &qubit,
        output_t_error_rate,
        same_ftp.max_code_distance(),
    );

    assert_eq!(tfactories.len(), 2);
    if let Some(factory1) = get_tfactory(&tfactories, 88000, 27900) {
        assert_eq!(factory1.num_rounds(), 2);
        assert_eq!(factory1.num_units_per_round(), vec![18, 1]);
        assert_eq!(
            factory1.unit_names(),
            vec![
                String::from("15-to-1 RM prep"),
                String::from("15-to-1 RM prep")
            ]
        );
        assert_eq!(factory1.code_distance_per_round(), vec![5, 15]);
    }

    if let Some(factory2) = get_tfactory(&tfactories, 92000, 18000) {
        assert_eq!(factory2.num_rounds(), 2);
        assert_eq!(factory2.num_units_per_round(), vec![18, 1]);
        assert_eq!(
            factory2.unit_names(),
            vec![
                String::from("15-to-1 space efficient"),
                String::from("15-to-1 RM prep")
            ]
        );
        assert_eq!(factory2.code_distance_per_round(), vec![5, 15]);
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
#[test]
pub fn test_hubbard_e2e_measurement_based() -> Result<()> {
    let ftp = Protocol::floquet_code();
    let qubit = Rc::new(PhysicalQubit::qubit_maj_ns_e6());
    let (layout_overhead, partitioning) = hubbard_overhead_and_partitioning()?;
    let estimation = PhysicalResourceEstimation::new(
        ftp,
        qubit.clone(),
        TFactoryBuilder::default(),
        layout_overhead,
        partitioning,
    );

    let result = estimation.estimate()?;

    let logical_qubit = result.logical_qubit();
    let tfactory = result.factory().expect("tfactory should be valid");

    assert_eq!(logical_qubit.code_distance(), 5);
    assert_eq!(logical_qubit.logical_cycle_time(), 1500);

    assert_eq!(result.layout_overhead().logical_qubits(), 72);
    assert_eq!(result.algorithmic_logical_depth(), 22623);
    assert_eq!(result.num_factories(), 10);
    assert_eq!(result.physical_qubits_for_factories(), 10400);
    assert_eq!(result.physical_qubits_for_algorithm(), 9504);
    assert_eq!(result.physical_qubits(), 19904);
    assert_eq!(result.runtime(), 33_934_500);

    assert_eq!(tfactory.physical_qubits(), 1040);
    assert_eq!(tfactory.num_rounds(), 2);
    assert_eq!(tfactory.num_units_per_round(), vec![23, 1]);
    assert_eq!(
        tfactory.unit_names(),
        vec![
            String::from("15-to-1 RM prep"),
            String::from("15-to-1 space efficient")
        ]
    );

    validate_result_invariants(&result);

    let output_t_error_rate = result
        .required_logical_magic_state_error_rate()
        .expect("required_logical_tstate_error_rate should be valid");
    let same_ftp = Protocol::floquet_code();
    let tfactories = TFactoryBuilder::default().find_factories(
        &same_ftp,
        &qubit,
        output_t_error_rate,
        same_ftp.max_code_distance(),
    );

    assert_eq!(tfactories.len(), 2);
    if let Some(factory1) = get_tfactory(&tfactories, 12300, 1612) {
        assert_eq!(factory1.num_rounds(), 2);
        assert_eq!(factory1.num_units_per_round(), vec![23, 1]);
        assert_eq!(
            factory1.unit_names(),
            vec![
                String::from("15-to-1 RM prep"),
                String::from("15-to-1 RM prep")
            ]
        );
        assert_eq!(factory1.code_distance_per_round(), vec![1, 3]);
    }

    if let Some(factory2) = get_tfactory(&tfactories, 14100, 1040) {
        assert_eq!(factory2.num_rounds(), 2);
        assert_eq!(factory2.num_units_per_round(), vec![23, 1]);
        assert_eq!(
            factory2.unit_names(),
            vec![
                String::from("15-to-1 RM prep"),
                String::from("15-to-1 space efficient")
            ]
        );
        assert_eq!(factory2.code_distance_per_round(), vec![1, 3]);
    }

    Ok(())
}

#[test]
pub fn test_hubbard_e2e_increasing_max_duration() -> Result<()> {
    let ftp = Protocol::floquet_code();
    let qubit = Rc::new(PhysicalQubit::qubit_maj_ns_e6());
    let (layout_overhead, partitioning) = hubbard_overhead_and_partitioning()?;
    let estimation = PhysicalResourceEstimation::new(
        ftp,
        qubit,
        TFactoryBuilder::default(),
        layout_overhead,
        partitioning,
    );

    let max_duration_in_nanoseconds1: u64 = 50_000_000_u64;
    let max_duration_in_nanoseconds2: u64 = 500_000_000_u64;

    let result1 = estimation.estimate_with_max_duration(max_duration_in_nanoseconds1)?;
    let result2 = estimation.estimate_with_max_duration(max_duration_in_nanoseconds2)?;

    assert!(result1.runtime() <= max_duration_in_nanoseconds1);
    assert!(result2.runtime() <= max_duration_in_nanoseconds2);
    assert!(result1.physical_qubits() >= result2.physical_qubits());

    assert_eq!(result1.physical_qubits(), 16784);
    assert_eq!(result2.physical_qubits(), 10544);
    Ok(())
}

#[test]
pub fn test_hubbard_e2e_increasing_max_num_qubits() -> Result<()> {
    let ftp = Protocol::floquet_code();
    let qubit = Rc::new(PhysicalQubit::qubit_maj_ns_e6());
    let (layout_overhead, partitioning) = hubbard_overhead_and_partitioning()?;
    let estimation = PhysicalResourceEstimation::new(
        ftp,
        qubit,
        TFactoryBuilder::default(),
        layout_overhead,
        partitioning,
    );

    let max_num_qubits1: u64 = 11000;
    let max_num_qubits2: u64 = 20000;

    let result1 = estimation.estimate_with_max_num_qubits(max_num_qubits1)?;
    let result2 = estimation.estimate_with_max_num_qubits(max_num_qubits2)?;

    assert!(result1.physical_qubits() <= max_num_qubits1);
    assert!(result2.physical_qubits() <= max_num_qubits2);
    assert!(result1.runtime() >= result2.runtime());

    assert_eq!(result1.runtime(), 329_010_000_u64);
    assert_eq!(result2.runtime(), 33_934_500_u64);
    Ok(())
}

fn prepare_chemistry_estimation_with_expected_majorana(
) -> PhysicalResourceEstimation<Protocol, TFactoryBuilder, LogicalResourceCounts> {
    let ftp = Protocol::floquet_code();
    let qubit = Rc::new(PhysicalQubit::qubit_maj_ns_e4());

    let value = r#"{
        "numQubits": 1318,
        "tCount": 55321680,
        "rotationCount": 205612244,
        "rotationDepth": 205151230,
        "cczCount": 134678785904,
        "ccixCount": 0,
        "measurementCount": 1374743748
    }"#;

    let counts: LogicalResourceCounts = serde_json::from_str(value).expect("json should be valid");

    let partitioning = ErrorBudgetSpecification::Total(1e-3)
        .partitioning(&counts)
        .expect("partitioning should succeed");
    PhysicalResourceEstimation::new(ftp, qubit, TFactoryBuilder::default(), counts, partitioning)
}

#[test]
pub fn test_chemistry_small_max_duration() {
    let max_duration_in_nanoseconds: u64 = 1_000_000_000_u64;

    let estimation = prepare_chemistry_estimation_with_expected_majorana();

    let result = estimation.estimate_with_max_duration(max_duration_in_nanoseconds);

    match result {
        Err(crate::estimates::Error::MaxDurationTooSmall) => {}
        _ => unreachable!("Expected MaxDurationTooSmall"),
    }
}

#[test]
pub fn test_chemistry_small_max_num_qubits() {
    let max_num_qubits: u64 = 10_000;
    let estimation = prepare_chemistry_estimation_with_expected_majorana();

    let result = estimation.estimate_with_max_num_qubits(max_num_qubits);

    match result {
        Err(crate::estimates::Error::MaxPhysicalQubitsTooSmall) => {}
        _ => unreachable!("Expected MaxNumQubitsTooSmall"),
    }
}

#[test]
pub fn test_chemistry_based_max_duration() -> Result<()> {
    let max_duration_in_nanoseconds: u64 = 365 * 24 * 3600 * 1_000_000_000_u64;

    let estimation = prepare_chemistry_estimation_with_expected_majorana();

    let result = estimation.estimate_with_max_duration(max_duration_in_nanoseconds)?;

    let logical_qubit = result.logical_qubit();
    let tfactory = result.factory().expect("tfactory should be valid");

    // constraint is not violated
    assert!(result.runtime() <= max_duration_in_nanoseconds);

    assert_eq!(logical_qubit.code_distance(), 19);
    assert_eq!(logical_qubit.logical_cycle_time(), 5700);

    assert_eq!(result.layout_overhead().logical_qubits(), 2740);
    assert_eq!(result.algorithmic_logical_depth(), 411_211_118_594_u64);
    assert_eq!(result.num_factories(), 2);
    assert_eq!(result.physical_qubits_for_factories(), 572_000);
    assert_eq!(result.physical_qubits_for_algorithm(), 4_351_120);
    assert_eq!(result.physical_qubits(), 4_923_120);

    assert_eq!(result.runtime(), 22_371_634_030_834_500_u64);

    assert_eq!(tfactory.physical_qubits(), 286_000);
    assert_eq!(tfactory.num_rounds(), 4);
    assert_eq!(tfactory.num_units_per_round(), vec![19994, 275, 16, 1]);
    assert_eq!(
        tfactory.unit_names(),
        vec![
            String::from("15-to-1 space efficient"),
            String::from("15-to-1 space efficient"),
            String::from("15-to-1 RM prep"),
            String::from("15-to-1 RM prep"),
        ]
    );
    assert_eq!(tfactory.code_distance_per_round(), vec![1, 3, 5, 15]);

    assert_eq!(
        result.physical_qubits(),
        result.physical_qubits_for_factories() + result.physical_qubits_for_algorithm()
    );
    assert_eq!(
        result.physical_qubits_for_factories(),
        result
            .factory()
            .expect("tfactory should be valid")
            .physical_qubits()
            * result.num_factories()
    );

    assert!(
        result.logical_qubit().logical_error_rate() <= result.required_logical_qubit_error_rate()
    );

    Ok(())
}

#[test]
pub fn test_chemistry_based_max_num_qubits() -> Result<()> {
    let max_num_qubits: u64 = 4_923_120;

    let estimation = prepare_chemistry_estimation_with_expected_majorana();

    let result = estimation.estimate_with_max_num_qubits(max_num_qubits)?;

    let logical_qubit = result.logical_qubit();
    let tfactory = result.factory().expect("tfactory should be valid");

    // constraint is not violated
    assert!(result.physical_qubits() <= max_num_qubits);

    assert_eq!(logical_qubit.code_distance(), 19);
    assert_eq!(logical_qubit.logical_cycle_time(), 5700);

    assert_eq!(result.layout_overhead().logical_qubits(), 2740);
    assert_eq!(result.algorithmic_logical_depth(), 411_211_118_594_u64);
    assert_eq!(result.num_factories(), 2);
    assert_eq!(result.physical_qubits_for_factories(), 572_000);
    assert_eq!(result.physical_qubits_for_algorithm(), 4_351_120);
    assert_eq!(result.physical_qubits(), 4_923_120);
    assert_eq!(result.runtime(), 22_371_634_030_834_500_u64);

    assert_eq!(tfactory.physical_qubits(), 286_000);
    assert_eq!(tfactory.num_rounds(), 4);
    assert_eq!(tfactory.num_units_per_round(), vec![19994, 275, 16, 1]);
    assert_eq!(
        tfactory.unit_names(),
        vec![
            String::from("15-to-1 space efficient"),
            String::from("15-to-1 space efficient"),
            String::from("15-to-1 RM prep"),
            String::from("15-to-1 RM prep"),
        ]
    );
    assert_eq!(tfactory.code_distance_per_round(), vec![1, 3, 5, 15]);

    assert_eq!(
        result.physical_qubits(),
        result.physical_qubits_for_factories() + result.physical_qubits_for_algorithm()
    );
    assert_eq!(
        result.physical_qubits_for_factories(),
        result
            .factory()
            .expect("tfactory should be valid")
            .physical_qubits()
            * result.num_factories()
    );

    assert!(
        result.logical_qubit().logical_error_rate() <= result.required_logical_qubit_error_rate()
    );

    Ok(())
}

fn prepare_factorization_estimation_with_optimistic_majorana(
) -> PhysicalResourceEstimation<Protocol, TFactoryBuilder, LogicalResourceCounts> {
    let ftp = Protocol::floquet_code();
    let qubit = Rc::new(PhysicalQubit::qubit_maj_ns_e6());

    let value = r#"{
        "numQubits": 12581,
        "tCount": 12,
        "rotationCount": 12,
        "rotationDepth": 12,
        "cczCount": 3731607428,
        "ccixCount": 0,
        "measurementCount": 1078154040
    }"#;

    let counts: LogicalResourceCounts = serde_json::from_str(value).expect("json should be valid");

    let partitioning = ErrorBudgetSpecification::Total(1e-3)
        .partitioning(&counts)
        .expect("partitioning should succeed");
    PhysicalResourceEstimation::new(ftp, qubit, TFactoryBuilder::default(), counts, partitioning)
}

#[test]
pub fn test_factorization_2048_max_duration_matches_regular_estimate() -> Result<()> {
    let estimation = prepare_factorization_estimation_with_optimistic_majorana();

    let result_no_max_duration = estimation.estimate_without_restrictions()?;

    let logical_qubit_no_max_duration = result_no_max_duration.logical_qubit();

    let max_duration_in_nanoseconds: u64 = result_no_max_duration.runtime();
    let result = estimation.estimate_with_max_duration(max_duration_in_nanoseconds)?;

    let logical_qubit = result.logical_qubit();

    assert_eq!(
        logical_qubit_no_max_duration.code_distance(),
        logical_qubit.code_distance()
    );

    assert_eq!(
        result_no_max_duration.layout_overhead().logical_qubits(),
        result.layout_overhead().logical_qubits()
    );

    assert_eq!(
        result_no_max_duration.algorithmic_logical_depth(),
        result.algorithmic_logical_depth()
    );

    assert_eq!(
        result_no_max_duration.num_factories(),
        result.num_factories()
    );

    assert_eq!(
        result_no_max_duration.physical_qubits_for_factories(),
        result.physical_qubits_for_factories()
    );

    assert_eq!(
        result_no_max_duration.physical_qubits_for_algorithm(),
        result.physical_qubits_for_algorithm()
    );

    assert_eq!(
        result_no_max_duration.physical_qubits(),
        result.physical_qubits()
    );

    assert_eq!(result_no_max_duration.runtime(), result.runtime());

    Ok(())
}

#[test]
pub fn test_factorization_2048_max_num_qubits_matches_regular_estimate() -> Result<()> {
    let estimation = prepare_factorization_estimation_with_optimistic_majorana();

    let result_no_max_num_qubits = estimation.estimate_without_restrictions()?;

    let logical_qubit_no_max_num_qubits = result_no_max_num_qubits.logical_qubit();

    let max_num_qubits = result_no_max_num_qubits.physical_qubits();
    let result = estimation.estimate_with_max_num_qubits(max_num_qubits)?;

    let logical_qubit = result.logical_qubit();

    assert_eq!(
        logical_qubit_no_max_num_qubits.code_distance(),
        logical_qubit.code_distance()
    );

    assert_eq!(
        result_no_max_num_qubits.layout_overhead().logical_qubits(),
        result.layout_overhead().logical_qubits()
    );

    assert_eq!(
        result_no_max_num_qubits.algorithmic_logical_depth(),
        result.algorithmic_logical_depth()
    );

    assert_eq!(
        result_no_max_num_qubits.num_factories(),
        result.num_factories()
    );

    assert_eq!(
        result_no_max_num_qubits.physical_qubits_for_factories(),
        result.physical_qubits_for_factories()
    );

    assert_eq!(
        result_no_max_num_qubits.physical_qubits_for_algorithm(),
        result.physical_qubits_for_algorithm()
    );

    assert_eq!(
        result_no_max_num_qubits.physical_qubits(),
        result.physical_qubits()
    );

    assert_eq!(result_no_max_num_qubits.runtime(), result.runtime());

    Ok(())
}

fn prepare_ising20x20_estimation_with_pessimistic_gate_based(
) -> PhysicalResourceEstimation<Protocol, TFactoryBuilder, LogicalResourceCounts> {
    let ftp = Protocol::surface_code_gate_based();
    let qubit = Rc::new(PhysicalQubit::qubit_gate_us_e3());

    let value = r#"{
        "numQubits": 100,
        "tCount": 0,
        "rotationCount": 112110,
        "rotationDepth": 2001,
        "cczCount": 0,
        "ccixCount": 0,
        "measurementCount": 0
    }"#;

    let counts: LogicalResourceCounts = serde_json::from_str(value).expect("cannot parse json");

    let partitioning = ErrorBudgetSpecification::Total(1e-3)
        .partitioning(&counts)
        .expect("cannot setup error budget partitioning");
    PhysicalResourceEstimation::new(ftp, qubit, TFactoryBuilder::default(), counts, partitioning)
}

#[test]
fn build_frontier_test() {
    let estimation = prepare_ising20x20_estimation_with_pessimistic_gate_based();

    let frontier_result = estimation.build_frontier();

    let points = frontier_result.expect("failed to estimate");
    assert_eq!(points.len(), 195);

    for i in 0..points.len() - 1 {
        assert!(points[i].runtime() <= points[i + 1].runtime());
        assert!(points[i].physical_qubits() >= points[i + 1].physical_qubits());
        assert!(points[i].num_factories() >= points[i + 1].num_factories());
        assert!(
            points[i].logical_qubit().code_distance()
                <= points[i + 1].logical_qubit().code_distance()
        );
    }

    let shortest_runtime_result = estimation.estimate().expect("failed to estimate");
    assert_eq!(points[0].runtime(), shortest_runtime_result.runtime());
    assert_eq!(
        points[0].physical_qubits(),
        shortest_runtime_result.physical_qubits()
    );
    assert_eq!(
        points[0].num_factories(),
        shortest_runtime_result.num_factories()
    );
    assert_eq!(
        points[0].logical_qubit().code_distance(),
        shortest_runtime_result.logical_qubit().code_distance()
    );

    let mut max_duration = shortest_runtime_result.runtime();
    let num_iterations = 100;
    let coefficient = 1.05;
    for _ in 0..num_iterations {
        max_duration = (max_duration as f64 * coefficient) as u64;
        let result = estimation
            .estimate_with_max_duration(max_duration)
            .expect("failed to estimate");

        assert!(
            points
                .iter()
                .filter(|point| point.runtime() <= result.runtime()
                    && point.physical_qubits() == result.physical_qubits())
                .count()
                == 1
        );
    }

    let mut max_num_qubits = shortest_runtime_result.physical_qubits();
    let num_iterations = 100;
    let coefficient = 1.10;
    for _ in 0..num_iterations {
        max_num_qubits = (max_num_qubits as f64 / coefficient) as u64;
        let result = estimation.estimate_with_max_num_qubits(max_num_qubits);

        if let Ok(result) = result {
            assert!(
                points
                    .iter()
                    .filter(|point| point.runtime() <= result.runtime()
                        && point.physical_qubits() == result.physical_qubits())
                    .count()
                    == 1
            );
        }
    }
}

fn prepare_bit_flip_code_resources_and_majorana_n6_qubit(
) -> PhysicalResourceEstimation<Protocol, TFactoryBuilder, LogicalResourceCounts> {
    let qubit = Rc::new(PhysicalQubit::qubit_maj_ns_e6());
    let ftp = Protocol::floquet_code();

    let value = r#"{
        "numQubits": 5,
        "tCount": 0,
        "rotationCount": 1,
        "rotationDepth": 1,
        "cczCount": 0,
        "ccixCount": 0,
        "measurementCount": 3
    }"#;

    let counts: LogicalResourceCounts = serde_json::from_str(value).expect("cannot parse json");

    let partitioning = ErrorBudgetSpecification::Total(1e-3)
        .partitioning(&counts)
        .expect("cannot setup error budget partitioning");
    PhysicalResourceEstimation::new(ftp, qubit, TFactoryBuilder::default(), counts, partitioning)
}

#[test]
fn build_frontier_bit_flip_code_test() {
    let estimation = prepare_bit_flip_code_resources_and_majorana_n6_qubit();

    let frontier_result = estimation.build_frontier();

    let points = frontier_result.expect("failed to estimate");
    assert_eq!(points.len(), 7);

    let shortest_runtime_result = estimation.estimate().expect("failed to estimate");

    assert_eq!(points[0].runtime(), shortest_runtime_result.runtime());
    assert_eq!(
        points[0]
            .factory()
            .expect("point should have valid tfactory value")
            .duration(),
        shortest_runtime_result
            .factory()
            .expect("shortest result should have valid tfactory value")
            .duration()
    );

    assert_eq!(
        points[0]
            .factory()
            .expect("point should have valid tfactory value")
            .physical_qubits(),
        shortest_runtime_result
            .factory()
            .expect("shortest result should have valid tfactory value")
            .physical_qubits()
    );

    assert_eq!(
        points[0].num_factories(),
        shortest_runtime_result.num_factories()
    );

    assert_eq!(
        points[0].physical_qubits(),
        shortest_runtime_result.physical_qubits()
    );
    assert_eq!(
        points[0].num_factories(),
        shortest_runtime_result.num_factories()
    );
}

#[allow(clippy::cast_lossless)]
#[test]
fn code_distance_tests() {
    let params = JobParams::default();

    let ftp = Protocol::surface_code_gate_based();

    for logical_qubits in (50..=1000).step_by(50) {
        for num_cycles in (50_000..=500_000).step_by(50_000) {
            for exp in 1..=15 {
                let budget_logical = 10.0_f64.powi(-exp);

                let required_logical_qubit_error_rate =
                    budget_logical / (logical_qubits * num_cycles) as f64;

                let qubit = params.qubit_params().clone();
                let code_distance =
                    ftp.compute_code_distance(&qubit, required_logical_qubit_error_rate);

                assert!(code_distance <= ftp.max_code_distance());
            }
        }
    }
}

fn get_tfactory(tfactories: &[TFactory], duration: u64, physical_qubits: u64) -> Option<&TFactory> {
    let tfactory = tfactories.iter().find(|tfactory| {
        tfactory.duration() == duration && tfactory.physical_qubits() == physical_qubits
    });
    assert!(tfactory.is_some());
    tfactory
}
