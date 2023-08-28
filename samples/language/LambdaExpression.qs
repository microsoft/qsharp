/// # Sample
/// Lambda Expressions
///
/// # Description
/// A lambda expression creates an anonymous function or operation, typically
/// used for local encapsulated functionality. Lambdas offer low sytactic
/// overhead, foregoing a fully namespaced function declaration, making them
/// useful for single-use or utility functions.
namespace MyQuantumApp {
    open Microsoft.Quantum.Arrays;
    @EntryPoint()
    operation Main() : Unit {
        // A lambda function is defined with an arrow `->`.
        // The below function takes two inputs and adds them.
        let add = (x, y) -> x + y;

        // A lambda operation is defined with a fat arrow `=>`.
        // The below operation closes over `qubit` and applies
        // a `CNOT` gate to it.
        use control = Qubit();
        let cnotOnControl = q => CNOT(control, q);

        // Lambdas can be used as higher-order callable inputs to
        // functions such as `Fold` and `Map`.
        // `Fold` folds an array into one element using a callable
        // that combines two elements into one.
        let intArray = [1, 2, 3, 4, 5];
        let sum = Fold(add, 0, intArray);

        // `Map` takes a callable and applies it to all elements in
        // an array
        let incremented = Mapped(x -> x + 1, intArray);
 
    }
}