use crate::fixed_point::*;

pub struct BankersRule;
impl RoundStrategy for BankersRule {
    /// Rounding algorithm per https://www.personalinvesting.jpmorgan.com/bankers-rounding
    fn round(a: FixedPoint, precision: P) -> FixedPoint {
            if a.precision == precision { a } else if (a.precision < precision) {
                change_precision(a, precision)
            } else {
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

                    if first_trailing > 5 { inc = 1; } else if first_trailing == 5 {
                        if last % 2 != 0 { inc = 1; }
                    }

                    change_precision(fixed_point((quantity - trailing) * sign, a.precision), precision) + fixed_point((inc) * sign, precision)
                }
            }

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

#[cfg(test)]
mod test {
    use crate::money::*;

    #[test]
    fn test_round_greater_than_1_no_round() {
        let m1 = fixed_point_from(1,114, 3);

        assert_eq!(BankersRule::round(m1, 2), fixed_point_from(1, 11, 2));
    }

    #[test]
    fn test_round_greater_than_1_round_up() {
        let m1 = fixed_point_from(1,115, 3);

        assert_eq!(BankersRule::round(m1, 2), fixed_point_from(1, 12, 2));
    }

    #[test]
    fn test_round_greater_than_1_round_up_greater_than_5() {
        let m1 = fixed_point_from(1,116, 3);

        assert_eq!(BankersRule::round(m1, 2), fixed_point_from(1, 12, 2));
    }

    #[test]
    fn test_round_greater_than_1_no_round_equal_5_even() {
        let m1 = fixed_point_from(1,125, 3);

        assert_eq!(BankersRule::round(m1, 2), fixed_point_from(1, 12, 2));
    }

    #[test]
    fn test_round_greater_than_1_no_round_equal_5_even_negative() {
        let m1 = fixed_point_from(-1,-125, 3);

        assert_eq!(BankersRule::round(m1, 2), fixed_point_from(-1, -12, 2));
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
}
