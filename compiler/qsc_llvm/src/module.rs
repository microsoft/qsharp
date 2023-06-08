// Portions copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::constant::ConstantRef;
use crate::debugloc::{DebugLoc, HasDebugLoc};
use crate::function::{Attribute, Declaration, Function, GroupID};
use crate::types::{FPType, Type, TypeRef, Types};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;

/// See [LLVM 14 docs on Module Structure](https://releases.llvm.org/14.0.0/docs/LangRef.html#module-structure)
#[derive(Clone)]
pub struct Module {
    /// The name of the module
    pub name: String,
    /// See [LLVM 14 docs on Source Filename](https://releases.llvm.org/14.0.0/docs/LangRef.html#source-filename)
    pub source_file_name: String,
    /// See [LLVM 14 docs on Data Layout](https://releases.llvm.org/14.0.0/docs/LangRef.html#data-layout)
    pub data_layout: Option<DataLayout>,
    /// See [LLVM 14 docs on Target Triple](https://releases.llvm.org/14.0.0/docs/LangRef.html#target-triple)
    pub target_triple: Option<String>,
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
    /// See [LLVM 14 docs on Module-Level Inline Assembly](https://releases.llvm.org/14.0.0/docs/LangRef.html#moduleasm)
    pub inline_assembly: String,
    /// Holds a reference to all of the `Type`s used in the `Module`, and
    /// facilitates lookups so you can get a `TypeRef` to the `Type` you want.
    pub types: Types,
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "; ModuleID = '<{}>'", self.name)?;
        if let Some(data_layout) = &self.data_layout {
            writeln!(f, "target datalayout = \"{data_layout}\"")?;
        }
        if let Some(target_triple) = &self.target_triple {
            writeln!(f, "target triple = \"{target_triple}\"")?;
        }
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

/// See [LLVM 14 docs on Data Layout](https://releases.llvm.org/14.0.0/docs/LangRef.html#data-layout)
#[derive(Clone, Debug)]
pub struct DataLayout {
    /// The data layout in string form, as described in the Data Layout docs linked above
    pub layout_str: String,
    /// Little-endian or big-endian?
    pub endianness: Endianness,
    /// Natural alignment of the stack, in bits. For more, see the Data Layout docs linked above
    pub stack_alignment: Option<u32>,
    /// Address space for program memory
    pub program_address_space: AddrSpace,
    /// Address space for objects created by `alloca`
    pub alloca_address_space: AddrSpace,
    /// Alignment for various types in memory
    pub alignments: Alignments,
    /// What mangling will be applied when the LLVM module is compiled to machine code
    pub mangling: Option<Mangling>,
    /// Native integer width(s) for the target CPU
    pub native_int_widths: Option<HashSet<u32>>,
    /// Address spaces with non-integral pointer types
    pub non_integral_ptr_types: HashSet<AddrSpace>,
}

impl fmt::Display for Endianness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Endianness::BigEndian => write!(f, "E"),
            Endianness::LittleEndian => write!(f, "e"),
        }
    }
}
impl fmt::Display for DataLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.layout_str.is_empty() {
            write!(f, "{}", self.endianness)?;
            write!(f, "-")?;
            write!(f, "{}", self.stack_alignment.unwrap_or(0))
            // TODO: finish
        } else {
            write!(f, "{}", self.layout_str)
        }
    }
}
impl PartialEq for DataLayout {
    fn eq(&self, other: &Self) -> bool {
        // The layout string fully specifies all the other information
        self.layout_str == other.layout_str
    }
}

impl Eq for DataLayout {}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Endianness {
    /// Least-significant bits are stored in the lowest address location
    LittleEndian,
    /// Most-significant bits are stored in the lowest address location
    BigEndian,
}

/// Alignment details for a type.
/// See [LLVM 14 docs on Data Layout](https://releases.llvm.org/14.0.0/docs/LangRef.html#data-layout)
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Alignment {
    /// Minimum alignment (in bits) per the ABI
    pub abi: u32,
    /// Preferred alignment (in bits)
    pub pref: u32,
}

/// Alignment details for function pointers.
/// See [LLVM 14 docs on Data Layout](https://releases.llvm.org/14.0.0/docs/LangRef.html#data-layout)
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct FunctionPtrAlignment {
    /// If `true`, function pointer alignment is independent of function alignment.
    /// If `false`, function pointer alignment is a multiple of function alignment.
    pub independent: bool,
    /// Minimum alignment (in bits) per the ABI
    pub abi: u32,
}

/// Layout details for pointers (other than function pointers).
/// See [LLVM 14 docs on Data Layout](https://releases.llvm.org/14.0.0/docs/LangRef.html#data-layout)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PointerLayout {
    /// Size of a pointer in bits
    pub size: u32,
    /// Alignment of a pointer
    pub alignment: Alignment,
    /// Size of an index used for address calculation, in bits
    pub index_size: u32,
}

