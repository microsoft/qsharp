// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_data_structures::{language_features::LanguageFeatures, target::TargetCapabilityFlags};
use qsc_frontend::compile::SourceMap;

use crate::codegen::get_qir;

#[test]
fn code_with_errors_returns_errors() {
    let source = "namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit()
                let pi_over_two = 4.0 / 2.0;
            }
        }";
    let sources = SourceMap::new([("test.qs".into(), source.into())], None);
    let language_features = LanguageFeatures::default();
    let capabilities = TargetCapabilityFlags::empty();

    match get_qir(sources, language_features, capabilities) {
        Ok(_) => panic!("Expected an error"),
        Err(e) => {
            assert!(e.contains("Failed to generate QIR. Could not compile sources."));
        }
    }
}

mod base_profile {
    use expect_test::expect;
    use qsc_data_structures::{language_features::LanguageFeatures, target::TargetCapabilityFlags};
    use qsc_frontend::compile::SourceMap;

    use crate::codegen::get_qir;

    #[test]
    fn simple() {
        let source = "namespace Test {
            open Microsoft.Quantum.Math;
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Result {
                use q = Qubit();
                let pi_over_two = 4.0 / 2.0;
                __quantum__qis__rz__body(pi_over_two, q);
                mutable some_angle = ArcSin(0.0);
                __quantum__qis__rz__body(some_angle, q);
                set some_angle = ArcCos(-1.0) / PI();
                __quantum__qis__rz__body(some_angle, q);
                __quantum__qis__mresetz__body(q)
            }
        }";
        let sources = SourceMap::new([("test.qs".into(), source.into())], None);
        let language_features = LanguageFeatures::default();
        let capabilities = TargetCapabilityFlags::empty();

        let qir =
            get_qir(sources, language_features, capabilities).expect("Failed to generate QIR");
        println!("{qir}");
        expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
            block_0:
              call void @__quantum__qis__rz__body(double 2.0, %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__rz__body(double 0.0, %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__rz__body(double 1.0, %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__qis__rz__body(double, %Qubit*)

            declare void @__quantum__rt__result_record_output(%Result*, i8*)

            declare void @__quantum__qis__mz__body(%Qubit*, %Result*) #1

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="1" "required_num_results"="1" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]]
        .assert_eq(&qir);
    }
}

mod adaptive_profile {
    use qsc_data_structures::{language_features::LanguageFeatures, target::TargetCapabilityFlags};
    use qsc_frontend::compile::SourceMap;

    use crate::codegen::get_qir;

    #[test]
    fn simple() -> Result<(), String> {
        let source = "namespace Test {
            open Microsoft.Quantum.Math;
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Result {
                use q = Qubit();
                let pi_over_two = 4.0 / 2.0;
                __quantum__qis__rz__body(pi_over_two, q);
                mutable some_angle = ArcSin(0.0);
                __quantum__qis__rz__body(some_angle, q);
                set some_angle = ArcCos(-1.0) / PI();
                __quantum__qis__rz__body(some_angle, q);
                __quantum__qis__mresetz__body(q)
            }
        }";
        let sources = SourceMap::new([("test.qs".into(), source.into())], None);
        let language_features = LanguageFeatures::default();
        let capabilities = TargetCapabilityFlags::Adaptive;

        let qir = get_qir(sources, language_features, capabilities)?;
        println!("{qir}");
        assert!(qir.contains("ENTRYPOINT"));
        Ok(())
    }
}

mod quantinuum_profile {
    use qsc_data_structures::{language_features::LanguageFeatures, target::TargetCapabilityFlags};
    use qsc_frontend::compile::SourceMap;

    use crate::codegen::get_qir;

    #[test]
    fn simple() -> Result<(), String> {
        let source = "namespace Test {
            open Microsoft.Quantum.Math;
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Result {
                use q = Qubit();
                let pi_over_two = 4.0 / 2.0;
                __quantum__qis__rz__body(pi_over_two, q);
                mutable some_angle = ArcSin(0.0);
                __quantum__qis__rz__body(some_angle, q);
                set some_angle = ArcCos(-1.0) / PI();
                __quantum__qis__rz__body(some_angle, q);
                __quantum__qis__mresetz__body(q)
            }
        }";
        let sources = SourceMap::new([("test.qs".into(), source.into())], None);
        let language_features = LanguageFeatures::default();
        let capabilities = TargetCapabilityFlags::Adaptive
            | TargetCapabilityFlags::QubitReset
            | TargetCapabilityFlags::IntegerComputations;

        let qir = get_qir(sources, language_features, capabilities)?;
        println!("{qir}");
        assert!(qir.contains("ENTRYPOINT"));
        Ok(())
    }
}
