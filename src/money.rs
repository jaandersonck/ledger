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
    /// Return a tuple, `(whole, decimal)`
    fn split(&self) -> (Q, Q) {
        (
            self.quantity / (10 as Q).pow(self.precision)
            ,
            self.quantity % (10 as Q).pow(self.precision)
        )
    }

    /// Compare `self` with a tuple `(Q, Q)`, in split form
    fn eq(&self, rhs: (Q, Q)) -> bool {
        return self.split() == rhs;
    }

    fn less_than_1(&self) -> bool {
        return self.quantity < (10 as Q).pow(self.precision)
    }

    fn positive(&self) -> bool {
        self.quantity > 0
    }
}

pub fn change_precision(a: FixedPoint, precision: P) -> FixedPoint {
    let dP: i32 = precision as i32 - a.precision as i32;

    if dP < 0 {
        panic!("there could be loss here, need to handle it");
    }

    // 3 - 2
    // consider (1,50,2)~150 to (1,500,3)~1500'
    fixed_point(
        a.quantity * (10 as Q).pow(dP as P),
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

// multiply two money objects together
// this makes more sense as an operation between decimals,
// not Money -- since there is no case where multiplying two
// amounts of money together makes sense

pub fn money_mult(lhs: FixedPoint, rhs: FixedPoint) -> FixedPoint {
    // consider, when multiplying decimals, that the deepest precision we'll have out of
    // some expression is lhs.precision * rhs.precision
    // we should always evaluate at this depth, then give an option for truncation/rounding
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
                money_mult(lhs_norm, fixed_point(n.1, norm_precision))
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
    fn test_d_mult() {
        let m1 = fixed_point_from(2, 50, 2);
        let m2 = fixed_point_from(0,0175, 4);

        let r = money_mult(m1, m2);
        assert_eq!(r.precision, m1.precision + m2.precision);
        assert_eq!(r, fixed_point_from(0, 043750, 6));
    }
}