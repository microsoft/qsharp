// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rustc_hash::FxHashSet;
use std::rc::Rc;

use super::super::super::{
    data::{
        TFactoryDistillationUnitSpecification,
        TFactoryProtocolSpecificDistillationUnitSpecification,
    },
    modeling::{LogicalQubit, PhysicalQubit, Protocol},
    stages::tfactory::{
        TFactoryDistillationUnit, TFactoryDistillationUnitTemplate, TFactoryDistillationUnitType,
    },
};

use super::DistillationUnitsMap;

fn create_default_units_for_distance<'a>(
    distillation_units_map: &'a DistillationUnitsMap<'a>,
    position: usize,
    distance: u64,
) -> Vec<&'a TFactoryDistillationUnit<'a>> {
    if position == 0 && distance == 1 {
        (0..distillation_units_map.num_physical_distillation_units)
            .filter_map(|idx| {
                distillation_units_map.get(
                    position,
                    distance,
                    idx + distillation_units_map.num_logical_distillation_units,
                )
            })
            .collect::<Vec<_>>()
    } else {
        (0..distillation_units_map.num_logical_distillation_units)
            .filter_map(|idx| distillation_units_map.get(position, distance, idx))
            .collect::<Vec<_>>()
    }
}

fn create_default_map<'a>(
    physical_qubit: PhysicalQubit,
    ftp: &'a Protocol,
    templates: &'a [TFactoryDistillationUnitTemplate],
) -> DistillationUnitsMap<'a> {
    let qubit = Rc::new(physical_qubit);

    let min_code_distance = 1;
    let max_code_distance = 11;
    let distances: Vec<_> = (min_code_distance..=max_code_distance).step_by(2).collect();
    let mut qubits = vec![None; max_code_distance as usize + 1];
    for &distance in &distances {
        qubits[distance as usize] = LogicalQubit::new(ftp, distance, qubit.clone())
            .ok()
            .map(Rc::new);
    }

    DistillationUnitsMap::create(&qubit, &qubits, distances, templates)
}

fn create_and_test(
    qubit: PhysicalQubit,
    ftp: &Protocol,
    position: usize,
    distance: u64,
    expected: &str,
    not_expected: &str,
) {
    let distillation_unit_templates =
        &TFactoryDistillationUnitTemplate::default_distillation_unit_templates();
    let distillation_units_map = create_default_map(qubit, ftp, distillation_unit_templates);
    let units = create_default_units_for_distance(&distillation_units_map, position, distance);
    for unit in &units {
        assert!(unit.name.contains(expected));
        assert!(!unit.name.contains(not_expected));
    }
}

fn create_distillation_unit_template(
    name: &str,
    unit_type: TFactoryDistillationUnitType,
) -> TFactoryDistillationUnitTemplate {
    let (
        physical_qubit_specification,
        logical_qubit_specification,
        logical_qubit_specification_first_round_override,
    ) = match unit_type {
        TFactoryDistillationUnitType::Physical => (
            Some(create_default_t_factory_protocol_specific_distillation_unit_specification()),
            None,
            None,
        ),
        TFactoryDistillationUnitType::Logical => (
            None,
            Some(create_default_t_factory_protocol_specific_distillation_unit_specification()),
            None, // or create_default_t_factory_protocol_specific_distillation_unit_specification(),
        ),
        TFactoryDistillationUnitType::Combined => (
            Some(create_default_t_factory_protocol_specific_distillation_unit_specification()),
            Some(create_default_t_factory_protocol_specific_distillation_unit_specification()),
            None, // or create_default_t_factory_protocol_specific_distillation_unit_specification(),
        ),
    };

    let specification = TFactoryDistillationUnitSpecification::Custom {
        display_name: name.to_owned(),
        num_input_ts: 1,
        num_output_ts: 1,
        failure_probability_formula: "0.5 * inputErrorRate".to_owned(),
        output_error_rate_formula: "0.5 * inputErrorRate".to_owned(),
        physical_qubit_specification,
        logical_qubit_specification,
        logical_qubit_specification_first_round_override,
    };

    TFactoryDistillationUnitTemplate::try_from(&specification)
        .expect("TFactory from specification should succeed")
}

