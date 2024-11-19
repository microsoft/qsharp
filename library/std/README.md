# Q# Standard Library

This folder defines the standard library for Q# that is included with the compiler and available to programs by default. The library scope and contents are not identical to the legacy libraries available from the [QuantumLibraries](https://github.com/microsoft/QuantumLibraries) repository, but rather tailored for the new stack and adjusted for broader hardware compatibility.

## Design Philosophies and Assumptions

### File Layout

Files in the library match the contained namespace. This makes it easier to find defined items from the fully-qualified name. While Q# (currently) allows defining namespace items across multiple files, this should be avoided here for consistency.

### Body Intrinsic Callables

Unless otherwise noted, callables that are defined as `body intrinsic` are generated into external declarations in QIR with exact callable name and signature matches. The containing namespace is not included in the generated name. This simplifies QIR code generation and makes correlating Q# declaration to the resulting QIR program easier. Body intrinsic callables should avoid using generics and aggregate types if possible to keep the corresponding LLVM signatures simple and increase the likelihood of hardware compatibility.

### QIR Quantum Gate Instructions

The library includes a set of one- and two-qubit gates represented as `__quantum__qis__*__body` or `__quantum__qis__*__adj` QIR declarations. The expectation is that these gates form a common QIR API surface that Q# is compiled into. Any target would either need to support these quantum gate instructions natively or provide a QIR definition of that gate instruction in terms of supported native gates. These definitions can then be linked with the QIR program via LLVM tools and resolve to the target specific gate set. This approach provides broader compatibility at the QIR level and avoids the need for front end language targeting; as a result, this Q# library design is not expected to require any target packages or operation substitution.

To avoid using any opaque array types in the quantum gate instruction set, the library utilizes decomposition strategies to express the Q# controlled specialization (which allows an arbitrary number of controls) in terms of singly- and doubly-controlled gates. While this makes simulation somewhat more expensive in terms of allocating extra controls and performing more operations, it more closely matches the patterns used by hardware and provides better API surface for QIR programs to have hardware compatibility.

### Doc comments

Use the following guidance when providing doc comments for functions and operations in the standard library. These doc comments are used to generate standard library documentation automatically.

| Section        | LaTeX allowed? | Mandatory? | What is it for
|---------------:|:--------------:|:----------:|----------------
| Summary        | No             | Yes        | Short summary of the functionality provided.
| Description    | Yes            | No         | Detailed explanation of the functionality provided.
| Remarks        | Yes            | No         | General knowledge that may be useful for understanding of the functionality provided.
| Type Parameters| No             | No         | Description of type parameters (for each type parameter).
| Input          | No             | No         | Description of input parameters (for each input parameter)
| Output         | No             | No         | Description of the output.
| Example        | No             | No         | Example(s) of use. Should be a code fragment that compiles.
| References     | No             | No         | References to papers and other information published on the web or elsewhere.
| See Also       | No             | No         | References to other functions of the standard library that are similar or relevant.

* Only `Summary` section is mandatory. There's no need to include other
  sections just to repeat the information that is already stated in the summary.
* For sections where LaTeX is not allowed, Unicode characters may be used
  to represent mathematical formulas even though it is not recommended in
  general. These sections are used in places where LaTeX rendering is
  not supported.
* For sections where LaTeX is supported, unicode characters may be used to
  represent mathematical formulas only if the rendering fidelity is
  not compromised.
* Use playground to check documentation rendering.
