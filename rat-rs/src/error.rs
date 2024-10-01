use core::{error::Error, fmt::Display};

#[derive(Debug, PartialEq, Eq)]
pub enum RationalError {
    ZeroDenominator,
    NumeratorOverflow,
    DenominatorOverflow,
    DivideByZero,
}

impl Error for RationalError {}

impl Display for RationalError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RationalError::ZeroDenominator => write!(f, "zero as denominator"),
            RationalError::NumeratorOverflow => write!(f, "numerator overflow"),
            RationalError::DenominatorOverflow => write!(f, "denominator overflow"),
            RationalError::DivideByZero => write!(f, "divided by zero"),
        }
    }
}