fn create_distillation_unit_template_with_override() -> TFactoryDistillationUnitTemplate {
    let specification = TFactoryDistillationUnitSpecification::Custom {
        display_name: "combined with override".to_owned(),
        num_input_ts: 1,
        num_output_ts: 1,
        failure_probability_formula: "0.5 * inputErrorRate".to_owned(),
        output_error_rate_formula: "0.5 * inputErrorRate".to_owned(),
        physical_qubit_specification: Some(TFactoryProtocolSpecificDistillationUnitSpecification {
            num_unit_qubits: 1,
            duration_in_qubit_cycle_time: 2,
        }),
        logical_qubit_specification: Some(TFactoryProtocolSpecificDistillationUnitSpecification {
            num_unit_qubits: 3,
            duration_in_qubit_cycle_time: 4,
        }),
        logical_qubit_specification_first_round_override: Some(
            TFactoryProtocolSpecificDistillationUnitSpecification {
                num_unit_qubits: 5,
                duration_in_qubit_cycle_time: 6,
            },
        ),
    };

    TFactoryDistillationUnitTemplate::try_from(&specification)
        .expect("TFactory from specification should succeed")
}

fn create_distillation_unit_template_without_override() -> TFactoryDistillationUnitTemplate {
    let specification = TFactoryDistillationUnitSpecification::Custom {
        display_name: "combined without override".to_owned(),
        num_input_ts: 1,
        num_output_ts: 1,
        failure_probability_formula: "0.5 * inputErrorRate".to_owned(),
        output_error_rate_formula: "0.5 * inputErrorRate".to_owned(),
        physical_qubit_specification: Some(TFactoryProtocolSpecificDistillationUnitSpecification {
            num_unit_qubits: 1,
            duration_in_qubit_cycle_time: 2,
        }),
        logical_qubit_specification: Some(TFactoryProtocolSpecificDistillationUnitSpecification {
            num_unit_qubits: 3,
            duration_in_qubit_cycle_time: 4,
        }),
        logical_qubit_specification_first_round_override: None,
    };

    TFactoryDistillationUnitTemplate::try_from(&specification)
        .expect("TFactory from specification should succeed")
}

fn create_default_t_factory_protocol_specific_distillation_unit_specification(
) -> TFactoryProtocolSpecificDistillationUnitSpecification {
    TFactoryProtocolSpecificDistillationUnitSpecification {
        num_unit_qubits: 1,
        duration_in_qubit_cycle_time: 1,
    }
}

#[test]
fn units_for_distance_test_position_0_distance_1() {
    create_and_test(
        PhysicalQubit::qubit_maj_ns_e4(),
        &Protocol::floquet_code(),
        0,
        1,
        "physical",
        "logical",
    );
}

#[test]
fn units_for_distance_test_position_0_distance_3() {
    create_and_test(
        PhysicalQubit::default(),
        &Protocol::default(),
        0,
        3,
        "logical",
        "physical",
    );
}

#[test]
fn units_for_distance_test_position_1_distance_1() {
    create_and_test(
        PhysicalQubit::default(),
        &Protocol::default(),
        1,
        1,
        "logical",
        "physical",
    );
}

#[test]
fn units_for_distance_test_position_1_distance_5() {
    create_and_test(
        PhysicalQubit::default(),
        &Protocol::default(),
        1,
        5,
        "logical",
        "physical",
    );
}

fn create_templates_list_2_2_2() -> Vec<TFactoryDistillationUnitTemplate> {
    let combined_template1 =
        create_distillation_unit_template("combined1", TFactoryDistillationUnitType::Combined);

    let logical_template1 =
        create_distillation_unit_template("logical1", TFactoryDistillationUnitType::Logical);

    let physical_template1 =
        create_distillation_unit_template("physical1", TFactoryDistillationUnitType::Physical);

    let combined_template2 =
        create_distillation_unit_template("combined2", TFactoryDistillationUnitType::Combined);

    let logical_template2 =
        create_distillation_unit_template("logical2", TFactoryDistillationUnitType::Logical);

    let physical_template2 =
        create_distillation_unit_template("physical2", TFactoryDistillationUnitType::Physical);

    Vec::from([
        combined_template1,
        logical_template1,
        physical_template1,
        combined_template2,
        logical_template2,
        physical_template2,
    ])
}

