namespace Kata {
    import Std.Convert.*;
    import Std.Math.*;

    function ComputeRepetitions(angle : Double, offset : Int, accuracy : Double) : Int {
        mutable pifactor = 0;
        while (true) {
            let pimultiple = PI() * IntAsDouble(2 * pifactor + offset);
            let times = Round(pimultiple / angle);
            if AbsD(pimultiple - (IntAsDouble(times) * angle)) / PI() < accuracy {
                return times;
            }
            set pifactor += 1;
        }
        return 0;
    }

    operation DistinguishRzFromRy(theta : Double, unitary : (Qubit => Unit is Adj + Ctl)) : Int {
        use q = Qubit();
        let times = ComputeRepetitions(theta, 1, 0.1);
        mutable attempt = 1;
        mutable measuredOne = false;
        repeat {
            for _ in 1..times {
                unitary(q);
            }
            // for Rz, we'll never venture away from |0⟩ state, so as soon as we got |1⟩ we know it's not Rz
            if MResetZ(q) == One {
                set measuredOne = true;
            }
            // if we try several times and still only get |0⟩s, chances are that it is Rz
        } until (attempt == 4 or measuredOne)
        fixup {
            set attempt += 1;
        }
        return measuredOne ? 1 | 0;
    }
}
