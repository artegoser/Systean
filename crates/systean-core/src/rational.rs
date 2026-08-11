use std::cmp::Ordering;
use std::ops::{AddAssign, Div, Mul, Neg};

use num_bigint::BigInt;
use num_traits::{One, Signed, Zero};

/// Exact normalized rational arithmetic backed by the workspace `num-bigint`.
///
/// This intentionally replaces `num-rational::BigRational`: num-rational 0.4.x
/// is still coupled to num-bigint 0.4.x, while Systean uses num-bigint 0.5.x.
/// Keeping the tiny arithmetic layer here prevents two incompatible BigInt
/// versions from leaking into the language engine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactRational {
    numerator: BigInt,
    denominator: BigInt,
}

impl ExactRational {
    pub fn new(mut numerator: BigInt, mut denominator: BigInt) -> Self {
        assert!(!denominator.is_zero(), "rational denominator cannot be zero");
        if denominator.is_negative() {
            numerator = -numerator;
            denominator = -denominator;
        }
        let gcd = gcd(numerator.clone().abs(), denominator.clone());
        Self {
            numerator: numerator / &gcd,
            denominator: denominator / gcd,
        }
    }

    pub fn from_integer(value: BigInt) -> Self {
        Self {
            numerator: value,
            denominator: BigInt::one(),
        }
    }

    pub fn numer(&self) -> &BigInt {
        &self.numerator
    }

    pub fn denom(&self) -> &BigInt {
        &self.denominator
    }

    pub fn is_negative(&self) -> bool {
        self.numerator.is_negative()
    }
}

fn gcd(mut left: BigInt, mut right: BigInt) -> BigInt {
    while !right.is_zero() {
        let remainder = &left % &right;
        left = right;
        right = remainder;
    }
    if left.is_zero() { BigInt::one() } else { left }
}

impl Neg for ExactRational {
    type Output = Self;

    fn neg(mut self) -> Self::Output {
        self.numerator = -self.numerator;
        self
    }
}

impl AddAssign for ExactRational {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self::new(
            &self.numerator * &rhs.denominator + &rhs.numerator * &self.denominator,
            &self.denominator * &rhs.denominator,
        );
    }
}

impl Mul for ExactRational {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(
            self.numerator * rhs.numerator,
            self.denominator * rhs.denominator,
        )
    }
}

impl Mul<&ExactRational> for ExactRational {
    type Output = Self;

    fn mul(self, rhs: &ExactRational) -> Self::Output {
        Self::new(
            self.numerator * &rhs.numerator,
            self.denominator * &rhs.denominator,
        )
    }
}

impl Div for ExactRational {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        assert!(!rhs.numerator.is_zero(), "cannot divide by zero rational");
        Self::new(
            self.numerator * rhs.denominator,
            self.denominator * rhs.numerator,
        )
    }
}

impl PartialOrd for ExactRational {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ExactRational {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.numerator * &other.denominator).cmp(&(&other.numerator * &self.denominator))
    }
}
