/// # Sample
/// String
///
/// # Description
/// Text as values that consist of a sequence of UTF-16 code units.
namespace MyQuantumApp {

    @EntryPoint()
    operation Main() : String {
        // Strings literals are declared with double quotes:
        let myString = "Foo";

        // Strings can be concatenated with `+`
        let myString = myString + "Bar";

        // Q# supports string interpolation with the `$` prefix.
        let myString = $"interpolated: {myString}";

        return myString;
    }
}