// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod samples {
    pub(crate) const BELL_STATE: &str = include_str!("../../../../samples/algorithms/BellState.qs");
    pub(crate) const BERNSTEIN_VAZIRANI_NISQ: &str =
        include_str!("../../../../samples/algorithms/BernsteinVaziraniNISQ.qs");
    pub(crate) const DEUTSCH_JOZSA_NISQ: &str =
        include_str!("../../../../samples/algorithms/DeutschJozsaNISQ.qs");
    pub(crate) const GROVER: &str = include_str!("../../../../samples/algorithms/Grover.qs");
}

mod base_profile {
    use qsc_data_structures::language_features::LanguageFeatures;
    use qsc_frontend::compile::{SourceMap, TargetCapabilityFlags};

    use crate::codegen::get_qir;

    #[test]
    fn simple() -> Result<(), String> {
        let source = r#"namespace Test {
            @EntryPoint()
            operation Test() : Result {
                use q = Qubit();
                H(q);
                MResetZ(q)
            }
        }"#;
        let sources = SourceMap::new([("test.qs".into(), source.into())], None);
        let language_features = LanguageFeatures::default();
        let capabilities = TargetCapabilityFlags::empty();

        let qir = get_qir(sources, language_features, capabilities)?;
        assert!(qir.contains("ENTRYPOINT"));
        Ok(())
    }

    #[test]
    fn bell_state_sample() -> Result<(), String> {
        let sources = SourceMap::new(
            [("BellState.qs".into(), super::samples::BELL_STATE.into())],
            None,
        );
        let language_features = LanguageFeatures::default();
        let capabilities = TargetCapabilityFlags::empty();

        let qir = get_qir(sources, language_features, capabilities)?;
        assert!(qir.contains("ENTRYPOINT"));
        Ok(())
    }

    #[test]
    fn bernstein_vazirani_nisq_sample() -> Result<(), String> {
        let sources = SourceMap::new(
            [(
                "BernsteinVaziraniNISQ.qs".into(),
                super::samples::BERNSTEIN_VAZIRANI_NISQ.into(),
            )],
            None,
        );
        let language_features = LanguageFeatures::default();
        let capabilities = TargetCapabilityFlags::empty();

        let qir = get_qir(sources, language_features, capabilities)?;
        assert!(qir.contains("ENTRYPOINT"));
        Ok(())
    }

    #[test]
    fn deutsch_jozsa_nisq_sample() -> Result<(), String> {
        let sources = SourceMap::new(
            [(
                "DeutschJozsaNISQ.qs".into(),
                super::samples::DEUTSCH_JOZSA_NISQ.into(),
            )],
            None,
        );
        let language_features = LanguageFeatures::default();
        let capabilities = TargetCapabilityFlags::empty();

        let qir = get_qir(sources, language_features, capabilities)?;
        assert!(qir.contains("ENTRYPOINT"));
        Ok(())
    }

    #[test]
    fn grover_sample() -> Result<(), String> {
        let sources = SourceMap::new([("Grover.qs".into(), super::samples::GROVER.into())], None);
        let language_features = LanguageFeatures::default();
        let capabilities = TargetCapabilityFlags::empty();

        let qir = get_qir(sources, language_features, capabilities)?;
        assert!(qir.contains("ENTRYPOINT"));
        Ok(())
    }
}
