// Portions copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::constant::ConstantRef;
use super::debugloc::{DebugLoc, HasDebugLoc};
use super::function::{Attribute, Declaration, Function, GroupID};
use super::types::{TypeRef, Types};
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
    /// See [LLVM 14 docs on Global Aliases](https://releases.llvm.org/14.0.0/docs/LangRef.html#aliases)
    pub global_aliases: Vec<GlobalAlias>,
    /// Holds a reference to all of the `Type`s used in the `Module`, and
    /// facilitates lookups so you can get a `TypeRef` to the `Type` you want.
    pub types: Types,
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "; ModuleID = '<{}>'", self.name)?;
        writeln!(f)?;
        for global_var in &self.global_vars {
            writeln!(f, "{global_var}")?;
        }
        for global_alias in &self.global_aliases {
            writeln!(f, "{global_alias}")?;
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

    /// Get the `GlobalAlias` having the given `name` (if any).
    /// Note that `GlobalAlias`es are named with `String`s and not `Name`s.
    #[must_use]
    pub fn get_global_alias_by_name(&self, name: &str) -> Option<&GlobalAlias> {
        self.global_aliases
            .iter()
            .find(|global| global.name == name)
    }
}

/// See [LLVM 14 docs on Global Variables](https://releases.llvm.org/14.0.0/docs/LangRef.html#global-variables)
#[derive(PartialEq, Clone, Debug)]
pub struct GlobalVariable {
    /// Globals' names must be strings
    pub name: String,
    pub linkage: Linkage,
    pub visibility: Visibility,
    pub is_constant: bool,
    pub ty: TypeRef,
    pub addr_space: AddrSpace,
    pub dll_storage_class: DLLStorageClass,
    pub thread_local_mode: ThreadLocalMode,
    pub unnamed_addr: Option<UnnamedAddr>,
    pub initializer: Option<ConstantRef>,
    pub section: Option<String>,
    pub comdat: Option<Comdat>, // llvm-hs-pure has Option<String> for some reason
    pub alignment: u32,
    pub debugloc: Option<DebugLoc>,
}

impl HasDebugLoc for GlobalVariable {
    fn get_debug_loc(&self) -> &Option<DebugLoc> {
        &self.debugloc
    }
}

impl fmt::Display for GlobalVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        /*
        @<GlobalVarName> = [Linkage] [PreemptionSpecifier] [Visibility]
                   [DLLStorageClass] [ThreadLocal]
                   [(unnamed_addr|local_unnamed_addr)] [AddrSpace]
                   [ExternallyInitialized]
                   <global | constant> <Type> [<InitializerConstant>]
                   [, section "name"] [, partition "name"]
                   [, comdat [($name)]] [, align <Alignment>]
                   (, !name !N)*
        */
        write!(f, "@{} = {} {}", self.name, self.linkage, self.visibility)?;
        writeln!(f)
    }
}

/// See [LLVM 14 docs on Global Aliases](https://releases.llvm.org/14.0.0/docs/LangRef.html#aliases)
#[derive(PartialEq, Clone, Debug)]
pub struct GlobalAlias {
    /// Globals' names must be strings, so this is `String` not `Name`
    pub name: String,
    pub aliasee: ConstantRef,
    pub linkage: Linkage,
    pub visibility: Visibility,
    pub ty: TypeRef,
    pub addr_space: AddrSpace,
    pub dll_storage_class: DLLStorageClass,
    pub thread_local_mode: ThreadLocalMode,
    pub unnamed_addr: Option<UnnamedAddr>,
}

impl fmt::Display for GlobalAlias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO
        // @<Name> = [Linkage] [PreemptionSpecifier] [Visibility] [DLLStorageClass] [ThreadLocal] [(unnamed_addr|local_unnamed_addr)] alias <AliaseeTy>, <AliaseeTy>* @<Aliasee> [, partition "name"]
        writeln!(
            f,
            "@{} = [Linkage] [PreemptionSpecifier] [Visibility] [DLLStorageClass] [ThreadLocal] [(unnamed_addr|local_unnamed_addr)] alias <AliaseeTy>, <AliaseeTy>* @<Aliasee> [, partition \"name\"]",
            self.name
        )
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum UnnamedAddr {
    Local,
    Global,
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

/// See [LLVM 14 docs on Visibility Styles](https://releases.llvm.org/14.0.0/docs/LangRef.html#visibility-styles)
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Visibility {
    Default,
    Hidden,
    Protected,
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Visibility::Default => write!(f, "default"),
            Visibility::Hidden => write!(f, "hidden"),
            Visibility::Protected => write!(f, "protected"),
        }
    }
}

/// See [LLVM 14 docs on DLL Storage Classes](https://releases.llvm.org/14.0.0/docs/LangRef.html#dllstorageclass)
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum DLLStorageClass {
    Default,
    Import,
    Export,
}

/// See [LLVM 14 docs on Thread Local Storage Models](https://releases.llvm.org/14.0.0/docs/LangRef.html#thread-local-storage-models)
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ThreadLocalMode {
    NotThreadLocal,
    GeneralDynamic,
    LocalDynamic,
    InitialExec,
    LocalExec,
}

/// For discussion of address spaces, see [LLVM 14 docs on Pointer Type](https://releases.llvm.org/14.0.0/docs/LangRef.html#pointer-type)
pub type AddrSpace = u32;

/// See [LLVM 14 docs on Attribute Groups](https://releases.llvm.org/14.0.0/docs/LangRef.html#attribute-groups)
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct FunctionAttributeGroup {
    pub group_id: GroupID,
    pub attrs: Vec<Attribute>,
}

/// See [LLVM 14 docs on Comdats](https://releases.llvm.org/14.0.0/docs/LangRef.html#langref-comdats)
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Comdat {
    pub name: String,
    pub selection_kind: SelectionKind,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum SelectionKind {
    Any,
    ExactMatch,
    Largest,
    NoDuplicates,
    SameSize,
}
