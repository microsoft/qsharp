use std::collections::{BTreeSet};

use clap::ValueEnum;
// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.
use serde::Deserialize;
#[derive(Deserialize, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum LanguageFeature {
    /// This language feature enables experimental syntax that will likely be stabilized in the next major version.
    /// It may include removals of outdated syntax and introductions of new syntax.
    V2PreviewSyntax,
}

impl LanguageFeature {
    pub fn try_parse(s: &str) -> Result<Self, UnrecognizedLanguageFeature> {
        match s {
            "v2-preview-syntax" => Ok(LanguageFeature::V2PreviewSyntax),
            _ => Err(UnrecognizedLanguageFeature::msg(format!(
                "Unrecognized language feature: {}",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LanguageFeatures(BTreeSet<LanguageFeature>);
pub type LanguageFeatureIncompatibility = miette::ErrReport;
pub type UnrecognizedLanguageFeature = miette::ErrReport;

impl LanguageFeatures {
    /// Checks that the current set of language features is compatible and well-formed.
    /// Returns a descriptive error message if not.
    pub fn check_compatibility(&self) -> Result<(), LanguageFeatureIncompatibility> {
        // we currently only have one language feature, so no checking is required.
        Ok(())
    }

    pub fn none() -> Self {
        Self(Default::default())
    }

    pub fn merge(&mut self, other: impl Into<BTreeSet<LanguageFeature>>) {
        self.0.append(&mut other.into());
    }

    pub fn contains(&self, feat: LanguageFeature) -> bool {
        self.0.contains(&feat)
    }


}
impl Into<BTreeSet<LanguageFeature>> for LanguageFeatures {
    fn into(self) -> BTreeSet<LanguageFeature> {
        self.0
    }
}
impl Into<LanguageFeatures> for BTreeSet<LanguageFeature> {
    fn into(self) -> LanguageFeatures {
        LanguageFeatures(self)
    }
}

impl Default for LanguageFeatures {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl ValueEnum for LanguageFeature {
    fn value_variants<'a>() -> &'a [Self] {
        &[LanguageFeature::V2PreviewSyntax]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            LanguageFeature::V2PreviewSyntax => {
                Some(clap::builder::PossibleValue::new("v2-preview-syntax"))
            }
        }
    }
}
