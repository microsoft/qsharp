/// # Sample
/// Unit
///
/// # Description
/// The `Unit` type is the singleton type whose only value is ().
/// Functions implicitly return `Unit` if no explicit or implicit
/// return is specified.
namespace MyQuantumApp {

    @EntryPoint()
    function ExplicitReturn() : Unit {
        // Question: It's worth having something that only returns Unit explicitly
        // as function, or should we support this as a valis operation?
        // Explicitly return `Unit`.
        return ();
    }
    operation NoReturn() : Unit {
        // No return, thus implicitly returning `Unit`.
    }
}
