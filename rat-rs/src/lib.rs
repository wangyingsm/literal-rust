#![cfg_attr(not(feature = "std"), no_std)]

pub mod conv;
pub mod error;
pub mod frac;
pub mod ops;

#[cfg(feature = "std")]
pub mod bigfrac;

pub use frac::Fraction;
pub use frac::FractionU16;
pub use frac::FractionU32;
pub use frac::FractionU8;

#[cfg(feature = "std")]
pub use bigfrac::BigFraction;
