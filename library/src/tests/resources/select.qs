namespace Test {
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Measurement;
    open Microsoft.Quantum.Random;
    open Microsoft.Quantum.Unstable.TableLookup;

    internal operation TestSelect(addressBits : Int, dataBits : Int) : Unit {
        use addressRegister = Qubit[addressBits];
        use temporaryRegister = Qubit[dataBits];
        use dataRegister = Qubit[dataBits];

        let data = DrawMany(_ => DrawMany(_ => (DrawRandomInt(0, 1) == 1), dataBits, 0), 2^addressBits, 0);

        for (index, expected) in Enumerated(data) {
            ApplyXorInPlace(index, addressRegister);

            // a temporary register is not necessary normally, but we want to
            // test the optimized adjoint operation as well.
            within {
                Select(data, addressRegister, temporaryRegister);
            } apply {
                ApplyToEach(CNOT, Zipped(temporaryRegister, dataRegister));
            }

            Fact(Mapped(ResultAsBool, MResetEachZ(dataRegister)) == expected, $"Invalid data result for address {index}");
            Fact(MeasureInteger(addressRegister) == index, $"Invalid address result for address {index}");
        }
    }

    internal operation TestSelectFuzz(rounds : Int) : Unit {
        for _ in 1..rounds {
            let addressBits = DrawRandomInt(2, 6);
            let dataBits = 10;
            let numData = DrawRandomInt(2^(addressBits - 1) + 1, 2^addressBits - 1);

            let data = DrawMany(_ => DrawMany(_ => (DrawRandomInt(0, 1) == 1), dataBits, 0), numData, 0);

            use addressRegister = Qubit[addressBits];
            use temporaryRegister = Qubit[dataBits];
            use dataRegister = Qubit[dataBits];

            for _ in 1..5 {
                let index = DrawRandomInt(0, numData - 1);

                ApplyXorInPlace(index, addressRegister);

                // a temporary register is not necessary normally, but we want to
                // test the optimized adjoint operation as well.
                within {
                    Select(data, addressRegister, temporaryRegister);
                } apply {
                    ApplyToEach(CNOT, Zipped(temporaryRegister, dataRegister));
                }

                Fact(Mapped(ResultAsBool, MResetEachZ(dataRegister)) == data[index], $"Invalid data result for address {index} (addressBits = {addressBits}, dataBits = {dataBits})");
                Fact(MeasureInteger(addressRegister) == index, $"Invalid address result for address {index} (addressBits = {addressBits}, dataBits = {dataBits})");
            }
        }
    }
}
