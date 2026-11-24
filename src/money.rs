pub struct Money {
    quantity: u64,
}

impl Money {
    fn normal(&self) -> (u64, u64) {
        (self.quantity / 100, self.quantity % 100)
    }
}

pub fn money_from(dollars: u64, cents: u64) -> Money {
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
}