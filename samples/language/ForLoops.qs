/// # Sample
/// For Loops
///
/// # Description
/// For loops are a basic looping structure in Q# for looping
/// over the elements of `Range` objects, arrays, and array slices.
/// They can be used in both `operation` and `function` callables.
namespace MyQuantumApp {

    @EntryPoint()
    function Main() : Unit {
        // For loop over `Range`
        for i in 0..5 {}

        // For loop over `Array`
        for element in [10, 11, 12] {}

        // For loop over array slice
        let array = [1.0, 2.0, 3.0, 4.0];
        for element in array[2...] {}
    }
}
