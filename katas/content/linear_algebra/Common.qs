namespace Kata.Verification {
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Convert;

    function ComplexAsString(x : Complex) : String {
        let precision = 3;        
        if x::Imag < 0.0 {
            $"{DoubleAsStringWithPrecision(x::Real,precision)} - {DoubleAsStringWithPrecision(AbsD(x::Imag),precision)}i"
        } else {
            $"{DoubleAsStringWithPrecision(x::Real,precision)} + {DoubleAsStringWithPrecision(x::Imag,precision)}i"
        }
    }

    function ArraysEqualD(actual : Double[][], expected : Double[][]) : Bool {
        let precision = 3;
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
                    Message($"For element in row {i}, column {j}, expected {DoubleAsStringWithPrecision(expected[i][j],precision)}, actual {DoubleAsStringWithPrecision(actual[i][j],precision)}");
                    return false;
                }
            }
        }
        
        Message("Correct!");
        return true;
    }

    function ArraysEqualC(actual : Complex[][], expected : Complex[][]) : Bool {
        let precision = 3;
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
                if AbsComplex(MinusC(actual[i][j], expected[i][j])) > 1e-9 {
                    Message("Incorrect");
                    Message($"For element in row {i}, column {j}, expected {ComplexAsString(expected[i][j])}, actual {ComplexAsString(actual[i][j])}");
                    return false;
                }
            }
        }
        
        Message("Correct!");
        return true;
    }
}
