namespace Kata.Verification {
    import Std.Math.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        let actual = Kata.EigenvectorsX();
        if Length(actual) != 2 {
            Message("The array of eigenvectors should have exactly two elements.");
            return false;
        }
        for i in 0..1 {
            if Length(actual[i]) != 2 {
                Message("Each eigenvector should have exactly two elements.");
                return false;
            }
            if AbsD(actual[i][0]) + AbsD(actual[i][1]) < 1E-9 {
                Message("Each eigenvector should be non-zero.");
                return false;
            }
        }

        // One eigenvector has to have equal components, the other one - opposite ones
        if AbsD(actual[0][0] - actual[0][1]) < 1e-9 and AbsD(actual[1][0] + actual[1][1]) < 1e-9 or
            AbsD(actual[0][0] + actual[0][1]) < 1e-9 and AbsD(actual[1][0] - actual[1][1]) < 1e-9 {
            Message("Correct!");
            return true;
        }

        Message("Incorrect value for one of the eigenvectors.");
        return false;
    }
}
