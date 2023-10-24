namespace Main {
    open Dependency1;
    open Dependency2;
    @EntryPoint()
    operation Main() : String {
        First() + Second()
    }
}
