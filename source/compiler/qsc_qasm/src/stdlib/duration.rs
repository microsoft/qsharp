// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{
    fmt::{self, Display, Formatter},
    ops::{Add, Div, Mul, Sub},
};

use crate::semantic::ast::{LiteralKind, TimeUnit};

#[derive(Clone, Copy, Debug, Default)]
pub struct Duration {
    pub value: f64,
    pub unit: TimeUnit,
}

impl Duration {
    pub fn new(value: f64, unit: TimeUnit) -> Self {
        Self { value, unit }
    }

    fn to_nanoseconds(self) -> f64 {
        match self.unit {
            TimeUnit::Ns => self.value,
            TimeUnit::Us => self.value * 1_000.0,
            TimeUnit::Ms => self.value * 1_000_000.0,
            TimeUnit::S => self.value * 1_000_000_000.0,
            TimeUnit::Dt => todo!("Duration in dt is not supported"),
        }
    }
}

impl Display for Duration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} {}", self.value, self.unit)
    }
}

impl PartialEq for Duration {
    fn eq(&self, other: &Self) -> bool {
        if self.unit == other.unit {
            f64::EPSILON > (self.value - other.value).abs()
        } else {
            // Convert both to nanoseconds for comparison
            let self_ns = self.to_nanoseconds();
            let other_ns = other.to_nanoseconds();
            f64::EPSILON > (self_ns - other_ns).abs()
        }
    }
}

impl Add for Duration {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.unit == TimeUnit::Dt || rhs.unit == TimeUnit::Dt {
            // either are dt, treat both as dt?
            let value = self.value + rhs.value;
            Self {
                value,
                unit: TimeUnit::Dt,
            }
        } else {
            // Normalize to a common unit (e.g., nanoseconds)
            let self_ns = self.to_nanoseconds();
            let rhs_ns = rhs.to_nanoseconds();
            let value = self_ns + rhs_ns;
            Self {
                value,
                unit: TimeUnit::Ns,
            }
        }
    }
}

impl Sub for Duration {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.unit == TimeUnit::Dt || rhs.unit == TimeUnit::Dt {
            // either are dt, treat both as dt?
            let value = self.value - rhs.value;
            Self {
                value,
                unit: TimeUnit::Dt,
            }
        } else {
            // Normalize to a common unit (e.g., nanoseconds)
            let self_ns = self.to_nanoseconds();
            let rhs_ns = rhs.to_nanoseconds();
            let value = self_ns - rhs_ns;
            Self {
                value,
                unit: TimeUnit::Ns,
            }
        }
    }
}

impl Mul<f64> for Duration {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        let value = self.value * rhs;
        Self {
            value,
            unit: self.unit,
        }
    }
}

impl Mul<i64> for Duration {
    type Output = Self;

    fn mul(self, rhs: i64) -> Self::Output {
        #[allow(clippy::cast_precision_loss)]
        let value = self.value * rhs as f64;
        Self {
            value,
            unit: self.unit,
        }
    }
}

impl Div<Duration> for Duration {
    type Output = f64;

    fn div(self, rhs: Self) -> Self::Output {
        if self.unit == TimeUnit::Dt || rhs.unit == TimeUnit::Dt {
            // either are dt, treat both as dt?
            self.value / rhs.value
        } else {
            // Normalize to a common unit (e.g., nanoseconds)
            let self_ns = self.to_nanoseconds();
            let rhs_ns = rhs.to_nanoseconds();
            self_ns / rhs_ns
        }
    }
}

impl Div<f64> for Duration {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        let value = self.value / rhs;
        Self {
            value,
            unit: self.unit,
        }
    }
}

impl Div<i64> for Duration {
    type Output = Self;

    fn div(self, rhs: i64) -> Self::Output {
        #[allow(clippy::cast_precision_loss)]
        let value = self.value / rhs as f64;
        Self {
            value,
            unit: self.unit,
        }
    }
}

impl From<Duration> for LiteralKind {
    fn from(value: Duration) -> Self {
        LiteralKind::Duration(value)
    }
}

impl From<f64> for LiteralKind {
    fn from(value: f64) -> Self {
        LiteralKind::Duration(Duration::new(value, TimeUnit::default()))
    }
}

impl From<i64> for LiteralKind {
    fn from(value: i64) -> Self {
        #[allow(clippy::cast_precision_loss)]
        let value = value as f64;
        LiteralKind::Duration(Duration::new(value, TimeUnit::default()))
    }
}
