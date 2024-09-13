//
namespace Fuzz.Testing {
    import Std.Arrays.*;
    import Std.Canon.*;
    import Std.Characterization.*;
    import Std.Convert.*;
    import Std.Diagnostics.*;
    import Std.Intrinsic.*;
    import Std.Logical.*;
    import Std.MachineLearning.*;
    import Std.MachineLearning.Datasets as Datasets.*;
    import Std.Math.*;
    import Std.Measurement.*;
    import Std.Preparation.*;
    import Std.Random.*;
    import Std.Simulation.*;
    import Std.Synthesis.*;
    import Std.Targeting.*;

    function IfRetExpr(cond : Bool, a : Int, b : Int) : Int {
        let x = if cond { a } else { b };
        x
    }

    function NestedRetFail(arg : Int ) : String {
        if arg < 5 {
            fail "< 5";
        }
        elif arg > 9 {
            return "> 9";
        }
        "[5, 9]"
    }

    operation Repeat(limit : Int) : Int {
        mutable dec = limit + 5;
        repeat {
            set dec = dec - 1;
        }
        until (dec < limit);
        dec
    }

    function Loops(limit : Int) : Int {

        mutable count = 0 - 7;
        while (count < limit)
        {
            set count = count + 1;
        }
        return count + 5;
    }

    function PrnArr(arr : Bool[]) : Unit {
        Message(AsString(arr));
    }

    function CopyAndUpdate() : Unit {
        let mask = [false, size = 10];

        for i in Length(mask)-2 .. -1 .. 0 {
            let nbPair = mask 
                w/ i     <- true
                w/ i + 1 <- true;
            PrnArr(nbPair);
        }
    }

    function Mul(d : Double) : Double {
        return 1.0 * d;
    }

    function Ternary(nSites : Int, amplitude : Double, idxQubit : Int) : Double {
        return idxQubit == nSites - 1
               ? 0.0
               | amplitude;
    }

    operation EmptyOpWithSomeParams(
        nSites : Int, hXInitial : Double, hXFinal : Double,
        jFinal : Double, adiabaticTime : Double,
        qubits : Qubit[]
    ) : Unit
    is Adj + Ctl {
    }

    operation RetResultArr(nSites : Int, hXInitial : Double, jFinal : Double, adiabaticTime : Double, trotterStepSize : Double, trotterOrder : Int) : Result[] {
        let hXFinal = 0.0;
        use qubits = Qubit[nSites];
        return [One];
    }


    operation SomeQubitManip(a : Qubit, b : Qubit) : Unit is Adj + Ctl {
        Message("Classical version");
        CNOT(a, b);
    }

    operation CallAndWithinApply(a : Qubit, b : Qubit) : Unit is Adj + Ctl {
        let _ = SomeQubitManip;

        within {
            CNOT(a, b);
            H(a);
        } apply {
            CNOT(a, b);
        }
    }

    @EntryPoint()
    operation PassQubits() : Unit {
        use a = Qubit();
        use b = Qubit();
        CallAndWithinApply(a, b);
    }

    @EntryPoint()
    operation CallControlled(time : Double, angle : Double, lambda : ((Double, Qubit[]) => Unit is Ctl), qs : Qubit[]) : Result {
        mutable result = Zero;

        use controlQubit = Qubit();
        H(controlQubit);
        Rz(-time * angle, controlQubit);
        Controlled lambda([controlQubit], (time, qs));
        return Zero; 
    }

    operation NestedFor() : Unit {
        let dt = 0.1;
        let nTimes = 101;
        let nSamples = 100;
        let eigenphase = PI();
        let angle = 0.5 * PI();

        use eigenstate = Qubit();
        within {
            X(eigenstate);
        } apply {
            for idxTime in 0 .. nTimes - 1 {
                let time = dt * IntAsDouble(idxTime);
                mutable nOnesObserved = 0;

                for idxSample in 0 .. nSamples - 1 {
                    let sample = Zero;

                    if (sample == One) {
                        set nOnesObserved += 1;
                    }
                }

                let obs = IntAsDouble(nOnesObserved) / IntAsDouble(nSamples);
                let mean = 0.1;

            }
        }
    }

    function Indexing(xs : Double[], ys : Double[]) : Double {
        mutable sum = 0.0;

        for idxPoint in 0 .. Length(xs) - 2 {
            let trapezoidalHeight = (ys[idxPoint + 1] + ys[idxPoint]) * 0.5;
            let trapezoidalBase = xs[idxPoint + 1] - xs[idxPoint];
            set sum += trapezoidalBase * trapezoidalHeight;
        }

        return sum;
    }

