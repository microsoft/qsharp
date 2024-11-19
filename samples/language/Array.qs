// # Sample
// Array
//
// # Description
// An array literal is a sequence of one or more expressions, separated by commas
// and enclosed in brackets `[` and `]`; for example, `[1,2,3]`. All expressions must
// have a common base type, which is the item type of the array.
//
// Arrays of arbitrary length, and in particular empty arrays, may be created using
// a new array expression. Such an expression is of the form new `[expr1, size = expr2]`,
// where `expr1` is an expression of any type and `expr2` is an expression of type `Int`.
// The expression `expr1` is used as the default value for all of the array items.
//
// For example, `[0, size = 10]` creates an array of zeroes containing ten items.
// The length of an array can be queried with the function `Length`. It is defined
// in the automatically opened namespace `Std.Core` and returns an `Int` value.
// Suitable initialization routines for arrays of `Qubit`s can be found in the
// `Std.Arrays` namespace.

function Main() : Int[] {

    // A basic Int Array literal
    let intArray : Int[] = [1, 2, 3, 4];
    Message($"Integer Array: {intArray} of length {Length(intArray)}");

    // A basic String Array literal
    let stringArray = ["a", "string", "array"];
    Message($"String Array: {stringArray}");

    // A new array expression creating the array `[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`
    let repeatedArray = [0, size = 10];
    Message($"Repeated Array: {repeatedArray}");
    let repeatedArray = Repeated(0, 10);  // contains [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    Message($"Repeated Array: {repeatedArray}");

    // Arrays can be sliced with ranges.
    let slice = intArray[1..2..4];  // contains [2,4]
    Message($"Sliced array: {slice}");
    let slice = intArray[2..-1..0]; // contains [3,2,1]
    Message($"Sliced array: {slice}");
    let slice = intArray[...]; // contains [1, 2, 3, 4];
    Message($"Sliced array: {slice}");

    return intArray;
}
