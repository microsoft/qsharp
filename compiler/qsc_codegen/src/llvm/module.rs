// Copyright (c) 2019 Craig Disselkoen
// Portions copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::constant::ConstantRef;
use super::debugloc::{DebugLoc, HasDebugLoc};
use super::function::{Attribute, Declaration, Function, GroupID};
use super::types::{Builder, TypeRef};
use std::fmt;

/// See [LLVM 14 docs on Module Structure](https://releases.llvm.org/14.0.0/docs/LangRef.html#module-structure)
#[derive(Clone)]
pub struct Module {
    /// The name of the module
    pub name: String,
    /// See [LLVM 14 docs on Source Filename](https://releases.llvm.org/14.0.0/docs/LangRef.html#source-filename)
    pub source_file_name: String,
    /// Functions which are defined (not just declared) in this `Module`.
    /// See [LLVM 14 docs on Functions](https://releases.llvm.org/14.0.0/docs/LangRef.html#functions)
    pub functions: Vec<Function>,
    /// Functions which are declared (but not defined) in this `Module`.
    /// See [LLVM 14 docs on Functions](https://releases.llvm.org/14.0.0/docs/LangRef.html#functions)
    pub func_declarations: Vec<Declaration>,
    /// See [LLVM 14 docs on Global Variables](https://releases.llvm.org/14.0.0/docs/LangRef.html#global-variables)
    pub global_vars: Vec<GlobalVariable>,
    /// See [LLVM 14 docs on Attribute Groups](https://releases.llvm.org/14.0.0/docs/LangRef.html#attribute-groups)
    pub function_attribute_groups: Vec<FunctionAttributeGroup>,
    /// Holds a reference to all of the `Type`s used in the `Module`, and
    /// facilitates lookups so you can get a `TypeRef` to the `Type` you want.
    pub ty_builder: Builder,
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "; ModuleID = '<{}>'", self.name)?;
        writeln!(f)?;
        writeln!(f, "{}", self.ty_builder)?;
        for global_var in &self.global_vars {
            writeln!(f, "{global_var}")?;
        }
        for func_decl in &self.func_declarations {
            writeln!(f, "{func_decl}")?;
        }
        for func in &self.functions {
            writeln!(f, "{func}")?;
        }
        Ok(())
    }
}

impl Module {
    /// Get the `Function` having the given `name` (if any).
    /// Note that functions are named with `String`s and not `Name`s.
    ///
    /// Note also that this will only find _fully defined_ functions, not
    /// `FunctionDeclaration`s.
    #[must_use]
    pub fn get_func_by_name(&self, name: &str) -> Option<&Function> {
        self.functions.iter().find(|func| func.name == name)
    }

    /// Get the `FunctionDeclaration` having the given `name` (if any).
    /// Note that functions are named with `String`s and not `Name`s.
    ///
    /// Note also that this will only find function _declarations_, and will not
    /// find defined functions (use `get_func_by_name()` for that).
    #[must_use]
    pub fn get_func_decl_by_name(&self, name: &str) -> Option<&Declaration> {
        self.func_declarations.iter().find(|decl| decl.name == name)
    }

    /// Get the `GlobalVariable` having the given `name` (if any).
    /// Note that `GlobalVariable`s are named with `String`s and not `Name`s.
    #[must_use]
    pub fn get_global_var_by_name(&self, name: &str) -> Option<&GlobalVariable> {
        self.global_vars.iter().find(|global| global.name == name)
    }
}

/// See [LLVM 14 docs on Global Variables](https://releases.llvm.org/14.0.0/docs/LangRef.html#global-variables)
#[derive(PartialEq, Clone, Debug)]
pub struct GlobalVariable {
    /// Globals' names must be strings
    pub name: String,
    pub linkage: Linkage,
    pub is_constant: bool,
    pub ty: TypeRef,
    pub initializer: Option<ConstantRef>,
    pub debugloc: Option<DebugLoc>,
}

impl HasDebugLoc for GlobalVariable {
    fn get_debug_loc(&self) -> &Option<DebugLoc> {
        &self.debugloc
    }
}

impl fmt::Display for GlobalVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "@{} = {} {} {}",
            self.name,
            self.linkage,
            if self.is_constant {
                "constant"
            } else {
                "global"
            },
            self.ty,
        )?;
        if let Some(init) = &self.initializer {
            write!(f, " {init}")?;
        }
        writeln!(f)
    }
}

/// See [LLVM 14 docs on Linkage Types](https://releases.llvm.org/14.0.0/docs/LangRef.html#linkage)
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Linkage {
    Private,
    Internal,
    External,
    ExternalWeak,
    AvailableExternally,
    LinkOnceAny,
    LinkOnceODR,
    LinkOnceODRAutoHide,
    WeakAny,
    WeakODR,
    Common,
    Appending,
    DLLImport,
    DLLExport,
    Ghost,
    LinkerPrivate,
    LinkerPrivateWeak,
}

impl fmt::Display for Linkage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Linkage::Private => write!(f, "private"),
            Linkage::Internal => write!(f, "internal"),
            Linkage::External => write!(f, "external"),
            Linkage::ExternalWeak => write!(f, "extern_weak"),
            Linkage::AvailableExternally => write!(f, "available_externally"),
            Linkage::LinkOnceAny => write!(f, "linkonce"),
            Linkage::LinkOnceODR => write!(f, "linkonce_odr"),
            Linkage::LinkOnceODRAutoHide => todo!(),
            Linkage::WeakAny => todo!(),
            Linkage::WeakODR => write!(f, "weak_odr"),
            Linkage::Common => write!(f, "common"),
            Linkage::Appending => write!(f, "appending"),
            Linkage::DLLImport => todo!(),
            Linkage::DLLExport => todo!(),
            Linkage::Ghost => todo!(),
            Linkage::LinkerPrivate => todo!(),
            Linkage::LinkerPrivateWeak => todo!(),
        }
    }
}

/// See [LLVM 14 docs on Attribute Groups](https://releases.llvm.org/14.0.0/docs/LangRef.html#attribute-groups)
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct FunctionAttributeGroup {
    pub group_id: GroupID,
    pub attrs: Vec<Attribute>,
}
