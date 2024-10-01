use core::{
    ops::{Add, Div, Mul, Neg, Sub},
    panic,
};

use crate::{
    error::RationalError,
    frac::{gcd, Fraction, FractionSign, UnsignedFractionInt},
};

impl<T> Add<Fraction<T>> for Fraction<T>
where
    u64: From<T>,
    T: Into<u64> + TryFrom<u64> + UnsignedFractionInt,
{
    type Output = Self;

    fn add(self, rhs: Fraction<T>) -> Self::Output {
        self.checked_add(rhs)
            .expect("numerator/denominator overflow")
    }
}

impl<T> Neg for Fraction<T>
where
    T: Into<u64> + TryFrom<u64> + UnsignedFractionInt,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(self.numer(), self.denom(), -self.sign()).unwrap()
    }
}

impl<T> Sub<Fraction<T>> for Fraction<T>
where
    u64: From<T>,
    T: Into<u64> + TryFrom<u64> + UnsignedFractionInt,
{
    type Output = Self;

    fn sub(self, rhs: Fraction<T>) -> Self::Output {
        self.checked_sub(rhs)
            .expect("numerator/denominator overflow")
    }
}

impl<T> Mul<Fraction<T>> for Fraction<T>
where
    u64: From<T>,
    T: Into<u64> + TryFrom<u64> + UnsignedFractionInt,
{
    type Output = Self;

    fn mul(self, rhs: Fraction<T>) -> Self::Output {
        self.checked_mul(rhs)
            .expect("numerator/denominator overflow")
    }
}

impl<T> Div<Fraction<T>> for Fraction<T>
where
    T: Into<u64> + TryFrom<u64> + UnsignedFractionInt,
    u32: From<T>,
    u64: From<T>,
{
    type Output = Self;

    fn div(self, rhs: Fraction<T>) -> Self::Output {
        match self.checked_div(rhs) {
            Ok(r) => r,
            Err(e) => panic!("divide error: {e}"),
        }
    }
}

impl<T> PartialEq<u32> for Fraction<T>
where
    T: Into<u64> + TryFrom<u64> + UnsignedFractionInt,
    u32: From<T>,
{
    fn eq(&self, other: &u32) -> bool {
        u32::from(self.numer()) == *other && self.denom().into() == 1
    }
}

impl<T> Fraction<T>
where
    T: Into<u64> + TryFrom<u64> + UnsignedFractionInt,
{
    pub fn checked_add(self, rhs: Self) -> Result<Self, RationalError>
    where
        u64: From<T>,
    {
        match (self.sign(), rhs.sign()) {
            (FractionSign::NonNegative, FractionSign::NonNegative)
            | (FractionSign::Negative, FractionSign::Negative) => {
                let numer = u64::from(self.numer()) * u64::from(rhs.denom())
                    + u64::from(self.denom()) * u64::from(rhs.numer());
                let denom = u64::from(self.denom()) * u64::from(rhs.denom());
                let gcd = gcd(numer, denom);
                let numer =
                    T::try_from(numer / gcd).map_err(|_| RationalError::NumeratorOverflow)?;
                let denom =
                    T::try_from(denom / gcd).map_err(|_| RationalError::DenominatorOverflow)?;
                Ok(Self::new(numer, denom, self.sign()).unwrap())
            }
            (FractionSign::NonNegative, FractionSign::Negative)
            | (FractionSign::Negative, FractionSign::NonNegative) => {
                let numer_part1 = u64::from(self.numer()) * u64::from(rhs.denom());
                let numer_part2 = u64::from(self.denom()) * u64::from(rhs.numer());
                let denom = u64::from(self.denom()) * u64::from(rhs.denom());
                let (numer, sign) = if numer_part1 >= numer_part2 {
                    (numer_part1 - numer_part2, FractionSign::NonNegative)
                } else {
                    (numer_part2 - numer_part1, FractionSign::Negative)
                };
                let gcd = gcd(numer, denom);
                let numer =
                    T::try_from(numer / gcd).map_err(|_| RationalError::NumeratorOverflow)?;
                let denom =
                    T::try_from(denom / gcd).map_err(|_| RationalError::DenominatorOverflow)?;
                Ok(Self::new(numer, denom, sign).unwrap())
            }
        }
    }

    pub fn checked_sub(self, rhs: Self) -> Result<Self, RationalError>
    where
        u64: From<T>,
    {
        self.checked_add(-rhs)
    }

    pub fn checked_mul(self, rhs: Self) -> Result<Self, RationalError>
    where
        u64: From<T>,
    {
        let numer = u64::from(self.numer()) * u64::from(rhs.numer());
        let denom = u64::from(self.denom()) * u64::from(rhs.denom());
        let gcd = gcd(numer, denom);
        let numer = T::try_from(numer / gcd).map_err(|_| RationalError::NumeratorOverflow)?;
        let denom = T::try_from(denom / gcd).map_err(|_| RationalError::DenominatorOverflow)?;
        let sign = FractionSign::from(self.sign() as u8 ^ rhs.sign() as u8);
        Ok(Self::new(numer, denom, sign).unwrap())
    }

    pub fn checked_div(self, rhs: Self) -> Result<Self, RationalError>
    where
        u64: From<T>,
        u32: From<T>,
    {
        if rhs == 0 {
            return Err(RationalError::DivideByZero);
        }
        let rhs = Self::new(rhs.denom(), self.numer(), self.sign())?;
        self.checked_mul(rhs)
    }
}

#[cfg(test)]
mod test {

    use crate::frac::FractionU32;

    use super::*;

    #[test]
    fn test_fraction_add() {
        assert_eq!(
            FractionU32::with_non_negative(1, 2).unwrap()
                + Fraction::with_non_negative(1, 2).unwrap(),
            Fraction::new(1, 1, FractionSign::NonNegative).unwrap()
        );
        assert_eq!(
            FractionU32::with_non_negative(1, 2).unwrap() + Fraction::with_negative(1, 2).unwrap(),
            Fraction::new(0, 1, FractionSign::NonNegative).unwrap()
        );
        assert_eq!(
            FractionU32::with_non_negative(1, 2).unwrap() + Fraction::with_negative(1, 3).unwrap(),
            Fraction::new(1, 6, FractionSign::NonNegative).unwrap()
        );
        assert_eq!(
            FractionU32::with_negative(1, 2).unwrap() + Fraction::with_negative(1, 3).unwrap(),
            Fraction::new(5, 6, FractionSign::Negative).unwrap()
        );
        assert_eq!(
            FractionU32::with_non_negative(0, 1).unwrap() + Fraction::with_negative(1, 2).unwrap(),
            Fraction::new(1, 2, FractionSign::Negative).unwrap()
        );
        assert_eq!(
            FractionU32::with_negative(1, 6).unwrap() + Fraction::with_negative(1, 2).unwrap(),
            Fraction::new(2, 3, FractionSign::Negative).unwrap()
        );
        assert_eq!(
            FractionU32::with_non_negative(1, 3).unwrap() + Fraction::with_negative(1, 2).unwrap(),
            Fraction::new(1, 6, FractionSign::Negative).unwrap()
        );
        assert_eq!(
            FractionU32::with_non_negative(1, 2).unwrap() + 1.into(),
            Fraction::with_non_negative(3, 2).unwrap()
        )
    }

    #[test]
    #[should_panic]
    fn test_fraction_add_with_overflow() {
        let f = Fraction::with_non_negative(u32::MAX - 1, u32::MAX).unwrap();
        let _ = f + f;
    }
}
