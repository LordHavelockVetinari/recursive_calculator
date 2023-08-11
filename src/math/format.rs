use crate::math::Value;
use malachite::num::arithmetic::traits::Abs;
use malachite::num::conversion::traits::{RoundingFrom, ToSci};
use malachite::rounding_modes::RoundingMode;
use malachite::{Integer, Rational};
use std::cmp::Ordering;
use std::fmt::{self, Display};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
pub enum Format {
    Fraction,
    Mixed,
    #[default]
    Scientific,
}

impl Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fraction => write!(f, "fraction"),
            Self::Mixed => write!(f, "mixed"),
            Self::Scientific => write!(f, "scientific"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid format (it should be \"fraction\", \"mixed\" or \"scientific\")")]
pub struct BadFormat;

impl FromStr for Format {
    type Err = BadFormat;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("fraction") {
            Ok(Self::Fraction)
        } else if s.eq_ignore_ascii_case("mixed") {
            Ok(Self::Mixed)
        } else if s.eq_ignore_ascii_case("scientific") {
            Ok(Self::Scientific)
        } else {
            Err(BadFormat)
        }
    }
}

#[derive(Debug)]
pub struct FormattedValue<'a>(pub Format, pub &'a Value);

impl Display for FormattedValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Format::*;
        use Value::*;
        match self {
            FormattedValue(_, Undefined(u)) => write!(f, "{u}"),
            FormattedValue(Fraction, Number(n)) => write!(f, "{n}"),
            FormattedValue(Mixed, Number(n)) => {
                let trunc = Integer::rounding_from(n, RoundingMode::Down);
                let fract = n - Rational::from(&trunc);
                if trunc == 0 {
                    return write!(f, "{fract}");
                }
                write!(f, "{}", trunc)?;
                match fract.partial_cmp(&0).unwrap() {
                    Ordering::Less => write!(f, " - ")?,
                    Ordering::Equal => return Ok(()),
                    Ordering::Greater => write!(f, " + ")?,
                };
                let fract = fract.abs();
                write!(f, "{fract}")
            }
            FormattedValue(Scientific, Number(n)) => write!(f, "{}", n.to_sci()),
        }
    }
}
