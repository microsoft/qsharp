namespace Kata.Verification {
    import Std.Convert.*;
    import KatasUtils.*;

    function F_Product(x : Bool[], r : Bool[]) : Bool {
        mutable product = false;
        for i in 0..Length(x) - 1 {
            if r[i] and x[i] {
                set product = not product;
            }
        }
        product
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let n = 3;
        for mask in 0..2^n - 1 {
            let r = IntAsBoolArray(mask, 3);
            if not CheckOracleImplementsFunction(n, Kata.Oracle_Product(_, _, r), F_Product(_, r)) {
                Message($"Test failed for n = {n}, r = {r}");
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
