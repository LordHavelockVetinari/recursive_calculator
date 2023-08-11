use malachite::num::arithmetic::traits::{Pow, Reciprocal, UnsignedAbs};
use malachite::num::basic::traits::{One, Zero};
use malachite::num::logic::traits::BitAccess;
use malachite::{Integer, Natural, Rational};
use std::cmp::Ordering;

fn nat_pow(b: &Natural, e: &Natural) -> Natural {
    if *b == 0 {
        return if *e == 0 { Natural::ONE } else { Natural::ZERO };
    }
    if *b == 1 {
        return Natural::ONE;
    }
    if let Ok(e) = u64::try_from(e) {
        return b.pow(e);
    }
    panic!("out of memory");
}

fn rat_nat_pow(mut b: Rational, e: &Natural) -> Rational {
    if b < 0 && !e.get_bit(0) {
        b = -b;
    }
    b.mutate_numerator_and_denominator(|numer, denom| {
        *numer = nat_pow(numer, e);
        *denom = nat_pow(denom, e);
    });
    b
}

fn rat_int_pow(mut b: Rational, e: &Integer) -> Option<Rational> {
    b = rat_nat_pow(b, e.unsigned_abs_ref());
    if *e >= 0 {
        return Some(b);
    }
    if b == 0 {
        return None;
    }
    Some(b.reciprocal())
}

fn nat_root(n: Natural, x: Natural) -> Option<Natural> {
    use std::cmp::Ordering::*;
    debug_assert_ne!(n, 0);
    debug_assert_ne!(x, 0);
    let mut lower = Natural::ZERO;
    let mut upper = x.clone();
    loop {
        let avg = (&lower + &upper) >> 1;
        if lower == avg {
            break;
        }
        let pow = nat_pow(&avg, &n);
        match pow.cmp(&x) {
            Less => lower = avg,
            Equal => return Some(avg),
            Greater => upper = avg,
        }
    }
    if nat_pow(&lower, &n) == x {
        return Some(lower);
    }
    if nat_pow(&upper, &n) == x {
        return Some(upper);
    }
    None
}

fn nth_root(n: Natural, x: Integer) -> Option<Integer> {
    debug_assert_ne!(n, 0);
    let x_sign = x.partial_cmp(&0).unwrap();
    let x_abs = x.unsigned_abs();
    Some(match x_sign {
        Ordering::Greater => Integer::from(nat_root(n, x_abs)?),
        Ordering::Less => {
            if !n.get_bit(0) {
                return None;
            }
            -Integer::from(nat_root(n, x_abs)?)
        }
        Ordering::Equal => unimplemented!(),
    })
}

