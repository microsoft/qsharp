namespace Kata {
    import Std.Arrays.*;
    import Std.Convert.*;
    import Std.Diagnostics.*;
    import Std.Random.*;

    @EntryPoint()
    operation RunBB84Protocol() : Unit {
        // 1. Alice chooses a random set of bits to encode in her qubits
        //    and a random set of bases to prepare her qubits in.
        // ...

        // 2. Alice allocates qubits, encodes them using her choices and sends them to Bob.
        //    You can not express "sending the qubits to Bob" in Q#,
        //    so Bob will just use the same qubits.
        // ...

        // 3. Bob chooses a random set of bases to measure Alice's qubits in.
        // ...

        // 4. Bob measures Alice's qubits in his chosen bases.
        // ...

        // 5. Alice and Bob compare their chosen bases and
        //    use the bits in the matching positions to create a shared key.
        // ...

        // If you did everything correctly, the generated keys will always match,
        // since there is no eavesdropping going on.
        // In the next lesson we will discuss introducing eavesdropping.
    }

    // You might find these helper operations from earlier tasks useful.
    operation RandomArray(N : Int) : Bool[] {
        mutable array = [false, size = N];
        for i in 0..N - 1 {
            set array w/= i <- DrawRandomInt(0, 1) == 0;
        }
        return array;
    }

    operation PrepareQubits(qs : Qubit[], bases : Bool[], bits : Bool[]) : Unit {
        for i in 0..Length(qs) - 1 {
            if bits[i] {
                X(qs[i]);
            }
            if bases[i] {
                H(qs[i]);
            }
        }
    }

    operation MeasureQubits(qs : Qubit[], bases : Bool[]) : Bool[] {
        for i in 0..Length(qs) - 1 {
            if bases[i] {
                H(qs[i]);
            }
        }
        return ResultArrayAsBoolArray(MeasureEachZ(qs));
    }

    function GenerateSharedKey(basesAlice : Bool[], basesBob : Bool[], bits : Bool[]) : Bool[] {
        mutable key = [];
        for i in 0..Length(bits) - 1 {
            if basesAlice[i] == basesBob[i] {
                set key += [bits[i]];
            }
        }
        return key;
    }
}
