use std::collections::BTreeSet;
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
                "Unrecognized language feature: {s}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[derive(Default)]
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

    #[must_use] pub fn none() -> Self {
        Self(BTreeSet::default())
    }

    pub fn merge(&mut self, other: impl Into<BTreeSet<LanguageFeature>>) {
        self.0.append(&mut other.into());
    }

    #[must_use] pub fn contains(&self, feat: LanguageFeature) -> bool {
        self.0.contains(&feat)
    }


}
impl From<LanguageFeatures> for BTreeSet<LanguageFeature> {
    fn from(val: LanguageFeatures) -> Self {
        val.0
    }
}
impl From<BTreeSet<LanguageFeature>> for LanguageFeatures {
    fn from(val: BTreeSet<LanguageFeature>) -> Self {
        LanguageFeatures(val)
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
