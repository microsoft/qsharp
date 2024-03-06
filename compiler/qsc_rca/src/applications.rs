// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    common::{initialize_locals_map, InputParam, InputParamIndex, Local, LocalKind, LocalsLookup},
    scaffolding::PackageComputeProperties,
    ApplicationGeneratorSet, ComputeKind, QuantumProperties, RuntimeFeatureFlags, RuntimeKind,
    ValueKind,
};
use qsc_fir::{
    fir::{BlockId, ExprId, LocalVarId, StmtId},
    ty::Ty,
};
use rustc_hash::FxHashMap;

/// Auxiliary data structure used to build multiple related application generator sets from individual application
/// instances for a particular callable specialization.
#[derive(Debug)]
pub struct GeneratorSetsBuilder {
    inherent: ApplicationInstance,
    dynamic_param_applications: Vec<ParamApplication>,
    _current_application: BuilderItemKey,
}

impl GeneratorSetsBuilder {
    /// Creates a new builder.
    pub fn new(input_params: &Vec<InputParam>, controls: Option<&Local>, return_type: &Ty) -> Self {
        let inherent = ApplicationInstance::new(input_params, controls, return_type, None);
        let mut dynamic_param_applications =
            Vec::<ParamApplication>::with_capacity(input_params.len());
        for input_param in input_params {
            let param_application =
                ParamApplication::new(input_param.index, input_params, controls, return_type);
            dynamic_param_applications.push(param_application);
        }

        Self {
            inherent,
            dynamic_param_applications,
            _current_application: BuilderItemKey::Inherent,
        }
    }

    pub fn advance_current_application_instance(&mut self) -> bool {
        unimplemented!();
    }

    pub fn get_current_application_instance(&self) -> &ApplicationInstance {
        unimplemented!();
    }

    pub fn get_current_application_instance_mut(&self) -> &mut ApplicationInstance {
        unimplemented!();
    }

    /// Saves the contents of the builder to the package compute properties data structure.
    /// If a main block ID is provided, it returns the applications generator set representing the block.
    pub fn save_to_package_compute_properties(
        self,
        package_compute_properties: &mut PackageComputeProperties,
        main_block: Option<BlockId>,
    ) -> Option<ApplicationGeneratorSet> {
        // Get the compute properties of the inherent application instance and the non-static parameter applications.
        let input_params_count = self.dynamic_param_applications.len();
        let mut inherent_application_compute_properties = self.inherent.close();
        let mut non_static_param_applications_compute_properties =
            Vec::<ParamApplicationComputeProperties>::with_capacity(input_params_count);
        for param_application in self.dynamic_param_applications {
            let param_application_compute_properties = match param_application {
                ParamApplication::Array(array_param_application) => {
                    ParamApplicationComputeProperties::Array(
                        ArrayParamApplicationComputeProperties {
                            static_content_dynamic_size: array_param_application
                                .static_content_dynamic_size
                                .close(),
                            dynamic_content_static_size: array_param_application
                                .dynamic_content_static_size
                                .close(),
                            dynamic_content_dynamic_size: array_param_application
                                .dynamic_content_dynamic_size
                                .close(),
                        },
                    )
                }
                ParamApplication::Element(application_instance) => {
                    ParamApplicationComputeProperties::Element(application_instance.close())
                }
            };
            non_static_param_applications_compute_properties
                .push(param_application_compute_properties);
        }

        // Save the compute properties to the package.
        Self::save_application_generator_sets(
            &mut inherent_application_compute_properties,
            &mut non_static_param_applications_compute_properties,
            package_compute_properties,
        );

        // If a main block was provided, create an applications generator that represents the specialization based on
        // the applications generator of the main block.
        let close_output = main_block.map(|main_block_id| {
            let mut applications_generator = package_compute_properties
                .blocks
                .get(main_block_id)
                .expect("block applications generator should exist")
                .clone();
            assert!(
                applications_generator.dynamic_param_applications.len()
                    == non_static_param_applications_compute_properties.len()
            );

            // Update the value kind of the generator's inherent compute kind.
            if let Some(inherent_value_kind) = inherent_application_compute_properties.value_kind {
                applications_generator
                    .inherent
                    .aggregate_value_kind(inherent_value_kind);
            }

            // Update the value kind of the generator's param applications.
            for (param_application, compute_properties) in applications_generator
                .dynamic_param_applications
                .iter_mut()
                .zip(non_static_param_applications_compute_properties)
            {
                Self::aggregate_param_application_value_kind(param_application, &compute_properties)
            }

            // Return the applications gene with the updated dynamism sources.
            applications_generator
        });

        close_output
    }

