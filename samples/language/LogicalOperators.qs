/// # Sample
/// Logical Operators
///
/// # Description
/// Logical operators in Q# are operators that operate on Boolean
/// values to produce an output Boolean value. The logical operators
/// in Q# are `and`, `or`, and `not`.
namespace MyQuantumApp {

    @EntryPoint()
    operation Main() : Unit {

        // `and` performs the Boolean And on two Boolean values.

        // The Boolean value `false`.
        let boolean = false and false;

        // The Boolean value `false`.
        let boolean = false and true;

        // The Boolean value `false`.
        let boolean = true and false;

        // The Boolean value `true`.
        let boolean = true and true;

        // `or` performs the Boolean Or on two Boolean values.

        // The Boolean value `false`.
        let boolean = false or false;

        // The Boolean value `true`.
        let boolean = false or true;

        // The Boolean value `true`.
        let boolean = true or false;

        // The Boolean value `true`.
        let boolean = true or true;

        // `not` performs the Boolean Not on one Boolean value.

        // The Boolean value `true`.
        let boolean = not false;

        // The Boolean value `false`.
        let boolean = not true;
    }
}
