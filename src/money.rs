use std::ops;
// todo: refactor this into a more generic fixed point class
// todo: so that we can implement other decimal objects, like
// todo: rates

type Q = i64;
type P = u32;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct FixedPoint {
    quantity: Q,
    precision: P,
}

impl FixedPoint {
    /// Split the number at the decimal, returning a tuple `(whole, decimal)`
    pub fn split(&self) -> (Q, Q) {
        (
            self.quantity / (10 as Q).pow(self.precision)
            ,
            self.quantity % (10 as Q).pow(self.precision)
        )
    }

    /// Compare `self` with a tuple in `(whole, decimal)` form
    pub fn eq(&self, rhs: (Q, Q)) -> bool {
        return self.split() == rhs;
    }

    pub fn less_than_1(&self) -> bool {
        return self.quantity < (10 as Q).pow(self.precision)
    }

    pub fn positive(&self) -> bool {
        self.quantity > 0
    }
}

fn pow_10(x: P) -> Q {
    (10 as Q).pow(x)
}

pub fn change_precision(a: FixedPoint, precision: P) -> FixedPoint {
    let dp: i32 = precision as i32 - a.precision as i32;
    let factor = (10 as Q).pow(dp.abs() as P);

    if dp < 0 {
        if a.quantity % factor == 0 {
            // trailing zeroes, no loss
            return fixed_point(
                a.quantity / factor,
                precision
            )
        }
        panic!("Not automatically rounding.");
    }

    // 3 - 2
    // consider (1,50,2)~150 to (1,500,3)~1500
    fixed_point(
        a.quantity * factor,
        precision
    )
}

impl ops::Add<FixedPoint> for FixedPoint {
    type Output = FixedPoint;
    fn add(self, _rhs: FixedPoint) -> FixedPoint {
        assert_eq!(self.precision, _rhs.precision);
        FixedPoint { quantity: self.quantity + _rhs.quantity, precision: self.precision }
    }
}

impl ops::Sub<FixedPoint> for FixedPoint {
    type Output = FixedPoint;

    fn sub(self, rhs: FixedPoint) -> Self::Output {
        assert_eq!(self.precision, rhs.precision);
        FixedPoint { quantity: self.quantity - rhs.quantity, precision: self.precision }
    }
}

impl ops::Mul<FixedPoint> for FixedPoint {
    type Output = FixedPoint;

    fn mul(self, rhs: FixedPoint) -> Self::Output {
        fp_mul(self, rhs)
    }
}

/// Divide some `FixedPoint` into `divisor` parts, with the remainder allocated to the last
/// part.
///
/// Return a vector of `FixedPoint`s, at precision of `dividend`.
pub fn lossless_divide(dividend: FixedPoint, divisor: u64) -> Vec<FixedPoint> {
    let mut v: Vec<FixedPoint> = vec![];
    for _ in 1..divisor {
        v.push(fixed_point((dividend.quantity / divisor as Q), dividend.precision));
    }
    v.push(
        fixed_point((dividend.quantity / divisor as Q) + (dividend.quantity % divisor as Q)
                    , dividend.precision)
    );
    v
}

/// Sum of nonempty `FixedPoint`s, with same precision
pub fn sum(elements: Vec<FixedPoint>) -> FixedPoint {
    assert!(elements.len() > 0);
    assert!(elements.iter().all(|x| x.precision == elements[0].precision));
    let mut s: FixedPoint = FixedPoint { quantity: 0, precision: elements[0].precision };
    for n in 0..elements.len() {
        s = s + elements[n];
    }
    s
}


fn fp_mul(lhs: FixedPoint, rhs: FixedPoint) -> FixedPoint {
    // consider, when multiplying decimals, that the deepest precision we'll have out of
    // some expression is lhs.precision + rhs.precision
    let norm_precision = lhs.precision + rhs.precision;
    let lhs_norm = change_precision(lhs, norm_precision);
    let rhs_norm = change_precision(rhs, norm_precision);

    let result: FixedPoint = || -> FixedPoint {
        if rhs_norm.less_than_1() {
            // consider we have something like (0,50) * (0,50) = (0,25)
            // 50 * 50 = 2500, 250/10 = 25
            fixed_point(lhs_norm.quantity * rhs_norm.quantity / (10 as Q).pow(norm_precision), norm_precision)
        } else {
            let n = rhs_norm.split();
            fixed_point(lhs_norm.quantity * n.0, norm_precision) +
                fp_mul(lhs_norm, fixed_point(n.1, norm_precision))
        }
    }();

    assert_eq!(norm_precision, result.precision);

    result
}

