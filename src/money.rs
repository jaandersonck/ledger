use std::ops;

#[derive(PartialEq, Eq, Debug)]
pub struct Money {
    quantity: i64,
}

impl Money {
    fn normal(&self) -> (i64, i64) {
        (self.quantity / 100, self.quantity % 100)
    }
}

impl ops::Add<Money> for Money {
    type Output = Money;
    fn add(self, _rhs: Money) -> Money {
        Money { quantity: self.quantity + _rhs.quantity }
    }
}

impl ops::Sub<Money> for Money {
    type Output = Money;

    fn sub(self, rhs: Money) -> Self::Output {
        Money { quantity: self.quantity - rhs.quantity }
    }
}

/// Create a `Money` object from dollar cent representation
///
/// Precondition: `dollars` and `cents` have the same sign
pub fn money_from(dollars: i64, cents: i64) -> Money {
    assert!((dollars >= 0 && cents >=0) || (dollars <= 0 && cents <= 0));
    Money {
        quantity: dollars * 100 + cents
    }
}

#[cfg(test)]
mod test {
    use crate::money::*;
    #[test]
    fn test_fixed_point_whole() {
        let money: Money = Money{
            quantity: 100
        };
        assert_eq!(money.normal(), (1,0));
    }

    #[test]
    fn test_fixed_point_50_cent() {
        let money: Money = Money{
            quantity: 150
        };
        assert_eq!(money.normal(), (1,50));
    }

    #[test]
    fn test_fixed_point_large_number() {
        let money: Money = Money{
            quantity: 1_984_893_93
        };
        assert_eq!(money.normal(), (1_984_893,93));
    }

    #[test]
    fn test_money_from_whole() {
        let money: Money = money_from(1, 0);
        assert_eq!(money.quantity, 100)
    }

    #[test]
    fn test_money_from_59_cent() {
        let money: Money = money_from(10, 59);
        assert_eq!(money.quantity, 1059)
    }

    #[test]
    fn test_money_from_large_number() {
        let money: Money = money_from(999_998_997, 96);
        assert_eq!(money.quantity,  999_998_997_96)
    }

    #[test]
    fn test_basic_add() {
        let m1 = money_from(1,50);
        let m2 = money_from(2, 00);

        assert_eq!(m1 + m2, money_from(3,50))
    }

    #[test]
    fn test_large_add() {
        let m1 = money_from(999_999_999,50);
        let m2 = money_from(999, 99);

        assert_eq!(m1 + m2, Money { quantity: 999_999_999_50 + 999_99 })
    }

    #[test]
    fn test_basic_sub() {
        let m1 = money_from(1,50);
        let m2 = money_from(2, 00);

        assert_eq!(m1 + m2, money_from(3,50))
    }

    #[test]
    fn test_large_sub() {
        let m1 = money_from(999_999_999,50);
        let m2 = money_from(999, 99);

        assert_eq!(m1 + m2, Money { quantity: 999_999_999_50 + 999_99 })
    }
}