/// Alignment for various types in memory.
/// See [LLVM 14 docs on Data Layout](https://releases.llvm.org/14.0.0/docs/LangRef.html#data-layout)
#[derive(Clone, Debug, Default)]
pub struct Alignments {
    /// Explicit alignments for various sizes of integers (in bits). Sizes not
    /// specified here are determined according to the rules described in the
    /// Data Layout docs.
    int_alignments: BTreeMap<u32, Alignment>,
    /// Explicit alignments for various sizes of vectors (in bits). Sizes not
    /// specified here are determined according to the rules described in the
    /// Data Layout docs.
    vec_alignments: BTreeMap<u32, Alignment>,
    /// Alignment for floating-point types, by size (in bits)
    fp_alignments: HashMap<u32, Alignment>,
    /// Alignment for aggregate types (structs, arrays)
    agg_alignment: Alignment,
    /// Alignment for function pointers
    fptr_alignment: FunctionPtrAlignment,
    /// Alignment for function pointers, as an `Alignment`
    fptr_alignment_as_alignment: Alignment,
    /// Layout details for (non-function-pointer) pointers, by address space
    pointer_layouts: HashMap<AddrSpace, PointerLayout>,
}

impl Alignments {
    /// Alignment of the given type (in bits)
    /// # Panics
    /// This function will panic if given a type with an unexpected layout.
    #[must_use]
    pub fn type_alignment(&self, ty: &Type) -> &Alignment {
        match ty {
            Type::IntegerType { bits } => self.int_alignment(*bits),
            Type::VectorType {
                element_type,
                num_elements,
                ..
            } => {
                let element_size_bits = match element_type.as_ref() {
                    Type::IntegerType { bits } => *bits,
                    Type::FPType(fpt) => Self::fpt_size(*fpt),
                    ty => panic!("Didn't expect a vector with element type {ty:?}"),
                };
                self.vec_alignment(element_size_bits * *num_elements)
            }
            Type::FPType(fpt) => self.fp_alignment(*fpt),
            Type::StructType { .. } | Type::NamedStructType { .. } | Type::ArrayType { .. } => {
                self.agg_alignment()
            }
            Type::PointerType {
                pointee_type,
                addr_space,
            } => match pointee_type.as_ref() {
                Type::FuncType { .. } => &self.fptr_alignment_as_alignment,
                _ => &self.ptr_alignment(*addr_space).alignment,
            },
            _ => panic!("Don't know how to get the alignment of {ty:?}"),
        }
    }

    /// Alignment of the integer type of the given size (in bits)
    #[must_use]
    pub fn int_alignment(&self, size: u32) -> &Alignment {
        // If we have an explicit entry for this size, use that
        if let Some(alignment) = self.int_alignments.get(&size) {
            return alignment;
        }
        // Find the next largest size that has an explicit entry and use that
        let next_largest_entry = self.int_alignments.iter().find(|(&k, _)| k > size);
        match next_largest_entry {
            Some((_, alignment)) => alignment,
            None => {
                // `size` is larger than any explicit entry: use the largest explicit entry
                self.int_alignments
                    .values()
                    .rev()
                    .next()
                    .expect("Should have at least one explicit entry")
            }
        }
    }

    /// Alignment of the vector type of the given total size (in bits)
    #[must_use]
    pub fn vec_alignment(&self, size: u32) -> &Alignment {
        // If we have an explicit entry for this size, use that
        if let Some(alignment) = self.vec_alignments.get(&size) {
            return alignment;
        }
        // Find the next smaller size that has an explicit entry and use that
        let next_smaller_entry = self.vec_alignments.iter().find(|(&k, _)| k < size);
        match next_smaller_entry {
            Some((_, alignment)) => alignment,
            None => {
                // `size` is smaller than any explicit entry. LLVM docs seem to
                // be not clear what happens here, I assume we just use the
                // smallest explicit entry
                self.vec_alignments
                    .values()
                    .next()
                    .expect("Should have at least one explicit entry")
            }
        }
    }

    /// Alignment of the given floating-point type
    #[must_use]
    pub fn fp_alignment(&self, fpt: FPType) -> &Alignment {
        self.fp_alignments.get(&Self::fpt_size(fpt)).expect(
            "No alignment information for floating point type - does the target support that type?",
        )
    }

    /// Alignment of aggregate types (structs, arrays)
    #[must_use]
    pub fn agg_alignment(&self) -> &Alignment {
        &self.agg_alignment
    }

    /// Alignment of function pointers
    #[must_use]
    pub fn fptr_alignment(&self) -> &FunctionPtrAlignment {
        &self.fptr_alignment
    }

    /// Alignment of (non-function-pointer) pointers in the given address space
    #[must_use]
    pub fn ptr_alignment(&self, addr_space: AddrSpace) -> &PointerLayout {
        match self.pointer_layouts.get(&addr_space) {
            Some(layout) => layout,
            None => self
                .pointer_layouts
                .get(&0)
                .expect("Should have a pointer layout for address space 0"),
        }
    }

    /// for internal use: size of an `FPType`, in bits
    fn fpt_size(fpt: FPType) -> u32 {
        match fpt {
            FPType::BFloat | FPType::Half => 16,
            FPType::Single => 32,
            FPType::Double => 64,
            FPType::FP128 | FPType::PPC_FP128 => 128,
            FPType::X86_FP80 => 80,
        }
    }
}

/// See [LLVM 14 docs on Data Layout](https://releases.llvm.org/14.0.0/docs/LangRef.html#data-layout)
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Mangling {
    ELF,
    MIPS,
    MachO,
    WindowsX86COFF,
    WindowsCOFF,

    XCOFF,
}
