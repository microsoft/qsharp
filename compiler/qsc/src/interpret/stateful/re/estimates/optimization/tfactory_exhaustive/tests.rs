// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    super::super::{
        data,
        modeling::{PhysicalQubit, Protocol},
        stages::tfactory::{TFactory, TFactoryDistillationUnitTemplate},
    },
    find_nondominated_population, find_nondominated_tfactories,
    population::Point2D,
};
use std::{rc::Rc, time::Instant};

#[test]
fn test_one_t_error_rate() {
    let start = Instant::now();
    let factories = find_nondominated_tfactories(
        &Protocol::default(),
        &Rc::new(PhysicalQubit::default()),
        &TFactoryDistillationUnitTemplate::default_distillation_unit_templates(),
        1e-18,
        35,
    );
    let elapsed = start.elapsed();

    // Here we know that the factory must have a valid normalized volume
    println!("elapsed   = {elapsed:?}");
    for factory in factories {
        println!(
            "found!    = {} ({})",
            factory.normalized_volume(),
            factory.normalized_volume().log10()
        );
    }
}

#[test]
pub fn chemistry_qubit_gate_us_e3_test() {
    let tfactories = find_tfactories(&Protocol::default(), "qubit_gate_us_e3");

    assert_eq!(tfactories.len(), 3);
    assert_eq!(tfactories[0].physical_qubits(), 50460);
    assert_eq!(tfactories[1].physical_qubits(), 33640);
    assert_eq!(tfactories[2].physical_qubits(), 30276);

    assert_eq!(tfactories[0].duration(), 104_400_000);
    assert_eq!(tfactories[1].duration(), 208_800_000);
    assert_eq!(tfactories[2].duration(), 400_200_000);
}

#[test]
pub fn chemistry_qubit_gate_us_e4_test() {
    let tfactories = find_tfactories(&Protocol::default(), "qubit_gate_us_e4");

    assert_eq!(tfactories.len(), 3);
    assert_eq!(tfactories[0].physical_qubits(), 13500);
    assert_eq!(tfactories[1].physical_qubits(), 9000);
    assert_eq!(tfactories[2].physical_qubits(), 8100);

    assert_eq!(tfactories[0].duration(), 54_000_000);
    assert_eq!(tfactories[1].duration(), 108_000_000);
    assert_eq!(tfactories[2].duration(), 207_000_000);
}

#[test]
pub fn chemistry_qubit_gate_ns_e3_test() {
    let tfactories = find_tfactories(&Protocol::default(), "qubit_gate_ns_e3");

    assert_eq!(tfactories.len(), 4);
    assert_eq!(tfactories[0].physical_qubits(), 82620);
    assert_eq!(tfactories[1].physical_qubits(), 55080);
    assert_eq!(tfactories[2].physical_qubits(), 50460);
    assert_eq!(tfactories[3].physical_qubits(), 49572);

    assert_eq!(tfactories[0].duration(), 91_200);
    assert_eq!(tfactories[1].duration(), 112_800);
    assert_eq!(tfactories[2].duration(), 152_400);
    assert_eq!(tfactories[3].duration(), 222_000);
}

#[test]
pub fn chemistry_qubit_gate_ns_e4_test() {
    let tfactories = find_tfactories(&Protocol::default(), "qubit_gate_ns_e4");

    assert_eq!(tfactories.len(), 3);
    assert_eq!(tfactories[0].physical_qubits(), 24000);
    assert_eq!(tfactories[1].physical_qubits(), 16000);
    assert_eq!(tfactories[2].physical_qubits(), 14400);

    assert_eq!(tfactories[0].duration(), 48000);
    assert_eq!(tfactories[1].duration(), 60000);
    assert_eq!(tfactories[2].duration(), 82000);
}

#[test]
pub fn chemistry_qubit_maj_ns_e4_test() {
    let tfactories = find_tfactories(&Protocol::floquet_code(), "qubit_maj_ns_e4");

    assert_eq!(tfactories.len(), 5);
    assert_eq!(tfactories[0].physical_qubits(), 619_814);
    assert_eq!(tfactories[1].physical_qubits(), 429_000);
    assert_eq!(tfactories[2].physical_qubits(), 379_886);
    assert_eq!(tfactories[3].physical_qubits(), 286_000);
    assert_eq!(tfactories[4].physical_qubits(), 257_400);

    assert_eq!(tfactories[0].duration(), 43800);
    assert_eq!(tfactories[1].duration(), 44600);
    assert_eq!(tfactories[2].duration(), 50000);
    assert_eq!(tfactories[3].duration(), 52500);
    assert_eq!(tfactories[4].duration(), 62400);
}

