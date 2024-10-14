use core::str::FromStr;

use crate::{
    error::RationalError,
    frac::{Fraction, FractionSign, FractionU16, FractionU32, FractionU8, UnsignedFractionInt},
};

macro_rules! primitive_unsign_conv {
    ($($unsign: ty,)*) => {
        $(
        impl<T> From<$unsign> for Fraction<T> where T: From<$unsign> + Into<u64> + TryFrom<u64> + UnsignedFractionInt,
{
            fn from(value: $unsign) -> Self {
                Self::new(value.into(), T::from(1), FractionSign::NonNegative).unwrap()
            }
        }) *
    };
}

primitive_unsign_conv!(u8, u16, u32,);

macro_rules! primitive_sign_conv {
    ($(($from: ty, $ttype: ty, $to: ty)),*) => {
        $(
        impl From<$from> for $to {
            fn from(value: $from) -> Self {
                if value < 0 {
                    Fraction::with_negative(-(value as i64) as $ttype, 1).unwrap()
                } else {
                    Fraction::with_non_negative(value as $ttype, 1).unwrap()
                }
            }
        }) *
    };
}

primitive_sign_conv!(
    (i8, u8, FractionU8),
    (i8, u16, FractionU16),
    (i8, u32, FractionU32)
);

primitive_sign_conv!((i16, u16, FractionU16), (i16, u32, FractionU32));
primitive_sign_conv!((i32, u32, FractionU32));

impl<T> TryFrom<u64> for Fraction<T>
where
    T: Into<u64> + TryFrom<u64> + UnsignedFractionInt,
{
    type Error = RationalError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::new(
            T::try_from(value).map_err(|_| RationalError::NumeratorOverflow)?,
            match T::try_from(1_u64) {
                Ok(v) => v,
                Err(_) => unreachable!("1 should always can be converted to T"),
            },
            FractionSign::NonNegative,
        )
    }
}

impl<T> TryFrom<u128> for Fraction<T>
where
    T: Into<u64> + TryFrom<u64> + UnsignedFractionInt,
    T: TryFrom<u128>,
{
    type Error = RationalError;

    fn try_from(value: u128) -> Result<Self, Self::Error> {
        Self::new(
            T::try_from(value).map_err(|_| RationalError::NumeratorOverflow)?,
            match T::try_from(1_u64) {
                Ok(v) => v,
                Err(_) => unreachable!("1 should always can be converted to T"),
            },
            FractionSign::NonNegative,
        )
    }
}

impl<T> TryFrom<i64> for Fraction<T>
where
    T: Into<u64> + TryFrom<u64> + UnsignedFractionInt,
    T: From<u32>,
{
    type Error = RationalError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if value > u32::MAX.into() && value < -(i64::from(u32::MAX) + 1) {
            Err(RationalError::NumeratorOverflow)
        } else if value < 0 {
            Self::with_negative(T::from(-value as u32), T::from(1))
        } else {
            Self::with_non_negative(T::from(value as u32), T::from(1))
        }
    }
}

impl<T> TryFrom<i128> for Fraction<T>
where
    T: Into<u64> + TryFrom<u64> + UnsignedFractionInt,
    T: From<u32>,
{
    type Error = RationalError;

    fn try_from(value: i128) -> Result<Self, Self::Error> {
        if value > u32::MAX.into() && value < -(i128::from(u32::MAX) + 1) {
            Err(RationalError::NumeratorOverflow)
        } else if value < 0 {
            Self::with_negative(T::from(-value as u32), T::from(1))
        } else {
            Self::with_non_negative(T::from(value as u32), T::from(1))
        }
    }
}

impl<T> FromStr for Fraction<T>
where
    T: Into<u64> + TryFrom<u64> + UnsignedFractionInt + FromStr + From<u8>,
{
    type Err = RationalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, sign) = match s.as_bytes()[0] {
            b'-' => (&s[1..], FractionSign::Negative),
            b'+' => (&s[1..], FractionSign::NonNegative),
            _ => (s, FractionSign::NonNegative),
        };
        if let Some((n, d)) = s.split_once('/') {
            let numer = n
                .parse::<T>()
                .map_err(|_| RationalError::ParseFractionError)?;
            let denom = d
                .parse::<T>()
                .map_err(|_| RationalError::ParseFractionError)?;
            Fraction::<T>::new(numer, denom, sign)
        } else {
            let numer = s
                .parse::<T>()
                .map_err(|_| RationalError::ParseFractionError)?;
            Fraction::<T>::new(numer, 1_u8.into(), sign)
        }
    }
}

#[cfg(test)]
mod test {

    use crate::frac::{FractionU16, FractionU32};

    use super::*;

    #[test]
    fn test_from_primitive() {
        assert_eq!(
            FractionU32::from(20_u32),
            Fraction::with_non_negative(20, 1).unwrap()
        );
        assert_eq!(
            FractionU16::from(-20_i8),
            Fraction::with_negative(20, 1).unwrap()
        );
        assert_eq!(
            Fraction::from(0_i32),
            Fraction::with_non_negative(0, 1).unwrap()
        );
        assert_eq!(
            FractionU8::from(0_i8),
            Fraction::with_non_negative(0, 1).unwrap()
        );
        assert_eq!(
            FractionU32::from(-128_i8),
            Fraction::with_negative(128, 1).unwrap()
        );
        assert_eq!(
            Fraction::try_from(u64::from(u32::MAX)),
            Ok(Fraction::with_non_negative(u32::MAX, 1).unwrap())
        );
        assert_eq!(
            FractionU32::try_from(u64::MAX),
            Err(RationalError::NumeratorOverflow)
        );
        assert_eq!(
            Fraction::try_from(i64::from(u32::MAX)),
            Ok(Fraction::with_non_negative(u32::MAX, 1).unwrap())
        );
        assert_eq!(
            FractionU32::try_from(-128_i64),
            Ok(Fraction::with_negative(128, 1).unwrap())
        );
    }

    #[test]
    fn test_parse_from_str() {
        assert_eq!(
            "1/2".parse::<FractionU32>().unwrap(),
            FractionU32::with_non_negative(1, 2).unwrap()
        );
        assert_eq!(
            "-1/2".parse::<FractionU32>().unwrap(),
            FractionU32::with_negative(1, 2).unwrap()
        );
        assert_eq!(
            "+1/2".parse::<FractionU32>().unwrap(),
            FractionU32::with_non_negative(1, 2).unwrap()
        );
        assert_eq!(
            "0".parse::<FractionU32>().unwrap(),
            FractionU32::with_non_negative(0, 1).unwrap()
        );
    }
}
