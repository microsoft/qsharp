// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::generate_docs;
use expect_test::expect;

#[test]
fn generates_standard_item() {
    let files = generate_docs(None, None, None);
    let (_, metadata, contents) = files
        .iter()
        .find(|(file_name, _, _)| &**file_name == "Std.Core/Length.md")
        .expect("Could not find doc file for Length");
    let full_contents = format!("{metadata}\n\n{contents}");

    expect![[r#"
        ---
        uid: Qdk.Std.Core.Length
        title: Length function
        description: "Q# Length function: Returns the number of elements in the input array `a`."
        ms.date: {TIMESTAMP}
        ms.topic: managed-reference
        qsharp.kind: function
        qsharp.package: __Core__
        qsharp.namespace: Std.Core
        qsharp.name: Length
        qsharp.summary: "Returns the number of elements in the input array `a`."
        ---

        # Length function

        Fully qualified name: Std.Core.Length

        ```qsharp
        function Length<'T>(a : 'T[]) : Int
        ```

        ## Summary
        Returns the number of elements in the input array `a`.

        ## Input
        ### a
        Input array.

        ## Output
        The total number of elements in the input array `a`.

        ## Example
        ```qsharp
        Message($"{ Length([0, 0, 0]) }"); // Prints 3
        ```
    "#]]
    .assert_eq(full_contents.as_str());
}

#[test]
fn generates_unrestricted_item() {
    let files = generate_docs(None, None, None);
    let (_, metadata, contents) = files
        .iter()
        .find(|(file_name, _, _)| &**file_name == "Std.Diagnostics/CheckZero.md")
        .expect("Could not file doc file for CheckZero");
    let full_contents = format!("{metadata}\n\n{contents}");

    expect![[r#"
        ---
        uid: Qdk.Std.Diagnostics.CheckZero
        title: CheckZero operation
        description: "Q# CheckZero operation: Checks whether a qubit is in the \|0⟩ state, returning true if it is."
        ms.date: {TIMESTAMP}
        ms.topic: managed-reference
        qsharp.kind: operation
        qsharp.package: __Std__
        qsharp.namespace: Std.Diagnostics
        qsharp.name: CheckZero
        qsharp.summary: "Checks whether a qubit is in the \|0⟩ state, returning true if it is."
        ---

        # CheckZero operation

        Fully qualified name: Std.Diagnostics.CheckZero

        ```qsharp
        operation CheckZero(qubit : Qubit) : Bool
        ```

        ## Summary
        Checks whether a qubit is in the |0⟩ state, returning true if it is.

        ## Description
        This operation checks whether a qubit is in the |0⟩ state. It will return true only
        if the qubit is deterministically in the |0⟩ state, and will return false otherwise. This operation
        does not change the state of the qubit.

        ## Input
        ### qubit
        The qubit to check.
        ## Output
        True if the qubit is in the |0⟩ state, false otherwise.

        ## Remarks
        This operation is useful for checking whether a qubit is in the |0⟩ state during simulation. It is not possible to check
        this on hardware without measuring the qubit, which could change the state.
    "#]]
    .assert_eq(full_contents.as_str());
}

#[test]
fn redirect_generation() {
    let files = generate_docs(None, None, None);
    let (_, metadata, contents) = files
        .iter()
        .find(|(file_name, _, _)| &**file_name == "Microsoft.Quantum.Core/Length.md")
        .expect("Could not find doc file for Length");
    let full_contents = format!("{metadata}\n\n{contents}");

    expect![[r#"
        ---
        uid: Qdk.Microsoft.Quantum.Core.Length
        title: Length exported item
        description: "Q# Length exported item: This is an exported item. The actual definition is found here: [Length](xref:Qdk.Std.Core.Length)"
        ms.date: {TIMESTAMP}
        ms.topic: managed-reference
        qsharp.kind: export
        qsharp.package: __Std__
        qsharp.namespace: Microsoft.Quantum.Core
        qsharp.name: Length
        qsharp.summary: "This is an exported item. The actual definition is found here: [Length](xref:Qdk.Std.Core.Length)"
        ---

        # Length exported item

        Fully qualified name: Microsoft.Quantum.Core.Length

        This is an exported item. The actual definition is found here: [Length](xref:Qdk.Std.Core.Length)
    "#]]
    .assert_eq(full_contents.as_str());
}

#[test]
fn index_file_generation() {
    let files = generate_docs(None, None, None);
    let (_, metadata, contents) = files
        .iter()
        .find(|(file_name, _, _)| &**file_name == "Std.Core/index.md")
        .expect("Could not find Std.Core Table of Contents file");
    let full_contents = format!("{metadata}\n\n{contents}");

    expect![[r#"
        ---
        uid: Qdk.Std.Core-toc
        title: Std.Core namespace
        description: Table of contents for the Q# Core namespace
        author: bradben
        ms.author: brbenefield
        ms.date: {TIMESTAMP}
        ms.topic: landing-page
        ---

        # Std.Core

        The Std.Core namespace contains the following items:

        | Name | Description |
        |------|-------------|
        | [Length](xref:Qdk.Std.Core.Length) | Returns the number of elements in the input array `a`. |
        | [Repeated](xref:Qdk.Std.Core.Repeated) | Creates an array of given `length` with all elements equal to given `value`. `length` must be a non-negative integer. |
    "#]]
    .assert_eq(full_contents.as_str());
}

#[allow(clippy::too_many_lines)]
#[test]
fn toc_generation() {
    let files = generate_docs(None, None, None);
    let (_, metadata, contents) = files
        .iter()
        .find(|(file_name, _, _)| &**file_name == "toc.yml")
        .expect("Could not find table of contents file");
    let full_contents = format!("{metadata}\n\n{contents}");

    expect![[r#"



        # This file is automatically generated.
        # Please do not modify this file manually, or your changes will be lost when
        # documentation is rebuilt.
        - items:
          name: Overview
          uid: Microsoft.Quantum.apiref-toc
        - items:
          - {name: Overview, uid: Qdk.Microsoft.Quantum.Core-toc}
          - {name: Length, uid: Qdk.Microsoft.Quantum.Core.Length}
          - {name: Repeated, uid: Qdk.Microsoft.Quantum.Core.Repeated}
          name: Microsoft.Quantum.Core
          uid: Qdk.Microsoft.Quantum.Core
        - items:
          - {name: Overview, uid: Qdk.Std.Arrays-toc}
          - {name: All, uid: Qdk.Std.Arrays.All}
          - {name: Any, uid: Qdk.Std.Arrays.Any}
          - {name: Chunks, uid: Qdk.Std.Arrays.Chunks}
          - {name: CircularlyShifted, uid: Qdk.Std.Arrays.CircularlyShifted}
          - {name: ColumnAt, uid: Qdk.Std.Arrays.ColumnAt}
          - {name: Count, uid: Qdk.Std.Arrays.Count}
          - {name: Diagonal, uid: Qdk.Std.Arrays.Diagonal}
          - {name: DrawMany, uid: Qdk.Std.Arrays.DrawMany}
          - {name: Enumerated, uid: Qdk.Std.Arrays.Enumerated}
          - {name: Excluding, uid: Qdk.Std.Arrays.Excluding}
          - {name: Filtered, uid: Qdk.Std.Arrays.Filtered}
          - {name: FlatMapped, uid: Qdk.Std.Arrays.FlatMapped}
          - {name: Flattened, uid: Qdk.Std.Arrays.Flattened}
          - {name: Fold, uid: Qdk.Std.Arrays.Fold}
          - {name: ForEach, uid: Qdk.Std.Arrays.ForEach}
          - {name: Head, uid: Qdk.Std.Arrays.Head}
          - {name: HeadAndRest, uid: Qdk.Std.Arrays.HeadAndRest}
          - {name: IndexOf, uid: Qdk.Std.Arrays.IndexOf}
          - {name: IndexRange, uid: Qdk.Std.Arrays.IndexRange}
          - {name: Interleaved, uid: Qdk.Std.Arrays.Interleaved}
          - {name: IsEmpty, uid: Qdk.Std.Arrays.IsEmpty}
          - {name: IsRectangularArray, uid: Qdk.Std.Arrays.IsRectangularArray}
          - {name: IsSorted, uid: Qdk.Std.Arrays.IsSorted}
          - {name: IsSquareArray, uid: Qdk.Std.Arrays.IsSquareArray}
          - {name: Mapped, uid: Qdk.Std.Arrays.Mapped}
          - {name: MappedByIndex, uid: Qdk.Std.Arrays.MappedByIndex}
          - {name: MappedOverRange, uid: Qdk.Std.Arrays.MappedOverRange}
          - {name: Most, uid: Qdk.Std.Arrays.Most}
          - {name: MostAndTail, uid: Qdk.Std.Arrays.MostAndTail}
          - {name: Padded, uid: Qdk.Std.Arrays.Padded}
          - {name: Partitioned, uid: Qdk.Std.Arrays.Partitioned}
          - {name: Rest, uid: Qdk.Std.Arrays.Rest}
          - {name: Reversed, uid: Qdk.Std.Arrays.Reversed}
          - {name: SequenceI, uid: Qdk.Std.Arrays.SequenceI}
          - {name: SequenceL, uid: Qdk.Std.Arrays.SequenceL}
          - {name: Sorted, uid: Qdk.Std.Arrays.Sorted}
          - {name: Subarray, uid: Qdk.Std.Arrays.Subarray}
          - {name: Swapped, uid: Qdk.Std.Arrays.Swapped}
          - {name: Tail, uid: Qdk.Std.Arrays.Tail}
          - {name: Transposed, uid: Qdk.Std.Arrays.Transposed}
          - {name: Unzipped, uid: Qdk.Std.Arrays.Unzipped}
          - {name: Where, uid: Qdk.Std.Arrays.Where}
          - {name: Windows, uid: Qdk.Std.Arrays.Windows}
          - {name: Zipped, uid: Qdk.Std.Arrays.Zipped}
          name: Std.Arrays
          uid: Qdk.Std.Arrays
        - items:
          - {name: Overview, uid: Qdk.Std.Canon-toc}
          - {name: ApplyCNOTChain, uid: Qdk.Std.Canon.ApplyCNOTChain}
          - {name: ApplyControlledOnBitString, uid: Qdk.Std.Canon.ApplyControlledOnBitString}
          - {name: ApplyControlledOnInt, uid: Qdk.Std.Canon.ApplyControlledOnInt}
          - {name: ApplyP, uid: Qdk.Std.Canon.ApplyP}
          - {name: ApplyPauli, uid: Qdk.Std.Canon.ApplyPauli}
          - {name: ApplyPauliFromBitString, uid: Qdk.Std.Canon.ApplyPauliFromBitString}
          - {name: ApplyPauliFromInt, uid: Qdk.Std.Canon.ApplyPauliFromInt}
          - {name: ApplyQFT, uid: Qdk.Std.Canon.ApplyQFT}
          - {name: ApplyToEach, uid: Qdk.Std.Canon.ApplyToEach}
          - {name: ApplyToEachA, uid: Qdk.Std.Canon.ApplyToEachA}
          - {name: ApplyToEachC, uid: Qdk.Std.Canon.ApplyToEachC}
          - {name: ApplyToEachCA, uid: Qdk.Std.Canon.ApplyToEachCA}
          - {name: ApplyXorInPlace, uid: Qdk.Std.Canon.ApplyXorInPlace}
          - {name: ApplyXorInPlaceL, uid: Qdk.Std.Canon.ApplyXorInPlaceL}
          - {name: CX, uid: Qdk.Std.Canon.CX}
          - {name: CY, uid: Qdk.Std.Canon.CY}
          - {name: CZ, uid: Qdk.Std.Canon.CZ}
          - {name: Fst, uid: Qdk.Std.Canon.Fst}
          - {name: Relabel, uid: Qdk.Std.Canon.Relabel}
          - {name: Snd, uid: Qdk.Std.Canon.Snd}
          - {name: SwapReverseRegister, uid: Qdk.Std.Canon.SwapReverseRegister}
          name: Std.Canon
          uid: Qdk.Std.Canon
        - items:
          - {name: Overview, uid: Qdk.Std.Convert-toc}
          - {name: BigIntAsBoolArray, uid: Qdk.Std.Convert.BigIntAsBoolArray}
          - {name: BoolArrayAsBigInt, uid: Qdk.Std.Convert.BoolArrayAsBigInt}
          - {name: BoolArrayAsInt, uid: Qdk.Std.Convert.BoolArrayAsInt}
          - {name: BoolArrayAsResultArray, uid: Qdk.Std.Convert.BoolArrayAsResultArray}
          - {name: BoolAsResult, uid: Qdk.Std.Convert.BoolAsResult}
          - {name: ComplexAsComplexPolar, uid: Qdk.Std.Convert.ComplexAsComplexPolar}
          - {name: ComplexPolarAsComplex, uid: Qdk.Std.Convert.ComplexPolarAsComplex}
          - {name: DoubleAsStringWithPrecision, uid: Qdk.Std.Convert.DoubleAsStringWithPrecision}
          - {name: IntAsBigInt, uid: Qdk.Std.Convert.IntAsBigInt}
          - {name: IntAsBoolArray, uid: Qdk.Std.Convert.IntAsBoolArray}
          - {name: IntAsDouble, uid: Qdk.Std.Convert.IntAsDouble}
          - {name: ResultArrayAsBoolArray, uid: Qdk.Std.Convert.ResultArrayAsBoolArray}
          - {name: ResultArrayAsInt, uid: Qdk.Std.Convert.ResultArrayAsInt}
          - {name: ResultAsBool, uid: Qdk.Std.Convert.ResultAsBool}
          name: Std.Convert
          uid: Qdk.Std.Convert
        - items:
          - {name: Overview, uid: Qdk.Std.Core-toc}
          - {name: Length, uid: Qdk.Std.Core.Length}
          - {name: Repeated, uid: Qdk.Std.Core.Repeated}
          name: Std.Core
          uid: Qdk.Std.Core
        - items:
          - {name: Overview, uid: Qdk.Std.Diagnostics-toc}
          - {name: CheckAllZero, uid: Qdk.Std.Diagnostics.CheckAllZero}
          - {name: CheckOperationsAreEqual, uid: Qdk.Std.Diagnostics.CheckOperationsAreEqual}
          - {name: CheckZero, uid: Qdk.Std.Diagnostics.CheckZero}
          - {name: DumpMachine, uid: Qdk.Std.Diagnostics.DumpMachine}
          - {name: DumpOperation, uid: Qdk.Std.Diagnostics.DumpOperation}
          - {name: DumpRegister, uid: Qdk.Std.Diagnostics.DumpRegister}
          - {name: Fact, uid: Qdk.Std.Diagnostics.Fact}
          - {name: StartCountingFunction, uid: Qdk.Std.Diagnostics.StartCountingFunction}
          - {name: StartCountingOperation, uid: Qdk.Std.Diagnostics.StartCountingOperation}
          - {name: StartCountingQubits, uid: Qdk.Std.Diagnostics.StartCountingQubits}
          - {name: StopCountingFunction, uid: Qdk.Std.Diagnostics.StopCountingFunction}
          - {name: StopCountingOperation, uid: Qdk.Std.Diagnostics.StopCountingOperation}
          - {name: StopCountingQubits, uid: Qdk.Std.Diagnostics.StopCountingQubits}
          name: Std.Diagnostics
          uid: Qdk.Std.Diagnostics
        - items:
          - {name: Overview, uid: Qdk.Std.Intrinsic-toc}
          - {name: AND, uid: Qdk.Std.Intrinsic.AND}
          - {name: CCNOT, uid: Qdk.Std.Intrinsic.CCNOT}
          - {name: CNOT, uid: Qdk.Std.Intrinsic.CNOT}
          - {name: Exp, uid: Qdk.Std.Intrinsic.Exp}
          - {name: H, uid: Qdk.Std.Intrinsic.H}
          - {name: I, uid: Qdk.Std.Intrinsic.I}
          - {name: M, uid: Qdk.Std.Intrinsic.M}
          - {name: Measure, uid: Qdk.Std.Intrinsic.Measure}
          - {name: Message, uid: Qdk.Std.Intrinsic.Message}
          - {name: R, uid: Qdk.Std.Intrinsic.R}
          - {name: R1, uid: Qdk.Std.Intrinsic.R1}
          - {name: R1Frac, uid: Qdk.Std.Intrinsic.R1Frac}
          - {name: RFrac, uid: Qdk.Std.Intrinsic.RFrac}
          - {name: Reset, uid: Qdk.Std.Intrinsic.Reset}
          - {name: ResetAll, uid: Qdk.Std.Intrinsic.ResetAll}
          - {name: Rx, uid: Qdk.Std.Intrinsic.Rx}
          - {name: Rxx, uid: Qdk.Std.Intrinsic.Rxx}
          - {name: Ry, uid: Qdk.Std.Intrinsic.Ry}
          - {name: Ryy, uid: Qdk.Std.Intrinsic.Ryy}
          - {name: Rz, uid: Qdk.Std.Intrinsic.Rz}
          - {name: Rzz, uid: Qdk.Std.Intrinsic.Rzz}
          - {name: S, uid: Qdk.Std.Intrinsic.S}
          - {name: SWAP, uid: Qdk.Std.Intrinsic.SWAP}
          - {name: T, uid: Qdk.Std.Intrinsic.T}
          - {name: X, uid: Qdk.Std.Intrinsic.X}
          - {name: Y, uid: Qdk.Std.Intrinsic.Y}
          - {name: Z, uid: Qdk.Std.Intrinsic.Z}
          name: Std.Intrinsic
          uid: Qdk.Std.Intrinsic
        - items:
          - {name: Overview, uid: Qdk.Std.Logical-toc}
          - {name: Xor, uid: Qdk.Std.Logical.Xor}
          name: Std.Logical
          uid: Qdk.Std.Logical
        - items:
          - {name: Overview, uid: Qdk.Std.Math-toc}
          - {name: AbsComplex, uid: Qdk.Std.Math.AbsComplex}
          - {name: AbsComplexPolar, uid: Qdk.Std.Math.AbsComplexPolar}
          - {name: AbsD, uid: Qdk.Std.Math.AbsD}
          - {name: AbsI, uid: Qdk.Std.Math.AbsI}
          - {name: AbsL, uid: Qdk.Std.Math.AbsL}
          - {name: AbsSquaredComplex, uid: Qdk.Std.Math.AbsSquaredComplex}
          - {name: AbsSquaredComplexPolar, uid: Qdk.Std.Math.AbsSquaredComplexPolar}
          - {name: ApproximateFactorial, uid: Qdk.Std.Math.ApproximateFactorial}
          - {name: ArcCos, uid: Qdk.Std.Math.ArcCos}
          - {name: ArcCosh, uid: Qdk.Std.Math.ArcCosh}
          - {name: ArcSin, uid: Qdk.Std.Math.ArcSin}
          - {name: ArcSinh, uid: Qdk.Std.Math.ArcSinh}
          - {name: ArcTan, uid: Qdk.Std.Math.ArcTan}
          - {name: ArcTan2, uid: Qdk.Std.Math.ArcTan2}
          - {name: ArcTanh, uid: Qdk.Std.Math.ArcTanh}
          - {name: ArgComplex, uid: Qdk.Std.Math.ArgComplex}
          - {name: ArgComplexPolar, uid: Qdk.Std.Math.ArgComplexPolar}
          - {name: Binom, uid: Qdk.Std.Math.Binom}
          - {name: BitSizeI, uid: Qdk.Std.Math.BitSizeI}
          - {name: BitSizeL, uid: Qdk.Std.Math.BitSizeL}
          - {name: Ceiling, uid: Qdk.Std.Math.Ceiling}
          - {name: Complex, uid: Qdk.Std.Math.Complex}
          - {name: ComplexPolar, uid: Qdk.Std.Math.ComplexPolar}
          - {name: ContinuedFractionConvergentI, uid: Qdk.Std.Math.ContinuedFractionConvergentI}
          - {name: ContinuedFractionConvergentL, uid: Qdk.Std.Math.ContinuedFractionConvergentL}
          - {name: Cos, uid: Qdk.Std.Math.Cos}
          - {name: Cosh, uid: Qdk.Std.Math.Cosh}
          - {name: DivRemI, uid: Qdk.Std.Math.DivRemI}
          - {name: DivRemL, uid: Qdk.Std.Math.DivRemL}
          - {name: DividedByC, uid: Qdk.Std.Math.DividedByC}
          - {name: DividedByCP, uid: Qdk.Std.Math.DividedByCP}
          - {name: E, uid: Qdk.Std.Math.E}
          - {name: ExpModI, uid: Qdk.Std.Math.ExpModI}
          - {name: ExpModL, uid: Qdk.Std.Math.ExpModL}
          - {name: ExtendedGreatestCommonDivisorI, uid: Qdk.Std.Math.ExtendedGreatestCommonDivisorI}
          - {name: ExtendedGreatestCommonDivisorL, uid: Qdk.Std.Math.ExtendedGreatestCommonDivisorL}
          - {name: FactorialI, uid: Qdk.Std.Math.FactorialI}
          - {name: FactorialL, uid: Qdk.Std.Math.FactorialL}
          - {name: Floor, uid: Qdk.Std.Math.Floor}
          - {name: GreatestCommonDivisorI, uid: Qdk.Std.Math.GreatestCommonDivisorI}
          - {name: GreatestCommonDivisorL, uid: Qdk.Std.Math.GreatestCommonDivisorL}
          - {name: HammingWeightI, uid: Qdk.Std.Math.HammingWeightI}
          - {name: InverseModI, uid: Qdk.Std.Math.InverseModI}
          - {name: InverseModL, uid: Qdk.Std.Math.InverseModL}
          - {name: IsCoprimeI, uid: Qdk.Std.Math.IsCoprimeI}
          - {name: IsCoprimeL, uid: Qdk.Std.Math.IsCoprimeL}
          - {name: IsInfinite, uid: Qdk.Std.Math.IsInfinite}
          - {name: IsNaN, uid: Qdk.Std.Math.IsNaN}
          - {name: LargestFixedPoint, uid: Qdk.Std.Math.LargestFixedPoint}
          - {name: Lg, uid: Qdk.Std.Math.Lg}
          - {name: Log, uid: Qdk.Std.Math.Log}
          - {name: Log10, uid: Qdk.Std.Math.Log10}
          - {name: LogFactorialD, uid: Qdk.Std.Math.LogFactorialD}
          - {name: LogGammaD, uid: Qdk.Std.Math.LogGammaD}
          - {name: LogOf2, uid: Qdk.Std.Math.LogOf2}
          - {name: Max, uid: Qdk.Std.Math.Max}
          - {name: MaxD, uid: Qdk.Std.Math.MaxD}
          - {name: MaxI, uid: Qdk.Std.Math.MaxI}
          - {name: MaxL, uid: Qdk.Std.Math.MaxL}
          - {name: Min, uid: Qdk.Std.Math.Min}
          - {name: MinD, uid: Qdk.Std.Math.MinD}
          - {name: MinI, uid: Qdk.Std.Math.MinI}
          - {name: MinL, uid: Qdk.Std.Math.MinL}
          - {name: MinusC, uid: Qdk.Std.Math.MinusC}
          - {name: MinusCP, uid: Qdk.Std.Math.MinusCP}
          - {name: ModulusI, uid: Qdk.Std.Math.ModulusI}
          - {name: ModulusL, uid: Qdk.Std.Math.ModulusL}
          - {name: NegationC, uid: Qdk.Std.Math.NegationC}
          - {name: NegationCP, uid: Qdk.Std.Math.NegationCP}
          - {name: PI, uid: Qdk.Std.Math.PI}
          - {name: PNorm, uid: Qdk.Std.Math.PNorm}
          - {name: PNormalized, uid: Qdk.Std.Math.PNormalized}
          - {name: PlusC, uid: Qdk.Std.Math.PlusC}
          - {name: PlusCP, uid: Qdk.Std.Math.PlusCP}
          - {name: PowC, uid: Qdk.Std.Math.PowC}
          - {name: PowCP, uid: Qdk.Std.Math.PowCP}
          - {name: RealMod, uid: Qdk.Std.Math.RealMod}
          - {name: Round, uid: Qdk.Std.Math.Round}
          - {name: SignD, uid: Qdk.Std.Math.SignD}
          - {name: SignI, uid: Qdk.Std.Math.SignI}
          - {name: SignL, uid: Qdk.Std.Math.SignL}
          - {name: Sin, uid: Qdk.Std.Math.Sin}
          - {name: Sinh, uid: Qdk.Std.Math.Sinh}
          - {name: SmallestFixedPoint, uid: Qdk.Std.Math.SmallestFixedPoint}
          - {name: Sqrt, uid: Qdk.Std.Math.Sqrt}
          - {name: SquaredNorm, uid: Qdk.Std.Math.SquaredNorm}
          - {name: Tan, uid: Qdk.Std.Math.Tan}
          - {name: Tanh, uid: Qdk.Std.Math.Tanh}
          - {name: TimesC, uid: Qdk.Std.Math.TimesC}
          - {name: TimesCP, uid: Qdk.Std.Math.TimesCP}
          - {name: TrailingZeroCountI, uid: Qdk.Std.Math.TrailingZeroCountI}
          - {name: TrailingZeroCountL, uid: Qdk.Std.Math.TrailingZeroCountL}
          - {name: Truncate, uid: Qdk.Std.Math.Truncate}
          name: Std.Math
          uid: Qdk.Std.Math
        - items:
          - {name: Overview, uid: Qdk.Std.Measurement-toc}
          - {name: MResetEachZ, uid: Qdk.Std.Measurement.MResetEachZ}
          - {name: MResetX, uid: Qdk.Std.Measurement.MResetX}
          - {name: MResetY, uid: Qdk.Std.Measurement.MResetY}
          - {name: MResetZ, uid: Qdk.Std.Measurement.MResetZ}
          - {name: MeasureAllZ, uid: Qdk.Std.Measurement.MeasureAllZ}
          - {name: MeasureEachZ, uid: Qdk.Std.Measurement.MeasureEachZ}
          - {name: MeasureInteger, uid: Qdk.Std.Measurement.MeasureInteger}
          name: Std.Measurement
          uid: Qdk.Std.Measurement
        - items:
          - {name: Overview, uid: Qdk.Std.Random-toc}
          - {name: DrawRandomBool, uid: Qdk.Std.Random.DrawRandomBool}
          - {name: DrawRandomDouble, uid: Qdk.Std.Random.DrawRandomDouble}
          - {name: DrawRandomInt, uid: Qdk.Std.Random.DrawRandomInt}
          name: Std.Random
          uid: Qdk.Std.Random
        - items:
          - {name: Overview, uid: Qdk.Std.Range-toc}
          - {name: IsRangeEmpty, uid: Qdk.Std.Range.IsRangeEmpty}
          - {name: RangeEnd, uid: Qdk.Std.Range.RangeEnd}
          - {name: RangeReverse, uid: Qdk.Std.Range.RangeReverse}
          - {name: RangeStart, uid: Qdk.Std.Range.RangeStart}
          - {name: RangeStep, uid: Qdk.Std.Range.RangeStep}
          name: Std.Range
          uid: Qdk.Std.Range
        - items:
          - {name: Overview, uid: Qdk.Std.ResourceEstimation-toc}
          - {name: AccountForEstimates, uid: Qdk.Std.ResourceEstimation.AccountForEstimates}
          - {name: AuxQubitCount, uid: Qdk.Std.ResourceEstimation.AuxQubitCount}
          - {name: BeginEstimateCaching, uid: Qdk.Std.ResourceEstimation.BeginEstimateCaching}
          - {name: BeginRepeatEstimates, uid: Qdk.Std.ResourceEstimation.BeginRepeatEstimates}
          - {name: CczCount, uid: Qdk.Std.ResourceEstimation.CczCount}
          - {name: EndEstimateCaching, uid: Qdk.Std.ResourceEstimation.EndEstimateCaching}
          - {name: EndRepeatEstimates, uid: Qdk.Std.ResourceEstimation.EndRepeatEstimates}
          - {name: MeasurementCount, uid: Qdk.Std.ResourceEstimation.MeasurementCount}
          - {name: PSSPCLayout, uid: Qdk.Std.ResourceEstimation.PSSPCLayout}
          - {name: RepeatEstimates, uid: Qdk.Std.ResourceEstimation.RepeatEstimates}
          - {name: RotationCount, uid: Qdk.Std.ResourceEstimation.RotationCount}
          - {name: RotationDepth, uid: Qdk.Std.ResourceEstimation.RotationDepth}
          - {name: SingleVariant, uid: Qdk.Std.ResourceEstimation.SingleVariant}
          - {name: TCount, uid: Qdk.Std.ResourceEstimation.TCount}
          name: Std.ResourceEstimation
          uid: Qdk.Std.ResourceEstimation
        - items:
          - {name: Overview, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic-toc}
          - {name: AddLE, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.AddLE}
          - {name: ApplyIfEqualL, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.ApplyIfEqualL}
          - {name: ApplyIfEqualLE, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.ApplyIfEqualLE}
          - {name: ApplyIfGreaterL, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.ApplyIfGreaterL}
          - {name: ApplyIfGreaterLE, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.ApplyIfGreaterLE}
          - {name: ApplyIfGreaterOrEqualL, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.ApplyIfGreaterOrEqualL}
          - {name: ApplyIfGreaterOrEqualLE, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.ApplyIfGreaterOrEqualLE}
          - {name: ApplyIfLessL, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.ApplyIfLessL}
          - {name: ApplyIfLessLE, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.ApplyIfLessLE}
          - {name: ApplyIfLessOrEqualL, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.ApplyIfLessOrEqualL}
          - {name: ApplyIfLessOrEqualLE, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.ApplyIfLessOrEqualLE}
          - {name: FourierTDIncByLE, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.FourierTDIncByLE}
          - {name: IncByI, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.IncByI}
          - {name: IncByIUsingIncByLE, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.IncByIUsingIncByLE}
          - {name: IncByL, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.IncByL}
          - {name: IncByLE, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.IncByLE}
          - {name: IncByLEUsingAddLE, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.IncByLEUsingAddLE}
          - {name: IncByLUsingIncByLE, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.IncByLUsingIncByLE}
          - {name: LookAheadDKRSAddLE, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.LookAheadDKRSAddLE}
          - {name: MAJ, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.MAJ}
          - {name: ReflectAboutInteger, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.ReflectAboutInteger}
          - {name: RippleCarryCGAddLE, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.RippleCarryCGAddLE}
          - {name: RippleCarryCGIncByLE, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.RippleCarryCGIncByLE}
          - {name: RippleCarryTTKIncByLE, uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic.RippleCarryTTKIncByLE}
          name: Microsoft.Quantum.Unstable.Arithmetic
          uid: Qdk.Microsoft.Quantum.Unstable.Arithmetic
        - items:
          - {name: Overview, uid: Qdk.Microsoft.Quantum.Unstable.StatePreparation-toc}
          - {name: ApproximatelyPreparePureStateCP, uid: Qdk.Microsoft.Quantum.Unstable.StatePreparation.ApproximatelyPreparePureStateCP}
          - {name: PreparePureStateD, uid: Qdk.Microsoft.Quantum.Unstable.StatePreparation.PreparePureStateD}
          name: Microsoft.Quantum.Unstable.StatePreparation
          uid: Qdk.Microsoft.Quantum.Unstable.StatePreparation
        - items:
          - {name: Overview, uid: Qdk.Microsoft.Quantum.Unstable.TableLookup-toc}
          - {name: Select, uid: Qdk.Microsoft.Quantum.Unstable.TableLookup.Select}
          name: Microsoft.Quantum.Unstable.TableLookup
          uid: Qdk.Microsoft.Quantum.Unstable.TableLookup"#]]
    .assert_eq(full_contents.as_str());
}

#[allow(clippy::too_many_lines)]
#[test]
fn docs_file_list() {
    let files = generate_docs(None, None, None);
    let names = files
        .iter()
        .map(|(name, _, _)| name.as_ref())
        .collect::<Vec<_>>()
        .join("\n");

    expect![[r#"
        Microsoft.Quantum.Core/index.md
        Microsoft.Quantum.Core/Length.md
        Microsoft.Quantum.Core/Repeated.md
        Microsoft.Quantum.Unstable.Arithmetic/index.md
        Microsoft.Quantum.Unstable.Arithmetic/AddLE.md
        Microsoft.Quantum.Unstable.Arithmetic/ApplyIfEqualL.md
        Microsoft.Quantum.Unstable.Arithmetic/ApplyIfEqualLE.md
        Microsoft.Quantum.Unstable.Arithmetic/ApplyIfGreaterL.md
        Microsoft.Quantum.Unstable.Arithmetic/ApplyIfGreaterLE.md
        Microsoft.Quantum.Unstable.Arithmetic/ApplyIfGreaterOrEqualL.md
        Microsoft.Quantum.Unstable.Arithmetic/ApplyIfGreaterOrEqualLE.md
        Microsoft.Quantum.Unstable.Arithmetic/ApplyIfLessL.md
        Microsoft.Quantum.Unstable.Arithmetic/ApplyIfLessLE.md
        Microsoft.Quantum.Unstable.Arithmetic/ApplyIfLessOrEqualL.md
        Microsoft.Quantum.Unstable.Arithmetic/ApplyIfLessOrEqualLE.md
        Microsoft.Quantum.Unstable.Arithmetic/FourierTDIncByLE.md
        Microsoft.Quantum.Unstable.Arithmetic/IncByI.md
        Microsoft.Quantum.Unstable.Arithmetic/IncByIUsingIncByLE.md
        Microsoft.Quantum.Unstable.Arithmetic/IncByL.md
        Microsoft.Quantum.Unstable.Arithmetic/IncByLE.md
        Microsoft.Quantum.Unstable.Arithmetic/IncByLEUsingAddLE.md
        Microsoft.Quantum.Unstable.Arithmetic/IncByLUsingIncByLE.md
        Microsoft.Quantum.Unstable.Arithmetic/LookAheadDKRSAddLE.md
        Microsoft.Quantum.Unstable.Arithmetic/MAJ.md
        Microsoft.Quantum.Unstable.Arithmetic/ReflectAboutInteger.md
        Microsoft.Quantum.Unstable.Arithmetic/RippleCarryCGAddLE.md
        Microsoft.Quantum.Unstable.Arithmetic/RippleCarryCGIncByLE.md
        Microsoft.Quantum.Unstable.Arithmetic/RippleCarryTTKIncByLE.md
        Microsoft.Quantum.Unstable.StatePreparation/index.md
        Microsoft.Quantum.Unstable.StatePreparation/ApproximatelyPreparePureStateCP.md
        Microsoft.Quantum.Unstable.StatePreparation/PreparePureStateD.md
        Microsoft.Quantum.Unstable.TableLookup/index.md
        Microsoft.Quantum.Unstable.TableLookup/Select.md
        Std.Arrays/index.md
        Std.Arrays/All.md
        Std.Arrays/Any.md
        Std.Arrays/Chunks.md
        Std.Arrays/CircularlyShifted.md
        Std.Arrays/ColumnAt.md
        Std.Arrays/Count.md
        Std.Arrays/Diagonal.md
        Std.Arrays/DrawMany.md
        Std.Arrays/Enumerated.md
        Std.Arrays/Excluding.md
        Std.Arrays/Filtered.md
        Std.Arrays/FlatMapped.md
        Std.Arrays/Flattened.md
        Std.Arrays/Fold.md
        Std.Arrays/ForEach.md
        Std.Arrays/Head.md
        Std.Arrays/HeadAndRest.md
        Std.Arrays/IndexOf.md
        Std.Arrays/IndexRange.md
        Std.Arrays/Interleaved.md
        Std.Arrays/IsEmpty.md
        Std.Arrays/IsRectangularArray.md
        Std.Arrays/IsSorted.md
        Std.Arrays/IsSquareArray.md
        Std.Arrays/Mapped.md
        Std.Arrays/MappedByIndex.md
        Std.Arrays/MappedOverRange.md
        Std.Arrays/Most.md
        Std.Arrays/MostAndTail.md
        Std.Arrays/Padded.md
        Std.Arrays/Partitioned.md
        Std.Arrays/Rest.md
        Std.Arrays/Reversed.md
        Std.Arrays/SequenceI.md
        Std.Arrays/SequenceL.md
        Std.Arrays/Sorted.md
        Std.Arrays/Subarray.md
        Std.Arrays/Swapped.md
        Std.Arrays/Tail.md
        Std.Arrays/Transposed.md
        Std.Arrays/Unzipped.md
        Std.Arrays/Where.md
        Std.Arrays/Windows.md
        Std.Arrays/Zipped.md
        Std.Canon/index.md
        Std.Canon/ApplyCNOTChain.md
        Std.Canon/ApplyControlledOnBitString.md
        Std.Canon/ApplyControlledOnInt.md
        Std.Canon/ApplyP.md
        Std.Canon/ApplyPauli.md
        Std.Canon/ApplyPauliFromBitString.md
        Std.Canon/ApplyPauliFromInt.md
        Std.Canon/ApplyQFT.md
        Std.Canon/ApplyToEach.md
        Std.Canon/ApplyToEachA.md
        Std.Canon/ApplyToEachC.md
        Std.Canon/ApplyToEachCA.md
        Std.Canon/ApplyXorInPlace.md
        Std.Canon/ApplyXorInPlaceL.md
        Std.Canon/CX.md
        Std.Canon/CY.md
        Std.Canon/CZ.md
        Std.Canon/Fst.md
        Std.Canon/Relabel.md
        Std.Canon/Snd.md
        Std.Canon/SwapReverseRegister.md
        Std.Convert/index.md
        Std.Convert/BigIntAsBoolArray.md
        Std.Convert/BoolArrayAsBigInt.md
        Std.Convert/BoolArrayAsInt.md
        Std.Convert/BoolArrayAsResultArray.md
        Std.Convert/BoolAsResult.md
        Std.Convert/ComplexAsComplexPolar.md
        Std.Convert/ComplexPolarAsComplex.md
        Std.Convert/DoubleAsStringWithPrecision.md
        Std.Convert/IntAsBigInt.md
        Std.Convert/IntAsBoolArray.md
        Std.Convert/IntAsDouble.md
        Std.Convert/ResultArrayAsBoolArray.md
        Std.Convert/ResultArrayAsInt.md
        Std.Convert/ResultAsBool.md
        Std.Diagnostics/index.md
        Std.Diagnostics/CheckAllZero.md
        Std.Diagnostics/CheckOperationsAreEqual.md
        Std.Diagnostics/CheckZero.md
        Std.Diagnostics/DumpMachine.md
        Std.Diagnostics/DumpOperation.md
        Std.Diagnostics/DumpRegister.md
        Std.Diagnostics/Fact.md
        Std.Diagnostics/StartCountingFunction.md
        Std.Diagnostics/StartCountingOperation.md
        Std.Diagnostics/StartCountingQubits.md
        Std.Diagnostics/StopCountingFunction.md
        Std.Diagnostics/StopCountingOperation.md
        Std.Diagnostics/StopCountingQubits.md
        Std.Intrinsic/index.md
        Std.Intrinsic/AND.md
        Std.Intrinsic/CCNOT.md
        Std.Intrinsic/CNOT.md
        Std.Intrinsic/Exp.md
        Std.Intrinsic/H.md
        Std.Intrinsic/I.md
        Std.Intrinsic/M.md
        Std.Intrinsic/Measure.md
        Std.Intrinsic/Message.md
        Std.Intrinsic/R.md
        Std.Intrinsic/R1.md
        Std.Intrinsic/R1Frac.md
        Std.Intrinsic/RFrac.md
        Std.Intrinsic/Reset.md
        Std.Intrinsic/ResetAll.md
        Std.Intrinsic/Rx.md
        Std.Intrinsic/Rxx.md
        Std.Intrinsic/Ry.md
        Std.Intrinsic/Ryy.md
        Std.Intrinsic/Rz.md
        Std.Intrinsic/Rzz.md
        Std.Intrinsic/S.md
        Std.Intrinsic/SWAP.md
        Std.Intrinsic/T.md
        Std.Intrinsic/X.md
        Std.Intrinsic/Y.md
        Std.Intrinsic/Z.md
        Std.Logical/index.md
        Std.Logical/Xor.md
        Std.Math/index.md
        Std.Math/AbsComplex.md
        Std.Math/AbsComplexPolar.md
        Std.Math/AbsD.md
        Std.Math/AbsI.md
        Std.Math/AbsL.md
        Std.Math/AbsSquaredComplex.md
        Std.Math/AbsSquaredComplexPolar.md
        Std.Math/ApproximateFactorial.md
        Std.Math/ArcCos.md
        Std.Math/ArcCosh.md
        Std.Math/ArcSin.md
        Std.Math/ArcSinh.md
        Std.Math/ArcTan.md
        Std.Math/ArcTan2.md
        Std.Math/ArcTanh.md
        Std.Math/ArgComplex.md
        Std.Math/ArgComplexPolar.md
        Std.Math/Binom.md
        Std.Math/BitSizeI.md
        Std.Math/BitSizeL.md
        Std.Math/Ceiling.md
        Std.Math/Complex.md
        Std.Math/ComplexPolar.md
        Std.Math/ContinuedFractionConvergentI.md
        Std.Math/ContinuedFractionConvergentL.md
        Std.Math/Cos.md
        Std.Math/Cosh.md
        Std.Math/DivRemI.md
        Std.Math/DivRemL.md
        Std.Math/DividedByC.md
        Std.Math/DividedByCP.md
        Std.Math/E.md
        Std.Math/ExpModI.md
        Std.Math/ExpModL.md
        Std.Math/ExtendedGreatestCommonDivisorI.md
        Std.Math/ExtendedGreatestCommonDivisorL.md
        Std.Math/FactorialI.md
        Std.Math/FactorialL.md
        Std.Math/Floor.md
        Std.Math/GreatestCommonDivisorI.md
        Std.Math/GreatestCommonDivisorL.md
        Std.Math/HammingWeightI.md
        Std.Math/InverseModI.md
        Std.Math/InverseModL.md
        Std.Math/IsCoprimeI.md
        Std.Math/IsCoprimeL.md
        Std.Math/IsInfinite.md
        Std.Math/IsNaN.md
        Std.Math/LargestFixedPoint.md
        Std.Math/Lg.md
        Std.Math/Log.md
        Std.Math/Log10.md
        Std.Math/LogFactorialD.md
        Std.Math/LogGammaD.md
        Std.Math/LogOf2.md
        Std.Math/Max.md
        Std.Math/MaxD.md
        Std.Math/MaxI.md
        Std.Math/MaxL.md
        Std.Math/Min.md
        Std.Math/MinD.md
        Std.Math/MinI.md
        Std.Math/MinL.md
        Std.Math/MinusC.md
        Std.Math/MinusCP.md
        Std.Math/ModulusI.md
        Std.Math/ModulusL.md
        Std.Math/NegationC.md
        Std.Math/NegationCP.md
        Std.Math/PI.md
        Std.Math/PNorm.md
        Std.Math/PNormalized.md
        Std.Math/PlusC.md
        Std.Math/PlusCP.md
        Std.Math/PowC.md
        Std.Math/PowCP.md
        Std.Math/RealMod.md
        Std.Math/Round.md
        Std.Math/SignD.md
        Std.Math/SignI.md
        Std.Math/SignL.md
        Std.Math/Sin.md
        Std.Math/Sinh.md
        Std.Math/SmallestFixedPoint.md
        Std.Math/Sqrt.md
        Std.Math/SquaredNorm.md
        Std.Math/Tan.md
        Std.Math/Tanh.md
        Std.Math/TimesC.md
        Std.Math/TimesCP.md
        Std.Math/TrailingZeroCountI.md
        Std.Math/TrailingZeroCountL.md
        Std.Math/Truncate.md
        Std.Measurement/index.md
        Std.Measurement/MResetEachZ.md
        Std.Measurement/MResetX.md
        Std.Measurement/MResetY.md
        Std.Measurement/MResetZ.md
        Std.Measurement/MeasureAllZ.md
        Std.Measurement/MeasureEachZ.md
        Std.Measurement/MeasureInteger.md
        Std.Random/index.md
        Std.Random/DrawRandomBool.md
        Std.Random/DrawRandomDouble.md
        Std.Random/DrawRandomInt.md
        Std.Range/index.md
        Std.Range/IsRangeEmpty.md
        Std.Range/RangeEnd.md
        Std.Range/RangeReverse.md
        Std.Range/RangeStart.md
        Std.Range/RangeStep.md
        Std.ResourceEstimation/index.md
        Std.ResourceEstimation/AccountForEstimates.md
        Std.ResourceEstimation/AuxQubitCount.md
        Std.ResourceEstimation/BeginEstimateCaching.md
        Std.ResourceEstimation/BeginRepeatEstimates.md
        Std.ResourceEstimation/CczCount.md
        Std.ResourceEstimation/EndEstimateCaching.md
        Std.ResourceEstimation/EndRepeatEstimates.md
        Std.ResourceEstimation/MeasurementCount.md
        Std.ResourceEstimation/PSSPCLayout.md
        Std.ResourceEstimation/RepeatEstimates.md
        Std.ResourceEstimation/RotationCount.md
        Std.ResourceEstimation/RotationDepth.md
        Std.ResourceEstimation/SingleVariant.md
        Std.ResourceEstimation/TCount.md
        Std.Core/index.md
        Std.Core/Length.md
        Std.Core/Repeated.md
        index.md
        toc.yml"#]]
    .assert_eq(names.as_str());
}
