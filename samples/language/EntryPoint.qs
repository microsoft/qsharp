/// # Sample
/// EntryPoint
///
/// # Description
/// The `@EntryPoint()` attribute is used to designate a particular operation as
/// the entry point of a Q# program rather than requiring entry points to have
// a particular name such as `main`, `Main`, or `__main__`.
namespace MyQuantumApp {

    // The Q# compiler identifies `Main` as the entry point operation because it is marked with the `@EntryPoint()` attribute.
    @EntryPoint()
    function Main() : Result[] {
        return [];
    }
}
