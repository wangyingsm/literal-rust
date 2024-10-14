//! frac是本lib最关键的数据结构，核心是Fraction结构体，字段包括分子numer,分母denom和符号sign
//!
//! # Example
//! ```rust
//! use rat_rs::frac::{Fraction, FractionU32, FractionSign};
//! let f = FractionU32::new(1, 2, FractionSign::NonNegative).unwrap();
//! let g = Fraction::with_negative(1, 2).unwrap();
//! assert_eq!(f + g, 0);
//! ```

use core::ops::Neg;

use crate::error::RationalError;

pub type FractionU8 = Fraction<u8>;
pub type FractionU16 = Fraction<u16>;
pub type FractionU32 = Fraction<u32>;

#[derive(Debug)]
pub struct Fraction<T> {
    pub(crate) numer: T,
    pub(crate) denom: T,
    pub(crate) sign: FractionSign,
}

impl<T> Clone for Fraction<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            numer: self.numer.clone(),
            denom: self.denom.clone(),
            sign: self.sign,
        }
    }
}

impl<T> Copy for Fraction<T> where T: Copy {}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FractionSign {
    NonNegative = 0,
    Negative = 1,
}

impl From<u8> for FractionSign {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::NonNegative,
            1 => Self::Negative,
            _ => panic!("invalid sign number"),
        }
    }
}

impl Neg for FractionSign {
    type Output = Self;

    fn neg(self) -> Self::Output {
        if self == Self::Negative {
            Self::NonNegative
        } else {
            Self::Negative
        }
    }
}

impl<T> PartialEq<Fraction<T>> for Fraction<T>
where
    T: PartialEq<T>,
{
    fn eq(&self, other: &Fraction<T>) -> bool {
        self.numer == other.numer && self.denom == other.denom && self.sign == other.sign
    }
}

impl<T> Eq for Fraction<T> where T: Eq {}

pub trait UnsignedFractionInt: Copy {}

impl UnsignedFractionInt for u8 {}
impl UnsignedFractionInt for u16 {}
impl UnsignedFractionInt for u32 {}

impl<T> Fraction<T>
where
    T: Into<u64> + TryFrom<u64> + UnsignedFractionInt,
{
    pub fn new(numer: T, denom: T, sign: FractionSign) -> Result<Self, RationalError> {
        if denom.into() == 0 {
            return Err(RationalError::ZeroDenominator);
        }
        let gcd = gcd(numer.into(), denom.into());
        let numer =
            T::try_from(numer.into() / gcd).map_err(|_| RationalError::NumeratorOverflow)?;
        let denom =
            T::try_from(denom.into() / gcd).map_err(|_| RationalError::DenominatorOverflow)?;
        Ok(Self { numer, denom, sign })
    }

    pub fn with_non_negative(numer: T, denom: T) -> Result<Self, RationalError> {
        Self::new(numer, denom, FractionSign::NonNegative)
    }

    pub fn with_negative(numer: T, denom: T) -> Result<Self, RationalError> {
        Self::new(numer, denom, FractionSign::Negative)
    }

    pub fn numer(&self) -> T {
        self.numer
    }

    pub fn denom(&self) -> T {
        self.denom
    }

    pub fn sign(&self) -> FractionSign {
        self.sign
    }
}

pub(crate) fn gcd(mut m: u64, mut n: u64) -> u64 {
    while n != 0 {
        let remainder = m % n;
        m = core::mem::replace(&mut n, remainder);
    }
    m
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_gcd_with_corner_cases() {
        assert_eq!(gcd(42, 12), 6);
        assert_eq!(gcd(12, 42), 6);
        assert_eq!(gcd(100, 0), 100);
        assert_eq!(gcd(0, 100), 100);
        assert_eq!(gcd(37, 73), 1);
        assert_eq!(gcd(42, 1), 1);
        assert_eq!(gcd(1, 42), 1);
    }

    #[test]
    fn test_new_fraction_with_corner_cases() {
        assert_eq!(
            FractionU32::new(42, 12, FractionSign::NonNegative),
            Ok(Fraction {
                numer: 7,
                denom: 2,
                sign: FractionSign::NonNegative
            })
        );
        assert_eq!(
            FractionU32::new(12, 42, FractionSign::Negative),
            Ok(Fraction {
                numer: 2,
                denom: 7,
                sign: FractionSign::Negative
            })
        );
        assert!(FractionU32::new(100, 0, FractionSign::NonNegative).is_err());
        assert_eq!(
            FractionU32::new(0, 100, FractionSign::NonNegative),
            Ok(Fraction {
                numer: 0,
                denom: 1,
                sign: FractionSign::NonNegative
            })
        );
        assert_eq!(
            FractionU32::new(37, 73, FractionSign::Negative),
            Ok(Fraction {
                numer: 37,
                denom: 73,
                sign: FractionSign::Negative,
            })
        );
        assert_eq!(
            FractionU32::new(42, 1, FractionSign::NonNegative),
            Ok(Fraction {
                numer: 42,
                denom: 1,
                sign: FractionSign::NonNegative
            })
        );
        assert_eq!(
            FractionU32::new(1, 42, FractionSign::Negative),
            Ok(Fraction {
                numer: 1,
                denom: 42,
                sign: FractionSign::Negative
            })
        );
    }
}
