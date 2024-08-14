import Unstable.Arithmetic.IncByI;
import Microsoft.Quantum.Arrays.Reversed;
@EntryPoint()
operation TestFunctions() : Int {
    use a = Qubit[2];
    use b = Qubit[2];
    use c = Qubit[4];

    X(a[0]);
    X(b[1]);
    Signed.MultiplyI(
        a, b, c
    );

    let res = MeasureInteger(c);
    ResetAll(a + b + c);
    return res;
}