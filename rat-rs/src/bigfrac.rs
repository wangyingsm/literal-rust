use num::{BigUint, Integer};

use crate::{
    error::RationalError,
    frac::{Fraction, FractionSign},
};

pub type BigFraction = Fraction<BigUint>;

impl BigFraction {
    pub fn new(numer: BigUint, denom: BigUint, sign: FractionSign) -> Result<Self, RationalError> {
        if denom == BigUint::from(0_u64) {
            return Err(RationalError::ZeroDenominator);
        }
        let gcd = numer.gcd(&denom);
        let numer = &numer / &gcd;
        let denom = denom / gcd;
        Ok(Self { numer, denom, sign })
    }
}

mod ops {
    use core::ops::Add;

    use super::*;

    impl Add<BigFraction> for BigFraction {
        type Output = Self;

        fn add(self, rhs: BigFraction) -> Self::Output {
            todo!()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bigfraction_new() {
        assert_eq!(
            BigFraction::new(
                BigUint::from(1_u64),
                BigUint::from(2_u64),
                FractionSign::NonNegative
            )
            .unwrap(),
            BigFraction::new(
                BigUint::from(2_u64),
                BigUint::from(4_u64),
                FractionSign::NonNegative
            )
            .unwrap()
        );
    }
    #[test]
    pub fn test_bigfraction_add() {
        assert_eq!(
            BigFraction::new(1_u64.into(), 2_u64.into(), FractionSign::NonNegative).unwrap()
                + BigFraction::new(1_u64.into(), 3_u64.into(), FractionSign::NonNegative).unwrap(),
            BigFraction::new(5_u64.into(), 6_u64.into(), FractionSign::NonNegative).unwrap()
        );
    }
}