    fn aggregate_param_application_value_kind(
        param_application: &mut crate::ParamApplication,
        compute_properties: &ParamApplicationComputeProperties,
    ) {
        match param_application {
            crate::ParamApplication::Array(array_param_application) => {
                let ParamApplicationComputeProperties::Array(array_compute_properties) =
                    compute_properties
                else {
                    panic!("expected an array param application");
                };
                array_compute_properties
                    .static_content_dynamic_size
                    .value_kind
                    .iter()
                    .for_each(|value_kind| {
                        array_param_application
                            .static_content_dynamic_size
                            .aggregate_value_kind(*value_kind)
                    });
                array_compute_properties
                    .dynamic_content_static_size
                    .value_kind
                    .iter()
                    .for_each(|value_kind| {
                        array_param_application
                            .dynamic_content_static_size
                            .aggregate_value_kind(*value_kind)
                    });
                array_compute_properties
                    .dynamic_content_dynamic_size
                    .value_kind
                    .iter()
                    .for_each(|value_kind| {
                        array_param_application
                            .dynamic_content_dynamic_size
                            .aggregate_value_kind(*value_kind)
                    });
            }
            crate::ParamApplication::Element(element_param_application) => {
                let ParamApplicationComputeProperties::Element(element_compute_properties) =
                    compute_properties
                else {
                    panic!("expected an element param application");
                };
                element_compute_properties
                    .value_kind
                    .iter()
                    .for_each(|value_kind| {
                        element_param_application.aggregate_value_kind(*value_kind)
                    });
            }
        };
    }

    fn save_application_generator_sets(
        inherent_application_compute_properties: &mut ApplicationInstanceComputeProperties,
        non_static_param_applications_compute_properties: &mut Vec<
            ParamApplicationComputeProperties,
        >,
        package_compute_properties: &mut PackageComputeProperties,
    ) {
        let input_params_count = non_static_param_applications_compute_properties.len();

        // Save an applications generator set for each block using their compute properties.
        for (block_id, block_inherent_compute_kind) in
            inherent_application_compute_properties.blocks.drain()
        {
            let mut block_non_static_param_applications =
                Vec::<crate::ParamApplication>::with_capacity(input_params_count);
            for param_application_compute_properties in
                non_static_param_applications_compute_properties.iter_mut()
            {
                let param_application = param_application_compute_properties
                    .remove_item(ApplicationInstanceItem::Block(block_id));
                block_non_static_param_applications.push(param_application);
            }
            let application_generator_set = ApplicationGeneratorSet {
                inherent: block_inherent_compute_kind,
                dynamic_param_applications: block_non_static_param_applications,
            };
            package_compute_properties
                .blocks
                .insert(block_id, application_generator_set);
        }

        // Save an applications generator set for each statement using their compute properties.
        for (stmt_id, stmt_inherent_compute_kind) in
            inherent_application_compute_properties.stmts.drain()
        {
            let mut stmt_non_static_param_applications =
                Vec::<crate::ParamApplication>::with_capacity(input_params_count);
            for param_application_compute_properties in
                non_static_param_applications_compute_properties.iter_mut()
            {
                let param_application = param_application_compute_properties
                    .remove_item(ApplicationInstanceItem::Stmt(stmt_id));
                stmt_non_static_param_applications.push(param_application);
            }
            let application_generator_set = ApplicationGeneratorSet {
                inherent: stmt_inherent_compute_kind,
                dynamic_param_applications: stmt_non_static_param_applications,
            };
            package_compute_properties
                .stmts
                .insert(stmt_id, application_generator_set);
        }

        // Save an applications generator set for each expression using their compute properties.
        for (expr_id, expr_inherent_compute_kind) in
            inherent_application_compute_properties.exprs.drain()
        {
            let mut expr_non_static_param_applications =
                Vec::<crate::ParamApplication>::with_capacity(input_params_count);
            for param_application_compute_properties in
                non_static_param_applications_compute_properties.iter_mut()
            {
                let param_application = param_application_compute_properties
                    .remove_item(ApplicationInstanceItem::Expr(expr_id));
                expr_non_static_param_applications.push(param_application);
            }
            let application_generator_set = ApplicationGeneratorSet {
                inherent: expr_inherent_compute_kind,
                dynamic_param_applications: expr_non_static_param_applications,
            };
            package_compute_properties
                .exprs
                .insert(expr_id, application_generator_set);
        }
    }
}

