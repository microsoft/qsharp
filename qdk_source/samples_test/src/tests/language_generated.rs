
// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This build-generated module contains tests for the samples in the `/samples/language` folder.
//! DO NOT MANUALLY EDIT THIS FILE. To regenerate this file, run `cargo check` or `cargo test` in the `samples_test` directory.

use super::language::*;
use super::{compile_and_run, compile_and_run_debug};
use qsc::SourceMap;

#[allow(non_snake_case)]
fn ArithmeticOperators_src() -> SourceMap {
    SourceMap::new(
        vec![("ArithmeticOperators.qs".into(), include_str!("../../../../../samples/language/ArithmeticOperators.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_ArithmeticOperators() {
    let output = compile_and_run(ArithmeticOperators_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample ArithmeticOperators.qs
    ARITHMETICOPERATORS_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_ArithmeticOperators() {
    let output = compile_and_run_debug(ArithmeticOperators_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample ArithmeticOperators.qs
    ARITHMETICOPERATORS_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Array_src() -> SourceMap {
    SourceMap::new(
        vec![("Array.qs".into(), include_str!("../../../../../samples/language/Array.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Array() {
    let output = compile_and_run(Array_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Array.qs
    ARRAY_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Array() {
    let output = compile_and_run_debug(Array_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Array.qs
    ARRAY_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn BigInt_src() -> SourceMap {
    SourceMap::new(
        vec![("BigInt.qs".into(), include_str!("../../../../../samples/language/BigInt.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_BigInt() {
    let output = compile_and_run(BigInt_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample BigInt.qs
    BIGINT_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_BigInt() {
    let output = compile_and_run_debug(BigInt_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample BigInt.qs
    BIGINT_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn BitwiseOperators_src() -> SourceMap {
    SourceMap::new(
        vec![("BitwiseOperators.qs".into(), include_str!("../../../../../samples/language/BitwiseOperators.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_BitwiseOperators() {
    let output = compile_and_run(BitwiseOperators_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample BitwiseOperators.qs
    BITWISEOPERATORS_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_BitwiseOperators() {
    let output = compile_and_run_debug(BitwiseOperators_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample BitwiseOperators.qs
    BITWISEOPERATORS_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Bool_src() -> SourceMap {
    SourceMap::new(
        vec![("Bool.qs".into(), include_str!("../../../../../samples/language/Bool.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Bool() {
    let output = compile_and_run(Bool_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Bool.qs
    BOOL_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Bool() {
    let output = compile_and_run_debug(Bool_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Bool.qs
    BOOL_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn ClassConstraints_src() -> SourceMap {
    SourceMap::new(
        vec![("ClassConstraints.qs".into(), include_str!("../../../../../samples/language/ClassConstraints.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_ClassConstraints() {
    let output = compile_and_run(ClassConstraints_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample ClassConstraints.qs
    CLASSCONSTRAINTS_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_ClassConstraints() {
    let output = compile_and_run_debug(ClassConstraints_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample ClassConstraints.qs
    CLASSCONSTRAINTS_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Comments_src() -> SourceMap {
    SourceMap::new(
        vec![("Comments.qs".into(), include_str!("../../../../../samples/language/Comments.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Comments() {
    let output = compile_and_run(Comments_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Comments.qs
    COMMENTS_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Comments() {
    let output = compile_and_run_debug(Comments_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Comments.qs
    COMMENTS_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn ComparisonOperators_src() -> SourceMap {
    SourceMap::new(
        vec![("ComparisonOperators.qs".into(), include_str!("../../../../../samples/language/ComparisonOperators.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_ComparisonOperators() {
    let output = compile_and_run(ComparisonOperators_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample ComparisonOperators.qs
    COMPARISONOPERATORS_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_ComparisonOperators() {
    let output = compile_and_run_debug(ComparisonOperators_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample ComparisonOperators.qs
    COMPARISONOPERATORS_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn ConditionalBranching_src() -> SourceMap {
    SourceMap::new(
        vec![("ConditionalBranching.qs".into(), include_str!("../../../../../samples/language/ConditionalBranching.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_ConditionalBranching() {
    let output = compile_and_run(ConditionalBranching_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample ConditionalBranching.qs
    CONDITIONALBRANCHING_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_ConditionalBranching() {
    let output = compile_and_run_debug(ConditionalBranching_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample ConditionalBranching.qs
    CONDITIONALBRANCHING_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn CopyAndUpdateOperator_src() -> SourceMap {
    SourceMap::new(
        vec![("CopyAndUpdateOperator.qs".into(), include_str!("../../../../../samples/language/CopyAndUpdateOperator.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_CopyAndUpdateOperator() {
    let output = compile_and_run(CopyAndUpdateOperator_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample CopyAndUpdateOperator.qs
    COPYANDUPDATEOPERATOR_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_CopyAndUpdateOperator() {
    let output = compile_and_run_debug(CopyAndUpdateOperator_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample CopyAndUpdateOperator.qs
    COPYANDUPDATEOPERATOR_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn CustomMeasurements_src() -> SourceMap {
    SourceMap::new(
        vec![("CustomMeasurements.qs".into(), include_str!("../../../../../samples/language/CustomMeasurements.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_CustomMeasurements() {
    let output = compile_and_run(CustomMeasurements_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample CustomMeasurements.qs
    CUSTOMMEASUREMENTS_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_CustomMeasurements() {
    let output = compile_and_run_debug(CustomMeasurements_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample CustomMeasurements.qs
    CUSTOMMEASUREMENTS_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn DataTypes_src() -> SourceMap {
    SourceMap::new(
        vec![("DataTypes.qs".into(), include_str!("../../../../../samples/language/DataTypes.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_DataTypes() {
    let output = compile_and_run(DataTypes_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample DataTypes.qs
    DATATYPES_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_DataTypes() {
    let output = compile_and_run_debug(DataTypes_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample DataTypes.qs
    DATATYPES_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Diagnostics_src() -> SourceMap {
    SourceMap::new(
        vec![("Diagnostics.qs".into(), include_str!("../../../../../samples/language/Diagnostics.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Diagnostics() {
    let output = compile_and_run(Diagnostics_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Diagnostics.qs
    DIAGNOSTICS_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Diagnostics() {
    let output = compile_and_run_debug(Diagnostics_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Diagnostics.qs
    DIAGNOSTICS_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Double_src() -> SourceMap {
    SourceMap::new(
        vec![("Double.qs".into(), include_str!("../../../../../samples/language/Double.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Double() {
    let output = compile_and_run(Double_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Double.qs
    DOUBLE_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Double() {
    let output = compile_and_run_debug(Double_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Double.qs
    DOUBLE_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn EntryPoint_src() -> SourceMap {
    SourceMap::new(
        vec![("EntryPoint.qs".into(), include_str!("../../../../../samples/language/EntryPoint.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_EntryPoint() {
    let output = compile_and_run(EntryPoint_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample EntryPoint.qs
    ENTRYPOINT_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_EntryPoint() {
    let output = compile_and_run_debug(EntryPoint_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample EntryPoint.qs
    ENTRYPOINT_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn FailStatement_src() -> SourceMap {
    SourceMap::new(
        vec![("FailStatement.qs".into(), include_str!("../../../../../samples/language/FailStatement.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_FailStatement() {
    let output = compile_and_run(FailStatement_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample FailStatement.qs
    FAILSTATEMENT_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_FailStatement() {
    let output = compile_and_run_debug(FailStatement_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample FailStatement.qs
    FAILSTATEMENT_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn ForLoops_src() -> SourceMap {
    SourceMap::new(
        vec![("ForLoops.qs".into(), include_str!("../../../../../samples/language/ForLoops.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_ForLoops() {
    let output = compile_and_run(ForLoops_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample ForLoops.qs
    FORLOOPS_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_ForLoops() {
    let output = compile_and_run_debug(ForLoops_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample ForLoops.qs
    FORLOOPS_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Functions_src() -> SourceMap {
    SourceMap::new(
        vec![("Functions.qs".into(), include_str!("../../../../../samples/language/Functions.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Functions() {
    let output = compile_and_run(Functions_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Functions.qs
    FUNCTIONS_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Functions() {
    let output = compile_and_run_debug(Functions_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Functions.qs
    FUNCTIONS_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn GettingStarted_src() -> SourceMap {
    SourceMap::new(
        vec![("GettingStarted.qs".into(), include_str!("../../../../../samples/language/GettingStarted.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_GettingStarted() {
    let output = compile_and_run(GettingStarted_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample GettingStarted.qs
    GETTINGSTARTED_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_GettingStarted() {
    let output = compile_and_run_debug(GettingStarted_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample GettingStarted.qs
    GETTINGSTARTED_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Int_src() -> SourceMap {
    SourceMap::new(
        vec![("Int.qs".into(), include_str!("../../../../../samples/language/Int.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Int() {
    let output = compile_and_run(Int_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Int.qs
    INT_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Int() {
    let output = compile_and_run_debug(Int_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Int.qs
    INT_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn LambdaExpression_src() -> SourceMap {
    SourceMap::new(
        vec![("LambdaExpression.qs".into(), include_str!("../../../../../samples/language/LambdaExpression.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_LambdaExpression() {
    let output = compile_and_run(LambdaExpression_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample LambdaExpression.qs
    LAMBDAEXPRESSION_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_LambdaExpression() {
    let output = compile_and_run_debug(LambdaExpression_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample LambdaExpression.qs
    LAMBDAEXPRESSION_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn LogicalOperators_src() -> SourceMap {
    SourceMap::new(
        vec![("LogicalOperators.qs".into(), include_str!("../../../../../samples/language/LogicalOperators.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_LogicalOperators() {
    let output = compile_and_run(LogicalOperators_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample LogicalOperators.qs
    LOGICALOPERATORS_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_LogicalOperators() {
    let output = compile_and_run_debug(LogicalOperators_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample LogicalOperators.qs
    LOGICALOPERATORS_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Namespaces_src() -> SourceMap {
    SourceMap::new(
        vec![("Namespaces.qs".into(), include_str!("../../../../../samples/language/Namespaces.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Namespaces() {
    let output = compile_and_run(Namespaces_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Namespaces.qs
    NAMESPACES_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Namespaces() {
    let output = compile_and_run_debug(Namespaces_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Namespaces.qs
    NAMESPACES_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Operations_src() -> SourceMap {
    SourceMap::new(
        vec![("Operations.qs".into(), include_str!("../../../../../samples/language/Operations.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Operations() {
    let output = compile_and_run(Operations_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Operations.qs
    OPERATIONS_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Operations() {
    let output = compile_and_run_debug(Operations_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Operations.qs
    OPERATIONS_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn PartialApplication_src() -> SourceMap {
    SourceMap::new(
        vec![("PartialApplication.qs".into(), include_str!("../../../../../samples/language/PartialApplication.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_PartialApplication() {
    let output = compile_and_run(PartialApplication_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample PartialApplication.qs
    PARTIALAPPLICATION_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_PartialApplication() {
    let output = compile_and_run_debug(PartialApplication_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample PartialApplication.qs
    PARTIALAPPLICATION_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Pauli_src() -> SourceMap {
    SourceMap::new(
        vec![("Pauli.qs".into(), include_str!("../../../../../samples/language/Pauli.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Pauli() {
    let output = compile_and_run(Pauli_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Pauli.qs
    PAULI_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Pauli() {
    let output = compile_and_run_debug(Pauli_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Pauli.qs
    PAULI_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn QuantumMemory_src() -> SourceMap {
    SourceMap::new(
        vec![("QuantumMemory.qs".into(), include_str!("../../../../../samples/language/QuantumMemory.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_QuantumMemory() {
    let output = compile_and_run(QuantumMemory_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample QuantumMemory.qs
    QUANTUMMEMORY_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_QuantumMemory() {
    let output = compile_and_run_debug(QuantumMemory_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample QuantumMemory.qs
    QUANTUMMEMORY_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Qubit_src() -> SourceMap {
    SourceMap::new(
        vec![("Qubit.qs".into(), include_str!("../../../../../samples/language/Qubit.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Qubit() {
    let output = compile_and_run(Qubit_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Qubit.qs
    QUBIT_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Qubit() {
    let output = compile_and_run_debug(Qubit_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Qubit.qs
    QUBIT_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Range_src() -> SourceMap {
    SourceMap::new(
        vec![("Range.qs".into(), include_str!("../../../../../samples/language/Range.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Range() {
    let output = compile_and_run(Range_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Range.qs
    RANGE_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Range() {
    let output = compile_and_run_debug(Range_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Range.qs
    RANGE_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn RepeatUntilLoops_src() -> SourceMap {
    SourceMap::new(
        vec![("RepeatUntilLoops.qs".into(), include_str!("../../../../../samples/language/RepeatUntilLoops.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_RepeatUntilLoops() {
    let output = compile_and_run(RepeatUntilLoops_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample RepeatUntilLoops.qs
    REPEATUNTILLOOPS_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_RepeatUntilLoops() {
    let output = compile_and_run_debug(RepeatUntilLoops_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample RepeatUntilLoops.qs
    REPEATUNTILLOOPS_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Result_src() -> SourceMap {
    SourceMap::new(
        vec![("Result.qs".into(), include_str!("../../../../../samples/language/Result.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Result() {
    let output = compile_and_run(Result_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Result.qs
    RESULT_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Result() {
    let output = compile_and_run_debug(Result_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Result.qs
    RESULT_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn ReturnStatement_src() -> SourceMap {
    SourceMap::new(
        vec![("ReturnStatement.qs".into(), include_str!("../../../../../samples/language/ReturnStatement.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_ReturnStatement() {
    let output = compile_and_run(ReturnStatement_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample ReturnStatement.qs
    RETURNSTATEMENT_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_ReturnStatement() {
    let output = compile_and_run_debug(ReturnStatement_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample ReturnStatement.qs
    RETURNSTATEMENT_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Specializations_src() -> SourceMap {
    SourceMap::new(
        vec![("Specializations.qs".into(), include_str!("../../../../../samples/language/Specializations.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Specializations() {
    let output = compile_and_run(Specializations_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Specializations.qs
    SPECIALIZATIONS_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Specializations() {
    let output = compile_and_run_debug(Specializations_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Specializations.qs
    SPECIALIZATIONS_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn String_src() -> SourceMap {
    SourceMap::new(
        vec![("String.qs".into(), include_str!("../../../../../samples/language/String.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_String() {
    let output = compile_and_run(String_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample String.qs
    STRING_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_String() {
    let output = compile_and_run_debug(String_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample String.qs
    STRING_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Ternary_src() -> SourceMap {
    SourceMap::new(
        vec![("Ternary.qs".into(), include_str!("../../../../../samples/language/Ternary.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Ternary() {
    let output = compile_and_run(Ternary_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Ternary.qs
    TERNARY_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Ternary() {
    let output = compile_and_run_debug(Ternary_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Ternary.qs
    TERNARY_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn TestAttribute_src() -> SourceMap {
    SourceMap::new(
        vec![("TestAttribute.qs".into(), include_str!("../../../../../samples/language/TestAttribute.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_TestAttribute() {
    let output = compile_and_run(TestAttribute_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample TestAttribute.qs
    TESTATTRIBUTE_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_TestAttribute() {
    let output = compile_and_run_debug(TestAttribute_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample TestAttribute.qs
    TESTATTRIBUTE_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Tuple_src() -> SourceMap {
    SourceMap::new(
        vec![("Tuple.qs".into(), include_str!("../../../../../samples/language/Tuple.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Tuple() {
    let output = compile_and_run(Tuple_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Tuple.qs
    TUPLE_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Tuple() {
    let output = compile_and_run_debug(Tuple_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Tuple.qs
    TUPLE_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn TypeDeclarations_src() -> SourceMap {
    SourceMap::new(
        vec![("TypeDeclarations.qs".into(), include_str!("../../../../../samples/language/TypeDeclarations.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_TypeDeclarations() {
    let output = compile_and_run(TypeDeclarations_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample TypeDeclarations.qs
    TYPEDECLARATIONS_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_TypeDeclarations() {
    let output = compile_and_run_debug(TypeDeclarations_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample TypeDeclarations.qs
    TYPEDECLARATIONS_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Unit_src() -> SourceMap {
    SourceMap::new(
        vec![("Unit.qs".into(), include_str!("../../../../../samples/language/Unit.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Unit() {
    let output = compile_and_run(Unit_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Unit.qs
    UNIT_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Unit() {
    let output = compile_and_run_debug(Unit_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Unit.qs
    UNIT_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Variables_src() -> SourceMap {
    SourceMap::new(
        vec![("Variables.qs".into(), include_str!("../../../../../samples/language/Variables.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Variables() {
    let output = compile_and_run(Variables_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Variables.qs
    VARIABLES_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Variables() {
    let output = compile_and_run_debug(Variables_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample Variables.qs
    VARIABLES_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn WhileLoops_src() -> SourceMap {
    SourceMap::new(
        vec![("WhileLoops.qs".into(), include_str!("../../../../../samples/language/WhileLoops.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_WhileLoops() {
    let output = compile_and_run(WhileLoops_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample WhileLoops.qs
    WHILELOOPS_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_WhileLoops() {
    let output = compile_and_run_debug(WhileLoops_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample WhileLoops.qs
    WHILELOOPS_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn WithinApply_src() -> SourceMap {
    SourceMap::new(
        vec![("WithinApply.qs".into(), include_str!("../../../../../samples/language/WithinApply.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_WithinApply() {
    let output = compile_and_run(WithinApply_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample WithinApply.qs
    WITHINAPPLY_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_WithinApply() {
    let output = compile_and_run_debug(WithinApply_src());
    // This constant must be defined in `samples_test/src/tests/language.rs` and
    // must contain the output of the sample WithinApply.qs
    WITHINAPPLY_EXPECT_DEBUG.assert_eq(&output);
}