pub fn pow(mut a: Rational, b: Rational) -> Option<Rational> {
    if a == 0 {
        return match Rational::partial_cmp(&b, &0).unwrap() {
            Ordering::Less => None,
            Ordering::Equal => Some(Rational::ONE),
            Ordering::Greater => Some(Rational::ZERO),
        };
    }
    let b_sign = b > 0;
    let (pow, root) = b.into_numerator_and_denominator();
    let pow = Integer::from_sign_and_abs(b_sign, pow);
    if root != 1 {
        let a_sign = a > 0;
        let (numer, denom) = a.into_numerator_and_denominator();
        let numer = nth_root(root.clone(), Integer::from_sign_and_abs(a_sign, numer))?;
        let denom = nth_root(root, Integer::from(denom))?;
        a = Rational::from_integers(numer, denom);
    }
    rat_int_pow(a, &pow)
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! frac {
        ($numer:literal) => {
            Rational::from($numer)
        };
        ($numer:literal / $denom:literal) => {
            Rational::from($numer) / Rational::from($denom)
        };
    }

    macro_rules! assert_pow {
        (
            $numer1:literal $(/ $denom1:literal)?,
            $numer2:literal $(/ $denom2:literal)?,
            None
        ) => {
            assert_eq!(
                pow(frac!($numer1 $(/ $denom1)?), frac!($numer2 $(/ $denom2)?)),
                None,
                stringify!(pow($numer1 $(/ $denom1)?, $numer2 $(/ $denom2)?))
            );
        };
        (
            $numer1:literal $(/ $denom1:literal)?,
            $numer2:literal $(/ $denom2:literal)?,
            $numer3:literal $(/ $denom3:literal)?
        ) => {
            assert_eq!(
                pow(frac!($numer1 $(/ $denom1)?), frac!($numer2 $(/ $denom2)?)),
                Some(frac!($numer3 $(/ $denom3)?)),
                stringify!(pow($numer1 $(/ $denom1)?, $numer2 $(/ $denom2)?))
            );
        };
    }

    #[test]
    fn pow_test() {
        assert_pow!(0, 0, 1);
        assert_pow!(0, 1, 0);
        assert_pow!(1, 0, 1);
        assert_pow!(1, 1, 1);
        assert_pow!(1, 4, 1);
        assert_pow!(2, 1, 2);
        assert_pow!(2, 2, 4);
        assert_pow!(0, 1 / 2, 0);
        assert_pow!(1, 1 / 2, 1);
        assert_pow!(2, 1 / 2, None);
        assert_pow!(4, 1 / 2, 2);
        assert_pow!(2, 3, 8);
        assert_pow!(8, 1 / 3, 2);
        assert_pow!(8, 1 / 2, None);
        assert_pow!(3, 3, 27);
        assert_pow!(27, 1 / 3, 3);
        assert_pow!(27, 2 / 3, 9);
        assert_pow!(9, 3 / 2, 27);
        assert_pow!(27, 1 / 2, None);
        assert_pow!(1 / 4, 1 / 2, 1 / 2);
        assert_pow!(1 / 4, 1 / 3, None);
        assert_pow!(1 / 2, 2, 1 / 4);
        assert_pow!(1 / 4, 3 / 2, 1 / 8);
        assert_pow!(10_000, 1 / 4, 10);
        assert_pow!(1024, 7 / 5, 16384);
        assert_pow!(1234, 1 / 2, None);
        assert_pow!(4567, 234 / 564, None);
        assert_pow!(-7, 0, 1);
        assert_pow!(-56 / 3, 0, 1);
        assert_pow!(-1, 1 / 2, None);
        assert_pow!(-1, 2, 1);
        assert_pow!(-1, 3, -1);
        assert_pow!(-1, 100, 1);
        assert_pow!(-7, 2, 49);
        assert_pow!(-5 / 3, 4, 625 / 81);
        assert_pow!(-8, 1 / 3, -2);
        assert_pow!(-8 / 27, 1 / 3, -2 / 3);
        assert_pow!(-4 / 1023, 6 / 100, None);
        assert_pow!(-535 / 24, 7 / 52, None);
        assert_pow!(-1 / 27, 2 / 6, -1 / 3);
        assert_pow!(-27, 2 / 3, 9);
        assert_pow!(0, -1, None);
        assert_pow!(0, -2 / 7, None);
        assert_pow!(0, -777 / 9, None);
        assert_pow!(1, -1, 1);
        assert_pow!(7 / 8, -1, 8 / 7);
        assert_pow!(2, -2, 1 / 4);
        assert_pow!(3 / 7, -1 / 2, None);
        assert_pow!(4 / 49, -1 / 2, 7 / 2);
        assert_pow!(27 / 125, -4 / 3, 625 / 81);
        assert_pow!(-1, -1, -1);
        assert_pow!(-2, -2, 1 / 4);
        assert_pow!(-2, -1 / 2, None);
        assert_pow!(-1, -7 / 3, -1);
        assert_pow!(-1, -8 / 3, 1);
        assert_pow!(-125, -1 / 3, -1 / 5);
        assert_pow!(-1024 / 243, -2 / 5, 9 / 16);
        assert_pow!(9 / 16, -5 / 2, 1024 / 243);
        assert_pow!(-9 / 16, -5 / 2, None);
        assert_pow!(-9 / 16, -5 / 3, None);
    }
}
