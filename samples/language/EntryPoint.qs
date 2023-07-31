/// # Sample
/// EntryPoint
///
/// # Description
/// The `@EntryPoint()` attribute is used to designate a particular operation as
/// the entry point of a Q# program rather than requiring entry points to have 
// a particular name such as `main`, `Main`, or `__main__`.
namespace MyQuantumApp {

    @EntryPoint()
    operation Main() : Result[] {
        return [];
    }
}