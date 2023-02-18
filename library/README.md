# Q# Standard Library

This folder defines the standard library for Q# that is included with the compiler and available to programs by default. The library scope and contents are not identical to the legacy libraries available from the [QuantumLibraries](https://github.com/microsoft/QuantumLibraries) repository, but rather tailored for the new stack and adjusted for broader hardware compatibility.

## Design Philosophies and Assumptions

### File Layout

Files in the library match the contained namespace. This makes it easier to find defined items from the fully-qualified name. While Q# (currently) allows defining namespace items across multiple files, this should be avoided here for consistency.

### Body Intrinsic Callables

Unless otherwise noted, callables that are defined as `body intrinsic` are generated into external declarations in QIR with exact callable name and signature matches. The containing namespace is not included in the generated name. This simplifies QIR code generation and makes correlating Q# declaration to the resulting QIR program easier. Body intrinsic callables should avoid using generics and aggregate types if possible to keep the corresponding LLVM signatures simple and increase the likelihood of hardware compatibility.

One notable excpetion to this is the `Microsoft.Quantum.Core.Length` function, which is both `body intrinsic` and generic across the array type. As a result it needs special handling during monomorphization and code geneation to correlate it to the right pattern for extracting length from the array structure. Open question: should this be transitioned to an operator or some other form that avoid such a prominent counterexample?

### QIR Quantum Gate Instructions

The library includes a set of one- and two-qubit gates represented as `__quantum__qis__*__body` or `__quantum__qis__*__adj` QIR declarations. The expectation is that these gates form a common QIR API surface that Q# is compiled into. Any target would either need to support these quantum gate instructions natively or provide a QIR definition of that gate instruction in terms of supported native gates. These definitions can then be linked with the QIR program via LLVM tools and resolve to the target specific gate set. This approach provides broader compatibility at the QIR level and avoids the need for front end language targetting; as a result, this Q# library design is not expected to require any target packages or operation substitution.

To avoid using any opaque array types in the quantum gate instruction set, the library utilizes decomposition strategies to express the Q# controlled functor (which allows an arbitrary number of controls) in terms of singly- and doubly-controlled gates. While this makes simulation somewhat more expensive in terms of allocating extra controls and performing more operations, it more closely matches the patterns used by hardware and provides better API surface for QIR programs to have hardware compatibility.