fn create_templates_list_0_2_1() -> Vec<TFactoryDistillationUnitTemplate> {
    let logical_template1 =
        create_distillation_unit_template("logical1", TFactoryDistillationUnitType::Logical);

    let physical_template1 =
        create_distillation_unit_template("physical1", TFactoryDistillationUnitType::Physical);

    let logical_template2 =
        create_distillation_unit_template("logical2", TFactoryDistillationUnitType::Logical);

    Vec::from([logical_template1, physical_template1, logical_template2])
}

#[test]
fn test_map_creation_no_purely_physical_templates_filtered_out_by_is_valid_condition() {
    let templates = create_templates_list_2_2_2();
    let ftp = Protocol::default();
    let map = create_default_map(PhysicalQubit::default(), &ftp, &templates);
    assert!(map.num_physical_distillation_units == 0);
    assert!(map.num_logical_distillation_units == 2);
    assert!(map.num_combined_distillation_units == 2);
    assert_eq!(map.get(0, 1, 0).map(|i| i.name.as_str()), Some("combined1"));
    assert_eq!(map.get(0, 1, 1).map(|i| i.name.as_str()), Some("combined2"));

    assert_eq!(map.get(0, 3, 0).map(|i| i.name.as_str()), Some("combined1"));
    assert_eq!(map.get(0, 3, 1).map(|i| i.name.as_str()), Some("combined2"));
    assert_eq!(map.get(0, 3, 2).map(|i| i.name.as_str()), Some("logical1"));
    assert_eq!(map.get(0, 3, 3).map(|i| i.name.as_str()), Some("logical2"));
}

#[test]
fn test_map_creation_with_purely_physical_templates() {
    let templates = create_templates_list_2_2_2();
    let ftp = Protocol::floquet_code();
    let map = create_default_map(PhysicalQubit::qubit_maj_ns_e4(), &ftp, &templates);
    assert!(map.num_physical_distillation_units == 2);
    assert!(map.num_logical_distillation_units == 2);
    assert!(map.num_combined_distillation_units == 2);
    assert_eq!(map.get(0, 1, 0).map(|i| i.name.as_str()), Some("combined1"));
    assert_eq!(map.get(0, 1, 1).map(|i| i.name.as_str()), Some("combined2"));
    assert_eq!(map.get(0, 1, 4).map(|i| i.name.as_str()), Some("physical1"));
    assert_eq!(map.get(0, 1, 5).map(|i| i.name.as_str()), Some("physical2"));

    assert_eq!(map.get(0, 3, 0).map(|i| i.name.as_str()), Some("combined1"));
    assert_eq!(map.get(0, 3, 1).map(|i| i.name.as_str()), Some("combined2"));
    assert_eq!(map.get(0, 3, 2).map(|i| i.name.as_str()), Some("logical1"));
    assert_eq!(map.get(0, 3, 3).map(|i| i.name.as_str()), Some("logical2"));
}

#[test]
fn test_map_creation_with_purely_physical_and_no_combined_templates() {
    let templates = create_templates_list_0_2_1();
    let ftp = Protocol::floquet_code();
    let map = create_default_map(PhysicalQubit::qubit_maj_ns_e4(), &ftp, &templates);
    assert!(map.num_physical_distillation_units == 1);
    assert!(map.num_logical_distillation_units == 2);
    assert!(map.num_combined_distillation_units == 0);
    assert_eq!(map.get(0, 1, 2).map(|i| i.name.as_str()), Some("physical1"));
    assert_eq!(map.get(0, 3, 0).map(|i| i.name.as_str()), Some("logical1"));
    assert_eq!(map.get(0, 3, 1).map(|i| i.name.as_str()), Some("logical2"));
}