/// An instance of a callable application.
#[derive(Debug, Default)]
pub struct ApplicationInstance {
    /// A map of locals with their associated compute kind.
    pub locals_map: LocalsComputeKindMap,
    /// The currently active dynamic scopes in the application instance.
    pub active_dynamic_scopes: Vec<ExprId>,
    /// The return expressions throughout the application instance.
    pub return_expressions: Vec<ExprId>,
    /// The return type of the application instance.
    return_type: Ty,
    /// The compute kind of the blocks related to the application instance.
    blocks: FxHashMap<BlockId, ComputeKind>,
    /// The compute kind of the statements related to the application instance.
    stmts: FxHashMap<StmtId, ComputeKind>,
    /// The compute kind of the expressions related to the application instance.
    exprs: FxHashMap<ExprId, ComputeKind>,
}

impl ApplicationInstance {
    pub fn find_block_compute_kind(&self, id: BlockId) -> Option<&ComputeKind> {
        self.blocks.get(&id)
    }

    pub fn find_expr_compute_kind(&self, id: ExprId) -> Option<&ComputeKind> {
        self.exprs.get(&id)
    }

    pub fn find_stmt_compute_kind(&self, id: StmtId) -> Option<&ComputeKind> {
        self.stmts.get(&id)
    }

    pub fn get_block_compute_kind(&self, id: BlockId) -> &ComputeKind {
        self.find_block_compute_kind(id)
            .expect("block compute kind should exist in application instance")
    }

    pub fn get_expr_compute_kind(&self, id: ExprId) -> &ComputeKind {
        self.find_expr_compute_kind(id)
            .expect("expression compute kind should exist in application instance")
    }

    pub fn get_stmt_compute_kind(&self, id: StmtId) -> &ComputeKind {
        self.find_stmt_compute_kind(id)
            .expect("expression compute kind should exist in application instance")
    }

    pub fn insert_block_compute_kind(&mut self, id: BlockId, value: ComputeKind) {
        self.blocks.insert(id, value);
    }

    pub fn insert_expr_compute_kind(&mut self, id: ExprId, value: ComputeKind) {
        self.exprs.insert(id, value);
    }

    pub fn insert_stmt_compute_kind(&mut self, id: StmtId, value: ComputeKind) {
        self.stmts.insert(id, value);
    }

    fn new(
        input_params: &Vec<InputParam>,
        controls: Option<&Local>,
        return_type: &Ty,
        non_static_param: Option<NonStaticParamDescriptor>,
    ) -> Self {
        // Initialize the locals map with the specialization controls (if any).
        let mut locals_map = LocalsComputeKindMap::default();
        if let Some(controls) = controls {
            // Controls compute properties are handled at the call expression, so just use quantum compute kind with
            // no runtime features here.
            let compute_kind = ComputeKind::Quantum(QuantumProperties {
                runtime_features: RuntimeFeatureFlags::empty(),
                value_kind: ValueKind::Array(RuntimeKind::Static, RuntimeKind::Static),
            });
            locals_map.insert(
                controls.var,
                LocalComputeKind {
                    local: controls.clone(),
                    compute_kind,
                },
            );
        }

        let mut unprocessed_locals_map = initialize_locals_map(input_params);
        for (local_var_id, local) in unprocessed_locals_map.drain() {
            let LocalKind::InputParam(input_param_index) = local.kind else {
                panic!("only input parameters are expected");
            };

            // If a non-static param is provided, set the compute kind associated to the parameter accordingly.
            let compute_kind = if let Some(non_static_param_descriptor) = non_static_param {
                if input_param_index == non_static_param_descriptor.index {
                    let value_kind = ValueKind::from(non_static_param_descriptor.kind);
                    ComputeKind::Quantum(QuantumProperties {
                        runtime_features: RuntimeFeatureFlags::empty(),
                        value_kind,
                    })
                } else {
                    ComputeKind::Classical
                }
            } else {
                ComputeKind::Classical
            };

            locals_map.insert(
                local_var_id,
                LocalComputeKind {
                    local,
                    compute_kind,
                },
            );
        }
        Self {
            locals_map,
            active_dynamic_scopes: Vec::new(),
            return_expressions: Vec::new(),
            return_type: return_type.clone(),
            blocks: FxHashMap::default(),
            stmts: FxHashMap::default(),
            exprs: FxHashMap::default(),
        }
    }

