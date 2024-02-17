// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use bitflags::bitflags;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Copy)]
pub struct LanguageFeatures(u8);

bitflags! {
    impl LanguageFeatures: u8 {
        const V2PreviewSyntax = 0b1;
    }
}

// #[derive(Debug, Clone, Deserialize, Default)]
// pub struct LanguageFeatures(BTreeSet<LanguageFeature>);
pub type LanguageFeatureIncompatibility = miette::ErrReport;
pub type UnrecognizedLanguageFeature = miette::ErrReport;

impl LanguageFeatures {
    /// Checks that the current set of language features is compatible and well-formed.
    /// Returns a descriptive error message if not.
    pub fn check_compatibility(&self) -> Result<(), LanguageFeatureIncompatibility> {
        // we currently only have one language feature, so no checking is required.
        Ok(())
    }

    #[must_use]
    pub fn none() -> Self {
        Self(0)
    }

    pub fn merge(&mut self, other: impl Into<LanguageFeatures>) {
        self.0 = self.0 | other.into().0
    }
}

impl Default for LanguageFeatures {
    fn default() -> Self {
        LanguageFeatures::empty()
    }
}

impl<'a, I> FromIterator<I> for LanguageFeatures
where
    I: AsRef<str>,
{
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        iter.into_iter().fold(LanguageFeatures::empty(), |acc, x| {
            acc | match x.as_ref() {
                "v2-preview-syntax" => LanguageFeatures::V2PreviewSyntax,
                _ => LanguageFeatures::empty(),
            }
        })
    }
}
