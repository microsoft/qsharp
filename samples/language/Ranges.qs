/// # Sample
/// Ranges
///
/// # Description
/// Ranges are a way of expressin a range of integer values in Q#.
/// Range expressions can be either closed ranges with definate start
/// and end values, such as 1..10 which contains all the integers from
/// 1 to 10 inclusively, or they can be open ranges that have
/// indeterminate start, end, or both. Ranges can also optionally specify
/// a step size with an integer in the middle of the expression, such as
/// 0..2..10, which contains all the even numbers from 0 to 10 inclusively.
namespace MyQuantumApp {

    @EntryPoint()
    operation Main() : Unit {

        // Ranges can be captured as local variables.
        let local_range = 0..10;

        // Ranges are used in for loop expressions. They must be closed
        // ranges.
        // Here we build an array of the squares of the numbers 0 through 10.
        mutable array = [0, size=10];
        for i in 0..10 {
            set array w/= i <- i*i;
        }
        Message($"{array}");

        // Ranges can be used to loop over elements of arrays by creating
        // array slices.
        Message("Even squares:");
        for i in array[0..2..10] {
            Message($"{i}");
        }
        // Array slices can also be printed directly.
        Message($"{array[0..2..10]}");

        // Ranges can be reversed by having a smaller end value than
        // start value and a negative step.
        Message($"{array[10..-1..0]}");

        // Open ranges can be used with array to create slices.
        Message($"Open start: {array[...4]}");
        Message($"Open end: {array[5...]}");
        Message($"Open end with step: {array[2..3...]}");
        Message($"Fully open: {array[...]}");
        Message($"Fully open with step: {array[...-3...]}");
    }
}