#[test]
fn test_first_round_overrides_applied() {
    let mut templates = create_templates_list_2_2_2();
    templates.push(create_distillation_unit_template_with_override());
    templates.push(create_distillation_unit_template_without_override());
    let ftp = Protocol::floquet_code();
    let map = create_default_map(PhysicalQubit::qubit_maj_ns_e4(), &ftp, &templates);

    assert!(map.num_physical_distillation_units == 2);
    assert!(map.num_logical_distillation_units == 2);
    assert!(map.num_combined_distillation_units == 4);

    // physical
    assert_eq!(
        map.get(0, 1, 2)
            .map(|i| (i.name.as_str(), i.physical_qubits(0), i.duration(0))),
        Some(("combined with override", 1, 200))
    );

    // physical
    assert_eq!(
        map.get(0, 1, 3)
            .map(|i| (i.name.as_str(), i.physical_qubits(0), i.duration(0))),
        Some(("combined without override", 1, 200))
    );

    // logical at subsequent round
    assert_eq!(
        map.get(1, 1, 2)
            .map(|i| (i.name.as_str(), i.physical_qubits(1), i.duration(1))),
        Some(("combined with override", 12, 1200))
    );

    // logical at subsequent round
    assert_eq!(
        map.get(1, 1, 3)
            .map(|i| (i.name.as_str(), i.physical_qubits(1), i.duration(1))),
        Some(("combined without override", 12, 1200))
    );

    // logical at first round. override
    assert_eq!(
        map.get(0, 3, 2)
            .map(|i| (i.name.as_str(), i.physical_qubits(0), i.duration(0))),
        Some(("combined with override", 260, 5400))
    );

    // logical at first round. no override
    assert_eq!(
        map.get(0, 3, 3)
            .map(|i| (i.name.as_str(), i.physical_qubits(0), i.duration(0))),
        Some(("combined without override", 156, 3600))
    );

    // logical at subsequent round
    assert_eq!(
        map.get(1, 3, 2)
            .map(|i| (i.name.as_str(), i.physical_qubits(1), i.duration(1))),
        Some(("combined with override", 156, 3600))
    );

    // logical at subsequent round
    assert_eq!(
        map.get(1, 3, 3)
            .map(|i| (i.name.as_str(), i.physical_qubits(1), i.duration(1))),
        Some(("combined without override", 156, 3600))
    );
}

#[test]
fn iterate_for_all_distillation_units_test() {
    let templates = create_templates_list_2_2_2();
    let ftp = Protocol::floquet_code();
    let map = create_default_map(PhysicalQubit::qubit_maj_ns_e4(), &ftp, &templates);

    let mut hashmap = FxHashSet::default();
    let mut callback = |indexes: &[usize]| {
        hashmap.insert(indexes[0]);
    };
    map.iterate_for_all_distillation_units(1, &mut callback);
    assert_eq!(hashmap.len(), 6);

    let mut hashmap = FxHashSet::default();
    let mut callback = |indexes: &[usize]| {
        hashmap.insert(indexes[0] * 10 + indexes[1]);
    };
    map.iterate_for_all_distillation_units(2, &mut callback);
    // 6 on the first position, 4 on the second (not purely physical)
    assert_eq!(hashmap.len(), 24);

    let mut hashmap = FxHashSet::default();
    let mut callback = |indexes: &[usize]| {
        hashmap.insert(indexes[0] * 100 + indexes[1] * 10 + indexes[2]);
    };
    map.iterate_for_all_distillation_units(3, &mut callback);
    // 6 on the first position, 4 on the second (not purely physical), 4 on the third (not purely physical)
    assert_eq!(hashmap.len(), 96);
}

#[test]
fn iterate_for_all_distillation_units_0_2_1_test() {
    let templates = create_templates_list_0_2_1();
    let ftp = Protocol::floquet_code();
    let map = create_default_map(PhysicalQubit::qubit_maj_ns_e4(), &ftp, &templates);
    let mut hashmap = FxHashSet::default();
    let mut callback = |indexes: &[usize]| {
        hashmap.insert(indexes[0]);
    };
    map.iterate_for_all_distillation_units(1, &mut callback);
    assert_eq!(hashmap.len(), 3);

    let mut hashmap = FxHashSet::default();
    let mut callback = |indexes: &[usize]| {
        hashmap.insert(indexes[0] * 10 + indexes[1]);
    };
    map.iterate_for_all_distillation_units(2, &mut callback);
    // 3 on the first position, 2 on the second (not purely physical)
    assert_eq!(hashmap.len(), 6);
}
