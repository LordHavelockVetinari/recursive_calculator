use malachite::num::basic::traits::{One, Zero};
use malachite::Rational;
use std::fmt::Display;
use std::ops;

#[derive(Clone, Default, Debug)]
pub enum Undefined {
    #[default]
    _Default,
    ZeroOverZero,
    Infinity,
    Irrational,
    InfiniteLoop,
}

impl Display for Undefined {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use self::Undefined::*;
        match self {
            _Default => panic!("Undefined::_Default"),
            ZeroOverZero => write!(f, "Undefined result: zero divided by zero"),
            Infinity => write!(f, "Undefined result: possibly infinite"),
            Irrational => write!(f, "Undefined result: possibly irrational"),
            InfiniteLoop => write!(f, "Undefined result: infinite loop detected"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Number(Rational),
    Undefined(Undefined),
}

use self::Value::*;

impl Default for Value {
    fn default() -> Self {
        Undefined(Undefined::_Default)
    }
}

impl ops::Neg for Value {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Number(n) => Number(-n),
            Undefined(u) => Undefined(u),
        }
    }
}

impl ops::Neg for &'_ Value {
    type Output = Value;

    fn neg(self) -> Self::Output {
        -self.clone()
    }
}

impl ops::Add<&Self> for Value {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self::Output {
        match (self, rhs) {
            (Number(n), Number(m)) => Number(n + m),
            (Undefined(u), _) => Undefined(u),
            (_, Undefined(u)) => Undefined(u.clone()),
        }
    }
}

impl ops::Sub<&Self> for Value {
    type Output = Self;

    fn sub(self, rhs: &Self) -> Self::Output {
        match (self, rhs) {
            (Number(n), Number(m)) => Number(n - m),
            (Undefined(u), _) => Undefined(u),
            (_, Undefined(u)) => Undefined(u.clone()),
        }
    }
}

impl ops::Mul<&Self> for Value {
    type Output = Self;

    fn mul(self, rhs: &Self) -> Self::Output {
        match (self, rhs) {
            (Number(n), Number(m)) => Number(n * m),
            (Number(z), Undefined(_)) if z == 0 => Number(Rational::ZERO),
            (Undefined(_), Number(z)) if *z == 0 => Number(Rational::ZERO),
            (Undefined(u), _) => Undefined(u),
            (_, Undefined(u)) => Undefined(u.clone()),
        }
    }
}

impl ops::Div<&Self> for Value {
    type Output = Self;

    fn div(self, rhs: &Self) -> Self::Output {
        match (self, rhs) {
            (Number(n), Number(z)) if *z == 0 => Undefined(if n == 0 {
                Undefined::ZeroOverZero
            } else {
                Undefined::Infinity
            }),
            (Number(n), Number(m)) => Number(n / m),
            (Undefined(u), _) => Undefined(u),
            (_, Undefined(u)) => Undefined(u.clone()),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number(n) => write!(f, "{n}"),
            Undefined(u) => write!(f, "{u}"),
        }
    }
}

impl Value {
    pub fn abs(self) -> Self {
        match self {
            Number(n) => Number(if n < 0 { -n } else { n }),
            Undefined(u) => Undefined(u),
        }
    }

    pub fn is_zero(&self) -> bool {
        matches!(self, Self::Number(z) if *z == 0)
    }

    pub fn is_one(&self) -> bool {
        matches!(self, Self::Number(one) if *one == 1)
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    pub fn is_undefined(&self) -> bool {
        matches!(self, Self::Undefined(_))
    }

    pub fn pow(self, other: &Self) -> Self {
        match (self, other) {
            (Number(z), v) if z == 0 => match v {
                Number(n) if *n == 0 => Number(Rational::ONE),
                Number(n) if *n < 0 => Undefined(Undefined::Infinity),
                Number(_) => Number(Rational::ZERO),
                Undefined(u) => Undefined(u.clone()),
            },
            (Number(one), _) if one == 1 => Number(Rational::ONE),
            (_, Number(z)) if *z == 0 => Number(Rational::ONE),
            (Number(n), Number(m)) => {
                if let Some(pow) = super::pow(n, m.clone()) {
                    Number(pow)
                } else {
                    Undefined(Undefined::Irrational)
                }
            }
            (Undefined(u), _) => Undefined(u),
            (_, Undefined(u)) => Undefined(u.clone()),
        }
    }
}
