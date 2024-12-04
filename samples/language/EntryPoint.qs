// # Sample
// EntryPoint
//
// # Description
// The `@EntryPoint()` attribute is used to designate a particular operation as
// the entry point of a Q# program rather than requiring entry points to have
// a particular name such as `main`, `Main`, or `__main__`.
// The Q# compiler identifies `Main` as the entry point operation by its name.
// An entry point can also have any name if it is also marked with the `@EntryPoint()` attribute.

function Main() : Result[] {
    return [];
}