#[test]
pub fn chemistry_qubit_maj_ns_e6_test() {
    let tfactories = find_tfactories(&Protocol::floquet_code(), "qubit_maj_ns_e6");

    assert_eq!(tfactories.len(), 3);
    assert_eq!(tfactories[0].physical_qubits(), 24960);
    assert_eq!(tfactories[1].physical_qubits(), 16640);
    assert_eq!(tfactories[2].physical_qubits(), 14976);

    assert_eq!(tfactories[0].duration(), 20400);
    assert_eq!(tfactories[1].duration(), 25800);
    assert_eq!(tfactories[2].duration(), 35700);
}

#[test]
fn required_logical_tstate_error_too_high() {
    let ftp = Protocol::default();
    let qubit: Rc<PhysicalQubit> = Rc::new(PhysicalQubit::default());
    let distillation_unit_templates =
        TFactoryDistillationUnitTemplate::default_distillation_unit_templates();
    let output_t_error_rate = 1e-1;
    let max_code_distance = 19;

    let population = find_nondominated_population::<Point2D<TFactory>>(
        &ftp,
        &qubit,
        &distillation_unit_templates,
        output_t_error_rate,
        max_code_distance,
    );

    assert_eq!(population.items().len(), 1);
    let tfactory = &population.items()[0].item;

    assert_eq!(tfactory.num_rounds(), 1);
    assert_eq!(tfactory.physical_qubits(), 722);
    assert_eq!(tfactory.duration(), 7600);
    assert_eq!(tfactory.unit_names(), vec!["trivial 1-to-1"]);
}

fn find_tfactories(ftp: &Protocol, qubit_name: &str) -> Vec<TFactory> {
    let qubit: Rc<PhysicalQubit> = serde_json::from_str(&format!(r#"{{"name": "{qubit_name}"}}"#))
        .expect("json should be valid");

    let output_t_error_rate = 6.123_826_261_916_663E-16;
    find_nondominated_tfactories(
        ftp,
        &qubit,
        &create_test_templates(),
        output_t_error_rate,
        ftp.max_code_distance(),
    )
}

fn create_test_templates() -> Vec<TFactoryDistillationUnitTemplate> {
    let template1 =
        create_custom_distillation_unit_1_to_15_template("15-to-1 RM prep", 31, 11, 31, 24);
    let template2 = create_custom_distillation_unit_1_to_15_template(
        "15-to-1 space efficient (dirail)",
        20,
        12,
        12,
        57,
    );
    let template3 = create_custom_distillation_unit_1_to_15_template(
        "15-to-1 space efficient (trirail)",
        30,
        6,
        19,
        32,
    );
    let template4 = create_custom_distillation_unit_1_to_15_template(
        "15-to-1 space efficient (minimal)",
        18,
        23,
        9,
        68,
    );
    vec![template1, template2, template3, template4]
}

fn create_custom_distillation_unit_1_to_15_template(
    name: &str,
    logical_num_unit_qubits: u64,
    logical_duration_in_qubit_cycle_time: u64,
    physical_num_unit_qubits: u64,
    physical_duration_in_qubit_cycle_time: u64,
) -> TFactoryDistillationUnitTemplate {
    let specification = data::TFactoryDistillationUnitSpecification::Custom {
        display_name: name.to_string(),
        num_input_ts: 15,
        num_output_ts: 1,
        failure_probability_formula: String::from(
            "15.0 * inputErrorRate + 356.0 * cliffordErrorRate",
        ),
        output_error_rate_formula: String::from(
            "35.0 * inputErrorRate ^ 3 + 7.1 * cliffordErrorRate",
        ),
        logical_qubit_specification: Some(
            data::TFactoryProtocolSpecificDistillationUnitSpecification {
                num_unit_qubits: logical_num_unit_qubits,
                duration_in_qubit_cycle_time: logical_duration_in_qubit_cycle_time,
            },
        ),
        physical_qubit_specification: Some(
            data::TFactoryProtocolSpecificDistillationUnitSpecification {
                num_unit_qubits: physical_num_unit_qubits,
                duration_in_qubit_cycle_time: physical_duration_in_qubit_cycle_time,
            },
        ),
        logical_qubit_specification_first_round_override: None,
    };

    TFactoryDistillationUnitTemplate::try_from(&specification)
        .expect("TFactory from specification should succeed")
}
