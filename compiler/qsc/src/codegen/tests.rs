// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod base_profile {
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
        let capabilities = TargetCapabilityFlags::empty();

        let qir = get_qir(sources, language_features, capabilities)?;
        println!("{qir}");
        assert!(qir.contains("ENTRYPOINT"));
        Ok(())
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
