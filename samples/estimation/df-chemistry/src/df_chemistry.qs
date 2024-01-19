// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.
namespace Microsoft.Quantum.Applications.Chemistry {
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Canon;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.ResourceEstimation;
    open Microsoft.Quantum.Unstable.Arithmetic;
    open Microsoft.Quantum.Unstable.TableLookup;

    // ------------------------------------------ //
    // DF chemistry (public operations and types) //
    // ------------------------------------------ //

    /// # Summary
    /// The description of a double-factorized Hamiltonian
    ///
    /// # Reference
    /// arXiv:2007.14460, p. 9, eq. 9
    newtype DoubleFactorizedChemistryProblem = (
        // Number of orbitals (N, p. 8)
        NumOrbitals: Int,
        // one-body norm (ǁL⁽⁻¹⁾ǁ, p. 8, eq. 16)
        OneBodyNorm: Double,
        // one-body norm (¼∑ǁL⁽ʳ⁾ǁ², p. 8, eq. 16)
        TwoBodyNorm: Double,
        // eigenvalues in the EVD of the one-electron Hamiltonian (λₖ, p. 54, eq. 67)
        OneBodyEigenValues: Double[],
        // eigenvectors in the EVD of the one-electron Hamiltonian (Rₖ, p. 54, eq. 67)
        OneBodyEigenVectors: Double[][],
        // norms inside Λ_SH (p. 56, eq. 77)
        Lambdas: Double[],
        // eigenvalues in the EVDs of the two-electron Hamiltonian for all r (λₖ⁽ʳ⁾, p. 56, eq. 77)
        TwoBodyEigenValues: Double[][],
        // eigenvectors in the EVDs of the two-electron Hamiltonian for all r (R⁽ʳ⁾ₖ, p. 56, eq. 77)
        TwoBodyEigenVectors: Double[][][],
    );

    newtype DoubleFactorizedChemistryParameters = (
        // Standard deviation (ΔE, p. 8, eq. 1)
        // Typically set to 0.001
        StandardDeviation: Double,
    );

    /// # Summary
    /// Performs quantum circuit in (p. 45, eq. 33) for double-factorized
    /// Hamiltonian; also prepares phase gradient registers to use phase
    /// gradient technique (p. 55)
    operation DoubleFactorizedChemistry(
        problem : DoubleFactorizedChemistryProblem,
        parameters: DoubleFactorizedChemistryParameters
    ) : Unit {
        let constants = ComputeConstants(problem, parameters);

        use register0 = Qubit[problem::NumOrbitals];
        use register1 = Qubit[problem::NumOrbitals];
        use phaseGradientRegister = Qubit[constants::RotationAngleBitPrecision];

        // compute number of repetitions
        let norm = problem::OneBodyNorm + problem::TwoBodyNorm;
        let repetitions = Ceiling((PI() * norm) / (2.0 * parameters::StandardDeviation));

        let walkStep = MakeWalkStep(problem, constants);
        use walkStepHelper = Qubit[walkStep::NGarbageQubits];

        within {
            X(phaseGradientRegister[0]);
            Adjoint ApplyQFT(Reversed(phaseGradientRegister));
        } apply {
            within {
                RepeatEstimates(repetitions);
            } apply {
                walkStep::StepOp(register0, register1, phaseGradientRegister, walkStepHelper);
            }
        }
    }

    // -------------------------------------------- //
    // DF chemistry (internal operations and types) //
    // -------------------------------------------- //

    /// # Summary
    /// Constants that are used in the implementation of the one- and two-
    /// electron operators, which are computed from the double factorized
    /// problem and parameters.
    internal newtype DoubleFactorizedChemistryConstants = (
        RotationAngleBitPrecision: Int,
        StatePreparationBitPrecision: Int,
        TargetError: Double
    );

    internal function ComputeConstants(
        problem : DoubleFactorizedChemistryProblem,
        parameters : DoubleFactorizedChemistryParameters)
    : DoubleFactorizedChemistryConstants {
        let pj = 0.1;
        let barEpsilon = Sqrt(pj);
        let norm = problem::OneBodyNorm + problem::TwoBodyNorm;
        let epsilon = 0.1 * parameters::StandardDeviation / norm;

        let RotationAngleBitPrecision = Ceiling(1.152
            + Lg(Sqrt((IntAsDouble((problem::NumOrbitals - 1) * 8) * PI() * norm) / parameters::StandardDeviation))
            + 0.5 * Lg(1.0 / barEpsilon));
        let StatePreparationBitPrecision = Ceiling(Lg(1.0 / epsilon) + 2.5);
        let TargetError = 2.0^IntAsDouble(1 - StatePreparationBitPrecision);

        DoubleFactorizedChemistryConstants(
            RotationAngleBitPrecision,
            StatePreparationBitPrecision,
            TargetError
        )
    }

    internal newtype WalkStep = (
        NGarbageQubits: Int,
        StepOp: (Qubit[], Qubit[], Qubit[], Qubit[]) => Unit
    );