    fn close(self) -> ApplicationInstanceComputeProperties {
        // Determine the value kind of the application instance by going through each return expression aggregating
        // their value kind (if any).
        let mut value_kinds = Vec::<ValueKind>::new();
        for expr_id in self.return_expressions {
            let expr_compute_kind = self
                .exprs
                .get(&expr_id)
                .expect("expression compute kind should exist");
            if let ComputeKind::Quantum(quantum_properties) = expr_compute_kind {
                value_kinds.push(quantum_properties.value_kind);
            }
        }

        // The application instance has a value kind only if at least one of its return expressions was quantum.
        let value_kind = if value_kinds.is_empty() {
            None
        } else {
            let initial_value_kind = if let Ty::Array(_) = self.return_type {
                ValueKind::Array(RuntimeKind::Static, RuntimeKind::Static)
            } else {
                ValueKind::Element(RuntimeKind::Static)
            };
            let value_kind = value_kinds.iter().fold(
                initial_value_kind,
                |aggregated_value_kind, current_value_kind| {
                    aggregated_value_kind.aggregate(*current_value_kind)
                },
            );
            Some(value_kind)
        };

        ApplicationInstanceComputeProperties {
            blocks: self.blocks,
            stmts: self.stmts,
            exprs: self.exprs,
            value_kind,
        }
    }
}

#[derive(Debug, Default)]
pub struct LocalsComputeKindMap(FxHashMap<LocalVarId, LocalComputeKind>);

impl LocalsLookup for LocalsComputeKindMap {
    fn find(&self, local_var_id: LocalVarId) -> Option<&Local> {
        self.0
            .get(&local_var_id)
            .map(|local_compute_kind| &local_compute_kind.local)
    }
}

impl LocalsComputeKindMap {
    pub fn aggregate_compute_kind(&mut self, local_var_id: LocalVarId, delta: ComputeKind) {
        let local_compute_kind = self
            .0
            .get_mut(&local_var_id)
            .expect("compute kind for local should exist");
        local_compute_kind.compute_kind = local_compute_kind.compute_kind.aggregate(delta);
    }

    pub fn find_compute_kind(&self, local_var_id: LocalVarId) -> Option<&ComputeKind> {
        self.0
            .get(&local_var_id)
            .map(|local_compute_kind| &local_compute_kind.compute_kind)
    }

    pub fn get_compute_kind(&self, local_var_id: LocalVarId) -> &ComputeKind {
        self.find_compute_kind(local_var_id)
            .expect("compute kind for local should exist")
    }

    pub fn insert(&mut self, local_var_id: LocalVarId, value: LocalComputeKind) {
        self.0.insert(local_var_id, value);
    }
}

#[derive(Debug)]
pub struct LocalComputeKind {
    pub local: Local,
    pub compute_kind: ComputeKind,
}

#[derive(Clone, Copy, Debug)]
enum BuilderItemKey {
    Inherent,
    Param(NonStaticParamDescriptor),
}

#[derive(Clone, Copy, Debug)]
struct NonStaticParamDescriptor {
    index: InputParamIndex,
    kind: NonStaticParamDescriptorKind,
}

#[derive(Clone, Copy, Debug)]
enum NonStaticParamDescriptorKind {
    Element,
    Array(NonStaticArrayParamDescriptor),
}

