// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::str::FromStr;

use qsc_frontend::compile::RuntimeCapabilityFlags;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Profile {
    Unrestricted,
    Base,
}

impl Profile {
    #[must_use]
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Unrestricted => "Unrestricted",
            Self::Base => "Base",
        }
    }
}

impl From<Profile> for RuntimeCapabilityFlags {
    fn from(value: Profile) -> Self {
        match value {
            Profile::Unrestricted => Self::all(),
            Profile::Base => Self::empty(),
        }
    }
}

impl FromStr for Profile {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Unrestricted" | "unrestricted" => Ok(Self::Unrestricted),
            "Base" | "base" => Ok(Self::Base),
            _ => Err(()),
        }
    }
}
