// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod samples {
    pub(crate) const BELL_STATE: &str = include_str!("../../../../samples/algorithms/BellState.qs");
}

mod base_profile {
    use qsc_data_structures::language_features::LanguageFeatures;
    use qsc_frontend::compile::{SourceMap, TargetCapabilityFlags};

    use crate::codegen::get_qir;

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
}
