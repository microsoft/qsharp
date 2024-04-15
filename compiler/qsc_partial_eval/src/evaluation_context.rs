// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_data_structures::functors::FunctorApp;
use qsc_eval::{val::Value, Env, Variable};
use qsc_fir::fir::{ExprId, LocalItemId, LocalVarId, PackageId};
use qsc_rca::{RuntimeKind, ValueKind};
use qsc_rir::rir::BlockId;
use rustc_hash::FxHashMap;

pub struct EvaluationContext {
    active_blocks: Vec<BlockNode>,
    scopes: Vec<Scope>,
}

impl EvaluationContext {
    pub fn new(package_id: PackageId, initial_block: BlockId) -> Self {
        let entry_callable_scope = Scope::new(package_id, None, Vec::new());
        Self {
            active_blocks: vec![BlockNode {
                id: initial_block,
                next: None,
            }],
            scopes: vec![entry_callable_scope],
        }
    }

    pub fn get_current_block_id(&self) -> BlockId {
        self.active_blocks.last().expect("no active blocks").id
    }

    pub fn get_current_scope(&self) -> &Scope {
        self.scopes
            .last()
            .expect("the evaluation context does not have a current scope")
    }

    pub fn get_current_scope_mut(&mut self) -> &mut Scope {
        self.scopes
            .last_mut()
            .expect("the evaluation context does not have a current scope")
    }

    pub fn pop_block_node(&mut self) -> BlockNode {
        self.active_blocks
            .pop()
            .expect("there are no active blocks in the evaluation context")
    }

    pub fn pop_scope(&mut self) -> Scope {
        self.scopes
            .pop()
            .expect("there are no scopes in the evaluation context")
    }

    pub fn push_block_node(&mut self, b: BlockNode) {
        self.active_blocks.push(b);
    }

    pub fn push_scope(&mut self, s: Scope) {
        self.scopes.push(s);
    }
}

pub struct BlockNode {
    pub id: BlockId,
    pub next: Option<BlockId>,
}

pub struct Scope {
    pub package_id: PackageId,
    pub callable: Option<(LocalItemId, FunctorApp)>,
    pub args_value_kind: Vec<ValueKind>,
    pub env: Env,
    hybrid_exprs: FxHashMap<ExprId, Value>,
    hybrid_vars: FxHashMap<LocalVarId, Value>,
}

impl Scope {
    pub fn new(
        package_id: PackageId,
        callable: Option<(LocalItemId, FunctorApp)>,
        args: Vec<Arg>,
    ) -> Self {
        // Determine the runtimne kind (static or dynamic) of the arguments.
        let args_runtime_kind: Vec<ValueKind> = args
            .iter()
            .map(|arg| {
                let value = match arg {
                    Arg::Discard(value) => value,
                    Arg::Var(_, var) => &var.value,
                };
                map_eval_value_to_value_kind(value)
            })
            .collect();

        // Add the static values to the environment.
        let mut env = Env::default();
        let mut hybrid_vars = FxHashMap::default();
        let arg_runtime_kind_tuple = args.into_iter().zip(args_runtime_kind.iter());
        for (arg, value_kind) in arg_runtime_kind_tuple {
            let Arg::Var(local_var_id, var) = arg else {
                continue;
            };

            if value_kind.is_dynamic() {
                hybrid_vars.insert(local_var_id, var.value);
            } else {
                env.bind_variable_in_top_frame(local_var_id, var);
            }
        }

        // Add the dynamic values to the hybrid variables
        Self {
            package_id,
            callable,
            args_value_kind: args_runtime_kind,
            env,
            hybrid_exprs: FxHashMap::default(),
            hybrid_vars,
        }
    }

    pub fn get_expr_value(&self, expr_id: ExprId) -> &Value {
        self.hybrid_exprs
            .get(&expr_id)
            .expect("expression value does not exist")
    }

    pub fn get_local_var_value(&self, local_var_id: LocalVarId) -> &Value {
        self.hybrid_vars
            .get(&local_var_id)
            .expect("local variable value does not exist")
    }

    pub fn insert_expr_value(&mut self, expr_id: ExprId, value: Value) {
        self.hybrid_exprs.insert(expr_id, value);
    }

    pub fn insert_local_var_value(&mut self, local_var_id: LocalVarId, value: Value) {
        self.hybrid_vars.insert(local_var_id, value);
    }
}

fn map_eval_value_to_value_kind(value: &Value) -> ValueKind {
    fn map_array_eval_value_to_value_kind(elements: &[Value]) -> ValueKind {
        let mut content_runtime_kind = RuntimeKind::Static;
        for element in elements {
            let element_value_kind = map_eval_value_to_value_kind(element);
            if element_value_kind.is_dynamic() {
                content_runtime_kind = RuntimeKind::Dynamic;
                break;
            }
        }

        // We assume the size of all arrays is static.
        ValueKind::Array(content_runtime_kind, RuntimeKind::Static)
    }

    fn map_tuple_eval_value_to_value_kind(elements: &[Value]) -> ValueKind {
        let mut runtime_kind = RuntimeKind::Static;
        for element in elements {
            let element_value_kind = map_eval_value_to_value_kind(element);
            if element_value_kind.is_dynamic() {
                runtime_kind = RuntimeKind::Dynamic;
                break;
            }
        }
        ValueKind::Element(runtime_kind)
    }

    match value {
        Value::Array(elements) => map_array_eval_value_to_value_kind(elements),
        Value::Tuple(elements) => map_tuple_eval_value_to_value_kind(elements),
        Value::Qubit(_) | Value::Result(_) | Value::Var(_) => {
            ValueKind::Element(RuntimeKind::Dynamic)
        }
        Value::BigInt(_)
        | Value::Bool(_)
        | Value::Closure(_)
        | Value::Double(_)
        | Value::Global(_, _)
        | Value::Int(_)
        | Value::Pauli(_)
        | Value::Range(_)
        | Value::String(_) => ValueKind::Element(RuntimeKind::Static),
    }
}

pub enum Arg {
    Discard(Value),
    Var(LocalVarId, Variable),
}