    function CopyAndUpd(left : Double[], right : Double[]) : Double[] {
        mutable product = [0.0, size = Length(left)];

        for idxElement in IndexRange(left) {
            set product w/= idxElement <- left[idxElement] * right[idxElement];
        }

        return product;
    }

    internal operation Fail(pattern : Bool[], queryRegister : Qubit[], target : Qubit) : Unit {
        if (Length(queryRegister) != Length(pattern)) {
            fail "Length of input register must be equal to the pattern length.";
        }

        for (patternBit, controlQubit) in [(pattern[0], queryRegister[0])] {
            if (patternBit) {
                Controlled X([controlQubit], target);
            }
        }
    }

    operation InvokedOp(data : Qubit, auxiliaryQubits : Qubit[]) : Unit
        is Adj + Ctl
    {
    }

    operation InvokeAdjoints () : Unit {
        use data = Qubit();
        use auxiliaryQubits = Qubit[2];
        let register = [data] + auxiliaryQubits;
        Rx(PI() / 3.0, data);

        InvokedOp(data, auxiliaryQubits);

        let parity01 = Measure([PauliZ, PauliZ, PauliI], register);
        let parity12 = Measure([PauliI, PauliZ, PauliZ], register);

        Adjoint InvokedOp(data, auxiliaryQubits);
        Adjoint Rx(PI() / 3.0, data);
    }

    operation InvokeCtrlAdjoint (control1 : Qubit, control2 : Qubit, target : Qubit) : Unit is Adj + Ctl {
        CCNOT(control1, control2, target);
        Controlled (Adjoint S)([control1], control2);
    }
    operation Body (control1 : Qubit, control2 : Qubit, target : Qubit) : Unit is Ctl {
        body (...) {
            Adjoint T(control1);
            H(target);
            CNOT(target, control1);
            T(target);
            Adjoint T(control1);
            T(control1);
        }

        adjoint self;
    }

    operation ThreeQubitParams (control1 : Qubit, control2 : Qubit, target : Qubit) : Unit is Adj + Ctl {
        use auxiliary = Qubit();
    }
    
    operation BodyControlled (control1 : Qubit, control2 : Qubit, target : Qubit) : Unit {
        body (...) {
            use auxillaryQubit = Qubit();
            ThreeQubitParams(control1, control2, auxillaryQubit);
            S(auxillaryQubit);
            CNOT(auxillaryQubit, target);
            H(auxillaryQubit);

            if (M(auxillaryQubit) == One) {
                Controlled Z([control2], control1);
                X(auxillaryQubit);
            }
        }

        controlled (controls, ...) {
            Controlled X(controls + [control1, control2], target);
        }

        adjoint self;
    }
    @EntryPoint() 
    operation LetTuple() : Int {
        use q = Qubit[3];
        mutable mismatch = 0;
        for _ in 1..q::Length - 2 {
            H(q[0]);
            CNOT(q[0], q[1]);

            let (r0, r1, r2) = (One, Zero, One);   

            if not (r0 == r1 and r1 == r2) {
                set mismatch += 1;
            }
        }
        return mismatch;
    }
    
    operation IntDiv (bitsPerColor : Int, register : Qubit[]) : Int[] {
        let nVertices = Length(register) / bitsPerColor;
        return [0];
    }

    operation NestedMath() : Unit {
        for nDatabaseQubits in 4 .. 6 {
            for nIterations in 0 .. 5 {
                use markedQubit = Qubit();
                use databaseRegister = Qubit[nDatabaseQubits];

                let markedElements = [1, 4, 9];
                let nMarkedElements = Length(markedElements);

                let successAmplitude = Sin(IntAsDouble(2 * nIterations + 1) * ArcSin(Sqrt(IntAsDouble(nMarkedElements) / IntAsDouble(2 ^ nDatabaseQubits))));
                let successProbability = successAmplitude * successAmplitude;

                let result = One;
                let number = 5;

                if (result == One) {
                    fail "Found index should be in MarkedElements.";
                }
            }
        }
    }
    function IsEven(element : Int) : Bool {
        element % 2 == 0;
    }

    function IsSingleDigit(element : Int) : Bool {
        return element >= 0 and element < 10;
    }

