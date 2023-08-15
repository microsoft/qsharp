/// # Sample
/// Range
///
/// # Description
/// Value literals for the Range type are expressions of the form `start..step..stop`, where `start`, `step`,
/// and `end` are expressions of type `Int`. If the step size is one, it may be omitted. For example,
/// `start..stop` is a valid `Range` literal and the same as `start..1..stop`.
namespace MyQuantumApp {

    @EntryPoint()
    operation Main() : Range {
        // The range 1, 2, 3.
        let range = 1..3;

        // The range 2, 4.
        let range = 2..2..5;

        // The range 2, 4, 6.
        let range = 2..2..6;

        // The range 6, 4, 2.
        let range = 6..-2..2;

        // The range 2.
        let range = 2..-2..2;

        // The empty range.
        let range = 2..1;

        return range;
    }
}