    /// # Summary
    /// Performs one B[H] and reflection (p. 45, eq. 33)
    internal function MakeWalkStep(
        problem : DoubleFactorizedChemistryProblem,
        constants : DoubleFactorizedChemistryConstants
    ) : WalkStep {
        let oneElectronOperator = MakeOneElectronOperator(problem::OneBodyEigenValues, problem::OneBodyEigenVectors, constants);
        let twoElectronOperator = MakeTwoElectronOperator(problem, constants);

        let NGarbageQubits = oneElectronOperator::NGarbageQubits + twoElectronOperator::NGarbageQubits;

        WalkStep(NGarbageQubits, WalkStepOperation(problem, oneElectronOperator, twoElectronOperator, _, _, _, _))
    }

    internal operation WalkStepOperation(
        problem : DoubleFactorizedChemistryProblem,
        oneElectronOperator : OneElectronOperator,
        twoElectronOperator : TwoElectronOperator,
        register0 : Qubit[],
        register1 : Qubit[],
        phaseGradientRegister : Qubit[],
        helper : Qubit[]
    ) : Unit {
        use ctl = Qubit();

        let helperParts = Partitioned([oneElectronOperator::NGarbageQubits, twoElectronOperator::NGarbageQubits], helper);
        Fact(IsEmpty(Tail(helperParts)), "wrong number of helper qubits in WalkStepOperation");

        // apply Hamiltionian
        within {
            PrepareSingleQubit(problem::OneBodyNorm, problem::TwoBodyNorm, ctl);
        } apply {
            within { X(ctl); } apply {
                Controlled oneElectronOperator::Apply([ctl], (register0, register1, phaseGradientRegister, [], helperParts[0]));
            }
            Controlled twoElectronOperator::Apply([ctl], (register0, register1, phaseGradientRegister, helperParts[1]));
        }

        // reflection
        ReflectAboutInteger(0, helper);
    }

    internal newtype OneElectronOperator = (
        NGarbageQubits: Int,
        Apply: (Qubit[], Qubit[], Qubit[], Qubit[], Qubit[]) => Unit is Adj + Ctl
    );

    /// # Summary
    /// Performs quantum circuit for one-electron operator (p. 54, eq. 70)
    internal function MakeOneElectronOperator(
        eigenValues : Double[],
        eigenVectors : Double[][],
        constants : DoubleFactorizedChemistryConstants
    ) : OneElectronOperator {
        let data = Mapped(eigenValue -> [eigenValue < 0.0], eigenValues);
        let prepare = MakePrepareArbitrarySuperpositionWithData(constants::TargetError, eigenValues, data);

        OneElectronOperator(
            prepare::NGarbageQubits,
            OneElectronOperatorOperation(eigenVectors, constants, prepare, _, _, _, _, _)
        )
    }

    internal operation OneElectronOperatorOperation(
        eigenVectors : Double[][],
        constants : DoubleFactorizedChemistryConstants,
        prepare : PrepareArbitrarySuperposition,
        register0 : Qubit[],
        register1 : Qubit[],
        phaseGradientRegister : Qubit[],
        offsets : Qubit[],
        helper : Qubit[]
    ) : Unit is Adj + Ctl {
        if BeginEstimateCaching("OneElectronOperator", IsEmpty(offsets) ? 0 | 1) {
            // assertions
            Fact(Length(helper) == prepare::NGarbageQubits, "invalid number of helper qubits in OneElectronOperatorOperation");

            let precision = constants::RotationAngleBitPrecision;
            let bitstrings = AllEigenVectorsAsBitString(eigenVectors, precision);

            use prepQubits = Qubit[prepare::NIndexQubits];
            use rotationQubits = Qubit[Length(Head(bitstrings))];
            use sign = Qubit();
            use spin = Qubit();

            within {
                if not IsEmpty(offsets) {
                    prepare::PrepareWithSelect(SelectWithOffset(_, offsets, _, _), prepQubits, [sign], helper);
                } else {
                    prepare::Prepare(prepQubits, [sign], helper);
                }
                H(spin);
                for i in IndexRange(register0) {
                    Controlled SWAP([spin], (register0[i], register1[i]));
                }
                if not IsEmpty(offsets) {
                    RippleCarryCGIncByLE(offsets, prepQubits);
                }
                Select(bitstrings, prepQubits, rotationQubits);
            } apply {
                ApplyGivensRotations(phaseGradientRegister, Chunks(precision, rotationQubits), register0);
                Z(sign);
            }

            EndEstimateCaching();
        }
    }

    internal operation SelectWithOffset(data : Bool[][], offset : Qubit[], address : Qubit[], target : Qubit[]) : Unit is Adj + Ctl {
        within {
            RippleCarryCGIncByLE(offset, address);
        } apply {
            Select(data, address, target);
        }
    }

    internal newtype TwoElectronOperator = (
        NGarbageQubits: Int,
        Apply: (Qubit[], Qubit[], Qubit[], Qubit[]) => Unit is Ctl
    );

