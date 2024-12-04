namespace Kata {
    import Std.Arrays.*;
    import Std.Convert.*;
    import Std.Math.*;

    operation BinaryFractionClassical(q : Qubit, j : Bool[]) : Unit is Adj + Ctl {
        let n = Length(j);
        let jIntBE = BoolArrayAsInt(Reversed(j));
        R1(2.0 * PI() * IntAsDouble(jIntBE) / IntAsDouble(1 <<< n), q);
    }
}