/// Create a `Money` object from a whole number, a decimal, and a precision
///
/// Precondition: `whole` and `decimal` have the same sign
pub fn fixed_point_from(whole: Q, decimal: Q, precision: P) -> FixedPoint {
    assert!((whole >= 0 && decimal >= 0) || (whole <= 0 && decimal <= 0));
    FixedPoint {
        quantity: whole * (10 as Q).pow(precision) + decimal,
        precision
    }
}

/// Shortcut to initializing `FixedPoint`
fn fixed_point(quantity: Q, precision: P) -> FixedPoint {
    FixedPoint { quantity, precision }
}

/// Round a number, `a`, based on an algorithm, to `precision`.
///
/// Design decisions:
/// If precisions are already equal, return the original value.
/// If precision of `a` is less than `precision`, return the original value
/// converted to the desired precision (extra zeros)
///
/// Rounding algorithm per https://www.personalinvesting.jpmorgan.com/bankers-rounding
pub fn bankers_round(a: FixedPoint, precision: P) -> FixedPoint {
    if a.precision == precision { a }
    else if (a.precision < precision) {
        change_precision(a, precision)
    }
    else {
        let sign = a.quantity / a.quantity.abs();
        let quantity = a.quantity.abs();

        if quantity < pow_10(a.precision) {
            panic!("need to solve less than 1 case")
        } else {
            let mut inc = 0;
            // then we have something like (x,000) where the zeros are maintained
            let extra = a.precision - precision;

            // Trailing part of the longer number - to be removed
            let trailing = quantity % pow_10(extra);
            let first_trailing = trailing / pow_10(extra - 1);

            // Last extra + 1 digits of the number, so we can get the digit to be rounded
            let trailing_plus_1 = quantity % pow_10(extra + 1);
            let last = trailing_plus_1 / pow_10(extra);

            if first_trailing > 5 { inc = 1; }
            else if first_trailing == 5 {
                if last % 2 != 0 { inc = 1; }
            }

            change_precision(fixed_point((quantity - trailing) * sign, a.precision), precision) + fixed_point((inc) * sign, precision)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::money::*;

    #[test]
    fn test_compare_50_cent() {
        assert!(
            fixed_point(150, 2).eq((1, 50))
        );
    }
    #[test]
    fn test_fixed_point_whole() {
        let money: FixedPoint = FixedPoint {
            quantity: 100,
            precision: 2
        };
        assert_eq!(money.split(), (1,0));
    }

    #[test]
    fn test_fixed_point_50_cent() {
        let money: FixedPoint = FixedPoint {
            quantity: 150, precision: 2
        };
        assert_eq!(money.split(), (1,50));
    }

    #[test]
    fn test_fixed_point_3_place() {
        let money: FixedPoint = FixedPoint {
            quantity: 150, precision: 3
        };
        assert_eq!(money.split(), (0,150));
    }

    #[test]
    fn test_fixed_point_large_number() {
        let money: FixedPoint = FixedPoint {
            quantity: 1_984_893_93, precision: 2
        };
        assert_eq!(money.split(), (1_984_893,93));
    }

    #[test]
    fn test_money_from_whole() {
        let money: FixedPoint = fixed_point_from(1, 0, 2);
        assert_eq!(money.quantity, 100)
    }

    #[test]
    fn test_money_from_59_cent() {
        let money: FixedPoint = fixed_point_from(10, 59, 2);
        assert_eq!(money.quantity, 1059)
    }

    #[test]
    fn test_money_from_large_number() {
        let money: FixedPoint = fixed_point_from(999_998_997, 96, 2);
        assert_eq!(money.quantity,  999_998_997_96)
    }

    #[test]
    fn test_basic_add() {
        let m1 = fixed_point_from(1, 50, 2);
        let m2 = fixed_point_from(2, 00, 2);

        assert_eq!(m1 + m2, fixed_point_from(3, 50, 2))
    }

    #[test]
    fn test_large_add() {
        let m1 = fixed_point_from(999_999_999, 50, 2);
        let m2 = fixed_point_from(999, 99, 2);

        assert_eq!(m1 + m2, FixedPoint { quantity: 999_999_999_50 + 999_99, precision: 2 })
    }

    #[test]
    fn test_basic_sub() {
        let m1 = fixed_point_from(1, 50, 2);
        let m2 = fixed_point_from(2, 00, 2);

        assert_eq!(m1 + m2, fixed_point_from(3, 50, 2))
    }

    #[test]
    fn test_large_sub() {
        let m1 = fixed_point_from(999_999_999, 50, 2);
        let m2 = fixed_point_from(999, 99, 2);

        assert_eq!(m1 + m2, FixedPoint { quantity: 999_999_999_50 + 999_99, precision: 2 })
    }

    #[test]
    fn test_sum_like_large_sub() {
        let m1 = fixed_point_from(999_999_999, 50, 2);
        let m2 = fixed_point_from(999, 99, 2);

        assert_eq!(sum(vec![m1, m2]), FixedPoint { quantity: 999_999_999_50 + 999_99, precision: 2 })
    }

    #[test]
    fn test_sum_3_terms() {
        let m1 = fixed_point_from(999_999_999, 50, 2);
        let m2 = fixed_point_from(999, 99, 2);
        let m3 = fixed_point_from(950, 99, 2);

        assert_eq!(sum(vec![m1, m2, m3]), FixedPoint { quantity: 999_999_999_50 + 999_99 + 950_99, precision: 2 })
    }

    #[test]
    fn test_lossless_divide_no_remainder() {
        let m1 = fixed_point_from(3, 00, 2);

        assert_eq!(sum(lossless_divide(m1, 2)), m1);
    }

    #[test]
    fn test_lossless_divide_with_remainder() {
        let m1 = fixed_point_from(42, 57, 2);

        assert_eq!(sum(lossless_divide(m1, 2)), m1);
    }

    // change of precision
    #[test]
    fn test_prec_3_to_2() {
        assert_eq!(
            change_precision(fixed_point_from(2,55,2), 3),
            fixed_point_from(2,550, 3)
        )
    }


    // todo: decimal multiplication tests

    #[test]
    fn test_d_mul() {
        let m1 = fixed_point_from(2, 50, 2);
        let m2 = fixed_point_from(0, 0175, 4);

        let r = m1 * m2;
        assert_eq!(r.precision, m1.precision + m2.precision);
        assert_eq!(r, fixed_point_from(0, 043750, 6));
    }

    // banker's rounding tests
    #[test]
    fn test_round_greater_than_1_no_round() {
        let m1 = fixed_point_from(1,114, 3);

        assert_eq!(bankers_round(m1, 2), fixed_point_from(1, 11, 2));
    }

    #[test]
    fn test_round_greater_than_1_round_up() {
        let m1 = fixed_point_from(1,115, 3);

        assert_eq!(bankers_round(m1, 2), fixed_point_from(1, 12, 2));
    }

    #[test]
    fn test_round_greater_than_1_round_up_greater_than_5() {
        let m1 = fixed_point_from(1,116, 3);

        assert_eq!(bankers_round(m1, 2), fixed_point_from(1, 12, 2));
    }

    #[test]
    fn test_round_greater_than_1_no_round_equal_5_even() {
        let m1 = fixed_point_from(1,125, 3);

        assert_eq!(bankers_round(m1, 2), fixed_point_from(1, 12, 2));
    }

    #[test]
    fn test_round_greater_than_1_no_round_equal_5_even_negative() {
        let m1 = fixed_point_from(-1,-125, 3);

        assert_eq!(bankers_round(m1, 2), fixed_point_from(-1, -12, 2));
    }
}