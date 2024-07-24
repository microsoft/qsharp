namespace Kata.Verification {
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        for (V, edges) in Most(ExampleGraphs()) {
            if not CheckOracleRecognizesColoring(V, edges, Kata.Oracle_VertexColoring, IsVertexColoringValid_Reference) {
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
