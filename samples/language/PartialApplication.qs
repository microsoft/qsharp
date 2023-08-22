/// # Sample
/// Partial Application
///
/// # Description
/// Partial application of functions and operations provides a way to tersely
/// define lambda functions and lambda operations. It is a convenient syntax
/// that uses an `_` to describe the "remaining element".
namespace MyQuantumApp {
    open Microsoft.Quantum.Arrays;
    @EntryPoint()
    operation Main(): Unit {
        // This takes our adding function and partially applies it,
        // filling the second argument with `1` and returning another
        // function that takes one argument and passes it in to `Add`.
        let incrementByOne = Add(_, 1);

        // The below lambda is equivalent to the above partial application
        let incrementByOneLambda = x -> Add(x, 1);

        // we can add `1` to any number using our partially applied function
        let five = incrementByOne(4);

        // More than one underscore can be used to define a function that takes
        // multiple arguments.
        let sumAndAddOne = AddMany(_, _, _, 1);

        // The below lambda is the same as the above partial application.
        let sumAndAddOneLambda = (a, b, c) -> AddMany(a, b, c, 1);

        let intArray = [1, 2, 3, 4, 5];
        // The below expression increments all values in an array by 1
        let incremented = Mapped(Add(_, 1), intArray);
    }

    function Add(x: Int, y: Int): Int {
        return x + y;
    }

    function AddMany(a: Int, b: Int, c: Int, d: Int): Int {
        return a + b + c + d;
    }
}