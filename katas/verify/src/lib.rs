pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use qsc_frontend::compile::{compile, PackageStore};

    fn verify_reference(reference: &str) {
        let store = PackageStore::new();
        let unit = compile(&store, [], [reference], "");
        assert!(
            unit.context.errors().is_empty(),
            "Compilation errors: {:?}",
            unit.context.errors()
        );
    }

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn verify_single_qubit_gates_kata() {
        verify_reference(
            indoc! {"
            namespace Quantum.Kata.SingleQubitGates {
                open Microsoft.Quantum.Intrinsic;
                operation ApplyY(q : Qubit) : Unit is Adj + Ctl {
                    // Apply the Pauli Y operation.
                    // Y(q);
                }
            }"})
    }
}
