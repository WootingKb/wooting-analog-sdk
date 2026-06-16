use std::{
    cmp::Ordering,
    fmt,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::key::KeyPosition;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[repr(C, u8)]
pub enum ValueMetadata {
    #[default]
    None,
    Basic {
        pos: KeyPosition,
        actuated: bool,
    },
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct AnalogValue {
    pub(crate) inner: f32,
    pub(crate) metadata: ValueMetadata,
}

impl AnalogValue {
    pub fn with_metadata(mut self, meta: ValueMetadata) -> Self {
        self.metadata = meta;
        self
    }
}

impl Eq for AnalogValue {}

impl PartialEq for AnalogValue {
    fn eq(&self, other: &Self) -> bool {
        self.inner.total_cmp(&other.inner) == Ordering::Equal
    }
}

impl PartialOrd for AnalogValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AnalogValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.total_cmp(&other.inner)
    }
}

impl PartialEq<f32> for AnalogValue {
    fn eq(&self, other: &f32) -> bool {
        self.inner.total_cmp(other) == Ordering::Equal
    }
}

impl PartialOrd<f32> for AnalogValue {
    fn partial_cmp(&self, other: &f32) -> Option<Ordering> {
        self.inner.partial_cmp(other)
    }
}

impl From<f32> for AnalogValue {
    fn from(value: f32) -> Self {
        Self {
            inner: value,
            metadata: ValueMetadata::None,
        }
    }
}

impl From<AnalogValue> for f32 {
    fn from(value: AnalogValue) -> Self {
        value.inner
    }
}

impl From<AnalogValue> for f64 {
    fn from(value: AnalogValue) -> Self {
        f64::from(value.inner)
    }
}

impl From<&AnalogValue> for f32 {
    fn from(value: &AnalogValue) -> Self {
        value.inner
    }
}

impl From<&AnalogValue> for f64 {
    fn from(value: &AnalogValue) -> Self {
        f64::from(value.inner)
    }
}

impl fmt::Display for AnalogValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl Add for AnalogValue {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from(self.inner + rhs.inner)
    }
}

impl Sub for AnalogValue {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from(self.inner - rhs.inner)
    }
}

impl Mul for AnalogValue {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::from(self.inner * rhs.inner)
    }
}

impl Div for AnalogValue {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::from(self.inner / rhs.inner)
    }
}

impl Rem for AnalogValue {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self::from(self.inner % rhs.inner)
    }
}

impl Neg for AnalogValue {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::from(-self.inner)
    }
}

impl Add<f32> for AnalogValue {
    type Output = Self;

    fn add(self, rhs: f32) -> Self::Output {
        Self::from(self.inner + rhs)
    }
}

impl Sub<f32> for AnalogValue {
    type Output = Self;

    fn sub(self, rhs: f32) -> Self::Output {
        Self::from(self.inner - rhs)
    }
}

impl Mul<f32> for AnalogValue {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::from(self.inner * rhs)
    }
}

impl Div<f32> for AnalogValue {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::from(self.inner / rhs)
    }
}

impl Rem<f32> for AnalogValue {
    type Output = Self;

    fn rem(self, rhs: f32) -> Self::Output {
        Self::from(self.inner % rhs)
    }
}

impl AddAssign for AnalogValue {
    fn add_assign(&mut self, rhs: Self) {
        self.inner += rhs.inner;
    }
}

impl SubAssign for AnalogValue {
    fn sub_assign(&mut self, rhs: Self) {
        self.inner -= rhs.inner;
    }
}

impl MulAssign for AnalogValue {
    fn mul_assign(&mut self, rhs: Self) {
        self.inner *= rhs.inner;
    }
}

impl DivAssign for AnalogValue {
    fn div_assign(&mut self, rhs: Self) {
        self.inner /= rhs.inner;
    }
}

impl RemAssign for AnalogValue {
    fn rem_assign(&mut self, rhs: Self) {
        self.inner %= rhs.inner;
    }
}

impl AddAssign<f32> for AnalogValue {
    fn add_assign(&mut self, rhs: f32) {
        self.inner += rhs;
    }
}

impl SubAssign<f32> for AnalogValue {
    fn sub_assign(&mut self, rhs: f32) {
        self.inner -= rhs;
    }
}

impl MulAssign<f32> for AnalogValue {
    fn mul_assign(&mut self, rhs: f32) {
        self.inner *= rhs;
    }
}

impl DivAssign<f32> for AnalogValue {
    fn div_assign(&mut self, rhs: f32) {
        self.inner /= rhs;
    }
}

impl RemAssign<f32> for AnalogValue {
    fn rem_assign(&mut self, rhs: f32) {
        self.inner %= rhs;
    }
}