    operation RetDoubleArr(coefficients : Double[], evaluationPoints : Double[],
                                 numBits : Int, pointPos : Int, odd : Bool, even : Bool)
                                 : Double[]
    {
        mutable results = [0.0, size = Length(evaluationPoints)];        
        for i in IndexRange(evaluationPoints) {
            let point = evaluationPoints[i];
            use xQubits = Qubit[numBits];
            use yQubits = Qubit[numBits];

            ResetAll(xQubits + yQubits);
        }
        return results;
    }
    function RetConstArrIndexed (idxBondLength : Int) : Double[] {
        return [
            [0.5678, -1.4508, 0.6799, 0.0791, 0.0791],
            [0.0984, 0.0679, 0.3329, 0.1475, 0.1475]
        ][idxBondLength];
    }

    function PowerCos(Results : Int, theta_1 : Double, theta_2 : Double, Measurements : Int): Unit{
        let DoubleVal = PI() * IntAsDouble(Results) / IntAsDouble(2 ^ (Measurements-1));
        let InnerProductValue = -Cos(DoubleVal);                                                      
    }
    operation TrippleFor() : Unit {
        let testList = [ (3, 5)
        ];

        for (actual, expected) in testList {
            for totalNumberOfQubits in 1 .. 8 {
                for numberOfControls in 1 .. totalNumberOfQubits - 1 {
                    Message("msg" + AsString(actual));
                }
            }
        }
    }
    function RichTrippleFor(func : Int[]) : Int[] {
        let bits = BitSizeI(func::Length - 1);
        mutable res = func;
        for m in 0..bits - 1 {
            mutable s = 1 <<< m;
            for i in 0..(2 * s)..Length(func) - 1 {
                mutable k = i + s;
                for j in i..i + s - 1 {
                    mutable t = res[j];
                    set res w/= j <- res[j] + res[k];
                    set res w/= k <- t - res[k];
                    set k = k + 1;
                }
            }
        }
        return res;
    }
    function DoublePower(sigma : Double, mu : Double, N : Int) : Double {
        let n = IntAsDouble(N);
        return -((n - mu) ^ 2.) / sigma ^ 2.; 
    }

    operation Fixup (target : Qubit) : Unit {

        body (...) {
            use aux0 = Qubit();
            use aux1 = Qubit();

            repeat {
                BodyControlled(aux0, aux1, target);
                S(target);

                BodyControlled(aux0, aux1, target);
                Z(target);

                let outcome0 = Measure([PauliX], [aux0]);
                let prob = outcome0 == One ? 0.5 | 5.0 / 6.0;
                let outcome1 = Measure([PauliX], [aux1]);
            }
            until (outcome0 == Zero and outcome1 == Zero)
            fixup {
                if (outcome1 == One) {
                    Z(aux1);
                }
            }

        }

        adjoint (...) {
            within {
                X(target);
            } apply {
                Fixup(target);
            }
        }
    }
    
    operation BitShift(
        generator : Int,
        modulus : Int,
        bitsize : Int
    )
    : Int {
        mutable frequencyEstimate = 0;
        let bitsPrecision =  2 * bitsize + 1;

        use eigenstateRegister = Qubit[bitsize];
        
        use c = Qubit();
        for idx in bitsPrecision - 1..-1..0 {
            within {
                H(c);
            } apply {
                R1Frac(frequencyEstimate, bitsPrecision - 1 - idx, c);
            }
            if true {
                set frequencyEstimate += 1 <<< (bitsPrecision - 1 - idx);
            }
        }

        ResetAll(eigenstateRegister);

        return frequencyEstimate;
    }

    internal operation RetTuple (
        inputValues : Bool[],
        encodingBases : Pauli[], 
        qubitIndices : Int[]
    ) : (Result, Result[]) {
        if ((Length(inputValues) != Length(encodingBases)) 
            or (Length(inputValues) != Length(qubitIndices))) {
            fail "Lengths of input values, encoding bases and qubitIndices must be equal.";
        }

        use block = Qubit[Length(inputValues)];
        use auxiliary = Qubit();
        for (qubit, value, basis) in [(block[0], inputValues[0], encodingBases[0])] {
        }

        H(auxiliary);
        for (index, basis) in [(qubitIndices[0], encodingBases[0])] { 
        }
        let auxiliaryResult = Measure([PauliX], [auxiliary]);
        let dataResult = [One];

        return (auxiliaryResult, dataResult);
    }

}
