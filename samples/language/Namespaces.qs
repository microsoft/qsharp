/// # Sample
/// Namespaces
///
/// # Description
/// Every Q# file typically starts with a namespace. A namespace helps
/// you organize related functionality this is useful when you are writing
/// libraries or reusable code.
namespace MyQuantumApp {

    // The following `open` directive is used to import all types and callables declared in the
    // Microsoft.Quantum.Diagnostics namespace.
    open Microsoft.Quantum.Diagnostics;
	
    @EntryPoint()
    operation Main() : Result[] {
        // `DumpMachine` is in the Microsoft.Quantum.Diagnostics namespace
        DumpMachine();
        return [];
    }
}