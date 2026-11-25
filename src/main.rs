pub mod fixed_point;
mod account;
mod money;

use fixed_point::*;
use money::*;

fn main() {
    let balance = fixed_point_from(6547, 31, 2);
    let parts = lossless_divide(balance, 2);
    let n: Vec<(i64, i64)> = parts.iter().map(|x| x.split()).collect();
    println!("{:?}", n);

    let balance = fixed_point_from(1, 11193, 5);
    println!("{:?} rounded: {:?}", balance.split(), BankersRule::round(balance, 2).split())
}
