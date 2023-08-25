/// # Sample
/// Within Apply
///
/// # Description
/// `within`...`apply` is a structure that provides a pattern for commonly used operations
/// in quantum algorithms. More specifically, it applies some operation in the `within` block,
/// applies smome other operation in the `apply` block, and then undoes the first operation using
/// the adjoint of the `within` block.
/// This is one of the features that makes Q# particularly ergonomic for expressing quantum algorithms.
namespace MyQuantumApp {
    @EntryPoint()
    operation Main() : Unit is Adj + Ctl {
        // Allocate a qubit.
        use qubit = Qubit();
        within {
            // Define some operation that prepares for the `apply` block.
            H(qubit);
        } apply {
            // Apply the main operation after the `within` block.
            X(qubit);
        }
        // After the apply block, the `H` gate will be adjointed, effectively undoing the
        // preparation in the `within` block by applying `H(qubit)` again.
    }
}