use std::ops;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Money {
    quantity: i64,
    precision: u32,
}

impl Money {
    fn normal(&self) -> (i64, i64) {
        (
            self.quantity / (10u16.pow(self.precision) as i64)
         ,
            self.quantity % (10u16.pow(self.precision) as i64)
        )
    }
}

impl ops::Add<Money> for Money {
    type Output = Money;
    fn add(self, _rhs: Money) -> Money {
        assert_eq!(self.precision, _rhs.precision);
        Money { quantity: self.quantity + _rhs.quantity, precision: self.precision }
    }
}

impl ops::Sub<Money> for Money {
    type Output = Money;

    fn sub(self, rhs: Money) -> Self::Output {
        assert_eq!(self.precision, rhs.precision);
        Money { quantity: self.quantity - rhs.quantity, precision: self.precision }
    }
}

pub fn lossless_divide(dividend: Money, divisor: u64) -> Vec<Money> {
    let mut v: Vec<Money> = vec![];
    for n in 1..divisor {
        v.push(new_money((dividend.quantity / divisor as i64), dividend.precision));
    }
    v.push(
        new_money((dividend.quantity / divisor as i64) + (dividend.quantity % divisor as i64)
                        , dividend.precision)
    );
    v
}

/// Sum of nonempty Money, with same precision
pub fn sum(elements: Vec<Money>) -> Money {
    assert!(elements.len() > 0);
    assert!(elements.iter().all(|x| x.precision == elements[0].precision));
    let mut s: Money = Money { quantity: 0, precision: elements[0].precision };
    for n in 0..elements.len() {
        s = s + elements[n];
    }
    s
}

/// Create a `Money` object from dollar cent representation
///
/// Precondition: `dollars` and `cents` have the same sign
pub fn money_from(dollars: i64, cents: i64, precision: u32) -> Money {
    assert!((dollars >= 0 && cents >=0) || (dollars <= 0 && cents <= 0));
    Money {
        quantity: dollars * (10u16.pow(precision) as i64) + cents,
        precision: 2
    }
}

/// Shortcut to initializing `Money`
fn new_money(quantity: i64, precision: u32) -> Money {
    Money { quantity, precision }
}

#[cfg(test)]
mod test {
    use crate::money::*;
    #[test]
    fn test_fixed_point_whole() {
        let money: Money = Money{
            quantity: 100,
            precision: 2
        };
        assert_eq!(money.normal(), (1,0));
    }

    #[test]
    fn test_fixed_point_50_cent() {
        let money: Money = Money{
            quantity: 150, precision: 2
        };
        assert_eq!(money.normal(), (1,50));
    }

    #[test]
    fn test_fixed_point_3_place() {
        let money: Money = Money{
            quantity: 150, precision: 3
        };
        assert_eq!(money.normal(), (0,150));
    }

    #[test]
    fn test_fixed_point_large_number() {
        let money: Money = Money{
            quantity: 1_984_893_93, precision: 2
        };
        assert_eq!(money.normal(), (1_984_893,93));
    }

    #[test]
    fn test_money_from_whole() {
        let money: Money = money_from(1, 0, 2);
        assert_eq!(money.quantity, 100)
    }

    #[test]
    fn test_money_from_59_cent() {
        let money: Money = money_from(10, 59, 2);
        assert_eq!(money.quantity, 1059)
    }

    #[test]
    fn test_money_from_large_number() {
        let money: Money = money_from(999_998_997, 96,2);
        assert_eq!(money.quantity,  999_998_997_96)
    }

    #[test]
    fn test_basic_add() {
        let m1 = money_from(1,50, 2);
        let m2 = money_from(2, 00, 2);

        assert_eq!(m1 + m2, money_from(3,50, 2))
    }

    #[test]
    fn test_large_add() {
        let m1 = money_from(999_999_999,50, 2);
        let m2 = money_from(999, 99, 2);

        assert_eq!(m1 + m2, Money { quantity: 999_999_999_50 + 999_99, precision: 2 })
    }

    #[test]
    fn test_basic_sub() {
        let m1 = money_from(1,50, 2);
        let m2 = money_from(2, 00, 2);

        assert_eq!(m1 + m2, money_from(3,50, 2))
    }

    #[test]
    fn test_large_sub() {
        let m1 = money_from(999_999_999,50, 2);
        let m2 = money_from(999, 99, 2);

        assert_eq!(m1 + m2, Money { quantity: 999_999_999_50 + 999_99, precision: 2 })
    }

    #[test]
    fn test_sum_like_large_sub() {
        let m1 = money_from(999_999_999,50, 2);
        let m2 = money_from(999, 99, 2);

        assert_eq!(sum(vec![m1, m2]), Money { quantity: 999_999_999_50 + 999_99, precision: 2 })
    }

    #[test]
    fn test_sum_3_terms() {
        let m1 = money_from(999_999_999,50, 2);
        let m2 = money_from(999, 99, 2);
        let m3 = money_from(950, 99, 2);

        assert_eq!(sum(vec![m1, m2, m3]), Money { quantity: 999_999_999_50 + 999_99 + 950_99, precision: 2 })
    }

    #[test]
    fn test_lossless_divide_no_remainder() {
        let m1 = money_from(3,00, 2);

        assert_eq!(sum(lossless_divide(m1, 2)), m1);
    }

    #[test]
    fn test_lossless_divide_with_remainder() {
        let m1 = money_from(42,57, 2);

        assert_eq!(sum(lossless_divide(m1, 2)), m1);
    }
}