impl From<NonStaticParamDescriptorKind> for ValueKind {
    fn from(value: NonStaticParamDescriptorKind) -> Self {
        match value {
            NonStaticParamDescriptorKind::Element => ValueKind::Element(RuntimeKind::Dynamic),
            NonStaticParamDescriptorKind::Array(array_descriptor) => match array_descriptor {
                NonStaticArrayParamDescriptor::StaticContentDynamicSize => {
                    ValueKind::Array(RuntimeKind::Static, RuntimeKind::Dynamic)
                }
                NonStaticArrayParamDescriptor::DynamicContentStaticSize => {
                    ValueKind::Array(RuntimeKind::Dynamic, RuntimeKind::Static)
                }
                NonStaticArrayParamDescriptor::DynamicContentDynamicSize => {
                    ValueKind::Array(RuntimeKind::Dynamic, RuntimeKind::Dynamic)
                }
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum NonStaticArrayParamDescriptor {
    StaticContentDynamicSize,
    DynamicContentStaticSize,
    DynamicContentDynamicSize,
}

/// Application instance(s) related to a parameter application.
#[derive(Debug)]
enum ParamApplication {
    Element(ApplicationInstance),
    Array(ArrayParamApplication),
}

impl ParamApplication {
    fn new(
        input_param_index: InputParamIndex,
        input_params: &Vec<InputParam>,
        controls: Option<&Local>,
        return_type: &Ty,
    ) -> Self {
        let input_param = input_params
            .get(usize::from(input_param_index))
            .expect("input parameter at index should exist");
        match input_param.ty {
            Ty::Array(_) => {
                let static_content_dynamic_size = ApplicationInstance::new(
                    input_params,
                    controls,
                    return_type,
                    Some(NonStaticParamDescriptor {
                        index: input_param_index,
                        kind: NonStaticParamDescriptorKind::Array(
                            NonStaticArrayParamDescriptor::StaticContentDynamicSize,
                        ),
                    }),
                );
                let dynamic_content_static_size = ApplicationInstance::new(
                    input_params,
                    controls,
                    return_type,
                    Some(NonStaticParamDescriptor {
                        index: input_param_index,
                        kind: NonStaticParamDescriptorKind::Array(
                            NonStaticArrayParamDescriptor::DynamicContentStaticSize,
                        ),
                    }),
                );
                let dynamic_content_dynamic_size = ApplicationInstance::new(
                    input_params,
                    controls,
                    return_type,
                    Some(NonStaticParamDescriptor {
                        index: input_param_index,
                        kind: NonStaticParamDescriptorKind::Array(
                            NonStaticArrayParamDescriptor::DynamicContentDynamicSize,
                        ),
                    }),
                );
                Self::Array(ArrayParamApplication {
                    static_content_dynamic_size,
                    dynamic_content_static_size,
                    dynamic_content_dynamic_size,
                })
            }
            _ => Self::Element(ApplicationInstance::new(
                input_params,
                controls,
                return_type,
                Some(NonStaticParamDescriptor {
                    index: input_param_index,
                    kind: NonStaticParamDescriptorKind::Element,
                }),
            )),
        }
    }
}

/// Application instances related to a parameter application for a parameter of type array.
#[derive(Debug)]
struct ArrayParamApplication {
    static_content_dynamic_size: ApplicationInstance,
    dynamic_content_static_size: ApplicationInstance,
    dynamic_content_dynamic_size: ApplicationInstance,
}

enum ParamApplicationComputeProperties {
    Element(ApplicationInstanceComputeProperties),
    Array(ArrayParamApplicationComputeProperties),
}

impl ParamApplicationComputeProperties {
    fn remove_item(&mut self, item: ApplicationInstanceItem) -> crate::ParamApplication {
        match self {
            Self::Element(compute_properties) => {
                let compute_kind = compute_properties.remove(item);
                crate::ParamApplication::Element(compute_kind)
            }
            Self::Array(array_param) => {
                let static_content_dynamic_size =
                    array_param.static_content_dynamic_size.remove(item);
                let dynamic_content_static_size =
                    array_param.dynamic_content_static_size.remove(item);
                let dynamic_content_dynamic_size =
                    array_param.dynamic_content_dynamic_size.remove(item);
                crate::ParamApplication::Array(crate::ArrayParamApplication {
                    static_content_dynamic_size,
                    dynamic_content_static_size,
                    dynamic_content_dynamic_size,
                })
            }
        }
    }
}

#[allow(clippy::struct_field_names)]
struct ArrayParamApplicationComputeProperties {
    static_content_dynamic_size: ApplicationInstanceComputeProperties,
    dynamic_content_static_size: ApplicationInstanceComputeProperties,
    dynamic_content_dynamic_size: ApplicationInstanceComputeProperties,
}

struct ApplicationInstanceComputeProperties {
    blocks: FxHashMap<BlockId, ComputeKind>,
    stmts: FxHashMap<StmtId, ComputeKind>,
    exprs: FxHashMap<ExprId, ComputeKind>,
    value_kind: Option<ValueKind>,
}

#[derive(Clone, Copy)]
enum ApplicationInstanceItem {
    Block(BlockId),
    Expr(ExprId),
    Stmt(StmtId),
}

impl ApplicationInstanceComputeProperties {
    fn remove(&mut self, item: ApplicationInstanceItem) -> ComputeKind {
        match item {
            ApplicationInstanceItem::Block(block_id) => self.remove_block(block_id),
            ApplicationInstanceItem::Expr(expr_id) => self.remove_expr(expr_id),
            ApplicationInstanceItem::Stmt(stmt_id) => self.remove_stmt(stmt_id),
        }
    }

    fn remove_block(&mut self, id: BlockId) -> ComputeKind {
        self.blocks.remove(&id).expect(
            "block to be removed should exist in the compute properties of the application instance",
        )
    }

    fn remove_stmt(&mut self, id: StmtId) -> ComputeKind {
        self.stmts.remove(&id).expect(
            "statement to be removed should exist in the compute properties of the application instance",
        )
    }

    fn remove_expr(&mut self, id: ExprId) -> ComputeKind {
        self.exprs.remove(&id).expect(
            "expression to be removed should exist in the compute properties of the application instance",
        )
    }
}
