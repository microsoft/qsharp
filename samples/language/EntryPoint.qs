/// # Sample
/// EntryPoint
///
/// # Description
/// When defining an entry point into a Q# program, the Q# compiler recognizes
/// the @EntryPoint() attribute rather than requiring that entry points have a
/// particular name, for example, main, Main, or __main__. That is, from the
/// perspective of a Q# developer, entry points are ordinary operations annotated
/// with @EntryPoint().
namespace MyQuantumApp {

    @EntryPoint()
    operation Main() : Result[] {
        return [];
    }
}