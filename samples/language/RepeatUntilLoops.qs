/// # Sample
/// Repeat-Until Loops
///
/// # Description
/// The repeat loop runs a block of code before evaluating a condition.
/// If the condition evaluates to true, the loop exits. If the condition
/// evaluates to false, an optional fixup block is executed before the
/// next loop iteration is run.
///
/// The compiler treats all parts of the repeat loop (both blocks and
/// the condition) as a single scope for each repetition. Symbols that
/// are defined within the repeat block are visible both to the
/// condition and within the fixup block. Symbols go out of scope after
/// each iteration, such that symbols defined in the fixup block are not
/// visible in the repeat block.
namespace MyQuantumApp {

    @EntryPoint()
    operation Main() : Unit {

        // Repeat-Until loop with fixup
        repeat {
            use qubit = Qubit();
            H(qubit);
            let result = M(qubit);
        } until result == Zero
        fixup {
            // Here, the fixup is used to put the qubit back
            // into its default state before releasing it.
            Reset(qubit);
        }

        // Repeat-Until loop
        repeat {
            set x += 1;
        } until x < 3;
    }
}
