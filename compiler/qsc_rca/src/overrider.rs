// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    scaffolding::PackageStoreComputeProperties, ApplicationGeneratorSet, ArrayParamApplication,
    ComputeKind, ParamApplication, QuantumProperties, RuntimeFeatureFlags, RuntimeKind, ValueKind,
};
use qsc_fir::{fir::PackageStore, ty::FunctorSetValue};
use rustc_hash::FxHashMap;
use std::rc::Rc;

struct ApplicationGeneratorSetOverride {
    _functor_set_value: FunctorSetValue,
    _application_generator_set: ApplicationGeneratorSet,
}

pub struct Overrider<'a> {
    _package_store: &'a PackageStore,
    package_store_compute_properties: PackageStoreComputeProperties,
    _spec_compute_properties: FxHashMap<Rc<str>, Vec<ApplicationGeneratorSetOverride>>,
}

impl<'a> Overrider<'a> {
    pub fn new(
        package_store: &'a PackageStore,
        package_store_compute_properties: PackageStoreComputeProperties,
    ) -> Self {
        let overrides: [(Rc<str>, Vec<ApplicationGeneratorSetOverride>); 1] = [(
            "Microsoft.Quantum.Core.Length".into(),
            vec![ApplicationGeneratorSetOverride {
                _functor_set_value: FunctorSetValue::Empty,
                _application_generator_set: ApplicationGeneratorSet {
                    inherent: ComputeKind::Classical,
                    dynamic_param_applications: vec![ParamApplication::Array(
                        ArrayParamApplication {
                            static_content_dynamic_size: ComputeKind::Quantum(QuantumProperties {
                                runtime_features: RuntimeFeatureFlags::UseOfDynamicallySizedArray,
                                value_kind: ValueKind::Element(RuntimeKind::Dynamic),
                            }),
                            dynamic_content_static_size: ComputeKind::Quantum(QuantumProperties {
                                runtime_features: RuntimeFeatureFlags::empty(),
                                value_kind: ValueKind::Element(RuntimeKind::Static),
                            }),
                            dynamic_content_dynamic_size: ComputeKind::Quantum(QuantumProperties {
                                runtime_features: RuntimeFeatureFlags::UseOfDynamicallySizedArray,
                                value_kind: ValueKind::Element(RuntimeKind::Dynamic),
                            }),
                        },
                    )],
                },
            }],
        )];
        let mut spec_compute_properties: FxHashMap<Rc<str>, Vec<ApplicationGeneratorSetOverride>> =
            FxHashMap::default();
        for (callable_name, application_generator_set_override) in overrides {
            spec_compute_properties.insert(callable_name, application_generator_set_override);
        }
        Self {
            _package_store: package_store,
            package_store_compute_properties,
            _spec_compute_properties: spec_compute_properties,
        }
    }

    pub fn populate_overrides(self) -> PackageStoreComputeProperties {
        self.package_store_compute_properties
    }
}
