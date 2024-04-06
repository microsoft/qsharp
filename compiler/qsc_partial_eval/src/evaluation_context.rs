// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_data_structures::functors::FunctorApp;
use qsc_eval::{val::Value, Env};
use qsc_fir::fir::{ExprId, LocalItemId, LocalVarId, PackageId};
use qsc_rca::ValueKind;
use qsc_rir::rir::BlockId;
use rustc_hash::FxHashMap;

pub struct EvaluationContext {
    pub current_block: BlockId,
    scopes: Vec<Scope>,
}

impl EvaluationContext {
    pub fn new(entry_package_id: PackageId, initial_block: BlockId) -> Self {
        let entry_callable_scope = Scope::new(entry_package_id, None, Vec::new());
        Self {
            current_block: initial_block,
            scopes: vec![entry_callable_scope],
        }
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

    pub fn pop_scope(&mut self) -> Scope {
        self.scopes
            .pop()
            .expect("there are no scopes in the evaluation context")
    }

    pub fn push_scope(&mut self, s: Scope) {
        self.scopes.push(s)
    }
}

pub struct Scope {
    pub package_id: PackageId,
    pub callable: Option<(LocalItemId, FunctorApp)>,
    pub args_runtime_properties: Vec<ValueKind>,
    pub env: Env,
    hybrid_exprs: FxHashMap<ExprId, Value>,
    hybrid_vars: FxHashMap<LocalVarId, Value>,
}

impl Scope {
    pub fn new(
        package_id: PackageId,
        callable: Option<(LocalItemId, FunctorApp)>,
        args_runtime_properties: Vec<ValueKind>,
    ) -> Self {
        Self {
            package_id,
            callable,
            args_runtime_properties,
            env: Env::default(),
            hybrid_exprs: FxHashMap::default(),
            hybrid_vars: FxHashMap::default(),
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