    /// # Summary
    /// Performs quantum circuit for two-electron operator (p. 57, eq. 78)
    internal function MakeTwoElectronOperator(problem : DoubleFactorizedChemistryProblem, constants : DoubleFactorizedChemistryConstants) : TwoElectronOperator {
        let lambdaPrepare = MakePrepareArbitrarySuperposition(
            constants::TargetError,
            problem::Lambdas
        );

        let eigenValuesFlattened = Flattened(problem::TwoBodyEigenValues);
        let eigenVectorsFlattened = Flattened(problem::TwoBodyEigenVectors);
        let oneElectronOperator = MakeOneElectronOperator(eigenValuesFlattened, eigenVectorsFlattened, constants);

        let numGarbageQubits = lambdaPrepare::NGarbageQubits + oneElectronOperator::NGarbageQubits;

        TwoElectronOperator(numGarbageQubits, TwoElectronOperatorOperation(problem, lambdaPrepare, oneElectronOperator, _, _, _, _))
    }

    internal operation TwoElectronOperatorOperation(
        problem : DoubleFactorizedChemistryProblem,
        lambdaPrepare : PrepareArbitrarySuperposition,
        oneElectronOperator : OneElectronOperator,
        register0 : Qubit[],
        register1 : Qubit[],
        phaseGradientRegister : Qubit[],
        helper : Qubit[]
    ) : Unit is Ctl {
        let helperParts = Partitioned([lambdaPrepare::NGarbageQubits], helper);
        let dataOffsets = ComputeOffsetDataSet(Mapped(Length, problem::TwoBodyEigenValues));

        use rankQubits = Qubit[lambdaPrepare::NIndexQubits];
        use offsetQubits = Qubit[Length(dataOffsets[0])];

        within {
            lambdaPrepare::Prepare(rankQubits, [], helperParts[0]);
            Select(dataOffsets, rankQubits, offsetQubits);
            oneElectronOperator::Apply(register0, register1, phaseGradientRegister, offsetQubits, helperParts[1]);
        } apply {
            ReflectAboutInteger(0, helperParts[1]);
        }
    }

    internal function ComputeOffsetDataSet(lengths : Int[]) : Bool[][] {
        mutable currentOffset = 0;
        mutable offsets = [0, size = Length(lengths)];

        for i in IndexRange(lengths) {
            set offsets w/= i <- currentOffset;
            set currentOffset += lengths[i];
        }

        let offsetWidth = Ceiling(Lg(IntAsDouble(currentOffset)));
        return Mapped(IntAsBoolArray(_, offsetWidth), offsets);
    }

    /// # Summary
    /// Performs quantum circuit for phase rotations (p. 54, eq. 73)
    internal operation ApplyGivensRotations(pgr : Qubit[], rotationQubits : Qubit[][], target : Qubit[]) : Unit is Adj + Ctl {
        within {
            let windows = Windows(2, target);
            for i in RangeReverse(IndexRange(rotationQubits)) {
                let rotationControls = rotationQubits[i];
                let qs = windows[i];
                ApplyWithMajoranaCliffords(1, ApplyRotationUsingRippleCarryAddition(pgr, rotationControls, _), qs);
                ApplyWithMajoranaCliffords(0, Adjoint ApplyRotationUsingRippleCarryAddition(pgr, rotationControls, _), qs);
            }
        } apply {
            X(Head(target));
            Y(Head(target));
        }
    }

    internal operation ApplyRotationUsingRippleCarryAddition(pgr : Qubit[], rotationQubits : Qubit[], target : Qubit) : Unit is Adj {
        Controlled Adjoint RippleCarryCGIncByLE([target], (Reversed(Rest(rotationQubits)), pgr));
    }

    internal operation ApplyWithMajoranaCliffords(x : Int, op : (Qubit => Unit is Adj), qs : Qubit[]) : Unit is Adj {
        // x is the Majorana type (which can be either 0 or 1, see p. 52, eq. 59)
        within {
            Adjoint S(qs[x]);
            H(qs[x]);
            H(qs[1 - x]);
            CNOT(qs[1], qs[0]);
        } apply {
            op(qs[0]);
        }
    }

    internal function AllEigenVectorsAsBitString(eigenVectors : Double[][], precision : Int) : Bool[][] {
        mutable bitstrings = [];

        let tau = 2.0 * PI();
        let preFactor = 2.0^IntAsDouble(precision);

        for eigenVector in eigenVectors {
            // Computes rotation angles for Majorana operator ($\vec u$ in p. 52, eq. 55)
            mutable result = [];
            mutable sins = 1.0;

            for index in 0..Length(eigenVector)-2 {
                // We apply MinD, such that rounding errors do not lead to
                // an argument for ArcCos which is larger than 1.0. (p. 52, eq. 56)
                let theta = sins == 0.0 ? 0.0 | 0.5 * ArcCos(MinD(eigenVector[index] / sins, 1.0));
                
                // all angles as bit string
                let factor = theta / tau;
                set result += Reversed(IntAsBoolArray(IsNaN(factor) ? 0 | Floor(preFactor * factor), precision));

                set sins *= Sin(2.0 * theta);
            }

            set bitstrings += [result];
        }

        bitstrings
    }

    internal function IsNaN(value : Double) : Bool {
        value != value
    }
}
