namespace Kata.Verification {
    open Microsoft.Quantum.Math;

    function ArraysEqualD(actual : Double[][], expected : Double[][]) : Bool {
        if Length(actual) != Length(expected) {
            Message("Incorrect");
            Message($"Expected number of rows {Length(expected)}, actual {Length(actual)}");
            return false;
        }
        for i in 0 .. Length(actual) - 1 {
            if Length(actual[i]) != Length(expected[i]) {
                Message("Incorrect");
                Message($"For row {i}, expected number of columns {Length(expected[i])}, actual {Length(actual[i])}");
                return false;
            }

            for j in 0 .. Length(actual[i]) - 1 {
                if AbsD(actual[i][j] - expected[i][j]) > 1e-9 {
                    Message("Incorrect");
                    Message($"For element in row {i}, column {j}, expected {expected[i][j]}, actual {actual[i][j]}");
                    return false;
                }
            }
        }
        
        Message("Correct!");
        return true;
    }
}
