namespace Kata {
    import Std.Convert.*;
    import Std.Math.*;

    operation FourBitstringSuperposition(qs : Qubit[], bits : Bool[][]) : Unit {
        FourBitstringSuperposition_Recursive([], qs, bits);
    }

    operation FourBitstringSuperposition_Recursive(currentBitString : Bool[], qs : Qubit[], bits : Bool[][]) : Unit {
        // an array of bit strings whose columns we are considering begin with |0⟩
        mutable zeroLeads = [];
        // an array of bit strings whose columns we are considering begin with |1⟩
        mutable oneLeads = [];
        // the number of bit strings we're considering
        let rows = Length(bits);
        // the current position we're considering
        let currentIndex = Length(currentBitString);

        if rows >= 1 and currentIndex < Length(qs) {
            // figure out what percentage of the bits should be |0⟩
            for row in 0..rows - 1 {
                if bits[row][currentIndex] {
                    set oneLeads = oneLeads + [bits[row]];
                } else {
                    set zeroLeads = zeroLeads + [bits[row]];
                }
            }
            // rotate the qubit to adjust coefficients based on the previous bit string
            // for the first path through, when the bit string has zero length,
            // the Controlled version of the rotation will perform a regular rotation
            let theta = ArcCos(Sqrt(IntAsDouble(Length(zeroLeads)) / IntAsDouble(rows)));
            ApplyControlledOnBitString(
                currentBitString,
                Ry,
                qs[0..currentIndex - 1],
                (2.0 * theta, qs[currentIndex])
            );

            // call state preparation recursively based on the bit strings so far
            FourBitstringSuperposition_Recursive(currentBitString + [false], qs, zeroLeads);
            FourBitstringSuperposition_Recursive(currentBitString + [true], qs, oneLeads);
        }
    }
}
