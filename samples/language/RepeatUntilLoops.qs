/// # Sample
/// Repeat-Until Loops
///
/// # Description
/// The repeat-until loop runs a block of code before evaluating a condition.
/// If the condition evaluates to true, the loop exits. If the condition
/// evaluates to false, an optional fixup block is executed before the
/// next loop iteration is run.
namespace MyQuantumApp {

    @EntryPoint()
    operation Main() : Unit {

        // Repeat-Until loop
        mutable x = 0;
        repeat {
            set x += 1;
        } until x > 3;

        // Repeat-Until loop with fixup
        use qubit = Qubit();
        repeat {
            H(qubit);
        } until M(qubit) == Zero
        fixup {
            Reset(qubit);
        }
    }
}
