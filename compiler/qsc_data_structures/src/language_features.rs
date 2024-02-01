// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.
use serde::{Deserialize};
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all="kebab-case")]
pub enum LanguageFeature {
    /// This language feature enables experimental syntax that will likely be stabilized in the next major version.
    /// It may include removals of outdated syntax and introductions of new syntax.
    V2PreviewSyntax,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LanguageFeatures(Vec<LanguageFeature>);
pub struct LanguageFeatureIncompatibility(String);

impl LanguageFeatures {
    /// Checks that the current set of language features is compatible and well-formed.
    /// Returns a descriptive error message if not.
    pub fn check_compatibility(&self) -> Result<(), LanguageFeatureIncompatibility> {
        // we currently only have one language feature, so no checking is required.
        Ok(())
    }

    pub fn none() -> Self { Self(vec![])  }
}

impl Default for LanguageFeatures {
    fn default() -> Self {
        Self(Default::default())
    }
}
